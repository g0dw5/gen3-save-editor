//! Time query scenarios never substitute the computer's clock for the game's clock.
use crate::{err, rom::Rom, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ClockRules {
    /// Verified hour boundaries in morning/day/dusk/night order.
    pub starts: [u8; 4],
    pub native_predicates: [usize; 3],
    pub forced_night_flag: Option<u16>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClockScenario {
    pub hour: Option<u8>,
    pub weekday: Option<u8>,
}
#[derive(Serialize)]
pub struct ClockReport {
    pub source: &'static str,
    pub effective_hour: Option<u8>,
    pub weekday: Option<u8>,
    pub period: Option<&'static str>,
    pub next_period_hour: Option<u8>,
    pub rules: Option<ClockRules>,
    pub current_clock_verified: bool,
    pub forced_night_state_verified: bool,
}
impl Rom {
    pub fn clock_query(&self, scenario: ClockScenario) -> Result<ClockReport> {
        if scenario.hour.is_some_and(|h| h > 23) || scenario.weekday.is_some_and(|d| d > 6) {
            return Err(err("clock_scenario_range", "hour 0–23; weekday 0–6"));
        }
        let rules = self.profile.clock;
        let selection = scenario.hour.zip(rules).map(|(hour, rules)| {
            let i = if hour < rules.starts[0] || hour >= rules.starts[3] {
                3
            } else if hour < rules.starts[1] {
                0
            } else if hour < rules.starts[2] {
                1
            } else {
                2
            };
            (
                ["morning", "day", "dusk", "night"][i],
                rules.starts[(i + 1) % 4],
            )
        });
        Ok(ClockReport {
            source: if scenario.hour.is_some() || scenario.weekday.is_some() {
                "scenario"
            } else {
                "unresolved"
            },
            effective_hour: scenario.hour,
            weekday: scenario.weekday,
            period: selection.map(|s| s.0),
            next_period_hour: selection.map(|s| s.1),
            rules,
            current_clock_verified: false,
            forced_night_state_verified: false,
        })
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
