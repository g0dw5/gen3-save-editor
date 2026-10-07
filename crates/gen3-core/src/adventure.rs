//! Player guide built from this ROM's referenced scripts; no shipped quest catalog.
//! Stage values are states, not percentages or a monotonically completed checklist.
use crate::{
    acquisition::{check, compare, ConditionCheck, Target, TargetKind},
    event_dependencies::{Index, Reference},
    event_state::EventSnapshot,
    map_events::EventCondition,
    rom::Rom,
    save::Save,
    world::World,
    Result,
};
pub mod journal;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Serialize)]
pub struct Task {
    pub journal: Option<journal::Journal>,
    pub id: String,
    pub kind: &'static str,
    pub map_id: String,
    pub x: Option<i16>,
    pub y: Option<i16>,
    pub actor: Option<u16>,
    pub goals: Vec<Target>,
    pub text: Vec<String>,
    pub checks: Vec<ConditionCheck>,
    pub status: &'static str,
    pub stage: Option<u16>,
    pub next_candidate: bool,
    /// Each list is alternative candidate actions for one prerequisite, not an AND chain.
    pub prerequisites: Vec<Vec<String>>,
    pub partial: bool,
    #[serde(skip)]
    pub effects: Vec<(&'static str, u16, Option<u16>)>,
}
#[derive(Serialize)]
pub struct Guide {
    pub rom_md5: String,
    pub current_stage: Option<u16>,
    pub story_supported: bool,
    pub tasks: Vec<Task>,
    pub partial: bool,
}
/// Exact-ROM engine semantics, not extracted quest names, dialogue or locations.
fn story_variable(rom: &Rom) -> Option<u16> {
    (rom.profile.md5 == crate::profile::ROCKET.md5).then_some(0x40f7)
}
fn status(receipt: Option<u16>, checks: &[ConditionCheck], save: bool) -> &'static str {
    if receipt == Some(1) {
        "completed"
    } else if !save {
        "unknown"
    } else if checks.iter().any(|c| c.satisfied == Some(false)) {
        "blocked"
    } else if !checks.is_empty() && checks.iter().all(|c| c.satisfied == Some(true)) {
        "ready"
    } else {
        "unknown"
    }
}
fn checked(
    state: Option<&EventSnapshot>,
    rom: &Rom,
    guards: &[EventCondition],
) -> Vec<ConditionCheck> {
    guards
        .iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|c| check(state, Some(rom), c))
        .collect()
}
fn current_stage_branch(guards: &[EventCondition], variable: u16, current: Option<u16>) -> bool {
    guards.iter().any(|c| {
        let op = if c.taken {
            c.comparison
        } else {
            [4, 5, 3, 2, 0, 1]
                .get(c.comparison as usize)
                .copied()
                .unwrap_or(255)
        };
        c.kind == "variable"
            && c.id == variable
            && op != 5
            && current == Some(c.value as u16)
            && current.is_some_and(|v| compare(v as u32, c) == Some(true))
    })
}
fn context(index: &Index, reference: &Reference, offset: usize) -> Vec<String> {
    let Some(script) = index.scripts.get(&reference.root) else {
        return vec![];
    };
    // Nearby referenced text is a clue, not a guaranteed dialogue on this branch.
    let mut text: Vec<_> = script
        .text
        .iter()
        .filter(|r| !r.text.contains("{FC}"))
        .collect();
    text.sort_by_key(|r| r.offset.abs_diff(offset));
    let mut seen = BTreeSet::new();
    text.into_iter()
        .filter(|r| seen.insert(r.text.clone()))
        .take(4)
        .map(|r| r.text.clone())
        .collect()
}

pub fn build(rom: &Rom, save: Option<&Save>, index: &Index, world: &World) -> Result<Guide> {
    index.check_rom(rom)?;
    let state = save
        .zip(rom.profile.event_state)
        .map(|(s, l)| EventSnapshot::new(s, l));
    let variable = story_variable(rom);
    let current_stage = variable.and_then(|id| state.as_ref()?.variable(id));
    let mut tasks = Vec::new();
    let mut seen = BTreeSet::new();
    if let Some(variable) = variable {
        for reference in &index.references {
            let Some(script) = index.scripts.get(&reference.root) else {
                continue;
            };
            for effect in script
                .effects
                .iter()
                .filter(|e| e.kind == "variable" && e.id == variable && e.value.is_some())
            {
                let id = format!("story:{}:{:x}", reference.map_id, effect.offset);
                if !seen.insert(id.clone()) {
                    continue;
                }
                let guards: Vec<_> = effect
                    .conditions
                    .iter()
                    .chain(&reference.conditions)
                    .cloned()
                    .collect();
                let checks = checked(state.as_ref(), rom, &guards);
                let stage_guard = current_stage_branch(&guards, variable, current_stage);
                // Initialization/rank-reset setters and multi-quest counters are not
                // guessed to be the next chronological event just from numeric order.
                let next_candidate = stage_guard
                    && !checks.iter().any(|c| c.satisfied == Some(false))
                    && current_stage != effect.value;
                let actor = world
                    .maps
                    .iter()
                    .find(|m| m.id == reference.map_id)
                    .and_then(|m| {
                        m.objects
                            .iter()
                            .find(|o| Some(o.local_id) == reference.local_id)
                    })
                    .map(|o| o.graphics_id);
                tasks.push(Task {
                    journal: None,
                    id,
                    kind: "main",
                    map_id: reference.map_id.clone(),
                    x: reference.x,
                    y: reference.y,
                    actor,
                    goals: vec![],
                    text: context(index, reference, effect.offset),
                    status: status(None, &checks, save.is_some()),
                    checks,
                    stage: effect.value,
                    next_candidate,
                    prerequisites: vec![],
                    partial: true,
                    effects: vec![("variable", variable, effect.value)],
                });
            }
        }
    }
    for report in &world.map_events {
        for marker in &report.markers {
            let reference = index.references.iter().find(|r| {
                r.map_id == report.map_id
                    && r.root == marker.script.unwrap_or(0)
                    && r.local_id == marker.local_id
            });
            for reward in marker
                .rewards
                .iter()
                .filter(|r| matches!(r.via, "gift" | "pc"))
            {
                let id = format!("reward:{}:{}:{:x}", report.map_id, marker.id, reward.offset);
                if !seen.insert(id.clone()) {
                    continue;
                }
                let guards = reward.conditions.clone();
                let checks = checked(state.as_ref(), rom, &guards);
                let receipt = reward
                    .receipt
                    .as_ref()
                    .and_then(|r| state.as_ref()?.flag(r.flag));
                tasks.push(Task {
                    journal: None,
                    id,
                    kind: "side",
                    map_id: report.map_id.clone(),
                    x: Some(marker.x),
                    y: Some(marker.y),
                    actor: marker.graphics_id,
                    goals: vec![Target {
                        kind: TargetKind::Item,
                        id: reward.item,
                    }],
                    text: reference
                        .map(|r| context(index, r, reward.offset))
                        .unwrap_or_default(),
                    status: status(receipt, &checks, save.is_some()),
                    checks,
                    stage: None,
                    next_candidate: false,
                    prerequisites: vec![],
                    partial: true,
                    effects: reward
                        .receipt
                        .as_ref()
                        .map(|r| vec![("flag", r.flag, Some(1))])
                        .unwrap_or_default(),
                });
            }
            for mon in marker
                .pokemon
                .iter()
                .filter(|m| matches!(m.method, "gift" | "egg"))
            {
                let id = format!("pokemon:{}:{}:{:x}", report.map_id, marker.id, mon.offset);
                if !seen.insert(id.clone()) {
                    continue;
                }
                let checks = checked(state.as_ref(), rom, &mon.conditions);
                tasks.push(Task {
                    journal: None,
                    id,
                    kind: "side",
                    map_id: report.map_id.clone(),
                    x: Some(marker.x),
                    y: Some(marker.y),
                    actor: marker.graphics_id,
                    goals: vec![Target {
                        kind: TargetKind::Species,
                        id: mon.species,
                    }],
                    text: reference
                        .map(|r| context(index, r, mon.offset))
                        .unwrap_or_default(),
                    status: if checks.iter().any(|c| c.satisfied == Some(false)) {
                        "blocked"
                    } else {
                        "unknown"
                    },
                    checks,
                    stage: None,
                    next_candidate: false,
                    prerequisites: vec![],
                    partial: true,
                    effects: vec![],
                });
            }
        }
    }
    // Follow prerequisite setters beyond rewarded NPCs. These are explicit
    // dialogue/location clues, not invented quest-completion records.
    for _ in 0..3 {
        let mut needed: BTreeMap<_, Vec<&EventCondition>> = BTreeMap::new();
        for task in &tasks {
            for check in &task.checks {
                let c = &check.condition;
                if c.kind == "variable"
                    || (c.kind == "flag"
                        && compare(1, c) == Some(true)
                        && compare(0, c) == Some(false))
                {
                    needed.entry((c.kind, c.id)).or_default().push(c);
                }
            }
        }
        let mut additions = vec![];
        for reference in &index.references {
            let Some(script) = index.scripts.get(&reference.root) else {
                continue;
            };
            for effect in &script.effects {
                let Some(guards_needed) = needed.get(&(effect.kind, effect.id)) else {
                    continue;
                };
                if !effect.value.is_some_and(|v| {
                    guards_needed
                        .iter()
                        .any(|c| compare(v as u32, c) == Some(true))
                }) {
                    continue;
                }
                if tasks.iter().any(|t| {
                    t.map_id == reference.map_id
                        && t.effects.contains(&(effect.kind, effect.id, effect.value))
                }) {
                    continue;
                }
                let id = format!("clue:{}:{:x}", reference.map_id, effect.offset);
                if !seen.insert(id.clone()) {
                    continue;
                }
                let text = context(index, reference, effect.offset);
                // Bulk initialization with no player-facing context isn't a task.
                if text.is_empty() && reference.local_id.is_none() {
                    continue;
                }
                let guards: Vec<_> = effect
                    .conditions
                    .iter()
                    .chain(&reference.conditions)
                    .cloned()
                    .collect();
                let checks = checked(state.as_ref(), rom, &guards);
                additions.push(Task {
                    journal: None,
                    id,
                    kind: "prerequisite",
                    map_id: reference.map_id.clone(),
                    x: reference.x,
                    y: reference.y,
                    actor: world
                        .maps
                        .iter()
                        .find(|m| m.id == reference.map_id)
                        .and_then(|m| {
                            m.objects
                                .iter()
                                .find(|o| Some(o.local_id) == reference.local_id)
                        })
                        .map(|o| o.graphics_id),
                    goals: vec![],
                    text,
                    status: if checks.iter().any(|c| c.satisfied == Some(false)) {
                        "blocked"
                    } else {
                        "unknown"
                    },
                    checks,
                    stage: None,
                    next_candidate: false,
                    prerequisites: vec![],
                    partial: true,
                    effects: vec![(effect.kind, effect.id, effect.value)],
                });
                if tasks.len() + additions.len() >= 4096 {
                    break;
                }
            }
            if tasks.len() + additions.len() >= 4096 {
                break;
            }
        }
        if additions.is_empty() {
            break;
        }
        tasks.extend(additions);
    }
    let mut writers: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for task in &tasks {
        for (kind, id, value) in &task.effects {
            writers
                .entry((*kind, *id))
                .or_default()
                .push((&task.id, *value));
        }
    }
    let dependencies: Vec<_> = tasks
        .iter()
        .map(|task| {
            task.checks
                .iter()
                .filter_map(|c| {
                    if !matches!(c.condition.kind, "flag" | "variable") {
                        return None;
                    }
                    let alternatives: Vec<_> = writers
                        .get(&(c.condition.kind, c.condition.id))
                        .into_iter()
                        .flatten()
                        .filter(|(id, value)| {
                            *id != &task.id
                                && value
                                    .is_some_and(|v| compare(v as u32, &c.condition) == Some(true))
                        })
                        .take(64)
                        .map(|(id, _)| (*id).clone())
                        .collect();
                    (!alternatives.is_empty()).then_some(alternatives)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    for (task, dependencies) in tasks.iter_mut().zip(dependencies) {
        task.prerequisites = dependencies;
    }
    tasks.sort_by_key(|t| (t.kind != "main", t.stage, t.map_id.clone(), t.id.clone()));
    let mut guide = Guide {
        rom_md5: rom.profile.md5.into(),
        current_stage,
        story_supported: variable.is_some(),
        tasks,
        partial: true,
    };
    journal::append(rom, state.as_ref(), &mut guide)?;
    if let Some(rules) = rom.profile.quest_journal {
        for task in guide.tasks.iter_mut().filter(|t| t.journal.is_some()) {
            let id = task
                .id
                .strip_prefix("journal:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap();
            let accepted = crate::binary::u16(&rom.data, rules.quests + id * 20 + 10)?;
            let mut seen = BTreeSet::new();
            let journal = task.journal.as_mut().unwrap();
            for r in &index.references {
                let Some(script) = index.scripts.get(&r.root) else {
                    continue;
                };
                if !script
                    .effects
                    .iter()
                    .any(|e| e.kind == "flag" && e.id == accepted && e.value == Some(1))
                    || !seen.insert((r.map_id.clone(), r.x, r.y))
                {
                    continue;
                }
                let actor = world
                    .maps
                    .iter()
                    .find(|m| m.id == r.map_id)
                    .and_then(|m| m.objects.iter().find(|o| Some(o.local_id) == r.local_id))
                    .map(|o| o.graphics_id);
                journal.locations.push(journal::Location {
                    map_id: r.map_id.clone(),
                    x: r.x,
                    y: r.y,
                    actor,
                });
            }
            // These are referenced actions, not a claim that every one is reachable now.
            if let Some(location) = journal.locations.first() {
                task.map_id = location.map_id.clone();
                task.x = location.x;
                task.y = location.y;
                task.actor = location.actor;
            }
        }
    }
    Ok(guide)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn next_clue_requires_the_current_branch_not_a_numeric_successor() {
        let mut guard = EventCondition {
            kind: "variable",
            id: 0x4000,
            value: 19,
            comparison: 2,
            taken: false,
        };
        // The current scene's native `not greater than 19` branch includes 19.
        assert!(current_stage_branch(&[guard.clone()], guard.id, Some(19)));
        // Passing a broad upper bound alone does not identify a next scene.
        assert!(!current_stage_branch(&[guard.clone()], guard.id, Some(18)));
        guard.taken = true;
        assert!(!current_stage_branch(&[guard.clone()], guard.id, Some(19)));
        guard.comparison = 1;
        assert!(current_stage_branch(&[guard.clone()], guard.id, Some(19)));
        guard.taken = false;
        assert!(!current_stage_branch(&[guard.clone()], guard.id, Some(19)));
        assert!(!current_stage_branch(&[guard.clone()], guard.id, None));
        assert!(!current_stage_branch(&[guard.clone()], 0x4001, Some(19)));
    }
    #[test]
    fn completion_uses_receipt_not_item_holdings_or_stage_order() {
        let guard = EventCondition {
            kind: "variable",
            id: 0x4000,
            value: 1,
            comparison: 1,
            taken: true,
        };
        let blocked = ConditionCheck {
            condition: guard.clone(),
            satisfied: Some(false),
            actual: Some(2),
            unresolved: None,
        };
        assert_eq!(
            status(Some(1), std::slice::from_ref(&blocked), true),
            "completed"
        );
        assert_eq!(status(Some(0), &[blocked], true), "blocked");
        assert_eq!(status(None, &[], true), "unknown");
        assert_eq!(
            status(
                None,
                &[ConditionCheck {
                    condition: guard,
                    satisfied: Some(true),
                    actual: Some(1),
                    unresolved: None
                }],
                false
            ),
            "unknown"
        );
    }
}
