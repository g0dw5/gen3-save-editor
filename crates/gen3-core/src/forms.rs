//! Battle transformations are derived references, never permanent evolution edges.
use crate::{
    binary::{pointer, u16, u32},
    err,
    rom::Rom,
    Result,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleFormRules {
    ExpansionEvolutionMethods,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BattleFormKind {
    Mega,
    Primal,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum BattleTrigger {
    HeldItem(u16),
    KnownMove(u16),
}
#[derive(Debug, Serialize)]
pub struct BattleForm {
    pub source: u16,
    pub target: u16,
    pub kind: BattleFormKind,
    pub trigger: BattleTrigger,
    pub offset: usize,
}
pub(crate) fn is_battle_method(rules: Option<BattleFormRules>, method: u16) -> bool {
    rules.is_some() && matches!(method, 0xfffd..=0xffff)
}
impl Rom {
    /// Persistent item/move transitions. Battle-, time- and item-use triggers
    /// are deliberately not run by unrelated save edits.
    pub fn storage_form(
        &self,
        species: u16,
        item: u16,
        moves: &[u16; 4],
        trigger: u16,
    ) -> Result<u16> {
        let Some(table) = self.profile.storage_forms else {
            return Ok(species);
        };
        self.valid_species(species)?;
        let entry = table + species as usize * 4;
        if u32(&self.data, entry)? == 0 {
            return Ok(species);
        }
        let list = pointer(&self.data, entry)?;
        let mut result = species;
        for index in 0..128 {
            let offset = list + index * 8;
            let method = u16(&self.data, offset)?;
            if method == 0 {
                return Ok(result);
            }
            let parameter = u16(&self.data, offset + 4)?;
            let matches = method == trigger
                && match method {
                    1 => item == parameter,
                    3 => (moves.contains(&parameter) as u16) != u16(&self.data, offset + 6)?,
                    _ => false,
                };
            if matches {
                result = u16(&self.data, offset + 2)?;
                self.valid_species(result)?;
            }
        }
        Err(err("form_terminator", species))
    }
    pub fn is_battle_species(&self, species: u16) -> Result<bool> {
        Ok(self
            .battle_forms(species)?
            .iter()
            .any(|form| form.target == species))
    }
    /// Associations only. Battle mode, unlocks and usage flags are not simulated.
    /// Incoming edges let a temporary species point back to its storage species.
    pub fn battle_forms(&self, species: u16) -> Result<Vec<BattleForm>> {
        self.species(species)?;
        Ok(self
            .all_battle_forms()?
            .into_iter()
            .filter(|form| form.source == species || form.target == species)
            .collect())
    }
    pub(crate) fn all_battle_forms(&self) -> Result<Vec<BattleForm>> {
        let Some(BattleFormRules::ExpansionEvolutionMethods) = self.profile.battle_forms else {
            return Ok(Vec::new());
        };
        let table = self.profile.evolutions;
        let mut out = Vec::new();
        for source in 1..table.count as u16 {
            for row in 0..table.stride / 8 {
                let offset = table.offset + source as usize * table.stride + row * 8;
                let method = u16(&self.data, offset)?;
                if !is_battle_method(self.profile.battle_forms, method) {
                    continue;
                }
                let target = u16(&self.data, offset + 4)?;
                self.valid_species(target)?;
                let parameter = u16(&self.data, offset + 2)?;
                let trigger = if method == 0xfffe {
                    self.move_info(parameter)?;
                    BattleTrigger::KnownMove(parameter)
                } else {
                    self.item(parameter)?;
                    BattleTrigger::HeldItem(parameter)
                };
                out.push(BattleForm {
                    source,
                    target,
                    kind: if method == 0xfffd {
                        BattleFormKind::Primal
                    } else {
                        BattleFormKind::Mega
                    },
                    trigger,
                    offset,
                });
            }
        }
        Ok(out)
    }
}
