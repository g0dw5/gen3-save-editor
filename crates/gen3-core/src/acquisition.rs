//! Runtime-only acquisition index. All content comes from the loaded ROM.
use crate::{
    err,
    map_events::EventCondition,
    profile::EventStateLayout,
    rom::{Evolution, LearnSource, Rom, Species},
    save::Save,
    world::World,
    Result,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Species,
    Item,
    Move,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub kind: TargetKind,
    pub id: u16,
}
#[derive(Clone, Serialize)]
pub struct ConditionCheck {
    pub condition: EventCondition,
    pub satisfied: Option<bool>,
    pub actual: Option<u16>,
}
#[derive(Clone, Serialize)]
pub struct AcquisitionSource {
    pub kind: String,
    pub map_id: Option<String>,
    pub region: Option<u8>,
    pub x: Option<i16>,
    pub y: Option<i16>,
    pub related: Vec<Target>,
    pub quantity: Option<u16>,
    pub min_level: Option<u8>,
    pub max_level: Option<u8>,
    /// Slot selection conditional on this encounter method, not held-item chance.
    pub encounter_percent: Option<u8>,
    /// Unknown until the exact native held-item selection routine is verified.
    pub held_percent: Option<u8>,
    pub periods: Vec<String>,
    pub conditions: Vec<ConditionCheck>,
    pub requirements: Vec<crate::rom::EvolutionRequirement>,
    pub evolution: Option<Evolution>,
    pub status: &'static str,
    pub receipt_flag: Option<u16>,
    pub repeatable: Option<bool>,
    pub offset: usize,
    pub partial: bool,
    pub in_scenario: Option<bool>,
}
#[derive(Serialize)]
pub struct AcquisitionReport {
    pub clock: Option<crate::clock::ClockReport>,
    pub target: Target,
    pub sources: Vec<AcquisitionSource>,
    pub partial: bool,
}
pub struct AcquisitionIndex {
    pub world: World,
    pub species: Vec<Species>,
    pub evolutions: BTreeMap<u16, Vec<Evolution>>,
    pub learnsets: BTreeMap<u16, Vec<LearnSource>>,
}
fn source(kind: &str, offset: usize) -> AcquisitionSource {
    AcquisitionSource {
        kind: kind.into(),
        map_id: None,
        region: None,
        x: None,
        y: None,
        related: vec![],
        quantity: None,
        min_level: None,
        max_level: None,
        encounter_percent: None,
        held_percent: None,
        periods: vec![],
        conditions: vec![],
        requirements: vec![],
        evolution: None,
        status: "unknown",
        receipt_flag: None,
        repeatable: None,
        offset,
        partial: true,
        in_scenario: None,
    }
}
fn flag(block: &[u8], layout: EventStateLayout, id: u16) -> Option<u16> {
    if id == 0 || id >= layout.flag_limit {
        return None;
    }
    block
        .get(layout.flags + id as usize / 8)
        .map(|b| ((b >> (id % 8)) & 1) as u16)
}
fn check(
    block: &[u8],
    layout: Option<EventStateLayout>,
    condition: &EventCondition,
) -> ConditionCheck {
    let actual = layout.and_then(|l| match condition.kind {
        "flag" => flag(block, l, condition.id),
        "variable" if condition.id >= 0x4000 && condition.id - 0x4000 < l.variable_count => {
            let o = l.variables + (condition.id - 0x4000) as usize * 2;
            block
                .get(o..o + 2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
        }
        _ => None,
    });
    let satisfied = actual.and_then(|v| {
        Some(
            match condition.comparison {
                0 => v < condition.value,
                1 => v == condition.value,
                2 => v > condition.value,
                3 => v <= condition.value,
                4 => v >= condition.value,
                5 => v != condition.value,
                _ => return None,
            } == condition.taken,
        )
    });
    ConditionCheck {
        condition: condition.clone(),
        actual,
        satisfied,
    }
}
impl AcquisitionIndex {
    pub fn build(rom: &Rom) -> Result<Self> {
        let species: Vec<_> = (1..rom.profile.species.count as u16)
            .filter_map(|id| rom.valid_species(id).ok())
            .collect();
        let mut evolutions = BTreeMap::new();
        let mut learnsets = BTreeMap::new();
        for s in &species {
            evolutions.insert(s.id, rom.evolutions(s.id)?);
            learnsets.insert(s.id, rom.learnset(s.id)?);
        }
        Ok(Self {
            world: rom.world()?,
            species,
            evolutions,
            learnsets,
        })
    }
    pub fn query_scenario(
        &self,
        rom: &Rom,
        save: Option<&Save>,
        target: Target,
        hour: Option<u8>,
        use_save_clock: bool,
    ) -> Result<AcquisitionReport> {
        let mut report = self.query(rom, save, target)?;
        if hour.is_some() || use_save_clock {
            let clock = rom.clock_query_with_save(
                if use_save_clock { save } else { None },
                crate::clock::ClockScenario {
                    hour,
                    weekday: None,
                },
            )?;
            self.mark_period(&mut report.sources, clock.period);
            report.clock = Some(clock);
        }
        Ok(report)
    }
    pub(crate) fn mark_period(&self, sources: &mut [AcquisitionSource], period: Option<&str>) {
        if let Some(period) = period {
            for source in sources {
                if source.periods.is_empty() {
                    continue;
                }
                let base_fallback = source.periods.iter().any(|p| p == "base")
                    && !self.world.encounters.iter().any(|e| {
                        Some(&e.map_id) == source.map_id.as_ref()
                            && e.method == source.kind
                            && e.periods.contains(&period)
                    });
                source.in_scenario =
                    Some(source.periods.iter().any(|p| p == period) || base_fallback);
            }
        }
    }
    pub fn query(
        &self,
        rom: &Rom,
        save: Option<&Save>,
        target: Target,
    ) -> Result<AcquisitionReport> {
        match target.kind {
            TargetKind::Species => {
                rom.valid_species(target.id)?;
            }
            TargetKind::Item => {
                rom.item(target.id)?;
            }
            TargetKind::Move => {
                rom.move_info(target.id)?;
            }
        }
        let block = save.map(|s| s.logical(1..=4));
        let mut sources = Vec::new();
        match target.kind {
            TargetKind::Item => {
                for report in &self.world.map_events {
                    let map = self
                        .world
                        .maps
                        .iter()
                        .find(|m| m.id == report.map_id)
                        .ok_or_else(|| err("map_id", &report.map_id))?;
                    for (marker, reward) in report
                        .markers
                        .iter()
                        .flat_map(|m| m.rewards.iter().map(move |r| (Some(m), r)))
                        .chain(report.unplaced_rewards.iter().map(|r| (None, r)))
                        .filter(|(_, r)| r.item == target.id)
                    {
                        let mut s = source(reward.via, reward.offset);
                        s.map_id = Some(map.id.clone());
                        s.region = Some(map.region);
                        s.quantity = reward.quantity;
                        s.x = marker.map(|m| m.x);
                        s.y = marker.map(|m| m.y);
                        s.conditions = reward
                            .conditions
                            .iter()
                            .map(|c| {
                                check(
                                    block.as_deref().unwrap_or(&[]),
                                    if save.is_some() {
                                        rom.profile.event_state
                                    } else {
                                        None
                                    },
                                    c,
                                )
                            })
                            .collect();
                        // Only the verified item-ball/hidden-item protocols own a receipt flag.
                        if rom.profile.event_state.is_some()
                            && matches!(reward.via, "pickup" | "hidden")
                        {
                            s.receipt_flag = marker.and_then(|m| m.flag);
                            s.repeatable = Some(false);
                        } else if reward.via == "shop" {
                            s.repeatable = Some(true);
                        }
                        let receipt = block.as_ref().and_then(|b| {
                            rom.profile
                                .event_state
                                .and_then(|l| s.receipt_flag.and_then(|f| flag(b, l, f)))
                        });
                        s.partial = marker.is_none_or(|m| !m.stopped_at.is_empty())
                            || !report.stopped_at.is_empty()
                            || rom.profile.event_state.is_none();
                        s.status = if receipt == Some(1) {
                            "completed"
                        } else if s.conditions.iter().any(|c| c.satisfied == Some(false)) {
                            "blocked"
                        } else if save.is_some()
                            && !s.partial
                            && s.conditions.iter().all(|c| c.satisfied == Some(true))
                        {
                            "available"
                        } else {
                            "unknown"
                        };
                        sources.push(s);
                    }
                }
                for species in self.species.iter().filter(|s| s.items.contains(&target.id)) {
                    let mut s = source("wild_held", species.offset);
                    s.related.push(Target {
                        kind: TargetKind::Species,
                        id: species.id,
                    });
                    s.repeatable = Some(true);
                    sources.push(s);
                }
            }
            TargetKind::Species => {
                for e in self
                    .world
                    .encounters
                    .iter()
                    .filter(|e| e.species == target.id)
                {
                    let mut s = source(&e.method, e.offset);
                    s.map_id = Some(e.map_id.clone());
                    s.region = Some(e.region);
                    s.min_level = Some(e.min_level);
                    s.max_level = Some(e.max_level);
                    s.encounter_percent = e.weight;
                    s.periods = e.periods.iter().map(|v| v.to_string()).collect();
                    if let Some(selector) = &e.selector {
                        s.conditions.push(check(
                            block.as_deref().unwrap_or(&[]),
                            if save.is_some() {
                                rom.profile.event_state
                            } else {
                                None
                            },
                            &EventCondition {
                                kind: "variable",
                                id: selector.variable,
                                value: selector.value,
                                comparison: if selector.fallback { 5 } else { 1 },
                                taken: true,
                            },
                        ));
                    }
                    s.repeatable = if matches!(
                        e.method.as_str(),
                        "land" | "water" | "old_rod" | "good_rod" | "super_rod" | "rock_smash"
                    ) {
                        Some(true)
                    } else {
                        None
                    };
                    // Scripted sources and clock selectors require context not captured by raw tables.
                    s.partial = e.conditional || !e.periods.is_empty();
                    if s.conditions.iter().any(|c| c.satisfied == Some(false)) {
                        s.status = "blocked";
                    }
                    sources.push(s);
                }
                for (parent, evos) in &self.evolutions {
                    for evo in evos.iter().filter(|e| e.target == target.id) {
                        let mut s = source("evolution", evo.offset);
                        s.related.push(Target {
                            kind: TargetKind::Species,
                            id: *parent,
                        });
                        s.evolution = Some(evo.clone());
                        s.requirements = evo.requirements.clone();
                        if matches!(
                            evo.condition,
                            "item" | "trade_item" | "held_item_day" | "held_item_night"
                        ) {
                            s.related.push(Target {
                                kind: TargetKind::Item,
                                id: evo.parameter,
                            });
                        } else if evo.condition == "move" {
                            s.related.push(Target {
                                kind: TargetKind::Move,
                                id: evo.parameter,
                            });
                        }
                        for r in &s.requirements {
                            if matches!(r.kind, "item" | "held_item") {
                                s.related.push(Target {
                                    kind: TargetKind::Item,
                                    id: r.value,
                                });
                            } else if r.kind == "move" {
                                s.related.push(Target {
                                    kind: TargetKind::Move,
                                    id: r.value,
                                });
                            }
                        }
                        sources.push(s);
                    }
                }
                let mon = self.species.iter().find(|s| s.id == target.id).unwrap();
                if !mon.egg_groups.iter().any(|g| matches!(g, 0 | 13 | 15))
                    && !self
                        .evolutions
                        .values()
                        .flatten()
                        .any(|e| e.target == target.id)
                {
                    let mut s = source("breeding_candidate", mon.offset);
                    s.related.push(target.clone());
                    sources.push(s);
                }
            }
            TargetKind::Move => {
                for (species, list) in &self.learnsets {
                    for l in list.iter().filter(|l| l.move_id == target.id) {
                        let mut s = source(&format!("learn_{}", l.source), l.offset);
                        s.related.push(Target {
                            kind: TargetKind::Species,
                            id: *species,
                        });
                        s.min_level = l.level;
                        sources.push(s);
                    }
                }
                for id in 1..rom.profile.items.count as u16 {
                    if let Ok(item) = rom.item(id) {
                        if item.tm_move == Some(target.id) {
                            let mut s = source("machine", item.offset);
                            s.related.push(Target {
                                kind: TargetKind::Item,
                                id,
                            });
                            sources.push(s);
                        }
                    }
                }
            }
        }
        Ok(AcquisitionReport {
            clock: None,
            target,
            sources,
            partial: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn time_table_fallback_is_per_map_and_method_not_per_target_species() {
        use crate::world::{Encounter, TrainerLocationIndex};
        let encounter = Encounter {
            selector: None,
            periods: vec!["night"],
            species: 2,
            map_id: "0-0".into(),
            map_name: "Synthetic map".into(),
            region: 1,
            method: "land".into(),
            min_level: 1,
            max_level: 1,
            weight: Some(100),
            encounter_rate: Some(20),
            slot: Some(0),
            offset: 0,
            conditional: true,
        };
        let index = AcquisitionIndex {
            world: World {
                maps: vec![],
                map_events: vec![],
                encounters: vec![encounter],
                trainers: vec![],
                trainer_locations: TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: vec![],
            evolutions: BTreeMap::new(),
            learnsets: BTreeMap::new(),
        };
        let mut sources: Vec<_> = ["land", "water", "land", "static"]
            .into_iter()
            .map(|kind| {
                let mut s = source(kind, 0);
                s.map_id = Some("0-0".into());
                s.periods = vec!["base".into()];
                s
            })
            .collect();
        sources[2].map_id = Some("0-1".into());
        sources[3].periods.clear();
        index.mark_period(&mut sources, Some("night"));
        assert_eq!(
            sources.iter().map(|s| s.in_scenario).collect::<Vec<_>>(),
            [Some(false), Some(true), Some(true), None]
        );
        // The target species need not occur in the overriding table: its base slots still lose priority.
        index.mark_period(&mut sources, Some("day"));
        assert_eq!(sources[0].in_scenario, Some(true));
    }
    #[test]
    fn predicates_are_tristate_and_compare_numeric_values() {
        let layout = EventStateLayout {
            flags: 0,
            flag_limit: 64,
            variables: 8,
            variable_count: 2,
        };
        let mut b = vec![0; 12];
        b[1] = 4;
        b[8] = 7;
        let c = EventCondition {
            kind: "flag",
            id: 10,
            value: 1,
            comparison: 1,
            taken: true,
        };
        assert_eq!(check(&b, Some(layout), &c).satisfied, Some(true));
        assert_eq!(check(&b, None, &c).satisfied, None);
        let c = EventCondition {
            kind: "variable",
            id: 0x4000,
            value: 6,
            comparison: 2,
            taken: false,
        };
        assert_eq!(check(&b, Some(layout), &c).satisfied, Some(false));
        let c = EventCondition { id: 0x800d, ..c };
        assert_eq!(check(&b, Some(layout), &c).satisfied, None);
    }
}
