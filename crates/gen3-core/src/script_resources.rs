//! Native script holdings checks. Checking a resource does not spend it.
use crate::{binary::*, err, map_events::EventCondition, rom::Rom, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub enum BagCountRule {
    Accumulate,
    FirstMatch,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ResourceCheckRules {
    pub commands: usize,
    pub item_code: usize,
    pub item_sanitizer: u32,
    pub money_code: usize,
    pub quantity_u8: bool,
    pub bag_count: BagCountRule,
    /// An unsaved map/temporary-flag context can select a separate facility bag.
    pub alternate_bag: bool,
}
pub const EMERALD: ResourceCheckRules = ResourceCheckRules {
    commands: 0x1db67c,
    item_code: 0x99a6c,
    item_sanitizer: 0x080d745c,
    money_code: 0x9b4c0,
    quantity_u8: true,
    bag_count: BagCountRule::Accumulate,
    alternate_bag: true,
};
pub const ROCKET: ResourceCheckRules = ResourceCheckRules {
    commands: 0x22b218,
    item_code: 0xcf95c,
    item_sanitizer: 0x0810f820,
    money_code: 0xd1420,
    ..EMERALD
};
pub const MERCURY: ResourceCheckRules = ResourceCheckRules {
    commands: 0x15f9b4,
    item_code: 0x6a6e4,
    item_sanitizer: 0x0809a8a4,
    money_code: 0x6c18c,
    quantity_u8: false,
    bag_count: BagCountRule::FirstMatch,
    alternate_bag: false,
};
impl Rom {
    /// Native category lookup sanitizes the ID before indexing the item table.
    /// In particular, Ultimate rejects some empty extended slots, including 377.
    /// The bag search still matches the original requested ID, not this index.
    pub(crate) fn resource_item_pocket(&self, item: u16) -> Result<u8> {
        let rules = self
            .profile
            .resource_checks
            .ok_or_else(|| err("resource_unverified", item))?;
        let mut sandbox = crate::native_trainer::Sandbox::new(&self.data);
        let index = sandbox
            .call(rules.item_sanitizer, [item as u32, 0, 0, 0], [0; 2], 4096)
            .map_err(|e| err("resource_native", e))? as u16 as usize;
        if index >= self.profile.items.count {
            return Err(err("resource_item", index));
        }
        Ok(bytes(
            &self.data,
            self.profile.items.offset + index * self.profile.items.stride + 26,
            1,
        )?[0])
    }
    /// Produces the predicate stored in VAR_RESULT by verified native handlers.
    /// Dynamic operands and changed dispatch targets remain unqualified.
    pub(crate) fn script_resource_check(
        &self,
        pc: usize,
        resolve: impl Fn(u16) -> Option<u16>,
        bag_changed: bool,
        money_changed: bool,
    ) -> Result<Option<EventCondition>> {
        let Some(rules) = self.profile.resource_checks else {
            return Ok(None);
        };
        let op = bytes(&self.data, pc, 1)?[0];
        let code = match op {
            0x47 => rules.item_code,
            0x92 => rules.money_code,
            _ => return Ok(None),
        };
        if pointer(&self.data, rules.commands + op as usize * 4)? & !1 != code {
            return Err(err("resource_dispatch", "native resource target changed"));
        }
        let (kind, id, value) = if op == 0x47 {
            let item =
                resolve(u16(&self.data, pc + 1)?).ok_or_else(|| err("resource_dynamic", pc))?;
            let quantity =
                resolve(u16(&self.data, pc + 3)?).ok_or_else(|| err("resource_dynamic", pc))?;
            self.item(item)?;
            (
                if bag_changed {
                    "bag_item_runtime"
                } else {
                    "bag_item"
                },
                item,
                if rules.quantity_u8 {
                    quantity as u8 as u32
                } else {
                    quantity as u32
                },
            )
        } else {
            if bytes(&self.data, pc, 6)?[5] != 0 {
                // Native ignore=1 leaves VAR_RESULT unchanged, even if stale.
                return Ok(None);
            }
            (
                if money_changed {
                    "money_runtime"
                } else {
                    "money"
                },
                0,
                u32(&self.data, pc + 1)?,
            )
        };
        Ok(Some(EventCondition {
            kind,
            id,
            value,
            comparison: 4,
            taken: true,
        }))
    }
}
