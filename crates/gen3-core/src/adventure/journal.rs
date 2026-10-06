//! Native quest journal tables. Content stays in the loaded ROM, state in SAV.
use super::{Guide, Task};
use crate::{binary::*, event_state::EventSnapshot, rom::Rom, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rules {
    pub quests: usize,
    pub books: usize,
    pub count: usize,
}
pub const MERCURY: Rules = Rules {
    quests: 0xe3ba28,
    books: 0x8f5644,
    count: 100,
};

#[derive(Serialize)]
pub struct Phase {
    pub title: String,
    pub text: String,
    pub visible: Option<bool>,
}
#[derive(Serialize)]
pub struct Location {
    pub map_id: String,
    pub x: Option<i16>,
    pub y: Option<i16>,
    pub actor: Option<u16>,
}
#[derive(Serialize)]
pub struct Journal {
    pub title: String,
    pub objective: String,
    pub accepted: Option<bool>,
    pub phases: Vec<Phase>,
    pub locations: Vec<Location>,
}
fn flag(state: Option<&EventSnapshot>, id: u16) -> Option<bool> {
    if id == 0 {
        Some(false)
    } else {
        state?.flag(id).map(|v| v != 0)
    }
}
fn eligible(
    accepted: Option<bool>,
    completed: Option<bool>,
    appearance: Option<bool>,
) -> Option<bool> {
    if accepted == Some(true) || completed == Some(true) || appearance == Some(true) {
        Some(true)
    } else if [accepted, completed, appearance].contains(&None) {
        None
    } else {
        Some(false)
    }
}
/// Mirrors DF7C40: completed pages may bypass their checks, depending on byte 15.
fn visible(
    completed: Option<bool>,
    eligible: Option<bool>,
    extra: Option<bool>,
    condition: Option<bool>,
    after_completion_checks: bool,
) -> Option<bool> {
    let possibilities =
        |v: Option<bool>| -> Vec<bool> { v.map(|b| vec![b]).unwrap_or_else(|| vec![false, true]) };
    let mut result = None;
    for done in possibilities(completed) {
        for known in possibilities(eligible) {
            for gate in possibilities(extra) {
                for condition in possibilities(condition) {
                    let value = if done && !after_completion_checks {
                        true
                    } else {
                        (done || known) && gate && condition
                    };
                    if result.is_some_and(|old| old != value) {
                        return None;
                    }
                    result = Some(value);
                }
            }
        }
    }
    result
}
fn clean(s: String) -> String {
    s.replace("{换行}", "\n")
        .replace("{翻页}", "\n")
        .trim()
        .to_string()
}
pub(super) fn append(rom: &Rom, state: Option<&EventSnapshot>, guide: &mut Guide) -> Result<()> {
    let Some(rules) = rom.profile.quest_journal else {
        return Ok(());
    };
    for id in 0..rules.count {
        let q = rules.quests + id * 20;
        let b = rules.books + id * 8;
        let accept_flag = u16(&rom.data, q + 10)?;
        let complete_flag = u16(&rom.data, q + 12)?;
        let accepted = flag(state, accept_flag);
        let completed = flag(state, complete_flag);
        let available = eligible(accepted, completed, flag(state, u16(&rom.data, b + 4)?));
        let pages = pointer(&rom.data, b)?;
        let mut phases = vec![];
        for page in 0..usize::from(bytes(&rom.data, b + 6, 1)?[0]) {
            let p = pages + page * 16;
            let extra_id = u16(&rom.data, p + 12)?;
            let extra = if extra_id == 0 {
                Some(true)
            } else {
                flag(state, extra_id)
            };
            let condition = match bytes(&rom.data, p + 14, 1)?[0] {
                0 => available,
                1 => flag(state, u16(&rom.data, p + 8)?),
                2 => state
                    .and_then(|s| s.variable(u16(&rom.data, p + 8).ok()?))
                    .map(|v| v >= u16(&rom.data, p + 10).unwrap_or_default()),
                3 => completed,
                _ => Some(false),
            };
            phases.push(Phase {
                title: clean(rom.ptr_text(p)),
                text: clean(rom.ptr_text(p + 4)),
                visible: if state.is_none() {
                    None
                } else {
                    visible(
                        completed,
                        available,
                        extra,
                        condition,
                        bytes(&rom.data, p + 15, 1)?[0] != 0,
                    )
                },
            });
        }
        // Associate only actual indexed writers. A journal's area is not an entrance.
        let location = guide
            .tasks
            .iter()
            .find(|t| t.effects.contains(&("flag", accept_flag, Some(1))))
            .or_else(|| {
                guide
                    .tasks
                    .iter()
                    .find(|t| t.effects.contains(&("flag", complete_flag, Some(1))))
            });
        let (map_id, x, y, actor) = location
            .map(|t| (t.map_id.clone(), t.x, t.y, t.actor))
            .unwrap_or_default();
        guide.tasks.push(Task {
            id: format!("journal:{id}"),
            kind: "journal",
            map_id,
            x,
            y,
            actor,
            goals: vec![],
            text: vec![],
            checks: vec![],
            stage: None,
            next_candidate: false,
            prerequisites: vec![],
            partial: true,
            status: if completed == Some(true) {
                "completed"
            } else if accepted == Some(true) {
                "in_progress"
            } else {
                "unknown"
            },
            effects: vec![("flag", complete_flag, Some(1))],
            journal: Some(Journal {
                title: clean(rom.ptr_text(q)),
                objective: clean(rom.ptr_text(q + 4)),
                accepted,
                phases,
                locations: vec![],
            }),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires private Mercury 1.2 ROM; exports native parity inputs only when requested"]
    fn exact_rom_journal_native_vectors() {
        let data = std::fs::read(std::env::var("GEN3_ROM_MERCURY12").unwrap()).unwrap();
        let rom = Rom::open(data.clone()).unwrap();
        assert_eq!(rom.profile.md5, crate::mercury::PROFILE.md5);
        let mut vectors = vec![];
        for pattern in [0, 255, 85, 170, 37] {
            let main = vec![pattern; 0x3d68];
            let extra = vec![pattern; 0x1900];
            let state = EventSnapshot::fixture(
                main.clone(),
                vec![],
                extra.clone(),
                rom.profile.event_state.unwrap(),
            );
            let mut guide = Guide {
                rom_md5: rom.profile.md5.into(),
                current_stage: None,
                story_supported: false,
                tasks: vec![],
                partial: true,
            };
            append(&rom, Some(&state), &mut guide).unwrap();
            assert_eq!(guide.tasks.len(), 100);
            assert!(guide
                .tasks
                .iter()
                .all(|t| !t.journal.as_ref().unwrap().title.is_empty()));
            let pages: Vec<_> = guide.tasks.iter().enumerate().flat_map(|(id, t)| {
                let first = pointer(&rom.data, MERCURY.books + id * 8).unwrap();
                t.journal.as_ref().unwrap().phases.iter().enumerate().map(move |(page, p)|
                    serde_json::json!({"quest": id, "page": first + page * 16, "expected": p.visible}))
            }).collect();
            assert_eq!(pages.len(), 306);
            vectors.push(serde_json::json!({"main": main, "extensions": extra, "pages": pages}));
        }
        if let Ok(path) = std::env::var("GEN3_JOURNAL_PROBES") {
            std::fs::write(path, serde_json::to_vec(&vectors).unwrap()).unwrap();
        }
        assert_eq!(&*rom.data, &data);
    }
    #[test]
    fn journal_visibility_retains_native_completion_bypass_and_unknown_state() {
        assert_eq!(
            visible(Some(true), Some(true), Some(false), Some(false), false),
            Some(true)
        );
        assert_eq!(
            visible(Some(true), Some(true), Some(false), Some(true), true),
            Some(false)
        );
        assert_eq!(
            visible(Some(false), Some(false), Some(true), Some(true), false),
            Some(false)
        );
        assert_eq!(
            visible(Some(false), Some(true), Some(true), Some(true), false),
            Some(true)
        );
        assert_eq!(
            visible(None, Some(true), Some(true), Some(true), false),
            Some(true)
        );
        assert_eq!(eligible(Some(false), None, Some(false)), None);
        assert_eq!(eligible(Some(true), None, Some(false)), Some(true));
    }
}
