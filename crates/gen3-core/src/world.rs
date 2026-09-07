//! Bounded table and script parsing. Missing evidence never means nonexistence.
use crate::{binary::*, err, rom::Rom, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Serialize)]
pub struct Map {
    pub id: String,
    pub group: u8,
    pub number: u8,
    pub name: String,
    pub region: u8,
    pub width: u32,
    pub height: u32,
    pub map_type: u8,
    pub header: usize,
    pub layout: usize,
    pub events: Option<usize>,
    pub scripts: Vec<usize>,
}
#[derive(Clone, Serialize)]
pub struct Encounter {
    pub species: u16,
    pub map_id: String,
    pub map_name: String,
    pub region: u8,
    pub method: String,
    pub min_level: u8,
    pub max_level: u8,
    pub weight: Option<u8>,
    pub encounter_rate: Option<u32>,
    pub slot: Option<u8>,
    pub offset: usize,
    pub conditional: bool,
}
#[derive(Clone, Serialize)]
pub struct TrainerMon {
    pub species: u16,
    pub level: u16,
    pub iv_quality: u16,
    pub held_item: u16,
    pub moves: Vec<u16>,
    pub moves_explicit: bool,
    pub offset: usize,
}
#[derive(Clone, Serialize)]
pub struct Trainer {
    pub diagnostics: Vec<String>,
    pub id: u16,
    pub name: String,
    pub class: u8,
    pub portrait: u8,
    pub female: bool,
    pub double_battle: bool,
    pub items: [u16; 4],
    pub ai: u32,
    pub party: Vec<TrainerMon>,
    pub offset: usize,
}
#[derive(Serialize)]
pub struct ScriptReport {
    pub encounters: Vec<Encounter>,
    pub trainer_ids: Vec<u16>,
    pub stopped_at: Vec<usize>,
}
impl Rom {
    pub fn maps(&self) -> Result<Vec<Map>> {
        let b = &self.data;
        let mut maps = Vec::new();
        for (group, count) in self.profile.map_counts.iter().enumerate() {
            let table = pointer(b, self.profile.maps + group * 4)?;
            for number in 0..*count {
                let h = pointer(b, table + number * 4)?;
                bytes(b, h, 28)?;
                let layout = pointer(b, h)?;
                let width = u32(b, layout)?;
                let height = u32(b, layout + 4)?;
                if width == 0 || height == 0 || width > 1024 || height > 1024 {
                    return Err(err("map_layout", format!("{group}-{number}")));
                }
                let events = pointer(b, h + 4).ok();
                let mut roots = BTreeSet::new();
                if let Some(ev) = events {
                    let counts = bytes(b, ev, 20)?;
                    for (count_off, ptr_off, stride, script_off) in
                        [(0, 4, 24, 16), (2, 12, 16, 12), (3, 16, 12, 8)]
                    {
                        if counts[count_off] > 0 {
                            let p = pointer(b, ev + ptr_off)?;
                            for i in 0..counts[count_off] as usize {
                                let o = p + i * stride;
                                if count_off == 3 && bytes(b, o + 5, 1)?[0] > 4 {
                                    continue;
                                }
                                if let Ok(p) = pointer(b, o + script_off) {
                                    roots.insert(p);
                                }
                            }
                        }
                    }
                }
                if let Ok(p) = pointer(b, h + 8) {
                    for i in 0..32 {
                        let o = p + i * 5;
                        let kind = bytes(b, o, 1)?[0];
                        if kind == 0 {
                            break;
                        }
                        if let Ok(script) = pointer(b, o + 1) {
                            if matches!(kind, 2 | 4) {
                                for j in 0..64 {
                                    let c = script + j * 8;
                                    if u16(b, c)? == 0 {
                                        break;
                                    }
                                    if let Ok(target) = pointer(b, c + 4) {
                                        roots.insert(target);
                                    }
                                }
                            } else {
                                roots.insert(script);
                            }
                        }
                    }
                }
                let region = b[h + 20];
                let name = self.ptr_text(self.profile.regions + region as usize * 8);
                maps.push(Map {
                    id: format!("{group}-{number}"),
                    group: group as u8,
                    number: number as u8,
                    name: format!(
                        "{} · {}-{}",
                        if name.is_empty() { "Map" } else { &name },
                        group,
                        number
                    ),
                    region,
                    width,
                    height,
                    map_type: b[h + 23],
                    header: h,
                    layout,
                    events,
                    scripts: roots.into_iter().collect(),
                });
            }
        }
        Ok(maps)
    }
    pub fn encounters(&self) -> Result<Vec<Encounter>> {
        let maps = self.maps()?;
        let by_id: BTreeMap<_, _> = maps.iter().map(|m| (m.id.clone(), m)).collect();
        let mut out = Vec::new();
        let b = &self.data;
        for i in 0..600 {
            let o = self.profile.wild + i * 20;
            let h = bytes(b, o, 20)?;
            if h[0] == 255 && h[1] == 255 {
                break;
            }
            let id = format!("{}-{}", h[0], h[1]);
            let Some(map) = by_id.get(&id) else {
                return Err(err("encounter_map", id));
            };
            for (ptr_off, method, count) in [
                (4, "grass", 12),
                (8, "surf", 5),
                (12, "rock_smash", 5),
                (16, "fishing", 10),
            ] {
                if u32(h, ptr_off)? == 0 {
                    continue;
                }
                let p = pointer(b, o + ptr_off)?;
                let rate = u32(b, p)?;
                let list = pointer(b, p + 4)?;
                for j in 0..count {
                    let off = list + j * 4;
                    let min = bytes(b, off, 2)?[0];
                    let max = b[off + 1];
                    let species = u16(b, off + 2)?;
                    self.valid_species(species)?;
                    if min > max || max > 100 {
                        return Err(err("encounter_level", off));
                    }
                    let (method, weight) = if method == "fishing" {
                        if j < 2 {
                            ("old_rod", [70, 30][j])
                        } else if j < 5 {
                            ("good_rod", [60, 20, 20][j - 2])
                        } else {
                            ("super_rod", [40, 40, 15, 4, 1][j - 5])
                        }
                    } else if method == "grass" {
                        (
                            if map.map_type == 4 { "cave" } else { method },
                            [20, 20, 10, 10, 10, 10, 5, 5, 4, 4, 1, 1][j],
                        )
                    } else {
                        (
                            if method == "surf" && map.map_type == 5 {
                                "dive"
                            } else {
                                method
                            },
                            [60, 30, 5, 4, 1][j],
                        )
                    };
                    out.push(Encounter {
                        species,
                        map_id: id.clone(),
                        map_name: map.name.clone(),
                        region: map.region,
                        method: method.into(),
                        min_level: min,
                        max_level: max,
                        weight: Some(weight),
                        encounter_rate: Some(rate),
                        slot: Some(j as u8),
                        offset: off,
                        conditional: false,
                    });
                }
            }
        }
        for map in maps {
            out.extend(self.script_report(&map)?.encounters);
        }
        Ok(out)
    }
    pub fn trainers(&self) -> Result<Vec<Trainer>> {
        // The old table at 0x1132660 has no base references. The active table is
        // referenced by code at 0x3587c, 0x6e624 (+4), and 0x13094c (+16).
        let b = &self.data;
        let mut out = Vec::new();
        for id in 1..self.profile.trainers.count {
            let o = self.profile.trainers.offset + id * self.profile.trainers.stride;
            let row = bytes(b, o, 40)?;
            let flags = row[0];
            let n = row[32] as usize;
            if n == 0 {
                continue;
            }
            if flags > 3 || n > 6 {
                return Err(err("trainer_layout", id));
            }
            let ptr = pointer(b, o + 36)?;
            let stride = if flags & 1 != 0 { 16 } else { 8 };
            let mut party = Vec::new();
            let mut diagnostics = Vec::new();
            for i in 0..n {
                let p = ptr + i * stride;
                let quality = u16(b, p)?;
                let lv = bytes(b, p + 2, 1)?[0] as u16;
                let species = u16(b, p + 4)?;
                if self.valid_species(species).is_err() {
                    diagnostics.push(format!("party[{i}].species={species}"));
                }
                if lv == 0 || lv > 100 {
                    diagnostics.push(format!("party[{i}].level={lv}"));
                }
                let item = if flags & 2 != 0 { u16(b, p + 6)? } else { 0 };
                if self.item(item).is_err() {
                    diagnostics.push(format!("party[{i}].held_item={item}"));
                }
                let mut moves = Vec::new();
                if flags & 1 != 0 {
                    let start = if flags & 2 != 0 { 8 } else { 6 };
                    for j in 0..4 {
                        let mv = u16(b, p + start + j * 2)?;
                        if self.move_info(mv).is_err() {
                            diagnostics.push(format!("party[{i}].moves[{j}]={mv}"));
                        }
                        moves.push(mv);
                    }
                } else {
                    for r in self.level_moves(species).unwrap_or_default() {
                        if r.level.unwrap_or(0) <= lv as u8 {
                            moves.retain(|m| *m != r.move_id);
                            moves.push(r.move_id);
                        }
                    }
                    if moves.len() > 4 {
                        moves = moves[moves.len() - 4..].to_vec();
                    }
                }
                party.push(TrainerMon {
                    species,
                    level: lv,
                    iv_quality: quality,
                    held_item: item,
                    moves,
                    moves_explicit: flags & 1 != 0,
                    offset: p,
                });
            }
            out.push(Trainer {
                diagnostics,
                id: id as u16,
                name: self.codec.decode(&row[4..16]),
                class: row[1],
                portrait: row[3],
                female: row[2] & 128 != 0,
                double_battle: row[24] != 0,
                items: std::array::from_fn(|i| u16(row, 16 + i * 2).unwrap()),
                ai: u32(row, 28)?,
                party,
                offset: o,
            });
        }
        Ok(out)
    }
    pub fn script_report(&self, map: &Map) -> Result<ScriptReport> {
        let b = &self.data;
        let mut found = BTreeMap::new();
        let mut trainers = BTreeSet::new();
        let mut stopped = BTreeSet::new();
        let mut visited = BTreeSet::new();
        // Every branch carries its own known constants. No arbitrary byte scan.
        let mut pending: VecDeque<_> = map
            .scripts
            .iter()
            .map(|p| (*p, BTreeMap::<u16, u16>::new()))
            .collect();
        while let Some((mut pc, mut vars)) = pending.pop_front() {
            for _ in 0..2048 {
                if visited.len() > 8192 {
                    stopped.insert(pc);
                    break;
                }
                let signature = (pc, vars.clone());
                if !visited.insert(signature) {
                    break;
                }
                let Some(op) = b.get(pc).copied() else {
                    break;
                };
                let len = match op {
                    0x5c => {
                        let typ = *b.get(pc + 1).unwrap_or(&255);
                        match typ {
                            0 | 5 | 9..=12 => 14,
                            1 | 2 | 4 | 7 => 18,
                            3 => 10,
                            6 | 8 => 22,
                            _ => 0,
                        }
                    }
                    0x00 | 0x01 | 0x02 | 0x03 | 0x27 | 0x28 | 0x30 | 0x31 | 0x32 | 0x33 | 0x5a
                    | 0x5b | 0x66 | 0x68 | 0x69 | 0x6a | 0x6b | 0x6c | 0x97 | 0xb7 | 0xc5 => 1,
                    0x04 | 0x05 | 0x16 | 0x17 | 0x18 | 0x19 | 0x1a | 0x21 | 0x26 | 0x43 | 0x44
                    | 0x45 | 0x67 | 0xa4 => 5,
                    0x06 | 0x07 | 0x0f | 0x4f | 0x51 | 0xb6 => 6,
                    0x08 | 0x09 | 0xdc => 2,
                    0x25 | 0x29 | 0x2a | 0x2b | 0x2f | 0x35 | 0x47 | 0x53 | 0x55 | 0x64 | 0x7a => 3,
                    0x79 => 15,
                    0x80 => 4,
                    _ => 0,
                };
                if len == 0 || bytes(b, pc, len).is_err() {
                    stopped.insert(pc);
                    break;
                }
                let resolve = |v: u16, vars: &BTreeMap<u16, u16>| {
                    if v >= 0x4000 {
                        vars.get(&v).copied()
                    } else {
                        Some(v)
                    }
                };
                if op == 0x16 {
                    vars.insert(u16(b, pc + 1)?, u16(b, pc + 3)?);
                } else if op == 0x19 || op == 0x1a {
                    let dest = u16(b, pc + 1)?;
                    let src = u16(b, pc + 3)?;
                    if let Some(v) = resolve(src, &vars) {
                        vars.insert(dest, v);
                    } else {
                        vars.remove(&dest);
                    }
                }
                let mut encounter = None;
                if matches!(op, 0xb6 | 0x79 | 0x7a) {
                    let species = resolve(u16(b, pc + 1)?, &vars);
                    let lv = if op == 0x7a {
                        Some(5)
                    } else {
                        b.get(pc + 3).copied()
                    };
                    if let (Some(s), Some(l)) = (species, lv) {
                        encounter = Some((
                            s,
                            l,
                            if op == 0xb6 {
                                "static"
                            } else if op == 0x79 {
                                "gift"
                            } else {
                                "egg"
                            },
                        ));
                    }
                }
                if op == 0x25 && u16(b, pc + 1)? == 0x1e2 {
                    if let (Some(s), Some(l)) = (vars.get(&0x8004), vars.get(&0x8005)) {
                        encounter = Some((*s, *l as u8, "special_battle"));
                    }
                }
                if let Some((s, l, method)) = encounter {
                    if self.valid_species(s).is_ok() && l > 0 && l <= 100 {
                        found.entry((pc, s)).or_insert(Encounter {
                            species: s,
                            map_id: map.id.clone(),
                            map_name: map.name.clone(),
                            region: map.region,
                            method: method.into(),
                            min_level: l,
                            max_level: l,
                            weight: None,
                            encounter_rate: None,
                            slot: None,
                            offset: pc,
                            conditional: true,
                        });
                    }
                }
                if op == 0x5c {
                    trainers.insert(u16(b, pc + 2)?);
                    let typ = b[pc + 1];
                    if matches!(typ, 1 | 2 | 6 | 8) {
                        if let Ok(p) = pointer(b, pc + len - 4) {
                            pending.push_back((p, vars.clone()));
                        }
                    }
                }
                if matches!(op, 0x04..=0x07) {
                    let off = if op == 4 || op == 5 { 1 } else { 2 };
                    if let Ok(p) = pointer(b, pc + off) {
                        pending.push_back((p, vars.clone()));
                    }
                    if op == 4 || op == 7 {
                        vars.clear();
                    }
                }
                if matches!(op, 0x02 | 0x03 | 0x05) {
                    break;
                }
                if op == 0x25 {
                    vars.clear();
                }
                pc += len;
            }
        }
        Ok(ScriptReport {
            encounters: found.into_values().collect(),
            trainer_ids: trainers.into_iter().collect(),
            stopped_at: stopped.into_iter().collect(),
        })
    }
}
