//! Read-only regional suggestions, not a shortest-path or legality solver.
use crate::{
    acquisition::{AcquisitionIndex, AcquisitionSource, Target, TargetKind},
    err,
    navigation::MapLink,
    rom::Rom,
    save::Save,
    Result,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionBasis {
    Dex,
    Individuals,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionRequest {
    pub basis: CollectionBasis,
    pub families: bool,
    #[serde(default)]
    pub include_unknown_rewards: bool,
}
#[derive(Serialize)]
pub struct CollectionTask {
    pub target: Target,
    pub family: Vec<u16>,
    pub existing_family_members: Vec<u16>,
    pub source: Option<AcquisitionSource>,
    pub alternatives: usize,
}
#[derive(Serialize)]
pub struct CollectionRegion {
    pub region: Option<u8>,
    pub tasks: Vec<CollectionTask>,
}
#[derive(Serialize)]
pub struct EntranceSuggestion {
    pub map_id: String,
    pub chains: Vec<Vec<MapLink>>,
    pub truncated: bool,
}
#[derive(Serialize)]
pub struct CollectionPlan {
    pub clock: Option<crate::clock::ClockReport>,
    pub rom_md5: &'static str,
    pub basis: CollectionBasis,
    pub families: bool,
    pub owned_count: usize,
    pub missing_count: usize,
    pub regions: Vec<CollectionRegion>,
    pub entrances: Vec<EntranceSuggestion>,
    pub partial: bool,
}
fn root(parents: &BTreeMap<u16, u16>, mut id: u16) -> u16 {
    while parents.get(&id).is_some_and(|p| *p != id) {
        id = parents[&id];
    }
    id
}
impl AcquisitionIndex {
    pub fn collection(
        &self,
        rom: &Rom,
        save: &Save,
        request: CollectionRequest,
    ) -> Result<CollectionPlan> {
        let clock = rom
            .profile
            .clock
            .map(|_| {
                rom.clock_query_with_save(
                    Some(save),
                    crate::clock::ClockScenario {
                        hour: None,
                        weekday: None,
                    },
                )
            })
            .transpose()?;
        let period = clock.as_ref().and_then(|c| c.period);
        let owned: BTreeSet<u16> = match request.basis {
            CollectionBasis::Individuals => save
                .all(rom)?
                .into_iter()
                .filter(|p| !p.pokemon.egg && p.pokemon.checksum_ok)
                .map(|p| p.pokemon.species)
                .collect(),
            CollectionBasis::Dex => {
                if rom.profile.save.dex.is_none() {
                    return Err(err(
                        "collection_dex_unverified",
                        "use current individuals; this ROM's expanded dex flags are unverified",
                    ));
                }
                let owned_numbers: BTreeSet<_> = save
                    .dex()?
                    .into_iter()
                    .filter(|d| d.owned)
                    .map(|d| d.number)
                    .collect();
                self.species
                    .iter()
                    .filter(|s| s.dex_number != 0 && owned_numbers.contains(&s.dex_number))
                    .map(|s| s.id)
                    .collect()
            }
        };
        let mut parents: BTreeMap<_, _> = self.species.iter().map(|s| (s.id, s.id)).collect();
        for (from, evos) in &self.evolutions {
            for evo in evos {
                if parents.contains_key(&evo.target) {
                    let a = root(&parents, *from);
                    let b = root(&parents, evo.target);
                    if a != b {
                        parents.insert(a.max(b), a.min(b));
                    }
                }
            }
        }
        let mut families: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
        for id in parents.keys() {
            families.entry(root(&parents, *id)).or_default().push(*id);
        }
        // Battle-only targets are not independent catching goals. Their source remains
        // visible in ROM form references; permanent forms remain separate species IDs.
        let permanent_references: BTreeSet<_> = self
            .evolutions
            .values()
            .flatten()
            .map(|e| e.target)
            .chain(self.world.encounters.iter().map(|e| e.species))
            .collect();
        let battle_targets: BTreeSet<_> = rom
            .all_battle_forms()?
            .iter()
            .map(|f| f.target)
            .filter(|id| !permanent_references.contains(id))
            .collect();
        let mut regions: BTreeMap<Option<u8>, Vec<CollectionTask>> = BTreeMap::new();
        let mut missing_count = 0;
        for family in families.values() {
            let existing: Vec<_> = family
                .iter()
                .filter(|id| owned.contains(id))
                .copied()
                .collect();
            if request.families && !existing.is_empty() {
                continue;
            }
            let goals: Vec<_> = family
                .iter()
                .filter(|id| !owned.contains(id) && !battle_targets.contains(id))
                .copied()
                .collect();
            if goals.is_empty() {
                continue;
            }
            let sets = if request.families {
                vec![goals]
            } else {
                goals.into_iter().map(|id| vec![id]).collect()
            };
            for goals in sets {
                missing_count += 1;
                let mut candidates = Vec::new();
                for id in &goals {
                    let mut report = self.query(
                        rom,
                        Some(save),
                        Target {
                            kind: TargetKind::Species,
                            id: *id,
                        },
                    )?;
                    self.mark_period(&mut report.sources, period);
                    for s in report.sources {
                        candidates.push((*id, s));
                    }
                }
                // Prefer a direct referenced source with no known failed condition.
                // Unknown conditions remain unknown, never silently counted as accessible.
                candidates.sort_by_key(|(id, s)| {
                    (
                        s.status == "blocked",
                        s.in_scenario == Some(false),
                        s.map_id.is_none(),
                        s.partial,
                        s.region,
                        *id,
                        s.offset,
                    )
                });
                let count = candidates.len();
                let chosen = candidates.into_iter().next();
                let (id, source) = chosen
                    .map(|(id, s)| (id, Some(s)))
                    .unwrap_or((goals[0], None));
                let task = CollectionTask {
                    target: Target {
                        kind: TargetKind::Species,
                        id,
                    },
                    family: family.clone(),
                    existing_family_members: existing.clone(),
                    source,
                    alternatives: count,
                };
                regions
                    .entry(task.source.as_ref().and_then(|s| s.region))
                    .or_default()
                    .push(task);
            }
        }
        let item_ids: BTreeSet<_> = self
            .world
            .map_events
            .iter()
            .flat_map(|r| {
                r.markers
                    .iter()
                    .flat_map(|m| m.rewards.iter())
                    .chain(r.unplaced_rewards.iter())
            })
            .filter(|r| r.via != "shop")
            .map(|r| r.item)
            .collect();
        for id in item_ids {
            let mut seen = BTreeSet::new();
            let mut sources = self
                .query(
                    rom,
                    Some(save),
                    Target {
                        kind: TargetKind::Item,
                        id,
                    },
                )?
                .sources;
            sources.sort_by_key(|s| match s.status {
                "completed" => 0,
                "available" => 1,
                "unknown" => 2,
                _ => 3,
            });
            let completed: BTreeSet<_> = sources
                .iter()
                .filter(|s| s.status == "completed")
                .map(|s| (s.map_id.clone(), s.offset))
                .collect();
            for s in sources {
                if completed.contains(&(s.map_id.clone(), s.offset))
                    || s.map_id.is_none()
                    || s.kind == "shop"
                    || s.status == "completed"
                    || s.status == "unknown" && !request.include_unknown_rewards
                {
                    continue;
                }
                if !seen.insert((s.map_id.clone(), s.offset)) {
                    continue;
                }
                let task = CollectionTask {
                    target: Target {
                        kind: TargetKind::Item,
                        id,
                    },
                    family: vec![],
                    existing_family_members: vec![],
                    source: Some(s),
                    alternatives: 1,
                };
                regions
                    .entry(task.source.as_ref().and_then(|s| s.region))
                    .or_default()
                    .push(task);
            }
        }
        let used_maps: BTreeSet<_> = regions
            .values()
            .flatten()
            .filter_map(|t| t.source.as_ref().and_then(|s| s.map_id.clone()))
            .collect();
        let (edges, _) = crate::navigation::links(&rom.data, &self.world.maps)?;
        let entrances = used_maps
            .into_iter()
            .map(|id| {
                let (chains, truncated) =
                    crate::navigation::approaches(&self.world.maps, &edges, &id);
                EntranceSuggestion {
                    map_id: id,
                    chains,
                    truncated,
                }
            })
            .collect();
        Ok(CollectionPlan {
            clock,
            rom_md5: rom.profile.md5,
            basis: request.basis,
            families: request.families,
            owned_count: owned.len(),
            missing_count,
            regions: regions
                .into_iter()
                .map(|(region, tasks)| CollectionRegion { region, tasks })
                .collect(),
            entrances,
            partial: true,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn family_union_has_no_cycles_and_preserves_branch_members() {
        let mut p = BTreeMap::from([(1, 1), (2, 2), (3, 3), (4, 4)]);
        for (a, b) in [(1, 2), (3, 2), (2, 4), (4, 1)] {
            let a = root(&p, a);
            let b = root(&p, b);
            if a != b {
                p.insert(a.max(b), a.min(b));
            }
        }
        assert!((1..=4).all(|id| root(&p, id) == 1));
    }
}
