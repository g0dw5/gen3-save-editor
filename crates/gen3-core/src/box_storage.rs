//! Native compact storage records; separate from the party's individual codec.
use crate::{
    binary::{bytes, put16, u16, u32},
    err,
    rom::Rom,
    Result,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub enum Block {
    Trainer,
    Main,
    Storage,
    Extensions,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Region {
    pub first: usize,
    pub count: usize,
    pub block: Block,
    pub offset: usize,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Layout {
    pub regions: &'static [Region],
    pub record_size: usize,
    pub primary_names: usize,
    pub names: usize,
    pub wallpapers: usize,
    pub extra_wallpapers: usize,
    pub tera_types: usize,
    pub tera_type_count: usize,
}

// Native GetCompressedMonPtr 1D5A3E0, pointer table 1DE4A7C, count 25, stride 58.
// Box 20–22 are in parasite RAM; 23–24 occupy SB1, and 25 occupies SB2.
pub const MERCURY: Layout = Layout {
    regions: &[
        Region {
            first: 0,
            count: 19,
            block: Block::Storage,
            offset: 4,
        },
        Region {
            first: 19,
            count: 3,
            block: Block::Extensions,
            offset: 0x19d0,
        },
        Region {
            first: 22,
            count: 2,
            block: Block::Main,
            offset: 0x1f08,
        },
        Region {
            first: 24,
            count: 1,
            block: Block::Trainer,
            offset: 0xb0,
        },
    ],
    record_size: 58,
    primary_names: 14,
    names: 0x8344,
    wallpapers: 0x83c2,
    extra_wallpapers: 0x8128,
    tera_types: 0x1de6920,
    tera_type_count: 19,
};

impl Layout {
    pub fn address(self, box_index: usize, slot: usize, slots: usize) -> Result<(Block, usize)> {
        let r = self
            .regions
            .iter()
            .find(|r| (r.first..r.first + r.count).contains(&box_index))
            .ok_or_else(|| err("location", box_index))?;
        Ok((
            r.block,
            r.offset + ((box_index - r.first) * slots + slot) * self.record_size,
        ))
    }
    pub fn metadata(self, index: usize) -> (usize, usize) {
        if index < self.primary_names {
            (self.names + index * 9, self.wallpapers + index)
        } else {
            let extra = index - self.primary_names;
            (self.names - (extra + 1) * 9, self.extra_wallpapers + extra)
        }
    }
    fn valid_type(self, value: u8, rom: &Rom) -> Result<bool> {
        Ok(bytes(&rom.data, self.tera_types, self.tera_type_count)?.contains(&value))
    }
    fn tera_code(self, raw: &[u8], rom: &Rom) -> Result<u8> {
        let stored = u16(raw, 30)?;
        let code = stored as u8;
        if stored & 0xff00 == 0xa500
            && code != 0
            && (code == 31 || self.valid_type(code - 1, rom)?)
        {
            return Ok(code);
        }
        let s = rom.species(u16(raw, 32)?)?;
        let kind =
            if u32(raw, 0)? & 1 != 0 && s.types[1] != 24 && self.valid_type(s.types[1], rom)? {
                s.types[1]
            } else if self.valid_type(s.types[0], rom)? {
                s.types[0]
            } else {
                0
            };
        Ok(kind + 1)
    }
    pub fn expand(self, compact: &[u8], rom: Option<&Rom>) -> Result<Vec<u8>> {
        bytes(compact, 0, self.record_size)?;
        if u16(compact, 28)? == 0 && compact[19] & 3 == 0 {
            return Ok(vec![0; 80]);
        }
        let mut raw = vec![0; 80];
        raw[..28].copy_from_slice(&compact[..28]);
        raw[19] &= 7;
        let code = compact[19] >> 3;
        let valid = code == 31
            || code != 0
                && match rom {
                    Some(r) => self.valid_type(code - 1, r)?,
                    None => true,
                };
        if valid {
            put16(&mut raw, 30, 0xa500 | u16::from(code));
        }
        raw[32..43].copy_from_slice(&compact[28..39]);
        raw[56..62].copy_from_slice(&compact[44..50]);
        raw[68..76].copy_from_slice(&compact[50..58]);
        let mut packed = [0; 8];
        packed[..5].copy_from_slice(&compact[39..44]);
        let moves = u64::from_le_bytes(packed);
        for i in 0..4 {
            let id = ((moves >> (i * 10)) & 1023) as u16;
            put16(&mut raw, 44 + i * 2, id);
            if let Some(r) = rom {
                raw[52 + i] = (u16::from(r.move_info(id)?.pp)
                    * (5 + u16::from((raw[40] >> (i * 2)) & 3))
                    / 5) as u8;
            }
        }
        Ok(raw)
    }
    pub fn compact(self, raw: &[u8], rom: &Rom) -> Result<Vec<u8>> {
        bytes(raw, 0, 80)?;
        let mut out = vec![0; self.record_size];
        if u16(raw, 32)? == 0 {
            return Ok(out);
        }
        out[..28].copy_from_slice(&raw[..28]);
        out[19] = (raw[19] & 7) | (self.tera_code(raw, rom)? << 3);
        out[28..39].copy_from_slice(&raw[32..43]);
        out[44..50].copy_from_slice(&raw[56..62]);
        out[50..58].copy_from_slice(&raw[68..76]);
        let mut moves = 0u64;
        for i in 0..4 {
            let id = u16(raw, 44 + i * 2)?;
            if id > 1023 {
                return Err(err("range", "compressed move"));
            }
            moves |= u64::from(id) << (i * 10);
        }
        out[39..44].copy_from_slice(&moves.to_le_bytes()[..5]);
        Ok(out)
    }
}
