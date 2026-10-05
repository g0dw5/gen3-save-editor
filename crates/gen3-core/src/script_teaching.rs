//! Positioned teaching offers from verified script/native dispatch paths.
//! An offer is not proof of eligibility, payment, completion or current access.
use crate::{binary::*, err, native_trainer::Sandbox, rom::Rom, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct TutorScriptRules {
    pub specials: usize,
    pub special: u16,
    pub code: usize,
    pub variable: u16,
    pub parameter: TutorParameter,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub enum TutorParameter {
    MoveId,
    /// The native getter includes this exact ROM's redirects and special cases.
    Index {
        count: u16,
        getter: u32,
    },
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TeachingSource {
    pub move_id: u16,
    pub parameter: u16,
    pub offset: usize,
    pub conditions: Vec<crate::map_events::EventCondition>,
}
impl Rom {
    pub(crate) fn tutor_move(&self, parameter: u16) -> Result<u16> {
        let rules = self
            .profile
            .tutor_scripts
            .ok_or_else(|| err("tutor_unverified", parameter))?;
        let move_id = match rules.parameter {
            TutorParameter::MoveId => parameter,
            TutorParameter::Index { count, getter, .. } => {
                // Indexed engines read the selector as u8 when teaching the move.
                let index = parameter as u8 as u16;
                if index >= count {
                    return Err(err("tutor_index", index));
                }
                let mut sandbox = Sandbox::new(&self.data);
                u16::try_from(
                    sandbox
                        .call(getter, [index as u32, 0, 0, 0], [0; 2], 4096)
                        .map_err(|e| err("tutor_native", e))?,
                )
                .map_err(|_| err("tutor_move", index))?
            }
        };
        if move_id == 0 {
            return Err(err("tutor_move", move_id));
        }
        self.move_info(move_id)?;
        Ok(move_id)
    }
    pub(crate) fn script_teaching_instruction(
        &self,
        pc: usize,
        resolve: impl Fn(u16) -> Option<u16>,
    ) -> Result<Option<TeachingSource>> {
        let Some(rules) = self.profile.tutor_scripts else {
            return Ok(None);
        };
        let op = *bytes(&self.data, pc, 1)?.first().unwrap();
        if !matches!(op, 0x25 | 0x26) {
            return Ok(None);
        }
        let special = u16(&self.data, pc + if op == 0x26 { 3 } else { 1 })?;
        if special != rules.special {
            return Ok(None);
        }
        if pointer(&self.data, rules.specials + special as usize * 4)? & !1 != rules.code {
            return Err(err("tutor_dispatch", "native teaching target changed"));
        }
        let Some(parameter) = resolve(rules.variable) else {
            return Ok(None);
        };
        Ok(Some(TeachingSource {
            move_id: self.tutor_move(parameter)?,
            parameter,
            offset: pc,
            conditions: vec![],
        }))
    }
}
