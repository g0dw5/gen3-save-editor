//! Verified read-only player conditions, decoded from the current native handler.
use crate::{binary::*, rom::Rom};

impl Rom {
    pub(crate) fn player_gender_offset(&self) -> Option<usize> {
        let rules = self.profile.event_state?.effects?;
        let (handler, save_pointer, result_pointer) = rules.player_gender;
        if pointer(&self.data, rules.commands + 0xa0 * 4).ok()? & !1 != handler {
            return None;
        }
        // Complete verified Thumb body: load SaveBlock2, read its gender byte,
        // write VAR_RESULT, return zero. A changed dispatcher/body stays unknown.
        let instructions = [0x4903, 0x4804, 0x6800, 0x7a00, 0x8008, 0x2000, 0x4770];
        for (i, expected) in instructions.into_iter().enumerate() {
            if u16(&self.data, handler + i * 2).ok()? != expected {
                return None;
            }
        }
        if u32(&self.data, handler + 0x10).ok()? != result_pointer
            || u32(&self.data, handler + 0x14).ok()? != save_pointer
        {
            return None;
        }
        Some(((u16(&self.data, handler + 6).ok()? >> 6) & 31) as usize)
    }

    pub(crate) fn script_player_gender(&self, pc: usize) -> bool {
        self.data.get(pc) == Some(&0xa0) && self.player_gender_offset().is_some()
    }
}
