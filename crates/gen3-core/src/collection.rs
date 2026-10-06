//! Read-only regional suggestions, not a shortest-path or legality solver.
use crate::{
    acquisition::{AcquisitionIndex, AcquisitionSource, Target, TargetKind},
    err,
    navigation::MapLink,
    rom::{Evolution, Rom},
    save::Save,
    Result,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
    pub preparation: Option<CollectionPreparation>,
}
#[derive(Clone, Serialize)]
pub struct EvolutionStep {
    pub from: u16,
    pub evolution: Evolution,
    pub related: Vec<Target>,
}
#[derive(Serialize)]
pub struct CollectionPreparation {
    pub origin: u16,
    /// Actual healthy non-egg individuals, even when goals use historical Dex records.
    pub current_count: usize,
    pub source: Option<AcquisitionSource>,
    /// Directed permanent evolution edges, ordered from origin to goal.
    pub steps: Vec<EvolutionStep>,
    pub breeding: Option<crate::breeding_collection::Route>,
    pub needs_hatching: bool,
    pub truncated: bool,
    pub partial: bool,
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
    pub unresolved_incoming: Vec<MapLink>,
}
#[derive(Serialize)]
pub struct CollectionPlan {
    pub dex_status: Option<crate::dex::DexReadStatus>,
    pub entrance_coverage: Option<crate::event_dependencies::Coverage>,
    pub entrance_diagnostics: Vec<String>,
    pub prerequisites: Option<crate::event_dependencies::Bundle>,
    pub clock: Option<crate::clock::ClockReport>,
    pub rom_md5: &'static str,
    pub basis: CollectionBasis,
    pub families: bool,
    pub owned_count: usize,
    pub missing_count: usize,
    pub regions: Vec<CollectionRegion>,
    pub entrances: Vec<EntranceSuggestion>,
    pub breeding_coverage: Option<crate::breeding_collection::Coverage>,
    pub partial: bool,
}
fn root(parents: &BTreeMap<u16, u16>, mut id: u16) -> u16 {
    while parents.get(&id).is_some_and(|p| *p != id) {
        id = parents[&id];
    }
    id
}
struct PreparationContext<'a> {
    current: &'a BTreeMap<u16, usize>,
    period: Option<&'a str>,
    cache: &'a mut BTreeMap<u16, Vec<AcquisitionSource>>,
    breeding: Option<&'a crate::breeding_collection::Suggestions>,
    daycare: Option<&'a AcquisitionSource>,
}
impl AcquisitionIndex {
    fn preparation(
        &self,
        rom: &Rom,
        save: &Save,
        goal: u16,
        context: &mut PreparationContext,
    ) -> Result<Option<CollectionPreparation>> {
        let current = context.current;
        let period = context.period;
        let cache = &mut *context.cache;
        let breeding = context.breeding;
        let daycare = context.daycare;
        // Bounded backwards traversal is only a preparation suggestion. Neither
        // possession nor a directed ROM edge proves that evolution can run now.
        let mut pending = VecDeque::from([(goal, Vec::<EvolutionStep>::new())]);
        let mut candidates = vec![];
        let mut truncated = false;
        let mut examined = 0;
        while let Some((id, steps)) = pending.pop_front() {
            examined += 1;
            if examined > 512 {
                truncated = true;
                break;
            }
            if let Some(route) = breeding.and_then(|b| b.children.get(&id)) {
                candidates.push(CollectionPreparation {
                    origin: id,
                    current_count: 0,
                    source: daycare.cloned(),
                    steps: steps.clone(),
                    breeding: Some(route.clone()),
                    needs_hatching: true,
                    truncated: false,
                    partial: true,
                });
            }
            if !steps.is_empty() {
                let count = current.get(&id).copied().unwrap_or(0);
                if count != 0 {
                    candidates.push(CollectionPreparation {
                        origin: id,
                        current_count: count,
                        source: None,
                        steps: steps.clone(),
                        breeding: None,
                        needs_hatching: false,
                        truncated: false,
                        partial: true,
                    });
                } else {
                    if let std::collections::btree_map::Entry::Vacant(e) = cache.entry(id) {
                        let mut sources = self
                            .query(
                                rom,
                                Some(save),
                                Target {
                                    kind: TargetKind::Species,
                                    id,
                                },
                            )?
                            .sources;
                        self.mark_period(&mut sources, period);
                        e.insert(sources);
                    }
                    for source in &cache[&id] {
                        if source.map_id.is_none()
                            || matches!(source.kind.as_str(), "evolution" | "breeding_candidate")
                            || source.status == "completed" && source.repeatable != Some(true)
                        {
                            continue;
                        }
                        candidates.push(CollectionPreparation {
                            origin: id,
                            current_count: 0,
                            needs_hatching: source.kind == "egg",
                            source: Some(source.clone()),
                            steps: steps.clone(),
                            breeding: None,
                            truncated: false,
                            partial: true,
                        });
                    }
                }
            }
            for (parent, edges) in &self.evolutions {
                for edge in edges.iter().filter(|e| e.target == id) {
                    if *parent == goal || steps.iter().any(|step| step.from == *parent) {
                        continue;
                    }
                    if steps.len() >= 8 || pending.len() + examined >= 512 {
                        truncated = true;
                        continue;
                    }
                    let mut chain = vec![EvolutionStep {
                        from: *parent,
                        evolution: edge.clone(),
                        related: crate::acquisition::evolution_targets(edge),
                    }];
                    chain.extend(steps.clone());
                    pending.push_back((*parent, chain));
                }
            }
        }
        candidates.sort_by_key(|p| {
            (
                p.current_count == 0,
                p.breeding.is_none() && p.source.is_none(),
                p.source.as_ref().is_some_and(|s| s.status == "blocked"),
                p.source
                    .as_ref()
                    .is_some_and(|s| s.in_scenario == Some(false)),
                p.steps.len(),
                p.origin,
                p.source.as_ref().map(|s| s.offset),
            )
        });
        Ok(candidates.into_iter().next().map(|mut p| {
            p.truncated = truncated;
            p
        }))
    }

    pub fn collection(
        &self,
        rom: &Rom,
        save: &Save,
        request: CollectionRequest,
    ) -> Result<CollectionPlan> {
        let passages = crate::event_dependencies::Index::build(rom, &self.world.maps)?;
        self.collection_with_passages(rom, save, request, &passages)
    }

    /// Share the ROM-bound script index with map/reference/prerequisite queries.
    pub fn collection_with_passages(
        &self,
        rom: &Rom,
        save: &Save,
        request: CollectionRequest,
        passages: &crate::event_dependencies::Index,
    ) -> Result<CollectionPlan> {
        let graph = passages.navigation_graph(rom, &self.world.maps, Some(save))?;
        let mut plan = self.collection_goals(rom, save, request)?;
        let ids: BTreeSet<_> = plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .flat_map(|t| {
                [
                    t.source.as_ref(),
                    t.preparation.as_ref().and_then(|p| p.source.as_ref()),
                ]
            })
            .flatten()
            .filter_map(|s| s.map_id.clone())
            .collect();
        plan.entrances = crate::navigation::suggestions(&self.world.maps, &graph, ids);
        plan.entrance_coverage = Some(graph.coverage);
        plan.entrance_diagnostics = graph.diagnostics;
        Ok(plan)
    }

    fn collection_goals(
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
        let breeding = self.breeding_cache.borrow_mut().get(rom, save)?;
        let mut daycare = self.daycare_sources(rom, Some(save));
        daycare.sort_by_key(|s| (s.status == "blocked", s.map_id.is_none(), s.offset));
        let mut current = BTreeMap::new();
        for stored in save
            .all(rom)?
            .into_iter()
            .filter(|p| !p.pokemon.egg && p.pokemon.checksum_ok)
        {
            *current.entry(stored.pokemon.species).or_insert(0) += 1;
        }
        let owned: BTreeSet<u16> = match request.basis {
            CollectionBasis::Individuals => current.keys().copied().collect(),
            CollectionBasis::Dex => {
                if save.dex_read_status()?.is_none() {
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
        let mut preparation_sources = BTreeMap::new();
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
                    preparation: self.preparation(
                        rom,
                        save,
                        id,
                        &mut PreparationContext {
                            current: &current,
                            period,
                            cache: &mut preparation_sources,
                            breeding: breeding.as_ref(),
                            daycare: daycare.first(),
                        },
                    )?,
                };
                regions
                    .entry(task.source.as_ref().and_then(|s| s.region).or_else(|| {
                        task.preparation
                            .as_ref()
                            .and_then(|p| p.source.as_ref())
                            .and_then(|s| s.region)
                    }))
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
            .chain(self.world.map_events.iter().flat_map(|r| {
                r.markers
                    .iter()
                    .flat_map(|m| &m.pokemon)
                    .chain(&r.unplaced_pokemon)
                    .filter(|p| p.trade.is_some() || p.method == "static")
                    .filter_map(|p| p.held_item.filter(|id| *id != 0))
            }))
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
                    || matches!(s.kind.as_str(), "npc_trade_item" | "static_held")
                        && !request.include_unknown_rewards
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
                    preparation: None,
                };
                regions
                    .entry(task.source.as_ref().and_then(|s| s.region))
                    .or_default()
                    .push(task);
            }
        }
        Ok(CollectionPlan {
            dex_status: if matches!(request.basis, CollectionBasis::Dex) {
                save.dex_read_status()?
            } else {
                None
            },
            entrance_coverage: None,
            entrance_diagnostics: Vec::new(),
            prerequisites: None,
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
            entrances: Vec::new(),
            breeding_coverage: breeding.map(|b| b.coverage),
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
