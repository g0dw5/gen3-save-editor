//! Read-only game-clock snapshots and explicit query scenarios, never device time.
use crate::{
    err,
    rom::Rom,
    save::{PocketBlock, Save},
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct SavedClockRules {
    pub block: PocketBlock,
    pub enabled_byte: usize,
    pub enabled_mask: u8,
    pub forced_night_byte: usize,
    pub forced_night_mask: u8,
    /// Four packed words: year, month/day, hour/minute, second/weekday.
    pub words: usize,
    pub speed_word: usize,
    pub year_min: u16,
    pub year_max: u16,
    /// Native month lengths as 32-bit entries; leap-year semantics verified separately.
    pub month_lengths: usize,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ClockRules {
    /// Verified hour boundaries in morning/day/dusk/night order.
    pub starts: [u8; 4],
    pub native_predicates: [usize; 3],
    pub forced_night_flag: Option<u16>,
    pub saved: Option<SavedClockRules>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClockScenario {
    pub hour: Option<u8>,
    pub weekday: Option<u8>,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SavedClock {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    /// Native weekday is independent of the calendar date (0 = Sunday).
    pub weekday: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub speed: u16,
}
#[derive(Serialize)]
pub struct ClockReport {
    pub source: &'static str,
    pub effective_hour: Option<u8>,
    pub weekday: Option<u8>,
    pub period: Option<&'static str>,
    pub next_period_hour: Option<u8>,
    pub seconds_until_next_period: Option<u32>,
    pub saved: Option<SavedClock>,
    pub forced_night: Option<bool>,
    pub issue: Option<&'static str>,
    pub rules: Option<ClockRules>,
    pub current_clock_verified: bool,
    pub forced_night_state_verified: bool,
}
fn word(block: &[u8], offset: usize) -> Option<u16> {
    block
        .get(offset..offset + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
}
impl Rom {
    pub fn clock_query(&self, scenario: ClockScenario) -> Result<ClockReport> {
        self.clock_query_with_save(None, scenario)
    }
    pub fn clock_query_with_save(
        &self,
        save: Option<&Save>,
        scenario: ClockScenario,
    ) -> Result<ClockReport> {
        if scenario.hour.is_some_and(|h| h > 23) || scenario.weekday.is_some_and(|d| d > 6) {
            return Err(err("clock_scenario_range", "hour 0–23; weekday 0–6"));
        }
        let mut report = ClockReport {
            source: "unresolved",
            effective_hour: None,
            weekday: None,
            period: None,
            next_period_hour: None,
            seconds_until_next_period: None,
            saved: None,
            forced_night: None,
            issue: None,
            rules: self.profile.clock,
            current_clock_verified: false,
            forced_night_state_verified: false,
        };
        if scenario.hour.is_some() || scenario.weekday.is_some() {
            // A simulated hour explicitly replaces SAVE time; it does not inherit its overrides.
            report.source = "scenario";
            report.effective_hour = scenario.hour;
            report.weekday = scenario.weekday;
        } else if let Some((save, rules)) = save.zip(self.profile.clock.and_then(|r| r.saved)) {
            // Refuse incompatible SAVE layouts rather than interpreting their bytes as this clock.
            if save.layout.pokemon_codec != self.profile.save.pokemon_codec
                || save.layout.sizes != self.profile.save.sizes
            {
                return Err(err("clock_save_layout", "SAVE and ROM layouts differ"));
            }
            let block = match rules.block {
                PocketBlock::Main => save.logical(1..=4),
                PocketBlock::SectorExtensions => save.extensions(),
            };
            let enabled = block
                .get(rules.enabled_byte)
                .map(|b| b & rules.enabled_mask != 0);
            report.forced_night = block
                .get(rules.forced_night_byte)
                .map(|b| b & rules.forced_night_mask != 0);
            report.forced_night_state_verified = report.forced_night.is_some();
            if enabled == Some(false) {
                // Native hardware RTC is not recoverable from these SAVE words; no host-clock fallback.
                report.issue = Some("hardware_rtc_unresolved");
            } else if enabled == Some(true) {
                let fields: Option<Vec<_>> =
                    (0..4).map(|i| word(&block, rules.words + i * 2)).collect();
                if let Some(fields) = fields {
                    let [year, md, hm, sw] = fields[..] else {
                        unreachable!()
                    };
                    let (month, day) = ((md >> 8) as u8, md as u8);
                    let days = if (1..=12).contains(&month) {
                        self.data
                            .get(rules.month_lengths + (month as usize - 1) * 4)
                            .copied()
                            .map(|d| {
                                d + u8::from(
                                    month == 2
                                        && year % 4 == 0
                                        && (year % 100 != 0 || year % 400 == 0),
                                )
                            })
                    } else {
                        None
                    };
                    let (hour, minute, second, weekday) =
                        ((hm >> 8) as u8, hm as u8, (sw >> 8) as u8, sw as u8);
                    if (rules.year_min..=rules.year_max).contains(&year)
                        && day != 0
                        && days.is_some_and(|d| day <= d)
                        && hour < 24
                        && minute < 60
                        && second < 60
                        && weekday < 7
                    {
                        let speed = word(&block, rules.speed_word).unwrap_or(1);
                        report.saved = Some(SavedClock {
                            year,
                            month,
                            day,
                            weekday,
                            hour,
                            minute,
                            second,
                            speed: if [1, 2, 5, 10, 30, 60].contains(&speed) {
                                speed
                            } else {
                                1
                            },
                        });
                        report.source = "save_virtual";
                        report.effective_hour = Some(hour);
                        report.weekday = Some(weekday);
                        report.current_clock_verified = true;
                    } else {
                        // Native code resets invalid virtual time from RTC. Without that RTC, stay unknown.
                        report.issue = Some("invalid_saved_clock");
                    }
                } else {
                    report.issue = Some("invalid_saved_clock");
                }
            } else {
                report.issue = Some("invalid_saved_clock");
            }
        }
        if let Some((hour, rules)) = report.effective_hour.zip(self.profile.clock) {
            let i = if report.forced_night == Some(true)
                || hour < rules.starts[0]
                || hour >= rules.starts[3]
            {
                3
            } else if hour < rules.starts[1] {
                0
            } else if hour < rules.starts[2] {
                1
            } else {
                2
            };
            report.period = Some(["morning", "day", "dusk", "night"][i]);
            if report.forced_night != Some(true) {
                let next = rules.starts[(i + 1) % 4];
                report.next_period_hour = Some(next);
                if let Some(saved) = &report.saved {
                    let now = hour as u32 * 3600 + saved.minute as u32 * 60 + saved.second as u32;
                    report.seconds_until_next_period =
                        Some((next as u32 * 3600 + 86400 - now) % 86400);
                }
            }
        }
        if report.forced_night == Some(true) {
            report.period = Some("night");
        }
        Ok(report)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn periods_wrap_at_native_boundaries_without_device_time() {
        let mut r = crate::tests::query_fixture_rom();
        r.profile.clock = crate::mercury::PROFILE.clock;
        for (h, p, next) in [
            (0, "night", 4),
            (3, "night", 4),
            (4, "morning", 8),
            (7, "morning", 8),
            (8, "day", 17),
            (16, "day", 17),
            (17, "dusk", 20),
            (19, "dusk", 20),
            (20, "night", 4),
            (23, "night", 4),
        ] {
            let report = r
                .clock_query(ClockScenario {
                    hour: Some(h),
                    weekday: None,
                })
                .unwrap();
            assert_eq!(report.period, Some(p));
            assert_eq!(report.next_period_hour, Some(next));
            assert!(!report.current_clock_verified);
        }
        assert!(r
            .clock_query(ClockScenario {
                hour: Some(24),
                weekday: None
            })
            .is_err());
        assert_eq!(
            r.clock_query(ClockScenario {
                hour: None,
                weekday: None
            })
            .unwrap()
            .effective_hour,
            None
        );
        r.profile.clock = None;
        assert_eq!(
            r.clock_query(ClockScenario {
                hour: Some(8),
                weekday: None
            })
            .unwrap()
            .period,
            None
        );
    }
}
