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
/// Bounded script proof, independent of the object's visibility flag.
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReceiptEvidence {
    pub flag: u16,
    pub root: usize,
    pub award_offset: usize,
    pub success_set_offsets: Vec<usize>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemReward {
    pub item: u16,
    pub quantity: Option<u16>,
    pub offset: usize,
    pub via: &'static str,
    pub conditions: Vec<EventCondition>,
    pub receipt: Option<ReceiptEvidence>,
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
    /// Packed FireRed hidden item is underfoot rather than in front of the player.
    pub underfoot: Option<bool>,
    /// Object visibility flag, or hidden-item collection flag. Not a gift receipt.
    pub flag: Option<u16>,
    /// Separately proven receipt protocol; visibility alone is not evidence.
    pub receipt_flag: Option<u16>,
    pub offset: usize,
    pub script: Option<usize>,
    pub rewards: Vec<ItemReward>,
    pub pokemon: Vec<crate::script_pokemon::PokemonSource>,
    pub teaching: Vec<crate::script_teaching::TeachingSource>,
    pub stopped_at: Vec<usize>,
}
#[derive(Serialize)]
pub struct MapEventReport {
    pub map_id: String,
    pub markers: Vec<MapMarker>,
    /// Map-level scripts have no reliable tile position.
    pub unplaced_rewards: Vec<ItemReward>,
    pub unplaced_pokemon: Vec<crate::script_pokemon::PokemonSource>,
    pub unplaced_teaching: Vec<crate::script_teaching::TeachingSource>,
    pub stopped_at: Vec<usize>,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RewardKey {
    offset: usize,
    item: u16,
    quantity: Option<u16>,
    via: &'static str,
}
impl ItemReward {
    fn key(&self) -> RewardKey {
        RewardKey {
            offset: self.offset,
            item: self.item,
            quantity: self.quantity,
            via: self.via,
        }
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AwardTrace {
    success: bool,
    guards: Vec<EventCondition>,
    sets: BTreeMap<u16, BTreeSet<usize>>,
}
struct Walk {
    rewards: Vec<ItemReward>,
    pokemon: Vec<crate::script_pokemon::PokemonSource>,
    teaching: Vec<crate::script_teaching::TeachingSource>,
    stopped: Vec<usize>,
    terminals: Vec<State>,
    complete: bool,
}
fn guards_unset(guards: &[EventCondition], flag: u16) -> bool {
    guards.iter().any(|c| {
        c.kind == "flag"
            && c.value == 1
            && c.id == flag
            && test(0, c.comparison) == Some(c.taken)
            && test(1, c.comparison) == Some(!c.taken)
    })
}
impl Walk {
    fn receipts(&self, root: usize) -> BTreeMap<RewardKey, ReceiptEvidence> {
        let mut result = BTreeMap::new();
        if !self.complete || !self.stopped.is_empty() {
            return result;
        }
        for reward in &self.rewards {
            let key = reward.key();
            if result.contains_key(&key) || reward.via != "gift" {
                continue;
            }
            for guard in &reward.conditions {
                let flag = guard.id;
                if flag == 0 || !guards_unset(&reward.conditions, flag) {
                    continue;
                }
                let mut sets = BTreeSet::new();
                let mut successes = 0;
                let proven = self.terminals.iter().all(|s| {
                    if let Some(trace) = s.awards.get(&key).filter(|t| t.success) {
                        successes += 1;
                        if !guards_unset(&trace.guards, flag) || s.flags.get(&flag) != Some(&true) {
                            return false;
                        }
                        let Some(offsets) = trace.sets.get(&flag) else {
                            return false;
                        };
                        sets.extend(offsets);
                        true
                    } else {
                        // Rejection, bag failure and alternate rewards must not set it.
                        s.flags.get(&flag) != Some(&true)
                    }
                });
                if proven && successes > 0 && !sets.is_empty() {
                    result.insert(
                        key.clone(),
                        ReceiptEvidence {
                            flag,
                            root,
                            award_offset: key.offset,
                            success_set_offsets: sets.into_iter().collect(),
                        },
                    );
                    break;
                }
            }
        }
        result
    }
}
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    pc: usize,
    vars: BTreeMap<u16, u16>,
    flags: BTreeMap<u16, bool>,
    stack: Vec<usize>,
    conditions: Vec<EventCondition>,
    comparison: Option<(&'static str, u16, u16)>,
    known_comparison: Option<u8>,
    awards: BTreeMap<RewardKey, AwardTrace>,
}
// Emerald opcode widths, including operands (trainerbattle is variable length).
// Source: pret/pokeemerald src/scrcmd.c. Unsupported control-flow constructs stop.
pub(crate) const LENGTHS: [u8; 221] = [
    1, 1, 1, 1, 5, 5, 6, 6, 2, 2, 3, 3, 1, 1, 2, 6, 3, 6, 6, 6, 3, 9, 5, 5, 5, 5, 5, 3, 3, 6, 6, 6,
    9, 5, 5, 5, 5, 3, 5, 1, 3, 3, 3, 3, 5, 1, 1, 3, 1, 3, 1, 4, 3, 1, 3, 2, 2, 8, 8, 8, 3, 8, 8, 8,
    8, 8, 5, 1, 5, 5, 5, 5, 3, 5, 5, 3, 3, 3, 3, 7, 9, 3, 5, 3, 5, 3, 5, 7, 5, 5, 1, 4, 0, 1, 1, 1,
    3, 3, 3, 7, 3, 4, 1, 5, 1, 1, 1, 1, 1, 1, 3, 5, 6, 6, 5, 5, 5, 5, 1, 2, 5, 15, 3, 5, 3, 4, 2,
    4, 4, 4, 4, 4, 4, 6, 5, 5, 5, 3, 4, 1, 1, 1, 1, 3, 6, 6, 6, 4, 3, 4, 3, 2, 3, 3, 2, 5, 3, 4, 3,
    3, 1, 5, 9, 1, 3, 1, 2, 3, 6, 5, 9, 3, 5, 5, 1, 5, 5, 8, 1, 3, 3, 3, 6, 1, 5, 5, 5, 6, 6, 5, 5,
    6, 3, 3, 3, 2, 8, 1, 4, 1, 1, 1, 1, 1, 1, 3, 3, 1, 1, 8, 4, 3, 1, 3, 1, 8, 1, 1, 1, 5, 2,
];
/// Expanded commands verified against Rocket's native dispatch table 0x22b218.
pub(crate) fn expanded_length(op: u8) -> usize {
    if op <= 0xdc {
        LENGTHS[op as usize] as usize
    } else {
        [4, 4, 5, 8, 4, 6, 3, 3, 3, 8, 1, 2, 1, 3]
            .get(op as usize - 0xdd)
            .copied()
            .unwrap_or(0)
    }
}
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
    /// A complete ordinary item-ball script, with no prelude, extra effects or
    /// changed LAST_TALKED identity. Command semantics are verified per adapter;
    /// item IDs and quantities still come from the current ROM's script bytes.
    fn ordinary_pickup_receipt(&self, root: usize) -> bool {
        if !self.profile.event_state.is_some_and(|l| l.pickup_receipt) {
            return false;
        }
        let Ok(code) = bytes(&self.data, root, 13) else {
            return false;
        };
        matches!(code[0], 0x16 | 0x1a)
            && code[1..3] == [0x00, 0x80]
            && u16(code, 3).is_ok_and(|v| v > 0 && v < 0x4000)
            && matches!(code[5], 0x16 | 0x1a)
            && code[6..8] == [0x01, 0x80]
            && u16(code, 8).is_ok_and(|v| v > 0 && v < 0x4000)
            && code[10..13] == [0x09, 0x01, 0x02]
    }

    #[cfg(test)]
    pub(crate) fn item_script(&self, root: usize) -> Result<(Vec<ItemReward>, Vec<usize>)> {
        let walk = self.event_script(root)?;
        Ok((walk.rewards, walk.stopped))
    }
    fn event_script(&self, root: usize) -> Result<Walk> {
        let mut walk = self.walk_item_script(root, false)?;
        // A second, bounded pass follows native boolean award outcomes. Keep the
        // broad catalog traversal independent of proof limits and branch expansion.
        if self.profile.event_state.is_some_and(|l| l.gift_result)
            && walk.stopped.is_empty()
            && walk.rewards.iter().any(|r| {
                r.via == "gift"
                    && r.conditions
                        .iter()
                        .any(|c| guards_unset(&r.conditions, c.id))
            })
        {
            let proof = self.walk_item_script(root, true)?.receipts(root);
            for reward in &mut walk.rewards {
                reward.receipt = proof.get(&reward.key()).cloned();
            }
        }
        Ok(walk)
    }

    fn walk_item_script(&self, root: usize, prove: bool) -> Result<Walk> {
        let b = &self.data;
        let mut pending = VecDeque::from([State {
            pc: root,
            ..State::default()
        }]);
        let mut visited = BTreeSet::new();
        let mut rewards = BTreeSet::new();
        let mut pokemon = BTreeSet::new();
        let mut teaching = BTreeSet::new();
        let mut stopped = BTreeSet::new();
        let mut steps = 0;
        let mut complete = true;
        let mut terminals = Vec::new();
        while let Some(mut s) = pending.pop_front() {
            loop {
                let pc = s.pc;
                if steps >= 8192 || s.stack.len() > 16 || s.conditions.len() > 32 {
                    stopped.insert(pc);
                    break;
                }
                steps += 1;
                if !visited.insert(s.clone()) {
                    // A cycle or merged path cannot certify all terminal outcomes.
                    complete = false;
                    break;
                }
                let Some(&op) = b.get(pc) else {
                    stopped.insert(pc);
                    break;
                };
                let len = if op == 0xb6 {
                    self.wild_command_length(pc).unwrap_or(0)
                } else if op == 0x5c {
                    match b.get(pc + 1) {
                        Some(0 | 5 | 9..=12) => 14,
                        Some(1 | 2 | 4 | 7) => 18,
                        Some(3) => 10,
                        Some(6 | 8) => 22,
                        _ => 0,
                    }
                } else {
                    if matches!(
                        self.profile.formats.scripts,
                        crate::adapter::ScriptFormat::EmeraldExpanded
                    ) {
                        expanded_length(op)
                    } else {
                        *LENGTHS.get(op as usize).unwrap_or(&0) as usize
                    }
                };
                if len == 0 || bytes(b, pc, len).is_err() {
                    stopped.insert(pc);
                    break;
                }
                if !prove {
                    match self.script_teaching_instruction(pc, |v| resolve(&s, v)) {
                        Ok(Some(mut source)) => {
                            source.conditions = s.conditions.clone();
                            teaching.insert(source);
                        }
                        Ok(None) => {}
                        Err(_) => {
                            stopped.insert(pc);
                        }
                    }
                    match self.script_pokemon_instruction(pc, |v| resolve(&s, v)) {
                        Ok(sources) => {
                            for mut source in sources {
                                // A script can set/clear the constructor's mode flag before
                                // the offer. Such a write is not a prerequisite in the SAV.
                                if source.conditions.iter().any(|c| {
                                    c.kind == "flag"
                                        && s.flags.get(&c.id) == Some(&true)
                                        && !c.taken
                                }) {
                                    continue;
                                }
                                source.conditions.retain(|c| {
                                    !(c.kind == "flag"
                                        && s.flags.get(&c.id) == Some(&false)
                                        && !c.taken)
                                });
                                source.conditions.splice(0..0, s.conditions.clone());
                                pokemon.insert(source);
                            }
                        }
                        Err(_) => {
                            stopped.insert(pc);
                        }
                    }
                }
                s.pc += len;
                let mut reward = None;
                let mut terminal = false;
                // Only commands with verified event/variable or presentation
                // semantics may participate in receipt proofs. Width alone is not proof.
                if prove
                    && !matches!(op,
                        0x00..=0x09 | 0x0f | 0x16..=0x19 | 0x1a | 0x21 | 0x22 | 0x29..=0x2b |
                        0x2f..=0x32 | 0x44 | 0x48 | 0x5a | 0x66 | 0x67 | 0x6a..=0x6d | 0x84
                    )
                {
                    complete = false;
                }
                match op {
                    0x02 => {
                        if prove {
                            terminals.push(s.clone());
                        }
                        break;
                    }
                    0x39 | 0x3a
                        if matches!(
                            self.profile.formats.scripts,
                            crate::adapter::ScriptFormat::EmeraldExpanded
                        ) =>
                    {
                        break
                    }
                    0x03 => {
                        if let Some(p) = s.stack.pop() {
                            s.pc = p;
                        } else {
                            if prove {
                                terminals.push(s.clone());
                            }
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
                        if prove && !matches!(std, 0 | 2..=6) {
                            complete = false;
                        }
                        if op == 8 {
                            if let Some(p) = s.stack.pop() {
                                s.pc = p;
                            } else {
                                terminal = true;
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
                    0x29 | 0x2a => {
                        let flag = u16(b, pc + 1)?;
                        s.flags.insert(flag, op == 0x29);
                        if prove && op == 0x29 {
                            for trace in s.awards.values_mut().filter(|t| t.success) {
                                trace.sets.entry(flag).or_default().insert(pc);
                            }
                        }
                    }
                    0x2b => {
                        s.comparison = Some(("flag", u16(b, pc + 1)?, 1));
                        s.known_comparison =
                            s.flags
                                .get(&u16(b, pc + 1)?)
                                .map(|v| if *v { 1 } else { 0 });
                    }
                    0x1b..=0x20 | 0x60 => {
                        s.comparison = None;
                        s.known_comparison = None;
                    }
                    0x23 | 0x25 | 0x26 | 0xb6 => {
                        s.vars.clear();
                        s.flags.clear();
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
                    0x86 => {
                        // Item lists, unlike decoration shops, terminate with item ID zero.
                        if let Ok(table) = pointer(b, pc + 1) {
                            let mut terminated = false;
                            for i in 0..256 {
                                let Some(item) = u16(b, table + i * 2).ok() else {
                                    break;
                                };
                                if item == 0 {
                                    terminated = true;
                                    break;
                                }
                                if self.item(item).is_err() {
                                    break;
                                }
                                rewards.insert(ItemReward {
                                    item,
                                    quantity: None,
                                    offset: pc,
                                    via: "shop",
                                    conditions: s.conditions.clone(),
                                    receipt: None,
                                });
                            }
                            if !terminated {
                                stopped.insert(pc);
                            }
                        } else {
                            stopped.insert(pc);
                        }
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
                    | 0xce
                    | 0xe3
                    | 0xe4
                    | 0xe5 => {
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
                        s.flags.clear();
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
                        let record = ItemReward {
                            item,
                            quantity,
                            offset: pc,
                            via,
                            conditions: s.conditions.clone(),
                            receipt: None,
                        };
                        if prove {
                            if via != "gift" || !quantity.is_some_and(|q| q > 0) {
                                complete = false;
                            } else {
                                let key = record.key();
                                if s.awards.contains_key(&key) {
                                    complete = false;
                                }
                                let trace = AwardTrace {
                                    success: true,
                                    guards: s.conditions.clone(),
                                    sets: BTreeMap::new(),
                                };
                                let mut failure = s.clone();
                                let mut failed_trace = trace.clone();
                                failed_trace.success = false;
                                failure.awards.insert(key.clone(), failed_trace);
                                failure.vars.insert(0x800d, 0);
                                if terminal {
                                    terminals.push(failure);
                                } else {
                                    pending.push_back(failure);
                                }
                                s.awards.insert(key, trace);
                                s.vars.insert(0x800d, 1);
                            }
                        }
                        rewards.insert(record);
                    } else {
                        stopped.insert(pc);
                    }
                }
                if terminal {
                    if prove {
                        terminals.push(s);
                    }
                    break;
                }
            }
        }
        Ok(Walk {
            rewards: rewards.into_iter().collect(),
            pokemon: pokemon.into_iter().collect(),
            teaching: teaching.into_iter().collect(),
            stopped: stopped.into_iter().collect(),
            terminals,
            complete,
        })
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
                        underfoot: None,
                        flag: None,
                        receipt_flag: None,
                        offset: o,
                        script: None,
                        rewards: Vec::new(),
                        pokemon: Vec::new(),
                        teaching: Vec::new(),
                        stopped_at: Vec::new(),
                    };
                    if count_off == 0 {
                        marker.local_id = Some(b[o]);
                        marker.graphics_id =
                            Some(u16(b, o + self.profile.formats.object_graphics_offset)?);
                        marker.movement_type = Some(b[o + 9]);
                        marker.flag = Some(u16(b, o + 20)?).filter(|f| *f != 0);
                        marker.script = pointer(b, o + 16).ok();
                    } else if count_off == 2 {
                        marker.script = pointer(b, o + 12).ok();
                    } else if b[o + 5] == 7 {
                        marker.kind = "hidden";
                        let item = u16(b, o + 8)?;
                        let mut quantity = None;
                        if let Some(rules) = self.profile.hidden_items {
                            let mut base = rules.flag_base;
                            if let Some((table, alternate)) = rules.region_override {
                                let regions = bytes(b, table, 256)?;
                                let end =
                                    regions.iter().position(|r| *r == 0xff).ok_or_else(|| {
                                        crate::err(
                                            "hidden_region_table",
                                            "unterminated region list",
                                        )
                                    })?;
                                if regions[..end].contains(&map.region) {
                                    base = alternate;
                                }
                            }
                            let index = if rules.packed {
                                b[o + 10] as u16
                            } else {
                                u16(b, o + 10)?
                            };
                            marker.flag = base.checked_add(index);
                            marker.receipt_flag = marker.flag;
                            if rules.packed {
                                marker.underfoot = Some(b[o + 11] & 0x80 != 0);
                                // Native underfoot consumer forces one, ignoring the packed quantity.
                                quantity = Some(if marker.underfoot == Some(true) {
                                    1
                                } else {
                                    (b[o + 11] & 0x7f) as u16
                                });
                            } else {
                                quantity = Some(1);
                            }
                        } else {
                            marker.stopped_at.push(o);
                        }
                        if item > 0 && self.item(item).is_ok() {
                            marker.rewards.push(ItemReward {
                                item,
                                quantity,
                                offset: o,
                                via: "hidden",
                                conditions: Vec::new(),
                                receipt: None,
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
                        let report = self.event_script(script)?;
                        marker.rewards = report.rewards;
                        marker.pokemon = report.pokemon;
                        marker.teaching = report.teaching;
                        marker.stopped_at = report.stopped;
                        if count_off == 2 {
                            let id = u16(b, o + 6)?;
                            if id != 0 {
                                for conditions in marker
                                    .rewards
                                    .iter_mut()
                                    .map(|r| &mut r.conditions)
                                    .chain(marker.pokemon.iter_mut().map(|p| &mut p.conditions))
                                    .chain(marker.teaching.iter_mut().map(|t| &mut t.conditions))
                                {
                                    conditions.insert(
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
                            if count_off == 0 && self.ordinary_pickup_receipt(script) {
                                marker.receipt_flag = marker.flag;
                            }
                        } else if !marker.rewards.is_empty() || !marker.pokemon.is_empty() {
                            marker.kind = "gift";
                        }
                    }
                    // NPC positions remain useful even when their script is unresolved.
                    if count_off == 0
                        || !marker.rewards.is_empty()
                        || !marker.pokemon.is_empty()
                        || !marker.teaching.is_empty()
                    {
                        markers.push(marker);
                    }
                }
            }
        }
        let mut unplaced = BTreeSet::new();
        let mut unplaced_pokemon = BTreeSet::new();
        let mut unplaced_teaching = BTreeSet::new();
        let mut stopped = BTreeSet::new();
        for root in map.scripts.iter().filter(|p| !positioned.contains(p)) {
            let report = self.event_script(*root)?;
            unplaced.extend(report.rewards);
            unplaced_pokemon.extend(report.pokemon);
            unplaced_teaching.extend(report.teaching);
            stopped.extend(report.stopped);
        }
        Ok(MapEventReport {
            map_id: map.id.clone(),
            markers,
            unplaced_rewards: unplaced.into_iter().collect(),
            unplaced_pokemon: unplaced_pokemon.into_iter().collect(),
            unplaced_teaching: unplaced_teaching.into_iter().collect(),
            stopped_at: stopped.into_iter().collect(),
        })
    }
}
