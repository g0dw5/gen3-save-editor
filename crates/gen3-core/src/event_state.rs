//! Read-only, exact-adapter event state. Temporary/RAM-only IDs stay unknown.
use crate::{profile::EventStateLayout, save::Save};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventBlock {
    Main,
    Trainer,
    Extensions,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct EventRange {
    pub first: u16,
    pub count: u16,
    pub block: EventBlock,
    pub offset: usize,
}
pub struct EventSnapshot {
    main: Vec<u8>,
    trainer: Vec<u8>,
    extensions: Vec<u8>,
    layout: EventStateLayout,
}
impl EventSnapshot {
    #[cfg(test)]
    pub(crate) fn fixture(
        main: Vec<u8>,
        trainer: Vec<u8>,
        extensions: Vec<u8>,
        layout: EventStateLayout,
    ) -> Self {
        Self {
            main,
            trainer,
            extensions,
            layout,
        }
    }
    pub fn new(save: &Save, layout: EventStateLayout) -> Self {
        Self {
            main: save.logical(1..=4),
            trainer: save.logical(0..=0),
            extensions: save.extensions(),
            layout,
        }
    }
    fn block(&self, block: EventBlock) -> &[u8] {
        match block {
            EventBlock::Main => &self.main,
            EventBlock::Trainer => &self.trainer,
            EventBlock::Extensions => &self.extensions,
        }
    }
    pub fn flag(&self, id: u16) -> Option<u16> {
        if id == 0 {
            return None;
        }
        let range = self
            .layout
            .flags
            .iter()
            .find(|r| id >= r.first && id - r.first < r.count)?;
        let bit = (id - range.first) as usize;
        self.block(range.block)
            .get(range.offset + bit / 8)
            .map(|b| ((b >> (bit % 8)) & 1) as u16)
    }
    pub fn variable(&self, id: u16) -> Option<u16> {
        let range = self
            .layout
            .variables
            .iter()
            .find(|r| id >= r.first && id - r.first < r.count)?;
        let offset = range.offset + (id - range.first) as usize * 2;
        self.block(range.block)
            .get(offset..offset + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{binary::*, profile, save::sector_checksum};
    fn pattern(index: usize, salt: u8) -> u8 {
        ((index * 73 + (index >> 5) * 19) as u8) ^ salt
    }
    fn synthetic_save(p: profile::Profile, salt: u8) -> Save {
        let mut data = vec![0u8; 0x20000];
        let mut main = 0;
        let mut extra = 0;
        for id in 0..14 {
            let o = id * 4096;
            let size = p.save.sizes[id];
            for index in 0..size {
                data[o + index] = if id == 0 {
                    pattern(index, salt)
                } else if id <= 4 {
                    pattern(main + index, salt)
                } else {
                    0
                };
            }
            if (1..=4).contains(&id) {
                main += size;
            }
            if id == 1 {
                data[o + p.save.party_count] = 0;
            }
            if p.save.extension_sectors.is_some() {
                for index in size..0xff0 {
                    data[o + index] = pattern(extra, salt);
                    extra += 1;
                }
            }
            put16(&mut data, o + 0xff4, id as u16);
            put32(&mut data, o + 0xff8, 0x08012025);
            put32(&mut data, o + 0xffc, 1);
            let sum = if p.save.sector_checksum == profile::SectorChecksum::NativeConstantOne {
                1
            } else {
                sector_checksum(&data[o..o + size])
            };
            put16(&mut data, o + 0xff6, sum);
        }
        if let Some(sectors) = p.save.extension_sectors {
            for sector in sectors {
                for index in 0..0xff0 {
                    data[sector * 4096 + index] = pattern(extra, salt);
                    extra += 1;
                }
            }
        }
        Save::open(data, p.save).unwrap()
    }
    #[test]
    fn segmented_event_state_is_read_only_bounded_and_matches_logical_save_blocks() {
        for p in profile::PROFILES {
            let layout = p.event_state.unwrap();
            for salt in [0x55, 0xaa] {
                let save = synthetic_save(p, salt);
                let before = save.data.clone();
                let state = EventSnapshot::new(&save, layout);
                for range in layout.flags {
                    for index in 0..range.count {
                        let id = range.first + index;
                        if id == 0 {
                            assert_eq!(state.flag(id), None);
                            continue;
                        }
                        assert_eq!(
                            state.flag(id),
                            Some(
                                ((pattern(range.offset + index as usize / 8, salt) >> (index % 8))
                                    & 1) as u16
                            ),
                            "{} flag {id:x}",
                            p.id
                        );
                    }
                }
                for range in layout.variables {
                    for index in 0..range.count {
                        let offset = range.offset + index as usize * 2;
                        assert_eq!(
                            state.variable(range.first + index),
                            Some(u16::from_le_bytes([
                                pattern(offset, salt),
                                pattern(offset + 1, salt)
                            ]))
                        );
                    }
                }
                assert_eq!(state.flag(0xffff), None);
                assert_eq!(state.variable(0x800d), None);
                assert_eq!(state.variable(0x3fff), None);
                assert_eq!(before, save.data);
            }
        }
    }
    #[test]
    #[ignore = "requires private native vectors via GEN3_EVENT_PROBES"]
    fn local_event_state_matches_native_ranges() {
        let probes: serde_json::Value = serde_json::from_slice(
            &std::fs::read(std::env::var("GEN3_EVENT_PROBES").unwrap()).unwrap(),
        )
        .unwrap();
        for (name, p) in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"]
            .into_iter()
            .zip(profile::PROFILES)
        {
            let report = &probes[name];
            assert_eq!(report["md5"], p.md5);
            for salt in [0x55, 0xaa] {
                let save = synthetic_save(p, salt);
                let state = EventSnapshot::new(&save, p.event_state.unwrap());
                for vector in report["vectors"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|v| v["salt"] == salt)
                {
                    let id = vector["id"].as_u64().unwrap() as u16;
                    let actual = if vector["kind"] == "flag" {
                        state.flag(id)
                    } else {
                        state.variable(id)
                    };
                    assert_eq!(
                        actual,
                        Some(vector["expected"].as_u64().unwrap() as u16),
                        "{name} {id:x}"
                    );
                }
            }
        }
    }
}
