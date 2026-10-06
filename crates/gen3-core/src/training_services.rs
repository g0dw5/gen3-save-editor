//! Runtime NPC training offers. An item name or an unused record is not a service.
//! Exact adapters describe verified script layouts; names, fees, choices, text,
//! requirements and NPC positions are read from the current ROM.
use crate::{
    acquisition::{check, ConditionCheck},
    binary::*,
    err,
    event_state::EventSnapshot,
    map_events::EventCondition,
    native_trainer::Sandbox,
    rom::Rom,
    save::Save,
    Result,
};
use armv4t_emu::Memory;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct CrownBranch {
    pub credit_check: usize,
    pub item_check: usize,
    pub credit_payment: usize,
    pub item_payment: usize,
    pub mask_command: usize,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct CrownRules {
    pub root: usize,
    pub level_check: usize,
    pub menu_command: usize,
    pub menu_table: usize,
    pub gold: CrownBranch,
    pub silver: CrownBranch,
    pub messages: &'static [usize],
    pub level_reader: u32,
    pub mark: u32,
    pub selection_mask: u32,
    pub selected_individual: u32,
    pub selected_choice: u32,
}
pub const ULTIMATE_CROWNS: CrownRules = CrownRules {
    root: 0x1812758,
    level_check: 0x181278f,
    menu_command: 0x181279a,
    menu_table: 0x1700000,
    gold: CrownBranch {
        credit_check: 0x18127bb,
        item_check: 0x18127c6,
        credit_payment: 0x18127eb,
        item_payment: 0x18127f0,
        mask_command: 0x18127d6,
    },
    silver: CrownBranch {
        credit_check: 0x1812805,
        item_check: 0x1812810,
        credit_payment: 0x1812830,
        item_payment: 0x1812835,
        mask_command: 0x18127fb,
    },
    messages: &[0x181275a, 0x181285a, 0x1812876, 0x181284e],
    level_reader: 0x098126d8,
    mark: 0x098126fc,
    selection_mask: 0x09812720,
    selected_individual: 0x020375e0,
    selected_choice: 0x020375f0,
};
#[derive(Serialize)]
pub struct ServiceLocation {
    pub map_id: String,
    pub map_name: String,
    pub x: i16,
    pub y: i16,
    pub local_id: Option<u8>,
    pub visibility: Vec<ConditionCheck>,
}
#[derive(Serialize)]
pub struct CrownChoice {
    pub menu_index: u8,
    pub name: String,
    pub item: u16,
    pub quantity: u16,
    pub stat: Option<usize>,
    pub mask: u8,
    pub credit: ConditionCheck,
    pub item_requirement: ConditionCheck,
}
#[derive(Serialize)]
pub struct CrownService {
    pub kind: &'static str,
    pub minimum_level: u16,
    pub choices: Vec<CrownChoice>,
    pub locations: Vec<ServiceLocation>,
    pub text: Vec<String>,
    pub evidence: CrownRules,
    pub partial: bool,
}
#[derive(Serialize)]
pub struct Report {
    pub rom_md5: &'static str,
    pub services: Vec<CrownService>,
    /// No verified service adapter is not evidence of nonexistence.
    pub partial: bool,
}
fn instruction(rom: &Rom, at: usize, op: u8) -> Result<()> {
    if bytes(&rom.data, at, 1)?[0] != op {
        return Err(err("training_service_script", at));
    }
    Ok(())
}
fn branch(rom: &Rom, rule: CrownBranch) -> Result<(u16, u16, u16, u8)> {
    for (at, op) in [
        (rule.credit_check, 0x21),
        (rule.item_check, 0x47),
        (rule.credit_payment, 0x18),
        (rule.item_payment, 0x45),
        (rule.mask_command, 0x16),
    ] {
        instruction(rom, at, op)?;
    }
    let credit = u16(&rom.data, rule.credit_check + 1)?;
    let item = u16(&rom.data, rule.item_check + 1)?;
    let quantity = u16(&rom.data, rule.item_check + 3)?;
    if u16(&rom.data, rule.credit_check + 3)? != 0
        || u16(&rom.data, rule.credit_payment + 1)? != credit
        || u16(&rom.data, rule.credit_payment + 3)? != 1
        || u16(&rom.data, rule.item_payment + 1)? != item
        || u16(&rom.data, rule.item_payment + 3)? != quantity
        || u16(&rom.data, rule.mask_command + 1)? != 0x8005
        || item == 0
        || item >= rom.profile.items.count as u16
        || quantity == 0
    {
        return Err(err("training_service_payment", rule.item_check));
    }
    let mask = u8::try_from(u16(&rom.data, rule.mask_command + 3)?)
        .map_err(|_| err("training_service_mask", rule.mask_command))?;
    Ok((credit, item, quantity, mask))
}
impl Rom {
    pub fn training_services(&self, save: Option<&Save>) -> Result<Report> {
        if let Some(save) = save {
            if save.layout.pokemon_codec != self.profile.save.pokemon_codec
                || save.layout.sizes != self.profile.save.sizes
            {
                return Err(err("training_save_layout", "SAVE and ROM layouts differ"));
            }
        }
        let mut report = Report {
            rom_md5: self.profile.md5,
            services: vec![],
            partial: true,
        };
        let Some(rule) = self.profile.training.and_then(|r| r.crown_service) else {
            return Ok(report);
        };
        instruction(self, rule.level_check, 0x21)?;
        instruction(self, rule.menu_command, 0x6f)?;
        if u16(&self.data, rule.level_check + 1)? != 0x8005 {
            return Err(err("training_service_level", rule.level_check));
        }
        let minimum_level = u16(&self.data, rule.level_check + 3)?;
        let menu = bytes(&self.data, rule.menu_command + 3, 1)?[0] as usize;
        let menu_at = rule.menu_table + menu * 8;
        let choices_at = pointer(&self.data, menu_at)?;
        let count = bytes(&self.data, menu_at + 4, 1)?[0];
        if count != 7 {
            return Err(err("training_service_menu", count));
        }
        let gold = branch(self, rule.gold)?;
        let silver = branch(self, rule.silver)?;
        if gold.3 != 0x7e || silver.3 != 1 {
            return Err(err("training_service_mask", "unverified stat selection"));
        }
        let state = save
            .zip(self.profile.event_state)
            .map(|(s, layout)| EventSnapshot::new(s, layout));
        let mut choices = vec![];
        let mut ram = Sandbox::new(&self.data);
        for menu_index in 0..count {
            let (credit, item, quantity, _) = if menu_index == 0 { gold } else { silver };
            let mask = if menu_index == 0 {
                gold.3
            } else {
                ram.w16(rule.selected_individual + 2, silver.3 as u16);
                ram.w8(rule.selected_choice, menu_index);
                ram.call(rule.selection_mask, [0; 4], [0; 2], 8192)?;
                u8::try_from(ram.r16(rule.selected_individual + 2))
                    .map_err(|_| err("training_service_mask", menu_index))?
            };
            if mask
                != (if menu_index == 0 {
                    0x7e
                } else {
                    2 << (menu_index - 1)
                })
            {
                return Err(err("training_service_mask", "native selection differs"));
            }
            choices.push(CrownChoice {
                menu_index,
                name: self.text(
                    pointer(&self.data, choices_at + menu_index as usize * 8)?,
                    256,
                )?,
                item,
                quantity,
                stat: (menu_index != 0).then(|| menu_index as usize - 1),
                mask,
                credit: check(
                    state.as_ref(),
                    Some(self),
                    &EventCondition {
                        kind: "variable",
                        id: credit,
                        value: 0,
                        comparison: 5,
                        taken: true,
                    },
                ),
                item_requirement: check(
                    state.as_ref(),
                    Some(self),
                    &EventCondition {
                        kind: "bag_item",
                        id: item,
                        value: quantity as u32,
                        comparison: 4,
                        taken: true,
                    },
                ),
            });
        }
        let mut locations = vec![];
        for map in self.maps()? {
            if !map.objects.iter().any(|o| o.script == Some(rule.root)) {
                continue;
            }
            for marker in self.map_events(&map)?.markers {
                if marker.script != Some(rule.root) {
                    continue;
                }
                let visibility = marker
                    .flag
                    .filter(|id| *id != 0)
                    .map(|id| {
                        check(
                            state.as_ref(),
                            Some(self),
                            &EventCondition {
                                kind: "flag",
                                id,
                                value: 0,
                                comparison: 1,
                                taken: true,
                            },
                        )
                    })
                    .into_iter()
                    .collect();
                locations.push(ServiceLocation {
                    map_id: map.id.clone(),
                    map_name: map.name.clone(),
                    x: marker.x,
                    y: marker.y,
                    local_id: marker.local_id,
                    visibility,
                });
            }
        }
        // Only offers referenced by a real map object are presented as services.
        if !locations.is_empty() {
            let text = rule
                .messages
                .iter()
                .map(|at| {
                    instruction(self, *at, 0x0f)?;
                    self.text(pointer(&self.data, at + 2)?, 1024)
                })
                .collect::<Result<Vec<_>>>()?;
            report.services.push(CrownService {
                kind: "hyper_training_flags",
                minimum_level,
                choices,
                locations,
                text,
                evidence: rule,
                partial: true,
            });
        }
        Ok(report)
    }
}
