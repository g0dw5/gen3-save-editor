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
        Ok(self.layout.dex.map(|layout| DexReadStatus {
            count: layout.count as usize,
            read_only: false,
            uninitialized_ranges: Vec::new(),
        }))
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
