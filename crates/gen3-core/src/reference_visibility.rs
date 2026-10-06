//! Runtime presentation filters, never deletion or a proof that an entry is unreachable.
use crate::{
    acquisition::{evolution_targets, TargetKind},
    event_dependencies::Index,
    rom::Rom,
    world::World,
    Result,
};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Default, Serialize)]
pub struct Visibility {
    pub items: Vec<Entry>,
    pub maps: Vec<MapEntry>,
}
#[derive(Serialize)]
pub struct Entry {
    pub id: u16,
    pub reason: &'static str,
}
#[derive(Serialize)]
pub struct MapEntry {
    pub id: String,
    pub reason: &'static str,
}

pub fn classify(rom: &Rom, world: &World, index: &Index) -> Result<Visibility> {
    let mut item_refs: BTreeSet<_> = rom.profile.fishing_rods.into_iter().collect();
    for id in 1..rom.profile.species.count as u16 {
        let s = rom.species(id)?;
        item_refs.extend(s.items.into_iter().filter(|v| *v != 0));
        for e in rom.evolutions(id)? {
            for r in evolution_targets(&e) {
                if r.kind == TargetKind::Item {
                    item_refs.insert(r.id);
                }
            }
        }
    }
    for form in rom.all_battle_forms()? {
        if let crate::forms::BattleTrigger::HeldItem(id) = form.trigger {
            item_refs.insert(id);
        }
    }
    for trainer in &world.trainers {
        item_refs.extend(trainer.items.iter().copied().filter(|v| *v != 0));
        item_refs.extend(
            trainer
                .party
                .iter()
                .map(|p| p.held_item)
                .filter(|v| *v != 0),
        );
    }
    for report in &world.map_events {
        for reward in report
            .markers
            .iter()
            .flat_map(|m| &m.rewards)
            .chain(&report.unplaced_rewards)
        {
            item_refs.insert(reward.item);
            item_refs.extend(
                reward
                    .conditions
                    .iter()
                    .filter(|c| matches!(c.kind, "bag_item" | "bag_item_runtime"))
                    .map(|c| c.id),
            );
        }
        for mon in report
            .markers
            .iter()
            .flat_map(|m| &m.pokemon)
            .chain(&report.unplaced_pokemon)
        {
            if let Some(i) = mon.held_item {
                item_refs.insert(i);
            }
            for member in &mon.battle_members {
                if let Some(i) = member.held_item {
                    item_refs.insert(i);
                }
            }
        }
    }
    let mut out = Visibility::default();
    for id in 1..rom.profile.items.count as u16 {
        let item = rom.item(id)?;
        if item_refs.contains(&id) {
            continue;
        }
        let placeholder = item.name.trim().is_empty()
            || item
                .name
                .chars()
                .all(|c| matches!(c, '?' | '？' | '-' | '—' | ' '));
        // Native machine/held/field functionality counts as use even without a
        // parsed acquisition source. A named, functional item is not a reserved slot.
        let field = crate::binary::u32(&rom.data, item.offset + 28).unwrap_or(0);
        let battle = crate::binary::u32(&rom.data, item.offset + 36).unwrap_or(0);
        if placeholder {
            out.items.push(Entry {
                id,
                reason: "reserved",
            });
        } else if item.tm_move.is_none() && item.hold_effect == 0 && field == 0 && battle == 0 {
            out.items.push(Entry {
                id,
                reason: "unreferenced",
            });
        }
    }
    let graph = index.navigation_graph(rom, &world.maps, None)?;
    let incoming: BTreeSet<_> = graph.edges.iter().filter_map(|e| e.to.as_deref()).collect();
    for map in &world.maps {
        let mismatch = rom
            .profile
            .map_graphics
            .native_mismatch_headers
            .contains(&map.header);
        let no_reference = !incoming.contains(map.id.as_str()) && !matches!(map.map_type, 1..=3);
        if mismatch || no_reference {
            out.maps.push(MapEntry {
                id: map.id.clone(),
                reason: if mismatch {
                    "invalid_layout"
                } else {
                    "unreferenced"
                },
            });
        }
    }
    Ok(out)
}
