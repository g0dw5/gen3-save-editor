//! Bounded native text buffering preserves event parameters, not dialogue access.
use crate::{binary::*, rom::Rom};

const OPCODES: [u8; 5] = [0x7d, 0x80, 0x82, 0x83, 0x85];
const TABLE_LITERALS: [usize; 5] = [0x38, 0x30, 0x38, 0x40, 0x24];
// Deliberately bounded below the smallest verified text-buffer allocation.
// Long/unterminated strings and unsupported native formatting stay unknown.
const SUPPORTED_STRING_BYTES: usize = 14;

impl Rom {
    pub(crate) fn script_buffer_preserves(
        &self,
        pc: usize,
        resolve: impl Fn(u16) -> Option<u16>,
    ) -> bool {
        self.script_buffer_effect(pc, resolve).is_some()
    }

    fn script_buffer_effect(&self, pc: usize, resolve: impl Fn(u16) -> Option<u16>) -> Option<()> {
        let opcode = *self.data.get(pc)?;
        let index = OPCODES.iter().position(|op| *op == opcode)?;
        let rules = self.profile.event_state?.effects?;
        let handler = rules.buffer_handlers[index];
        if pointer(&self.data, rules.commands + opcode as usize * 4).ok()? & !1 != handler {
            return None;
        }
        if u16(&self.data, handler).ok()? != if opcode == 0x83 { 0xb530 } else { 0xb510 }
            || u16(&self.data, handler + 2).ok()? != 0x6881
        {
            return None;
        }
        let slot = *self.data.get(pc + 1)? as usize;
        if slot >= 3 {
            return None;
        }
        let table = pointer(&self.data, handler + TABLE_LITERALS[index]).ok()?;
        let destination = u32(&self.data, table + slot * 4).ok()?;
        if destination != rules.buffer_destinations[slot] {
            return None;
        }
        if !(0x02000000..=0x02040000 - SUPPORTED_STRING_BYTES as u32).contains(&destination) {
            return None;
        }
        let terminated = |offset: usize| {
            self.data
                .get(offset..offset + SUPPORTED_STRING_BYTES)
                .is_some_and(|b| b.contains(&0xff))
        };
        if opcode == 0x85 {
            return terminated(pointer(&self.data, pc + 2).ok()?).then_some(());
        }
        let operand = u16(&self.data, pc + 2).ok()?;
        if opcode == 0x83 {
            // Every valid VarGet result is a u16, at most five decimal digits.
            // Do not turn an invalid variable pointer into a literal.
            let layout = self.profile.event_state?;
            let valid = operand < 0x4000
                || (0x8000..0x8010).contains(&operand)
                || layout
                    .variables
                    .iter()
                    .any(|r| operand >= r.first && operand - r.first < r.count);
            return valid.then_some(());
        }
        let id = resolve(operand)? as usize;
        let source = match opcode {
            0x7d | 0x82 => {
                let limit = if opcode == 0x7d {
                    self.profile.species.count
                } else {
                    self.profile.moves.count
                };
                if id >= limit {
                    return None;
                }
                // Use the table/stride actually loaded by this native command.
                // In particular, do not silently substitute the editor's split
                // move-name table for an unpatched older script formatter.
                let instruction = u16(&self.data, handler + 0x22).ok()?;
                let stride = if opcode == 0x7d { 11 } else { 13 };
                if instruction != 0x2100 + stride {
                    return None;
                }
                pointer(&self.data, handler + 0x3c).ok()? + id * stride as usize
            }
            0x80 => {
                if id >= self.profile.items.count {
                    return None;
                }
                if let Some(compare) = rules.buffer_item_dynamic_compare {
                    let instruction = u16(&self.data, compare).ok()?;
                    if instruction & 0xff00 != 0x2800 || id == (instruction & 255) as usize {
                        // This item formats a name from runtime/custom berry
                        // state. A ROM item-name field cannot bound that copy.
                        return None;
                    }
                }
                self.profile.items.offset + id * self.profile.items.stride
            }
            _ => return None,
        };
        terminated(source).then_some(())
    }
}
