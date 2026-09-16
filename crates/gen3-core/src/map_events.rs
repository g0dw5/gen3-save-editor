//! ROM event positions and bounded reward-script traversal, independent of a save.
//! Coordinates use the map layout origin (no runtime seven-tile border).
use crate::{binary::*, rom::Rom, world::Map, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventCondition {
    pub kind: &'static str,
    pub id: u16,
    pub value: u16,
    pub comparison: u8,
    pub taken: bool,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemReward {
    pub item: u16,
    pub quantity: Option<u16>,
    pub offset: usize,
    pub via: &'static str,
    pub conditions: Vec<EventCondition>,
}
#[derive(Clone, Serialize)]
pub struct MapMarker {
    pub id: String,
    pub kind: &'static str,
    pub x: i16,
    pub y: i16,
    pub elevation: u8,
    pub local_id: Option<u8>,
    pub graphics_id: Option<u16>,
    pub movement_type: Option<u8>,
    /// Object visibility flag, or hidden-item collection flag. Not a gift receipt.
    pub flag: Option<u16>,
    pub offset: usize,
    pub script: Option<usize>,
    pub rewards: Vec<ItemReward>,
    pub stopped_at: Vec<usize>,
}
#[derive(Serialize)]
pub struct MapEventReport {
    pub map_id: String,
    pub markers: Vec<MapMarker>,
    /// Map-level scripts have no reliable tile position.
    pub unplaced_rewards: Vec<ItemReward>,
    pub stopped_at: Vec<usize>,
}
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    pc: usize,
    vars: BTreeMap<u16, u16>,
    stack: Vec<usize>,
    conditions: Vec<EventCondition>,
    comparison: Option<(&'static str, u16, u16)>,
    known_comparison: Option<u8>,
}
// Emerald opcode widths, including operands (trainerbattle is variable length).
// Source: pret/pokeemerald src/scrcmd.c. Unsupported control-flow constructs stop.
const LENGTHS: [u8; 221] = [
    1, 1, 1, 1, 5, 5, 6, 6, 2, 2, 3, 3, 1, 1, 2, 6, 3, 6, 6, 6, 3, 9, 5, 5, 5, 5, 5, 3, 3, 6, 6, 6,
    9, 5, 5, 5, 5, 3, 5, 1, 3, 3, 3, 3, 5, 1, 1, 3, 1, 3, 1, 4, 3, 1, 3, 2, 2, 8, 8, 8, 3, 8, 8, 8,
    8, 8, 5, 1, 5, 5, 5, 5, 3, 5, 5, 3, 3, 3, 3, 7, 9, 3, 5, 3, 5, 3, 5, 7, 5, 5, 1, 4, 0, 1, 1, 1,
    3, 3, 3, 7, 3, 4, 1, 5, 1, 1, 1, 1, 1, 1, 3, 5, 6, 6, 5, 5, 5, 5, 1, 2, 5, 15, 3, 5, 3, 4, 2,
    4, 4, 4, 4, 4, 4, 6, 5, 5, 5, 3, 4, 1, 1, 1, 1, 3, 6, 6, 6, 4, 3, 4, 3, 2, 3, 3, 2, 5, 3, 4, 3,
    3, 1, 5, 9, 1, 3, 1, 2, 3, 6, 5, 9, 3, 5, 5, 1, 5, 5, 8, 1, 3, 3, 3, 6, 1, 5, 5, 5, 6, 6, 5, 5,
    6, 3, 3, 3, 2, 8, 1, 4, 1, 1, 1, 1, 1, 1, 3, 3, 1, 1, 8, 4, 3, 1, 3, 1, 8, 1, 1, 1, 5, 2,
];
fn test(value: u8, condition: u8) -> Option<bool> {
    Some(match condition {
        0 => value < 1,
        1 => value == 1,
        2 => value > 1,
        3 => value <= 1,
        4 => value >= 1,
        5 => value != 1,
        _ => return None,
    })
}
fn resolve(s: &State, v: u16) -> Option<u16> {
    if v < 0x4000 {
        Some(v)
    } else {
        s.vars.get(&v).copied()
    }
}
impl Rom {
    pub(crate) fn item_script(&self, root: usize) -> Result<(Vec<ItemReward>, Vec<usize>)> {
        let b = &self.data;
        let mut pending = VecDeque::from([State {
            pc: root,
            ..State::default()
        }]);
        let mut visited = BTreeSet::new();
        let mut rewards = BTreeSet::new();
        let mut stopped = BTreeSet::new();
        let mut steps = 0;
        while let Some(mut s) = pending.pop_front() {
            loop {
                let pc = s.pc;
                if steps >= 8192 || s.stack.len() > 16 || s.conditions.len() > 32 {
                    stopped.insert(pc);
                    break;
                }
                steps += 1;
                if !visited.insert(s.clone()) {
                    break;
                }
                let Some(&op) = b.get(pc) else {
                    stopped.insert(pc);
                    break;
                };
                let len = if op == 0x5c {
                    match b.get(pc + 1) {
                        Some(0 | 5 | 9..=12) => 14,
                        Some(1 | 2 | 4 | 7) => 18,
                        Some(3) => 10,
                        Some(6 | 8) => 22,
                        _ => 0,
                    }
                } else {
                    *LENGTHS.get(op as usize).unwrap_or(&0) as usize
                };
                if len == 0 || bytes(b, pc, len).is_err() {
                    stopped.insert(pc);
                    break;
                }
                s.pc += len;
                let mut reward = None;
                match op {
                    0x02 => break,
                    0x03 => {
                        if let Some(p) = s.stack.pop() {
                            s.pc = p;
                        } else {
                            break;
                        }
                    }
                    0x04 | 0x05 => {
                        if let Ok(p) = pointer(b, pc + 1) {
                            if op == 4 {
                                s.stack.push(s.pc);
                            }
                            s.pc = p;
                        } else {
                            stopped.insert(pc);
                            break;
                        }
                    }
                    0x06 | 0x07 => {
                        let condition = b[pc + 1];
                        if condition > 5 {
                            stopped.insert(pc);
                            break;
                        }
                        let Ok(target) = pointer(b, pc + 2) else {
                            stopped.insert(pc);
                            break;
                        };
                        let known = s.known_comparison.and_then(|v| test(v, condition));
                        let mut branch = s.clone();
                        branch.pc = target;
                        if op == 7 {
                            branch.stack.push(s.pc);
                        }
                        if known.is_none() {
                            let (kind, id, value) = s.comparison.unwrap_or(("unknown", 0, 0));
                            let mut guard = EventCondition {
                                kind,
                                id,
                                value,
                                comparison: condition,
                                taken: true,
                            };
                            if !branch.conditions.contains(&guard) {
                                branch.conditions.push(guard.clone());
                            }
                            guard.taken = false;
                            if !s.conditions.contains(&guard) {
                                s.conditions.push(guard);
                            }
                        }
                        if known != Some(false) {
                            pending.push_back(branch);
                        }
                        if known == Some(true) {
                            break;
                        }
                    }
                    0x08 | 0x09 => {
                        let std = b[pc + 1];
                        if matches!(std, 0 | 1) {
                            reward = Some((
                                resolve(&s, 0x8000),
                                resolve(&s, 0x8001),
                                if std == 1 { "pickup" } else { "gift" },
                            ));
                        }
                        // Standard scripts can overwrite temporary variables/results.
                        s.vars.retain(|k, _| *k < 0x8000);
                        s.comparison = None;
                        s.known_comparison = None;
                        if op == 8 {
                            // Record the reward before returning to the calling script.
                            if let Some((Some(item), quantity, via)) = reward.take() {
                                if item > 0 && self.item(item).is_ok() {
                                    rewards.insert(ItemReward {
                                        item,
                                        quantity,
                                        offset: pc,
                                        via,
                                        conditions: s.conditions.clone(),
                                    });
                                }
                            }
                            if let Some(p) = s.stack.pop() {
                                s.pc = p;
                            } else {
                                break;
                            }
                        }
                    }
                    // Conditional standard scripts, virtual/RAM jumps and trainer engine
                    // continuations need an interpreter; never walk through their operands.
                    0x0a..=0x0d | 0x24 | 0x5e | 0x5f | 0xb8..=0xbf | 0xcf => {
                        stopped.insert(pc);
                        break;
                    }
                    0x16 | 0x19 | 0x1a => {
                        let dst = u16(b, pc + 1)?;
                        let src = u16(b, pc + 3)?;
                        let value = if op == 0x16 {
                            Some(src)
                        } else {
                            resolve(&s, src)
                        };
                        if let Some(v) = value {
                            s.vars.insert(dst, v);
                        } else {
                            s.vars.remove(&dst);
                        }
                    }
                    0x17 | 0x18 => {
                        let dst = u16(b, pc + 1)?;
                        let a = resolve(&s, dst);
                        let v = u16(b, pc + 3)?;
                        if let Some(a) = a {
                            s.vars.insert(
                                dst,
                                if op == 0x17 {
                                    a.wrapping_add(v)
                                } else {
                                    a.wrapping_sub(v)
                                },
                            );
                        } else {
                            s.vars.remove(&dst);
                        }
                    }
                    0x21 | 0x22 => {
                        let id = u16(b, pc + 1)?;
                        let rhs = u16(b, pc + 3)?;
                        let v = if op == 0x21 {
                            Some(rhs)
                        } else {
                            resolve(&s, rhs)
                        };
                        s.known_comparison = resolve(&s, id).zip(v).map(|(a, b)| {
                            if a < b {
                                0
                            } else if a == b {
                                1
                            } else {
                                2
                            }
                        });
                        s.comparison =
                            v.map(|v| (if id < 0x8000 { "variable" } else { "unknown" }, id, v));
                    }
                    0x2b => {
                        s.comparison = Some(("flag", u16(b, pc + 1)?, 1));
                        s.known_comparison = None;
                    }
                    0x1b..=0x20 | 0x60 => {
                        s.comparison = None;
                        s.known_comparison = None;
                    }
                    0x23 | 0x25 | 0x26 => {
                        s.vars.clear();
                        s.comparison = None;
                        s.known_comparison = None;
                        // Dynamic/native rewards cannot be inferred from an item table.
                        stopped.insert(pc);
                    }
                    0x44 | 0x49 => {
                        reward = Some((
                            resolve(&s, u16(b, pc + 1)?),
                            resolve(&s, u16(b, pc + 3)?),
                            if op == 0x49 { "pc" } else { "gift" },
                        ));
                        s.vars.remove(&0x800d);
                    }
                    0x42 => {
                        s.vars.remove(&u16(b, pc + 1)?);
                        s.vars.remove(&u16(b, pc + 3)?);
                    }
                    0x43
                    | 0x45..=0x48
                    | 0x4a..=0x4e
                    | 0x6e..=0x71
                    | 0x79
                    | 0x7a
                    | 0x7c
                    | 0x8f
                    | 0x92
                    | 0x96
                    | 0xa0
                    | 0xb3
                    | 0xce => {
                        s.vars.remove(&0x800d);
                    }
                    0x5c | 0x5d | 0xb7 => {
                        // Battle outcomes and post-battle jumps are conditional.
                        stopped.insert(pc);
                        s.conditions.push(EventCondition {
                            kind: "battle",
                            id: 0,
                            value: 0,
                            comparison: 1,
                            taken: true,
                        });
                        s.vars.clear();
                        s.comparison = None;
                        s.known_comparison = None;
                        if op == 0x5c && matches!(b[pc + 1], 1 | 2 | 6 | 8) {
                            if let Ok(target) = pointer(b, pc + len - 4) {
                                let mut branch = s.clone();
                                branch.pc = target;
                                pending.push_back(branch);
                            }
                        }
                    }
                    _ => {}
                }
                if let Some((item, quantity, via)) = reward {
                    if let Some(item) = item.filter(|i| *i > 0 && self.item(*i).is_ok()) {
                        rewards.insert(ItemReward {
                            item,
                            quantity,
                            offset: pc,
                            via,
                            conditions: s.conditions.clone(),
                        });
                    } else {
                        stopped.insert(pc);
                    }
                }
            }
        }
        Ok((rewards.into_iter().collect(), stopped.into_iter().collect()))
    }

    pub fn map_events(&self, map: &Map) -> Result<MapEventReport> {
        let b = &self.data;
        let mut markers = Vec::new();
        let mut positioned = BTreeSet::new();
        if let Some(ev) = map.events {
            bytes(b, ev, 20)?;
            for (count_off, ptr_off, stride) in [(0, 4, 24), (2, 12, 16), (3, 16, 12)] {
                let count = b[ev + count_off] as usize;
                if count == 0 {
                    continue;
                }
                let table = pointer(b, ev + ptr_off)?;
                bytes(b, table, count * stride)?;
                for i in 0..count {
                    let o = table + i * stride;
                    let (x, y, elevation) = if count_off == 0 {
                        (u16(b, o + 4)? as i16, u16(b, o + 6)? as i16, b[o + 8])
                    } else {
                        (u16(b, o)? as i16, u16(b, o + 2)? as i16, b[o + 4])
                    };
                    let mut marker = MapMarker {
                        id: format!("{}:{o:x}", map.id),
                        kind: if count_off == 0 { "npc" } else { "event" },
                        x,
                        y,
                        elevation,
                        local_id: None,
                        graphics_id: None,
                        movement_type: None,
                        flag: None,
                        offset: o,
                        script: None,
                        rewards: Vec::new(),
                        stopped_at: Vec::new(),
                    };
                    if count_off == 0 {
                        marker.local_id = Some(b[o]);
                        marker.graphics_id = Some(u16(b, o + 1)?);
                        marker.movement_type = Some(b[o + 9]);
                        marker.flag = Some(u16(b, o + 20)?).filter(|f| *f != 0);
                        marker.script = pointer(b, o + 16).ok();
                    } else if count_off == 2 {
                        marker.script = pointer(b, o + 12).ok();
                    } else if b[o + 5] == 7 {
                        marker.kind = "hidden";
                        let item = u16(b, o + 8)?;
                        let index = u16(b, o + 10)?;
                        // Emerald stores a hidden-item index, not an absolute flag ID.
                        marker.flag = index.checked_add(0x1f4);
                        if item > 0 && self.item(item).is_ok() {
                            marker.rewards.push(ItemReward {
                                item,
                                quantity: Some(1),
                                offset: o,
                                via: "hidden",
                                conditions: Vec::new(),
                            });
                        } else {
                            marker.stopped_at.push(o);
                        }
                    } else if b[o + 5] <= 4 {
                        marker.script = pointer(b, o + 8).ok();
                    } else {
                        continue;
                    }
                    if let Some(script) = marker.script {
                        positioned.insert(script);
                        let (rewards, stopped) = self.item_script(script)?;
                        marker.rewards = rewards;
                        marker.stopped_at = stopped;
                        if count_off == 2 {
                            let id = u16(b, o + 6)?;
                            if id != 0 {
                                for reward in &mut marker.rewards {
                                    reward.conditions.insert(
                                        0,
                                        EventCondition {
                                            kind: "variable",
                                            id,
                                            value: u16(b, o + 8)?,
                                            comparison: 1,
                                            taken: true,
                                        },
                                    );
                                }
                            }
                        }
                        if marker.rewards.iter().any(|r| r.via == "pickup") {
                            marker.kind = "pickup";
                        } else if !marker.rewards.is_empty() {
                            marker.kind = "gift";
                        }
                    }
                    // NPC positions remain useful even when their script is unresolved.
                    if count_off == 0 || !marker.rewards.is_empty() {
                        markers.push(marker);
                    }
                }
            }
        }
        let mut unplaced = BTreeSet::new();
        let mut stopped = BTreeSet::new();
        for root in map.scripts.iter().filter(|p| !positioned.contains(p)) {
            let (items, stops) = self.item_script(*root)?;
            unplaced.extend(items);
            stopped.extend(stops);
        }
        Ok(MapEventReport {
            map_id: map.id.clone(),
            markers,
            unplaced_rewards: unplaced.into_iter().collect(),
            stopped_at: stopped.into_iter().collect(),
        })
    }
}
