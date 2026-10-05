//! Bounded collection suggestions from exact current party/box parent records.
//! Observes native species selection; never reverses evolution or inserts eggs.
use crate::{
    binary::*,
    err,
    native_trainer::Sandbox,
    pokemon,
    rom::Rom,
    save::{Location, Save},
    Result,
};
use armv4t_emu::Memory;
use serde::Serialize;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rules {
    /// Stop before native creation, where r1 holds the selected offspring species.
    pub checkpoint: u32,
}
pub const EMERALD: Rules = Rules {
    checkpoint: 0x080708e8,
};
pub const ROCKET: Rules = Rules {
    checkpoint: 0x0809e8d0,
};
pub const MERCURY: Rules = Rules {
    checkpoint: 0x09d1b68c,
};
const PAIR_LIMIT: usize = 2048;
// Explicit scenarios, not a complete enumeration of the game's live RNG.
const SCENARIOS: [(u32, u32); 4] = [(42, 24), (42, 0x8018), (1, 24), (1, 0x8018)];

#[derive(Clone, Serialize)]
pub struct ParentRef {
    pub location: Location,
    pub species: u16,
    pub nickname: String,
    pub gender: String,
    pub held_item: u16,
}
#[derive(Clone, Serialize)]
pub struct Route {
    pub parents: [ParentRef; 2],
    pub compatibility: u8,
    pub seed: u32,
    pub offspring_pid: u32,
    pub partial: bool,
}
#[derive(Clone, Serialize)]
pub struct Coverage {
    pub parent_count: usize,
    pub checked_pairs: usize,
    pub total_pairs: usize,
    pub failed_pairs: usize,
    pub truncated: bool,
    pub sampled: bool,
    pub issue: Option<String>,
}
#[derive(Clone)]
pub(crate) struct Suggestions {
    pub children: BTreeMap<u16, Route>,
    pub coverage: Coverage,
}
#[derive(Default)]
pub(crate) struct Cache {
    data: Option<Arc<Vec<u8>>>,
    save_hash: String,
    value: Option<Suggestions>,
}
impl Cache {
    pub(crate) fn get(&mut self, rom: &Rom, save: &Save) -> Result<Option<Suggestions>> {
        let save_hash = sha256(&save.data);
        if self
            .data
            .as_ref()
            .is_some_and(|d| Arc::ptr_eq(d, &rom.data))
            && self.save_hash == save_hash
        {
            return Ok(self.value.clone());
        }
        let value = suggestions(rom, save)?;
        self.data = Some(rom.data.clone());
        self.save_hash = save_hash;
        self.value = value.clone();
        Ok(value)
    }
}

/// Executes native compatibility and the constructor up to the verified selection
/// checkpoint. Writes occur only in disposable RAM. Original SAV/ROM stay intact.
pub(crate) fn select(
    rom: &Rom,
    save: Option<&Save>,
    parents: &[Vec<u8>; 2],
    seed: u32,
    pid: u32,
) -> Result<(u8, Option<u16>)> {
    let b = rom
        .profile
        .breeding
        .ok_or_else(|| err("breeding_unverified", "collection"))?;
    let rules = b
        .selection
        .ok_or_else(|| err("breeding_selection_unverified", "collection"))?;
    if pid == 0 || b.pending_width == 2 && pid > u16::MAX as u32 {
        return Err(err("breeding_pending_pid", pid));
    }
    // Checkpoint must still be a Thumb BL; identity is already exact-ROM gated.
    let at = rules
        .checkpoint
        .checked_sub(0x08000000)
        .ok_or_else(|| err("breeding_selection_rule", "checkpoint"))? as usize;
    if u16(&rom.data, at)? & 0xf800 != 0xf000 || u16(&rom.data, at + 2)? & 0xf800 != 0xf800 {
        return Err(err("breeding_selection_rule", "creation call changed"));
    }
    for raw in parents {
        pokemon::checked_unpack_with(raw, rom.profile.save.pokemon_codec)?;
        let mon = pokemon::decode(raw, rom)?;
        rom.valid_species(mon.species)?;
        if mon.egg {
            return Err(err("breeding_parent", "non-egg parent required"));
        }
    }
    let mut ram = Sandbox::new(&rom.data);
    for (pointer, address) in b.save_pointers.into_iter().zip([0x02030000, 0x02034000]) {
        ram.w32(pointer, address);
    }
    if let Some(save) = save {
        for (base, data) in [
            (0x02030000, save.logical(1..=4)),
            (0x02034000, save.logical(0..=0)),
        ] {
            if data.len() > 0x4000 {
                return Err(err("breeding_save_block", data.len()));
            }
            for (i, byte) in data.into_iter().enumerate() {
                ram.w8(base + i as u32, byte);
            }
        }
    }
    let daycare = 0x02030000 + b.daycare as u32;
    for i in 0..b.parent_stride * 2 + 8 {
        ram.w8(daycare + i as u32, 0);
    }
    for (slot, raw) in parents.iter().enumerate() {
        for (i, byte) in raw[..80].iter().enumerate() {
            ram.w8(daycare + (slot * b.parent_stride + i) as u32, *byte);
        }
    }
    ram.w32(b.rng, seed);
    if b.pending_width == 2 {
        ram.w16(daycare + b.pending_pid as u32, pid as u16);
    } else {
        ram.w32(daycare + b.pending_pid as u32, pid);
    }
    let compatibility =
        u8::try_from(ram.call(b.compatibility, [daycare, 0, 0, 0], [0; 2], 1_000_000)?)
            .map_err(|_| err("breeding_compatibility", "selection"))?;
    let child = if compatibility == 0 {
        None
    } else {
        let regs = ram.observe(
            b.constructor,
            [daycare, 0, 0, 0],
            [0; 2],
            1_000_000,
            rules.checkpoint,
        )?;
        let id = u16::try_from(regs[1]).map_err(|_| err("breeding_child", "selection width"))?;
        rom.valid_species(id)?;
        Some(id)
    };
    for (slot, raw) in parents.iter().enumerate() {
        if (0..80).any(|i| ram.r8(daycare + (slot * b.parent_stride + i) as u32) != raw[i]) {
            return Err(err("breeding_parent_changed", slot));
        }
    }
    Ok((compatibility, child))
}

pub(crate) fn suggestions(rom: &Rom, save: &Save) -> Result<Option<Suggestions>> {
    if rom.profile.breeding.and_then(|b| b.selection).is_none() {
        return Ok(None);
    }
    let parents: Vec<_> = save
        .all(rom)?
        .into_iter()
        .filter(|p| !p.pokemon.egg && p.pokemon.checksum_ok)
        .collect();
    let raws: Vec<_> = parents
        .iter()
        .map(|p| save.raw(p.location).map(|r| r[..80].to_vec()))
        .collect::<Result<_>>()?;
    let n = parents.len();
    let total_pairs = n.saturating_mul(n.saturating_sub(1)) / 2;
    let mut result = Suggestions {
        children: BTreeMap::new(),
        coverage: Coverage {
            parent_count: n,
            checked_pairs: 0,
            total_pairs,
            failed_pairs: 0,
            truncated: total_pairs > PAIR_LIMIT,
            sampled: true,
            issue: None,
        },
    };
    'pairs: for a in 0..n {
        for b in a + 1..n {
            if result.coverage.checked_pairs >= PAIR_LIMIT {
                break 'pairs;
            }
            result.coverage.checked_pairs += 1;
            // Preserve order and every individual byte; do not synthesize a Ditto,
            // change held items, genders, OT or personality to obtain an outcome.
            let pair = [raws[a].clone(), raws[b].clone()];
            for (seed, pid) in SCENARIOS {
                match select(rom, Some(save), &pair, seed, pid) {
                    Ok((_, None)) => break,
                    Ok((compatibility, Some(child))) => {
                        result.children.entry(child).or_insert_with(|| Route {
                            parents: [a, b].map(|i| {
                                let p = &parents[i];
                                ParentRef {
                                    location: p.location,
                                    species: p.pokemon.species,
                                    nickname: p.pokemon.nickname.clone(),
                                    gender: p.pokemon.gender.clone(),
                                    held_item: p.pokemon.held_item,
                                }
                            }),
                            compatibility,
                            seed,
                            offspring_pid: pid,
                            partial: true,
                        });
                    }
                    Err(e) => {
                        result.coverage.failed_pairs += 1;
                        result.coverage.issue.get_or_insert_with(|| e.to_string());
                        break;
                    }
                }
            }
        }
    }
    Ok(Some(result))
}
