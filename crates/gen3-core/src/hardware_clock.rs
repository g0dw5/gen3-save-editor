//! Read-only saved RTC offsets/checkpoints; these are not the live device clock.
use crate::{err, rom::Rom, save::Save, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rules {
    pub save_offset: usize,
    pub last_update: usize,
}
pub const EMERALD: Rules = Rules {
    save_offset: 0x98,
    last_update: 0xa0,
};
pub const ROCKET: Rules = Rules {
    save_offset: 0x98,
    last_update: 0xa0,
};
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Time {
    pub days: i16,
    pub hour: i8,
    pub minute: i8,
    pub second: i8,
}
impl Time {
    fn valid(self) -> bool {
        (0..24).contains(&self.hour)
            && (0..60).contains(&self.minute)
            && (0..60).contains(&self.second)
    }
    fn from_bytes(raw: &[u8]) -> Self {
        Self {
            days: i16::from_le_bytes([raw[0], raw[1]]),
            hour: raw[2] as i8,
            minute: raw[3] as i8,
            second: raw[4] as i8,
        }
    }
}
#[derive(Serialize)]
pub struct Snapshot {
    pub offset: Time,
    pub last_update: Time,
    pub offset_valid: bool,
    pub last_update_valid: bool,
}
impl Rom {
    pub fn hardware_clock_snapshot(&self, save: Option<&Save>) -> Result<Option<Snapshot>> {
        let Some((rules, save)) = self.profile.hardware_clock.zip(save) else {
            return Ok(None);
        };
        if save.layout.pokemon_codec != self.profile.save.pokemon_codec
            || save.layout.sizes != self.profile.save.sizes
        {
            return Err(err("clock_save_layout", "SAVE and ROM layouts differ"));
        }
        let data = save.logical(0..=0);
        let read = |offset| {
            data.get(offset..offset + 6)
                .map(Time::from_bytes)
                .ok_or_else(|| err("clock_saved_offset", offset))
        };
        let offset = read(rules.save_offset)?;
        let last_update = read(rules.last_update)?;
        Ok(Some(Snapshot {
            offset,
            last_update,
            offset_valid: offset.valid(),
            last_update_valid: last_update.valid(),
        }))
    }
}
