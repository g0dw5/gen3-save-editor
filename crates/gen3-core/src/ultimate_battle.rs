//! Ordinary trainer-party generation for the fingerprinted Ultimate Emerald ROM.
//! All roster and move data comes from the ROM opened by the user.

use crate::{err, rom::Rom, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainerBattleRequest {
    pub trainer_id: u16,
    pub difficulty: u8,
    pub player_max_level: Option<u8>,
}

#[derive(Serialize)]
pub struct TrainerBattlePreview {
    pub trainer_id: u16,
    pub difficulty: u8,
    pub player_max_level: Option<u8>,
    pub mons: Vec<TrainerBattleMon>,
}

#[derive(Serialize)]
pub struct TrainerBattleMon {
    pub species: u16,
    pub base_level: u16,
    pub level: Option<u8>,
    pub level_source: &'static str,
    pub moves: Option<Vec<u16>>,
    pub ivs: Option<[u8; 6]>,
    pub evs: Option<[u8; 6]>,
}

/// Preview the ordinary trainer constructor, excluding scripted substitutions,
/// facility constructors and the separate flag-0x268 forced-level-50 path.
pub fn preview(rom: &Rom, request: &TrainerBattleRequest) -> Result<TrainerBattlePreview> {
    if rom.profile.id != "ultimate-emerald-55" {
        return Err(err("trainer_battle_rom", rom.profile.id));
    }
    if !(1..=4).contains(&request.difficulty)
        || request
            .player_max_level
            .is_some_and(|level| level == 0 || level > rom.profile.max_level)
    {
        return Err(err("trainer_battle_scenario", "difficulty or player level"));
    }
    let trainer = rom
        .trainers()?
        .into_iter()
        .find(|trainer| trainer.id == request.trainer_id)
        .ok_or_else(|| err("trainer_id", request.trainer_id))?;
    let mut mons = Vec::with_capacity(trainer.party.len());
    for mon in trainer.party {
        let (level, source) = resolve_level(
            mon.level,
            request.difficulty,
            request.player_max_level,
            rom.profile.max_level,
        );
        let moves = if mon.moves_explicit {
            Some(mon.moves.clone())
        } else if rom.valid_species(mon.species).is_ok() {
            level
                .map(|level| level_moves(rom, mon.species, level))
                .transpose()?
        } else {
            None
        };
        // CreateMon receives fixed IV 0 in the ordinary trainer constructor.
        // The enhanced-template branch overwrites IVs and EVs afterward;
        // flag-0 parties skip that branch and retain their zero training values.
        let plain = mon
            .generation
            .as_ref()
            .is_some_and(|generation| generation.context == "ultimate_plain");
        mons.push(TrainerBattleMon {
            species: mon.species,
            base_level: mon.level,
            level,
            level_source: source,
            moves,
            ivs: plain.then_some([0; 6]),
            evs: plain.then_some([0; 6]),
        });
    }
    Ok(TrainerBattlePreview {
        trainer_id: trainer.id,
        difficulty: request.difficulty,
        player_max_level: request.player_max_level,
        mons,
    })
}

fn resolve_level(
    base: u16,
    difficulty: u8,
    player_max: Option<u8>,
    max_level: u8,
) -> (Option<u8>, &'static str) {
    if base == 0 {
        return match player_max {
            Some(level) => (Some(level), "player_max"),
            None => (None, "needs_player_max"),
        };
    }
    if base > u16::from(max_level) {
        return (None, "unsupported_raw");
    }
    let base = base as u8;
    if difficulty == 4 && base > 1 && base < max_level {
        return match player_max {
            Some(level) if level > base => (Some(level), "lunatic_scaled"),
            Some(_) => (Some(base), "rom"),
            None => (None, "needs_player_max"),
        };
    }
    (Some(base), "rom")
}

pub(crate) fn level_moves(rom: &Rom, species: u16, level: u8) -> Result<Vec<u16>> {
    let mut moves = Vec::new();
    for learned in rom.level_moves(species)? {
        if learned.level.unwrap_or(0) <= level {
            moves.retain(|id| *id != learned.move_id);
            moves.push(learned.move_id);
        }
    }
    Ok(moves
        .into_iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_level_rules_require_player_level_only_when_needed() {
        assert_eq!(resolve_level(8, 2, None, 100), (Some(8), "rom"));
        assert_eq!(resolve_level(8, 4, None, 100), (None, "needs_player_max"));
        assert_eq!(
            resolve_level(8, 4, Some(30), 100),
            (Some(30), "lunatic_scaled")
        );
        assert_eq!(resolve_level(10, 4, Some(7), 100), (Some(10), "rom"));
        assert_eq!(resolve_level(0, 2, Some(30), 100), (Some(30), "player_max"));
        assert_eq!(resolve_level(1, 4, None, 100), (Some(1), "rom"));
        assert_eq!(resolve_level(100, 4, None, 100), (Some(100), "rom"));
    }

    #[test]
    fn native_rom_plain_trainers_have_rom_only_preview() {
        let Ok(path) = std::env::var("GEN3_ULTIMATE_ROM") else {
            return;
        };
        let rom = Rom::open(std::fs::read(path).unwrap()).unwrap();
        let standard = preview(
            &rom,
            &TrainerBattleRequest {
                trainer_id: 57,
                difficulty: 2,
                player_max_level: None,
            },
        )
        .unwrap();
        assert_eq!(
            standard
                .mons
                .iter()
                .map(|mon| mon.level)
                .collect::<Vec<_>>(),
            [Some(8), Some(10)]
        );
        let trainer = rom
            .trainers()
            .unwrap()
            .into_iter()
            .find(|row| row.id == 57)
            .unwrap();
        assert!(trainer.party.iter().all(|mon| {
            mon.generation.as_ref().is_some_and(|generation| {
                generation.context == "ultimate_plain"
                    && generation.nature == 0
                    && generation.ivs == Some([0; 6])
                    && generation.evs == Some([0; 6])
            })
        }));
        assert!(standard
            .mons
            .iter()
            .all(|mon| mon.ivs == Some([0; 6]) && mon.evs == Some([0; 6])));
        let lunatic = preview(
            &rom,
            &TrainerBattleRequest {
                trainer_id: 57,
                difficulty: 4,
                player_max_level: Some(30),
            },
        )
        .unwrap();
        assert_eq!(
            lunatic.mons.iter().map(|mon| mon.level).collect::<Vec<_>>(),
            [Some(30), Some(30)]
        );
        assert!(lunatic.mons.iter().all(|mon| mon.moves.is_some()));
        let mut app = crate::app::App {
            session: Some(crate::session::Session::new(rom)),
            ..Default::default()
        };
        let response = app
            .dispatch(crate::app::Request {
                command: "trainer_battle_preview".into(),
                payload: serde_json::json!({
                    "trainer_id": 57,
                    "difficulty": 4,
                    "player_max_level": 30
                }),
            })
            .unwrap();
        assert_eq!(response["mons"][0]["level"], 30);
    }
}
