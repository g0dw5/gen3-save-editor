//! Bounded effects of native field presentation on script dataflow.
//! Standard-script tables and bodies are read from the current ROM. Unsupported
//! instructions keep conservative invalidation; a readable prompt is not access.
use crate::{binary::*, rom::Rom};

const OPCODES: [u8; 8] = [0x08, 0x09, 0x03, 0x66, 0x67, 0x6e, 0x28, 0x68];

#[derive(Clone, Copy)]
pub(crate) struct Presentation {
    /// The native menu writes VAR_RESULT, but the player's choice is unknown.
    pub choice: bool,
    /// A valid standard script returns; an out-of-range index does nothing.
    pub returns: bool,
}

impl Rom {
    fn presentation_dispatch(&self, opcode: u8) -> Option<usize> {
        let rules = self.profile.event_state?.effects?;
        let index = OPCODES.iter().position(|op| *op == opcode)?;
        let handler = rules.presentation_handlers[index];
        (handler != 0
            && pointer(&self.data, rules.commands + opcode as usize * 4).ok()? & !1 == handler)
            .then_some(handler)
    }

    pub(crate) fn script_presentation(&self, pc: usize) -> Option<Presentation> {
        let opcode = *self.data.get(pc)?;
        let handler = self.presentation_dispatch(opcode)?;
        if matches!(opcode, 0x28 | 0x66 | 0x67 | 0x68 | 0x6e) {
            return Some(Presentation {
                choice: opcode == 0x6e,
                returns: false,
            });
        }
        if !matches!(opcode, 0x08 | 0x09) {
            return None;
        }
        // Both verified native dispatchers load their table bounds from these
        // literals. Do not assume an index (e.g. 5) always means a yes/no prompt.
        let first = pointer(&self.data, handler + 0x28).ok()?;
        let index = *self.data.get(pc + 1)? as usize;
        let hook = self.profile.event_state?.effects?.standard_call_hook;
        let length = if let Some(hook) = hook.filter(|_| opcode == 0x09) {
            if u16(&self.data, handler + 0x12).ok()? != 0x4b06
                || u16(&self.data, handler + 0x14).ok()? != 0x4718
                || pointer(&self.data, handler + 0x2c).ok()? & !1 != hook
            {
                return None;
            }
            let limit = u16(&self.data, hook + 6).ok()?;
            if limit & 0xff00 != 0x2800 {
                return None;
            }
            // This gate prepares messages for selected indices before entering
            // their standard bodies. Those extra effects are not summarized.
            for offset in [0x0a, 0x0e, 0x12, 0x16] {
                let compare = u16(&self.data, hook + offset).ok()?;
                if compare & 0xff00 != 0x2800 || index == (compare & 255) as usize {
                    return None;
                }
            }
            ((limit & 255) as usize + 1) * 4
        } else {
            pointer(&self.data, handler + 0x2c)
                .ok()?
                .checked_sub(first)?
        };
        if first % 4 != 0 || length % 4 != 0 || length > 256 * 4 {
            return None;
        }
        bytes(&self.data, first, length).ok()?;
        if index >= length / 4 {
            return Some(Presentation {
                choice: false,
                returns: false,
            });
        }
        let mut cursor = pointer(&self.data, first + index * 4).ok()?;
        let mut choice = false;
        // Only a short, linear verified presentation body is summarized. Native
        // calls, writes, branches, end commands and cycles are not skipped.
        for _ in 0..32 {
            let op = *self.data.get(cursor)?;
            self.presentation_dispatch(op)?;
            let width = match op {
                0x03 => {
                    return Some(Presentation {
                        choice,
                        returns: true,
                    });
                }
                0x67 => 5,
                0x66 | 0x68 => 1,
                0x28 => 3,
                0x6e => {
                    choice = true;
                    3
                }
                _ => return None,
            };
            bytes(&self.data, cursor, width).ok()?;
            cursor += width;
        }
        None
    }
}
