//! Ordinary daycare production checks, observed in the current ROM's code.
//! These are query scenarios, not forecasts of live daycare/RNG state or access.
use crate::{binary::*, breeding, err, native_trainer::Sandbox, rom::Rom, save::Save, Result};
use armv4t_emu::Memory;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub enum ItemOperand {
    Immediate { offset: usize, shift: u8 },
    Word { offset: usize },
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rules {
    pub step: u32,
    pub comparison: u32,
    pub roll: usize,
    pub bag_descriptors: u32,
    pub modifier: Option<ItemOperand>,
    pub item_check: Option<u32>,
    pub pending_flag: Option<PendingFlag>,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PendingFlag {
    pub operand: usize,
    pub clear: u32,
}
pub const EMERALD: Rules = Rules {
    step: 0x08070ac4,
    comparison: 0x08070b2c,
    roll: 0x70b1e,
    bag_descriptors: 0x02039dd8,
    modifier: Some(ItemOperand::Immediate {
        offset: 0x310ea2,
        shift: 0,
    }),
    item_check: Some(0x080d6724),
    pending_flag: None,
};
pub const ULTIMATE: Rules = Rules {
    modifier: None,
    item_check: None,
    ..EMERALD
};
pub const ROCKET: Rules = Rules {
    step: 0x0809eaac,
    comparison: 0x0809eb1c,
    roll: 0x9eb0e,
    bag_descriptors: 0x0203addc,
    modifier: Some(ItemOperand::Word { offset: 0x9f35c }),
    item_check: Some(0x0810eae0),
    pending_flag: None,
};
pub const MERCURY: Rules = Rules {
    step: 0x080462c4,
    comparison: 0x0804632c,
    roll: 0x4631e,
    bag_descriptors: 0x0203988c,
    modifier: Some(ItemOperand::Immediate {
        offset: 0x1d1c800,
        shift: 1,
    }),
    item_check: Some(0x08099f40),
    pending_flag: Some(PendingFlag {
        operand: 0x1d1c490,
        clear: 0x0806e6a8,
    }),
};
#[derive(Debug, Serialize)]
pub struct Preview {
    pub scenario: &'static str,
    pub bag_context: &'static str,
    pub modifier_item: Option<u16>,
    pub modifier_present: Option<bool>,
    pub interval_steps: u16,
    pub threshold: u32,
    pub numerator: u32,
    pub denominator: u32,
    pub percent: f64,
    pub scenario_roll: u32,
    pub scenario_passed: bool,
    pub partial: bool,
    pub simulated_pending_flag: Option<u16>,
}

/// Validated instruction semantics. Parameters remain in ROM; no default roll
/// formula is substituted for an unknown instruction sequence or divisor.
pub(crate) fn roll_parameters(rom: &Rom, rules: Rules) -> Result<(u32, u32)> {
    let d = &rom.data;
    let at = rules.roll;
    if bytes(d, at, 4)? != [0, 4, 0, 12]
        || u16(d, at + 4)? & 0xff00 != 0x2100
        || u16(d, at + 6)? != 0x4348
        || u16(d, at + 8)? & 0xff00 != 0x4900
        || rules.comparison as usize - 0x08000000 != at + 14
        || bytes(d, at + 14, 4)? != [0x84, 0x42, 1, 0xd9]
        || u16(d, rules.step as usize - 0x08000000 + 0x46)? != 0x28ff
    {
        return Err(err(
            "breeding_production_rule",
            "native roll/step instructions changed",
        ));
    }
    let scale = u32::from(u16(d, at + 4)? & 255);
    let literal = ((at + 12) & !3) + usize::from(u16(d, at + 8)? & 255) * 4;
    let divisor = u32(d, literal)?;
    if divisor == 0 || scale == 0 {
        return Err(err("breeding_production_rule", "zero native scale/divisor"));
    }
    Ok((scale, divisor))
}
fn modifier_item(rom: &Rom, rules: Rules) -> Result<Option<u16>> {
    rules
        .modifier
        .map(|operand| {
            let id = match operand {
                ItemOperand::Immediate { offset, shift } => {
                    let instruction = u16(&rom.data, offset)?;
                    if instruction & 0xff00 != 0x2000 {
                        return Err(err(
                            "breeding_production_rule",
                            "modifier instruction changed",
                        ));
                    }
                    (instruction & 255) << shift
                }
                ItemOperand::Word { offset } => u16::try_from(u32(&rom.data, offset)?)
                    .map_err(|_| err("breeding_production_rule", "modifier item width"))?,
            };
            rom.item(id)?;
            Ok(id)
        })
        .transpose()
}

pub(crate) fn preview(
    rom: &Rom,
    save: Option<&Save>,
    request: &breeding::Request,
    parents: &[Vec<u8>; 2],
) -> Result<Option<Preview>> {
    let breeding = rom
        .profile
        .breeding
        .ok_or_else(|| err("breeding_unverified", "production"))?;
    let Some(rules) = breeding.production else {
        return if request.production_item.is_none() {
            Ok(None)
        } else {
            Err(err(
                "breeding_production_item",
                "unverified production context",
            ))
        };
    };
    let (scale, divisor) = roll_parameters(rom, rules)?;
    let modifier = modifier_item(rom, rules)?;
    if modifier.is_none() && request.production_item.is_some() {
        return Err(err(
            "breeding_production_item",
            "no verified modifier in this scenario",
        ));
    }
    let mut ram = Sandbox::new(&rom.data);
    const SB1: u32 = 0x02030000;
    const SB2: u32 = 0x02034000;
    for (pointer, address) in breeding.save_pointers.into_iter().zip([SB1, SB2]) {
        ram.w32(pointer, address);
    }
    if let Some(save) = save {
        for (address, data) in [(SB1, save.logical(1..=4)), (SB2, save.logical(0..=0))] {
            if data.len() > 0x4000 {
                return Err(err("breeding_save_block", data.len()));
            }
            for (i, byte) in data.into_iter().enumerate() {
                ram.w8(address + i as u32, byte);
            }
        }
    }
    // Mercury's availability lives in a flag, not the old personality slot.
    // Clear it only in disposable RAM to establish the stated no-pending scenario.
    let pending_flag = rules
        .pending_flag
        .map(|flag| {
            let id = u16::try_from(u32(&rom.data, flag.operand)?)
                .map_err(|_| err("breeding_production_rule", "pending flag width"))?;
            ram.call(flag.clear, [u32::from(id), 0, 0, 0], [0; 2], 100_000)?;
            Ok::<_, crate::Error>(id)
        })
        .transpose()?;
    // Ordinary bag projection preserves slot order/duplicates and the native
    // security key. Descriptor pointers refer only to isolated fixture RAM.
    let bag = save.map(Save::bag).transpose()?.unwrap_or_default();
    let key = ram.r32(SB2 + rom.profile.save.key as u32) as u16;
    let mut cursor = 0x02010000u32;
    for pocket in rom.profile.save.pockets.iter().filter(|p| p.category != 0) {
        let descriptor = rules.bag_descriptors + u32::from(pocket.category - 1) * 8;
        ram.w32(descriptor, cursor);
        ram.w16(descriptor + 4, pocket.count as u16);
        let mut entries: Vec<_> = bag
            .iter()
            .filter(|e| e.pocket == pocket.id)
            .map(|e| (e.slot, e.item, e.quantity))
            .collect();
        if let (Some(item), Some(present)) = (modifier, request.production_item) {
            let category = rom.resource_item_pocket(item)?;
            if category == pocket.category {
                entries.retain(|e| e.1 != item);
                if present {
                    // This is a simulated query, not a capacity check or inventory edit.
                    let empty = (0..pocket.count)
                        .find(|slot| !entries.iter().any(|e| e.0 == *slot && e.1 != 0))
                        .ok_or_else(|| {
                            err(
                                "breeding_production_bag",
                                "no empty ordinary-bag slot for simulated modifier",
                            )
                        })?;
                    entries.retain(|e| e.0 != empty);
                    entries.push((empty, item, 1));
                }
            }
        }
        for slot in 0..pocket.count {
            let (_, item, quantity) = entries
                .iter()
                .find(|e| e.0 == slot)
                .copied()
                .unwrap_or((slot, 0, 0));
            let at = cursor + slot as u32 * 4;
            ram.w16(at, item);
            ram.w16(at + 2, quantity ^ if pocket.encrypted { key } else { 0 });
        }
        cursor += pocket.count as u32 * 4;
        if cursor > 0x02020000 {
            return Err(err("breeding_production_bag", "fixture capacity"));
        }
    }
    let present = match (modifier, rules.item_check) {
        (Some(item), Some(code)) => {
            Some(ram.call(code, [u32::from(item), 1, 0, 0], [0; 2], 100_000)? != 0)
        }
        (None, None) => None,
        _ => {
            return Err(err(
                "breeding_production_rule",
                "missing native modifier predicate",
            ))
        }
    };
    let daycare = SB1 + breeding.daycare as u32;
    for i in 0..breeding.parent_stride * 2 + 8 {
        ram.w8(daycare + i as u32, 0);
    }
    for (slot, parent) in parents.iter().enumerate() {
        for (i, byte) in parent.iter().enumerate() {
            ram.w8(daycare + (slot * breeding.parent_stride + i) as u32, *byte);
        }
        // The complete step increments these to 255, triggering an ordinary
        // production check with no pending egg. This is an explicit scenario.
        ram.w32(daycare + (slot * breeding.parent_stride + 0x88) as u32, 254);
    }
    ram.w8(breeding.party_count, 0);
    ram.w32(breeding.rng, request.seed);
    let regs = ram.observe(
        rules.step,
        [daycare, 0, 0, 0],
        [0; 2],
        1_000_000,
        rules.comparison,
    )?;
    let threshold = regs[4];
    let numerator = (0..=u16::MAX)
        .filter(|n| u32::from(*n) * scale / divisor < threshold)
        .count() as u32;
    Ok(Some(Preview {
        scenario: "ordinary_no_pending_egg_check",
        bag_context: if request.production_item.is_some() {
            "simulated_item_override"
        } else if save.is_some() {
            "ordinary_save_projection"
        } else {
            "empty_bag_simulation"
        },
        modifier_item: modifier,
        modifier_present: present,
        interval_steps: (u16(&rom.data, rules.step as usize - 0x08000000 + 0x46)? & 255) + 1,
        threshold,
        numerator,
        denominator: 65536,
        percent: f64::from(numerator) * 100.0 / 65536.0,
        scenario_roll: regs[0],
        scenario_passed: threshold > regs[0],
        partial: true,
        simulated_pending_flag: pending_flag,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_production_is_not_replaced_by_a_default_or_silent_item_override() {
        let mut rom = crate::tests::query_fixture_rom();
        let before = rom.data.clone();
        assert_eq!(
            roll_parameters(&rom, EMERALD).unwrap_err().code,
            "breeding_production_rule"
        );
        rom.profile.breeding = Some(breeding::BreedingRules {
            production: None,
            ..breeding::EMERALD
        });
        let mut request: breeding::Request = serde_json::from_value(serde_json::json!({
            "parents":[{"kind":"simulated","species":1,"gender":"female"},
                {"kind":"simulated","species":1,"gender":"male"}],
            "offspring_pid":24,"production_item":true}))
        .unwrap();
        assert_eq!(
            preview(&rom, None, &request, &[vec![], vec![]])
                .unwrap_err()
                .code,
            "breeding_production_item"
        );
        request.production_item = None;
        assert!(preview(&rom, None, &request, &[vec![], vec![]])
            .unwrap()
            .is_none());
        assert_eq!(*rom.data, *before);
    }
}
