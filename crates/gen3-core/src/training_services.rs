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
    pub menu_ignore_b_mask: u8,
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
    menu_ignore_b_mask: 0xff,
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
/// NPC services can change base IVs or mark effective training. They must not
/// share field semantics merely because the consumables have similar names.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(untagged)]
pub enum ServiceRules {
    Flags(CrownRules),
    BaseIvs(IvCrownRules),
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct IvCrownRules {
    pub root: usize,
    pub entry: usize,
    pub level_checks: [usize; 2],
    pub item_checks: [usize; 2],
    pub payments: [usize; 2],
    pub names: [usize; 2],
    pub stat_names: usize,
    pub stat_dispatch: usize,
    pub gold_stats: usize,
    pub messages: &'static [usize],
    pub level_reader: u32,
    pub setter: u32,
    pub selected_individual: u32,
    pub menu_commands: [usize; 2],
    pub menu_ignore_b_mask: u8,
}
pub const MERCURY_CROWNS: IvCrownRules = IvCrownRules {
    root: 0x7b05ba,
    entry: 0x7b000a,
    level_checks: [0x7b07b0, 0x7b099b],
    item_checks: [0x7b0097, 0x7b00c0],
    payments: [0x7b0116, 0x7b019c],
    names: [0x7b03d8, 0x7b03e3],
    stat_names: 0x7b01a6,
    stat_dispatch: 0x7b0229,
    gold_stats: 0x7b0123,
    messages: &[0x7b00a7, 0x7b078e],
    level_reader: 0x0896a730,
    setter: 0x09d5c1d8,
    selected_individual: 0x020370c0,
    menu_commands: [0x7b003d, 0x7b01fb],
    menu_ignore_b_mask: 1,
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
    pub credit: Option<ConditionCheck>,
    pub item_requirement: ConditionCheck,
    pub payment: ServicePayment,
}
#[derive(Serialize)]
pub struct ServicePayment {
    pub item: u16,
    pub quantity: u16,
    /// Some ROM scripts attempt payment before a stat menu and
    /// continue even when removal fails. Preserve this distinction as evidence.
    pub before_stat_selection: bool,
    pub result_checked: bool,
}
#[derive(Serialize)]
pub struct ServiceMenu {
    pub stage: &'static str,
    pub single_stat_only: bool,
    pub cancel_with_b: bool,
    pub payment_precedes_menu: bool,
    pub command: usize,
}
#[derive(Serialize)]
pub struct CrownService {
    pub kind: &'static str,
    pub minimum_level: u16,
    pub choices: Vec<CrownChoice>,
    pub locations: Vec<ServiceLocation>,
    pub text: Vec<String>,
    pub conditions: Vec<ConditionCheck>,
    pub menus: Vec<ServiceMenu>,
    pub evidence: ServiceRules,
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
    // Opcode 0x6f's fourth argument reaches the exact engine's ignore-B flag.
    // Mercury masks bit 0 (bit 1 is presentation); Ultimate retains the byte.
    fn service_menu(
        &self,
        command: usize,
        stage: &'static str,
        single_stat_only: bool,
        payment_precedes_menu: bool,
        ignore_b_mask: u8,
    ) -> Result<ServiceMenu> {
        instruction(self, command, 0x6f)?;
        Ok(ServiceMenu {
            stage,
            single_stat_only,
            cancel_with_b: bytes(&self.data, command + 4, 1)?[0] & ignore_b_mask == 0,
            payment_precedes_menu,
            command,
        })
    }
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
        let rule = match rule {
            ServiceRules::Flags(rule) => rule,
            ServiceRules::BaseIvs(rule) => {
                if let Some(service) = self.iv_crown_service(rule, save)? {
                    report.services.push(service);
                }
                return Ok(report);
            }
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
                credit: Some(check(
                    state.as_ref(),
                    Some(self),
                    &EventCondition {
                        kind: "variable",
                        id: credit,
                        value: 0,
                        comparison: 5,
                        taken: true,
                    },
                )),
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
                payment: ServicePayment {
                    item,
                    quantity,
                    before_stat_selection: false,
                    result_checked: false,
                },
            });
        }
        let locations = self.service_locations(rule.root, state.as_ref())?;
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
                conditions: vec![],
                menus: vec![self.service_menu(
                    rule.menu_command,
                    "training_choice",
                    false,
                    false,
                    rule.menu_ignore_b_mask,
                )?],
                evidence: ServiceRules::Flags(rule),
                partial: true,
            });
        }
        Ok(report)
    }
    fn service_locations(
        &self,
        root: usize,
        state: Option<&EventSnapshot>,
    ) -> Result<Vec<ServiceLocation>> {
        let mut locations = vec![];
        for map in self.maps()? {
            if !map.objects.iter().any(|o| o.script == Some(root)) {
                continue;
            }
            for marker in self.map_events(&map)?.markers {
                if marker.script != Some(root) {
                    continue;
                }
                let visibility = marker
                    .flag
                    .filter(|id| *id != 0)
                    .map(|id| {
                        check(
                            state,
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
        Ok(locations)
    }

    fn iv_crown_service(
        &self,
        rule: IvCrownRules,
        save: Option<&Save>,
    ) -> Result<Option<CrownService>> {
        let state = save
            .zip(self.profile.event_state)
            .map(|(s, layout)| EventSnapshot::new(s, layout));
        let locations = self.service_locations(rule.root, state.as_ref())?;
        if locations.is_empty() {
            return Ok(None);
        }
        instruction(self, rule.root, 0x2b)?;
        instruction(self, rule.root + 3, 0x06)?;
        if bytes(&self.data, rule.root + 4, 1)?[0] != 1
            || pointer(&self.data, rule.root + 5)? != rule.entry
        {
            return Err(err("training_service_unlock", rule.root));
        }
        let conditions = vec![check(
            state.as_ref(),
            Some(self),
            &EventCondition {
                kind: "flag",
                id: u16(&self.data, rule.root + 1)?,
                value: 1,
                comparison: 1,
                taken: true,
            },
        )];
        let mut minimum_level = None;
        for at in rule.level_checks {
            instruction(self, at, 0x21)?;
            instruction(self, at + 5, 0x06)?;
            if u16(&self.data, at + 1)? != 0x8006 || bytes(&self.data, at + 6, 1)?[0] != 0 {
                return Err(err("training_service_level", at));
            }
            let level = u16(&self.data, at + 3)?;
            if minimum_level.is_some_and(|n| n != level) {
                return Err(err("training_service_level", at));
            }
            minimum_level = Some(level);
        }
        let mut choices = vec![];
        let names = [
            self.text(rule.names[0], 256)?,
            self.text(rule.names[1], 256)?,
        ];
        for menu_index in 0..7u8 {
            let branch = usize::from(menu_index != 0);
            let check_at = rule.item_checks[branch];
            let pay_at = rule.payments[branch];
            instruction(self, check_at, 0x47)?;
            instruction(self, pay_at, 0x45)?;
            let item = u16(&self.data, check_at + 1)?;
            let quantity = u16(&self.data, check_at + 3)?;
            let payment_item = u16(&self.data, pay_at + 1)?;
            let payment_quantity = u16(&self.data, pay_at + 3)?;
            if item == 0
                || payment_item == 0
                || item as usize >= self.profile.items.count
                || payment_item as usize >= self.profile.items.count
                || quantity != 1
                || payment_quantity != 1
            {
                return Err(err("training_service_payment", pay_at));
            }
            // Silver attempts payment before setting up the single-stat menu.
            // No comparison/branch inspects the remove command's result here.
            if branch == 1 {
                instruction(self, pay_at + 5, 0x16)?;
                if u16(&self.data, pay_at + 6)? != 0x8006 || u16(&self.data, pay_at + 8)? != 0 {
                    return Err(err("training_service_payment_order", pay_at));
                }
            }
            let stat = (menu_index != 0).then(|| menu_index as usize - 1);
            let name = if let Some(stat) = stat {
                let label_at = rule.stat_names + stat * 14;
                instruction(self, label_at, 0x0f)?;
                instruction(self, label_at + 6, 0x25)?;
                if u16(&self.data, label_at + 7)? != 0x25 {
                    return Err(err("training_service_menu", label_at));
                }
                let dispatch = rule.stat_dispatch + stat * 11;
                instruction(self, dispatch, 0x21)?;
                instruction(self, dispatch + 5, 0x06)?;
                if u16(&self.data, dispatch + 1)? != 0x800d
                    || u16(&self.data, dispatch + 3)? != stat as u16
                    || bytes(&self.data, dispatch + 6, 1)?[0] != 1
                {
                    return Err(err("training_service_selection", dispatch));
                }
                let selected = pointer(&self.data, dispatch + 7)?;
                self.verify_iv_service_stat(selected + 10, stat)?;
                format!(
                    "{} · {}",
                    names[1],
                    self.text(pointer(&self.data, label_at + 2)?, 256)?
                )
            } else {
                for stat in 0..6 {
                    self.verify_iv_service_stat(rule.gold_stats + stat * 13, stat)?;
                }
                names[0].clone()
            };
            choices.push(CrownChoice {
                menu_index,
                name,
                item,
                quantity,
                stat,
                mask: 0,
                credit: None,
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
                payment: ServicePayment {
                    item: payment_item,
                    quantity: payment_quantity,
                    before_stat_selection: branch == 1,
                    result_checked: false,
                },
            });
        }
        let text = rule
            .messages
            .iter()
            .map(|at| {
                instruction(self, *at, 0x0f)?;
                self.text(pointer(&self.data, at + 2)?, 1024)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Some(CrownService {
            kind: "base_iv_training",
            minimum_level: minimum_level.unwrap(),
            choices,
            locations,
            conditions,
            text,
            menus: vec![
                self.service_menu(
                    rule.menu_commands[0],
                    "training_choice",
                    false,
                    false,
                    rule.menu_ignore_b_mask,
                )?,
                self.service_menu(
                    rule.menu_commands[1],
                    "stat_choice",
                    true,
                    true,
                    rule.menu_ignore_b_mask,
                )?,
            ],
            evidence: ServiceRules::BaseIvs(rule),
            partial: true,
        }))
    }
    fn verify_iv_service_stat(&self, at: usize, stat: usize) -> Result<()> {
        for (offset, op) in [(0, 0x16), (5, 0x16), (10, 0x25)] {
            instruction(self, at + offset, op)?;
        }
        if u16(&self.data, at + 1)? != 0x8005
            || u16(&self.data, at + 3)? != stat as u16
            || u16(&self.data, at + 6)? != 0x8006
            || u16(&self.data, at + 8)? != 31
            || u16(&self.data, at + 11)? != 0x10
        {
            return Err(err("training_service_stat", at));
        }
        Ok(())
    }
}
