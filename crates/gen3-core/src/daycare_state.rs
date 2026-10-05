//! Saved ordinary daycare state; native reads in disposable RAM, never live RNG.
use crate::{binary::*, err, native_trainer::Sandbox, pokemon, rom::Rom, save::Save, Result};
use armv4t_emu::Memory;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rules {
    pub getter: u32,
    pub available: u32,
    pub service_state: u32,
    pub parent_steps: usize,
}
pub const EMERALD: Rules = Rules {
    getter: 0x0806a674,
    available: 0x08070bf0,
    service_state: 0x08070cb0,
    parent_steps: 0x88,
};
pub const ROCKET: Rules = Rules {
    getter: 0x080977d0,
    available: 0x0809ebe0,
    service_state: 0x0809eca0,
    parent_steps: 0x88,
};
pub const MERCURY: Rules = Rules {
    getter: 0x0803fd44,
    available: 0x080463fc,
    service_state: 0x080464b4,
    parent_steps: 0x88,
};
const SB1: u32 = 0x02030000;
const SB2: u32 = 0x02034000;

#[derive(Serialize)]
pub struct ParentState {
    pub slot: usize,
    pub present: bool,
    /// Individual data at deposit; accumulated daycare steps are separate.
    pub pokemon: Option<pokemon::Pokemon>,
    pub accumulated_steps: u32,
    pub issue: Option<String>,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub rom_md5: &'static str,
    pub source: &'static str,
    pub status: &'static str,
    pub parents: Vec<ParentState>,
    pub egg_available: bool,
    pub compatibility: Option<u8>,
    /// Distance to the ordinary check only, never steps until an egg is produced.
    pub next_check_steps: Option<u16>,
    pub native_service_state: Option<u32>,
    pub legacy_pending_value: u32,
    pub partial: bool,
}

fn ram<'a>(rom: &'a Rom, save: &Save) -> Result<Sandbox<'a>> {
    let rules = rom
        .profile
        .breeding
        .ok_or_else(|| err("breeding_unverified", "saved state"))?;
    let mut ram = Sandbox::new(&rom.data);
    for (pointer, address) in rules.save_pointers.into_iter().zip([SB1, SB2]) {
        ram.w32(pointer, address);
    }
    for (address, data) in [(SB1, save.logical(1..=4)), (SB2, save.logical(0..=0))] {
        if data.len() > 0x4000 {
            return Err(err("breeding_save_block", data.len()));
        }
        for (i, byte) in data.into_iter().enumerate() {
            ram.w8(address + i as u32, byte);
        }
    }
    Ok(ram)
}
fn read_parent(rom: &Rom, ram: &mut Sandbox, slot: usize) -> Result<(ParentState, Vec<u8>)> {
    let breeding = rom
        .profile
        .breeding
        .ok_or_else(|| err("breeding_unverified", "saved parent"))?;
    let rules = breeding
        .saved
        .ok_or_else(|| err("daycare_state_unverified", "saved parent"))?;
    if slot >= 2 {
        return Err(err("daycare_parent_slot", slot));
    }
    let at = SB1 + (breeding.daycare + slot * breeding.parent_stride) as u32;
    let raw: Vec<_> = (0..80).map(|i| ram.r8(at + i)).collect();
    let present = ram.call(rules.getter, [at, 5, 0, 0], [0; 2], 100_000)?;
    if present > 1 {
        return Err(err("daycare_state_rule", "unexpected presence"));
    }
    let steps = ram.r32(at + rules.parent_steps as u32);
    let (pokemon, issue) = if present == 0 {
        (None, None)
    } else {
        let decoded = pokemon::checked_unpack_with(&raw, rom.profile.save.pokemon_codec)
            .and_then(|_| pokemon::decode(&raw, rom))
            .and_then(|p| {
                rom.valid_species(p.species)?;
                if p.egg {
                    return Err(err("breeding_parent", "deposited egg"));
                }
                Ok(p)
            });
        match decoded {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(e.to_string())),
        }
    };
    if (0..80).any(|i| ram.r8(at + i) != raw[i as usize]) {
        return Err(err("breeding_parent_changed", slot));
    }
    Ok((
        ParentState {
            slot,
            present: present != 0,
            pokemon,
            accumulated_steps: steps,
            issue,
        },
        raw,
    ))
}

pub(crate) fn parent_raw(rom: &Rom, save: &Save, slot: usize) -> Result<Vec<u8>> {
    let (state, raw) = read_parent(rom, &mut ram(rom, save)?, slot)?;
    if state.pokemon.is_none() {
        return Err(err(
            "breeding_parent",
            "a healthy non-egg deposited individual is required",
        ));
    }
    Ok(raw)
}

pub fn snapshot(rom: &Rom, save: &Save) -> Result<Option<Snapshot>> {
    let Some(breeding) = rom.profile.breeding else {
        return Ok(None);
    };
    let Some(rules) = breeding.saved else {
        return Ok(None);
    };
    let mut ram = ram(rom, save)?;
    let main = save.logical(1..=4);
    let trainer = save.logical(0..=0);
    let parents = (0..2)
        .map(|i| read_parent(rom, &mut ram, i).map(|p| p.0))
        .collect::<Result<Vec<_>>>()?;
    let at = SB1 + breeding.daycare as u32;
    let available = ram.call(rules.available, [at, 0, 0, 0], [0; 2], 100_000)?;
    if available > 1 {
        return Err(err("daycare_state_rule", "unexpected availability"));
    }
    // Avoid native checksum repair for invalid individual records. Availability
    // is still a separate native saved marker, not inferred from decoded parents.
    let healthy = parents.iter().all(|p| p.issue.is_none());
    let service = if healthy {
        Some(ram.call(rules.service_state, [0; 4], [0; 2], 100_000)?)
    } else {
        None
    };
    let count = parents.iter().filter(|p| p.present).count();
    let compatibility = if healthy && count == 2 {
        Some(
            u8::try_from(ram.call(breeding.compatibility, [at, 0, 0, 0], [0; 2], 1_000_000)?)
                .map_err(|_| err("breeding_compatibility", "saved state"))?,
        )
    } else {
        None
    };
    let pending = if breeding.pending_width == 2 {
        u32::from(ram.r16(at + breeding.pending_pid as u32))
    } else {
        ram.r32(at + breeding.pending_pid as u32)
    };
    let status = match service {
        Some(0) if count == 0 && available == 0 => "empty",
        Some(1) if available == 1 => "egg_available",
        Some(2) if count == 1 && available == 0 => "one_parent",
        Some(3) if count == 2 && available == 0 => "two_parents",
        _ => "unknown",
    };
    let next_check = if status == "two_parents" && pending == 0 {
        if let Some(production) = breeding.production {
            crate::breeding_production::roll_parameters(rom, production)?;
            let gate = (u16(&rom.data, production.step as usize - 0x08000000 + 0x46)? & 255) as u8;
            let remaining = gate.wrapping_sub(parents[1].accumulated_steps as u8);
            Some(if remaining == 0 {
                u16::from(gate) + 1
            } else {
                u16::from(remaining)
            })
        } else {
            None
        }
    } else {
        None
    };
    for (base, bytes) in [(SB1, main), (SB2, trainer)] {
        if bytes
            .into_iter()
            .enumerate()
            .any(|(i, b)| ram.r8(base + i as u32) != b)
        {
            return Err(err(
                "daycare_state_changed",
                "native read changed a saved block",
            ));
        }
    }
    Ok(Some(Snapshot {
        rom_md5: rom.profile.md5,
        source: "saved_ordinary_daycare",
        status,
        parents,
        egg_available: available != 0,
        compatibility,
        next_check_steps: next_check,
        native_service_state: service,
        legacy_pending_value: pending,
        partial: true,
    }))
}
