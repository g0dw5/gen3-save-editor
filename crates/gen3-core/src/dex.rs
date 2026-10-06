//! Read-only native Dex banks, separate from editable legacy flag layouts.
use crate::{
    binary::u16,
    err,
    save::{DexFlag, Save},
    Result,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct DexRange {
    pub first: u16,
    pub count: u16,
}
#[derive(Serialize)]
pub struct DexReadStatus {
    pub count: usize,
    pub read_only: bool,
    pub uninitialized_ranges: Vec<DexRange>,
    /// Flag numbers whose raw positive records fail the native read checks.
    /// Read-only queries project the native result without clearing these bytes.
    pub inconsistent_numbers: Vec<u16>,
}

impl Save {
    pub fn dex_read_status(&self) -> Result<Option<DexReadStatus>> {
        if let Some(banks) = self.layout.dex_read {
            let main = self.logical(1..=4);
            let trainer = self.logical(0..=0);
            let mut status = DexReadStatus {
                count: 0,
                read_only: true,
                uninitialized_ranges: Vec::new(),
                inconsistent_numbers: Vec::new(),
            };
            for bank in banks {
                let block = if bank.main_block { &main } else { &trainer };
                if bank.count == 0
                    || bank.first as usize != status.count + 1
                    || bank.first.checked_add(bank.count - 1).is_none()
                    || block
                        .get(bank.seen..bank.seen + (bank.count as usize).div_ceil(8))
                        .is_none()
                    || block
                        .get(bank.owned..bank.owned + (bank.count as usize).div_ceil(8))
                        .is_none()
                {
                    return Err(err("save_layout", "invalid native Dex bank"));
                }
                if let Some((offset, marker)) = bank.initialization {
                    if u16(block, offset)? != marker {
                        status.uninitialized_ranges.push(DexRange {
                            first: bank.first,
                            count: bank.count,
                        });
                    }
                }
                status.count += bank.count as usize;
            }
            return Ok(Some(status));
        }
        if self.layout.dex.is_some() {
            return Ok(Some(self.read_legacy_dex()?.1));
        }
        Ok(None)
    }

    pub(crate) fn read_legacy_dex(&self) -> Result<(Vec<DexFlag>, DexReadStatus)> {
        let layout = self
            .layout
            .dex
            .ok_or_else(|| err("unsupported_feature", "dex_read"))?;
        let main = self.logical(1..=4);
        let trainer = self.logical(0..=0);
        let block = if layout.main_block { &main } else { &trainer };
        let mut status = DexReadStatus {
            count: layout.count as usize,
            read_only: false,
            uninitialized_ranges: Vec::new(),
            inconsistent_numbers: Vec::new(),
        };
        let mut result = Vec::with_capacity(status.count);
        for number in 1..=layout.count {
            let index = number as usize - 1 + layout.bit_bias as usize;
            let byte = index / 8;
            let mask = 1 << (index % 8);
            let bit = |data: &[u8], offset: usize| -> Result<bool> {
                let value = offset
                    .checked_add(byte)
                    .and_then(|at| data.get(at))
                    .ok_or_else(|| err("save_layout", "invalid native Dex flag"))?;
                Ok(value & mask != 0)
            };
            let raw_seen = bit(block, layout.seen)?;
            let raw_owned = bit(block, layout.owned)?;
            let mut seen = raw_seen;
            for offset in layout.seen_mirrors {
                seen &= bit(&main, *offset)?;
            }
            let owned = raw_owned && (!layout.owned_requires_seen || seen);
            if (seen, owned) != (raw_seen, raw_owned) {
                status.inconsistent_numbers.push(number);
            }
            result.push(DexFlag {
                number,
                seen,
                owned,
            });
        }
        Ok((result, status))
    }

    pub(crate) fn read_dex_banks(&self) -> Result<Vec<DexFlag>> {
        let status = self
            .dex_read_status()?
            .ok_or_else(|| err("unsupported_feature", "dex_read"))?;
        let main = self.logical(1..=4);
        let trainer = self.logical(0..=0);
        let mut result = Vec::with_capacity(status.count);
        for bank in self.layout.dex_read.unwrap_or_default() {
            let block = if bank.main_block { &main } else { &trainer };
            let initialized = !status
                .uninitialized_ranges
                .iter()
                .any(|r| r.first == bank.first);
            for index in 0..bank.count {
                let byte = index as usize / 8;
                let mask = 1 << (index % 8);
                result.push(DexFlag {
                    number: bank.first + index,
                    seen: initialized && block[bank.seen + byte] & mask != 0,
                    owned: initialized && block[bank.owned + byte] & mask != 0,
                });
            }
        }
        Ok(result)
    }
}
