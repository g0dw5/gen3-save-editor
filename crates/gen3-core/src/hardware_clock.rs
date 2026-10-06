//! Explicit RTC-input projections through the current ROM, never host-clock guesses.
use crate::{err, native_trainer::Sandbox, rom::Rom, save::Save, Result};
use armv4t_emu::Memory;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rules {
    pub validate: usize,
    pub difference: usize,
    pub getter: usize,
    pub save_offset: usize,
    pub last_update: usize,
}
pub const EMERALD: Rules = Rules {
    validate: 0x2f2fc,
    difference: 0x2f504,
    getter: 0x2f588,
    save_offset: 0x98,
    last_update: 0xa0,
};
pub const ROCKET: Rules = Rules {
    validate: 0x441a0,
    difference: 0x443a8,
    getter: 0x4442c,
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
    fn bytes(self) -> [u8; 6] {
        let days = self.days.to_le_bytes();
        [
            days[0],
            days[1],
            self.hour as u8,
            self.minute as u8,
            self.second as u8,
            0,
        ]
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub expected_rom_md5: String,
    pub rtc: Input,
    /// None uses this SAV's offset. Explicit offsets are standalone scenarios.
    pub offset: Option<Time>,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub offset: Time,
    pub last_update: Time,
    pub offset_valid: bool,
    pub last_update_valid: bool,
}
#[derive(Serialize)]
pub struct Projection {
    pub rom_md5: &'static str,
    pub source: &'static str,
    pub input: Input,
    pub offset: Time,
    pub offset_source: &'static str,
    pub local_time: Time,
    pub current_clock_verified: bool,
    /// The native difference routine does not produce an effective weekday.
    pub weekday: Option<u8>,
    pub period: Option<&'static str>,
    pub partial: bool,
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
    pub fn hardware_clock_preview(
        &self,
        save: Option<&Save>,
        request: Request,
    ) -> Result<Projection> {
        if request.expected_rom_md5 != self.profile.md5 {
            return Err(err("rom_mismatch", "RTC scenario"));
        }
        let rules = self
            .profile
            .hardware_clock
            .ok_or_else(|| err("clock_rtc_unverified", self.profile.id))?;
        let input = request.rtc;
        // Bound inputs before native validation can index its month table. Calendar
        // validity (including leap years) is then checked by the actual ROM.
        if !(2000..=2099).contains(&input.year)
            || !(1..=12).contains(&input.month)
            || !(1..=31).contains(&input.day)
            || input.hour >= 24
            || input.minute >= 60
            || input.second >= 60
        {
            return Err(err(
                "clock_rtc_range",
                "RTC input outside supported native range",
            ));
        }
        let (offset, offset_source) = match request.offset {
            Some(offset) => (offset, "scenario"),
            None => (
                self.hardware_clock_snapshot(save)?
                    .ok_or_else(|| {
                        err(
                            "clock_save_required",
                            "RTC projection requires SAV or explicit offset",
                        )
                    })?
                    .offset,
                "save",
            ),
        };
        if !offset.valid() {
            return Err(err("clock_saved_offset", "invalid time components"));
        }
        let mut ram = Sandbox::new(&self.data);
        const RTC: u32 = 0x02010000;
        const RESULT: u32 = 0x02010100;
        const OFFSET: u32 = 0x02010200;
        let bcd = |v: u8| (v / 10) * 16 + v % 10;
        let raw = [
            bcd((input.year - 2000) as u8),
            bcd(input.month),
            bcd(input.day),
            0,
            bcd(input.hour),
            bcd(input.minute),
            bcd(input.second),
            0x40,
            0,
            0,
            0,
            0,
        ];
        for (i, byte) in raw.into_iter().enumerate() {
            ram.w8(RTC + i as u32, byte);
        }
        for (i, byte) in offset.bytes().into_iter().enumerate() {
            ram.w8(OFFSET + i as u32, byte);
        }
        let check = ram.call(
            0x08000000 + rules.validate as u32,
            [RTC, 0, 0, 0],
            [0, 0],
            100_000,
        )?;
        if check != 0 {
            return Err(err(
                "clock_rtc_invalid",
                format!("native RTC validation {check:04X}"),
            ));
        }
        ram.call(
            0x08000000 + rules.difference as u32,
            [RTC, RESULT, OFFSET, 0],
            [0, 0],
            100_000,
        )?;
        let raw: [u8; 6] = std::array::from_fn(|i| ram.r8(RESULT + i as u32));
        let local_time = Time::from_bytes(&raw);
        if !local_time.valid() {
            return Err(err("clock_native_time", "invalid native projection"));
        }
        Ok(Projection {
            rom_md5: self.profile.md5,
            source: "rtc_scenario",
            input,
            offset,
            offset_source,
            local_time,
            current_clock_verified: false,
            weekday: None,
            period: None,
            partial: true,
        })
    }
}
