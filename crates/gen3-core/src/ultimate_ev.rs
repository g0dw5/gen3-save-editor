//! Bounded execution of the verified Ultimate Emerald trainer EV constructor.
//! The program and all game data come from the loaded ROM.

use crate::{err, pokemon, rom::Rom, Result};
use armv4t_emu::{reg, Cpu, Memory, Mode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const ROM_BASE: u32 = 0x0800_0000;
const EWRAM_BASE: u32 = 0x0200_0000;
const IWRAM_BASE: u32 = 0x0300_0000;
const MON: u32 = 0x0202_4744;
const SUMMARY: u32 = 0x0203_0000;
const GET_MON_DATA: u32 = 0x0806_a518;
const SET_MON_DATA: u32 = 0x0806_acac;
const VAR_GET: u32 = 0x0809_d694;
const RANDOM: u32 = 0x0806_f5cc;
const END: u32 = 0x0f00_0000;
const TRAINER_EV_ROUTINE: u32 = 0x09f0_42bc;
const PLAYER_SPEED_ROUTINE: u32 = 0x09f0_3f0c;
const PLAYER_ROLE_ROUTINE: u32 = 0x09f0_229c;
const SORT_PLAYER_SUMMARY: u32 = 0x09f0_3e6c;

/// A battle participant supplied by the currently opened save or by the user.
/// Speed is the visible pre-battle Speed stat, before held-item adjustments.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerBattleMon {
    pub species: u16,
    pub held_item: u16,
    pub ability_slot: u8,
    pub nature: u8,
    pub speed: u16,
    pub current_hp: u16,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainerEvRequest {
    pub trainer_id: u16,
    pub difficulty: u8,
    pub player_party: Vec<PlayerBattleMon>,
    #[serde(default)]
    pub opponent_levels: Vec<u8>,
}

#[derive(Serialize)]
pub struct TrainerEvMon {
    pub species: u16,
    pub level: u8,
    pub ivs: Option<[u8; 6]>,
    pub alternate_ivs: Option<[u8; 6]>,
    pub evs: Option<[u8; 6]>,
    pub alternate_evs: Option<[u8; 6]>,
}

/// Execute the loaded ROM's EV constructor for one explicit battle scenario.
/// Scripted trainer overrides and the game's dynamic level selection are outside
/// this routine; the caller supplies the actual levels when those are known.
pub fn preview(rom: &Rom, request: &TrainerEvRequest) -> Result<TrainerEvPreview> {
    if rom.profile.id != "ultimate-emerald-55" {
        return Err(err("trainer_ev_rom", rom.profile.id));
    }
    if !(1..=4).contains(&request.difficulty) || !(1..=6).contains(&request.player_party.len()) {
        return Err(err("trainer_ev_scenario", "difficulty or party size"));
    }
    for p in &request.player_party {
        rom.valid_species(p.species)?;
        if p.nature >= 25 || p.ability_slot > 2 || p.speed == 0 {
            return Err(err("trainer_ev_player", p.species));
        }
        if p.held_item != 0 {
            rom.item(p.held_item)?;
        }
    }
    let trainer = rom
        .trainers()?
        .into_iter()
        .find(|t| t.id == request.trainer_id)
        .ok_or_else(|| err("trainer_id", request.trainer_id))?;
    if request.opponent_levels.len() > trainer.party.len() {
        return Err(err("trainer_ev_levels", request.opponent_levels.len()));
    }
    let simulate = |rng_bit: u8| -> Result<Vec<TrainerEvMon>> {
        let mut mem = Sandbox::new(&rom.data);
        mem.difficulty = request.difficulty;
        mem.rng_bit = rng_bit;
        mem.set_byte(0x0202_44e9, request.player_party.len() as u8);
        mem.set_byte(0x0202_4743, trainer.party.len() as u8);
        mem.put32(0x0202_2fec, 8 | u32::from(trainer.double_battle));
        mem.put16(0x0203_8bca, trainer.id);
        let mut active_speed_sum = 0u32;
        let mut active_count = 0;
        for (i, p) in request.player_party.iter().enumerate() {
            let addr = 0x0202_44ec + i as u32 * 100;
            mem.mon(
                addr,
                p.species,
                p.held_item,
                p.ability_slot,
                p.nature,
                p.speed,
                p.current_hp,
            );
            mem.set_byte(SUMMARY + i as u32 * 4 + 2, i as u8);
            if p.current_hp > 0 {
                let speed = mem.native(PLAYER_SPEED_ROUTINE, [addr, 0, 0, 0], [0, 0])? as u16;
                let role = mem.native(PLAYER_ROLE_ROUTINE, [addr, 0, 0, 0], [0, 0])? as u8;
                mem.put16(SUMMARY + i as u32 * 4, speed);
                mem.set_byte(SUMMARY + i as u32 * 4 + 3, role);
                if active_count < if trainer.double_battle { 2 } else { 1 } {
                    active_speed_sum += u32::from(speed);
                    active_count += 1;
                }
            }
        }
        mem.native(
            SORT_PLAYER_SUMMARY,
            [SUMMARY, request.player_party.len() as u32, 1, 0],
            [0, 0],
        )?;
        let mut result = Vec::with_capacity(trainer.party.len());
        for (i, p) in trainer.party.iter().enumerate() {
            let level = request
                .opponent_levels
                .get(i)
                .copied()
                .unwrap_or(p.level as u8);
            if level == 0 || level > rom.profile.max_level {
                return Err(err("trainer_ev_level", format!("{i}:{level}")));
            }
            let mut row = TrainerEvMon {
                species: p.species,
                level,
                ivs: None,
                alternate_ivs: None,
                evs: None,
                alternate_evs: None,
            };
            if let Some(g) = p
                .generation
                .as_ref()
                .filter(|g| g.context == "ultimate_template")
            {
                let iv = g.ivs.ok_or_else(|| err("trainer_ev_ivs", i))?[0];
                let template = crate::binary::bytes(
                    &rom.data,
                    0x1f0af60 + g.personality_parameter as usize * 8,
                    8,
                )?;
                let species = rom.valid_species(p.species)?;
                let stats = pokemon::stats_with_changes(
                    species.stats,
                    [iv; 6],
                    [0; 6],
                    level,
                    rom.nature_changes(g.nature)?,
                    rom.profile.nature_product_u16,
                );
                let addr = MON + i as u32 * 100;
                mem.mon(
                    addr,
                    p.species,
                    p.held_item,
                    template[2],
                    g.nature,
                    stats[3],
                    stats[0],
                );
                let moves = if p.moves_explicit {
                    p.moves.clone()
                } else {
                    crate::ultimate_battle::level_moves(rom, p.species, level)?
                };
                for (j, move_id) in moves.iter().take(4).enumerate() {
                    mem.mons.get_mut(&addr).unwrap().fields[13 + j] = u32::from(*move_id);
                }
                mem.native(
                    TRAINER_EV_ROUTINE,
                    [MON, SUMMARY, i as u32, u32::from(iv)],
                    [u32::from(template[0]), active_speed_sum],
                )?;
                if mem.random_calls > 1 {
                    return Err(err("trainer_ev_random_complex", mem.random_calls));
                }
                let fields = &mem.mons[&addr].fields;
                row.ivs = Some(std::array::from_fn(|j| fields[39 + j] as u8));
                row.evs = Some(std::array::from_fn(|j| fields[26 + j] as u8));
            }
            result.push(row);
        }
        Ok(result)
    };
    let mut mons = simulate(0)?;
    let alternate = simulate(1)?;
    for (mon, other) in mons.iter_mut().zip(alternate) {
        if mon.ivs != other.ivs {
            mon.alternate_ivs = other.ivs;
        }
        if mon.evs != other.evs {
            mon.alternate_evs = other.evs;
        }
    }
    Ok(TrainerEvPreview {
        trainer_id: trainer.id,
        difficulty: request.difficulty,
        player_count: request.player_party.len(),
        mons,
    })
}

#[derive(Serialize)]
pub struct TrainerEvPreview {
    pub trainer_id: u16,
    pub difficulty: u8,
    pub player_count: usize,
    pub mons: Vec<TrainerEvMon>,
}

#[derive(Clone)]
struct NativeMon {
    fields: [u32; 64],
}
impl Default for NativeMon {
    fn default() -> Self {
        Self { fields: [0; 64] }
    }
}

struct Sandbox<'a> {
    rom: &'a [u8],
    ewram: Box<[u8; 0x40000]>,
    iwram: Box<[u8; 0x8000]>,
    mons: BTreeMap<u32, NativeMon>,
    difficulty: u8,
    rng_bit: u8,
    random_calls: u32,
    bad_address: Option<u32>,
}

impl<'a> Sandbox<'a> {
    fn new(rom: &'a [u8]) -> Self {
        Self {
            rom,
            ewram: Box::new([0; 0x40000]),
            iwram: Box::new([0; 0x8000]),
            mons: BTreeMap::new(),
            difficulty: 2,
            rng_bit: 0,
            random_calls: 0,
            bad_address: None,
        }
    }

    fn byte(&mut self, addr: u32) -> u8 {
        match addr {
            ROM_BASE..=0x09ff_ffff => self
                .rom
                .get(((addr - ROM_BASE) & 0x01ff_ffff) as usize)
                .copied()
                .unwrap_or_else(|| {
                    self.bad_address = Some(addr);
                    0
                }),
            EWRAM_BASE..=0x0203_ffff => self.ewram[(addr - EWRAM_BASE) as usize],
            IWRAM_BASE..=0x0300_7fff => self.iwram[(addr - IWRAM_BASE) as usize],
            _ => {
                self.bad_address = Some(addr);
                0
            }
        }
    }

    fn set_byte(&mut self, addr: u32, value: u8) {
        match addr {
            EWRAM_BASE..=0x0203_ffff => self.ewram[(addr - EWRAM_BASE) as usize] = value,
            IWRAM_BASE..=0x0300_7fff => self.iwram[(addr - IWRAM_BASE) as usize] = value,
            _ => self.bad_address = Some(addr),
        }
    }

    fn put16(&mut self, addr: u32, value: u16) {
        for (i, byte) in value.to_le_bytes().iter().enumerate() {
            self.set_byte(addr + i as u32, *byte);
        }
    }

    fn put32(&mut self, addr: u32, value: u32) {
        for (i, byte) in value.to_le_bytes().iter().enumerate() {
            self.set_byte(addr + i as u32, *byte);
        }
    }

    fn mon(
        &mut self,
        address: u32,
        species: u16,
        item: u16,
        ability_slot: u8,
        nature: u8,
        speed: u16,
        hp: u16,
    ) {
        let mut mon = NativeMon::default();
        mon.fields[11] = u32::from(species);
        mon.fields[12] = u32::from(item);
        mon.fields[46] = u32::from(ability_slot);
        self.mons.insert(address, mon);
        self.set_byte(address + 0x1f, nature);
        self.put16(address + 0x5e, speed);
        self.put16(address + 0x56, hp);
    }

    fn native(&mut self, start: u32, args: [u32; 4], stack: [u32; 2]) -> Result<u32> {
        let sp = 0x0300_7e00;
        self.put32(sp, stack[0]);
        self.put32(sp + 4, stack[1]);
        self.bad_address = None;
        self.random_calls = 0;
        let mut cpu = Cpu::new();
        cpu.reg_set(Mode::User, reg::CPSR, 0x30);
        cpu.reg_set(Mode::User, reg::PC, start);
        cpu.reg_set(Mode::User, reg::SP, sp);
        cpu.reg_set(Mode::User, reg::LR, END | 1);
        for (register, value) in args.into_iter().enumerate() {
            cpu.reg_set(Mode::User, register as u8, value);
        }
        for _ in 0..50_000 {
            let pc = cpu.reg_get(Mode::User, reg::PC);
            if pc == END {
                return Ok(cpu.reg_get(Mode::User, 0));
            }
            if pc == GET_MON_DATA || pc == SET_MON_DATA {
                let address = cpu.reg_get(Mode::User, 0);
                let field = cpu.reg_get(Mode::User, 1) as usize;
                if field >= 64 {
                    return Err(err("trainer_ev_native_field", field));
                }
                if pc == GET_MON_DATA {
                    let value = self
                        .mons
                        .get(&address)
                        .ok_or_else(|| err("trainer_ev_native_mon", address))?
                        .fields[field];
                    cpu.reg_set(Mode::User, 0, value);
                } else {
                    let ptr = cpu.reg_get(Mode::User, 2);
                    let value = if (26..=31).contains(&field) || (39..=44).contains(&field) {
                        u32::from(self.r8(ptr))
                    } else {
                        self.r32(ptr)
                    };
                    self.mons
                        .get_mut(&address)
                        .ok_or_else(|| err("trainer_ev_native_mon", address))?
                        .fields[field] = value;
                }
                cpu.reg_set(Mode::User, reg::PC, cpu.reg_get(Mode::User, reg::LR) & !1);
            } else if pc == VAR_GET {
                if cpu.reg_get(Mode::User, 0) != 0x409b {
                    return Err(err("trainer_ev_native_var", cpu.reg_get(Mode::User, 0)));
                }
                cpu.reg_set(Mode::User, 0, u32::from(self.difficulty));
                cpu.reg_set(Mode::User, reg::PC, cpu.reg_get(Mode::User, reg::LR) & !1);
            } else if pc == RANDOM {
                self.random_calls += 1;
                cpu.reg_set(Mode::User, 0, u32::from(self.rng_bit));
                cpu.reg_set(Mode::User, reg::PC, cpu.reg_get(Mode::User, reg::LR) & !1);
            } else if !cpu.step(self) {
                return Err(err("trainer_ev_native_instruction", format!("{pc:08X}")));
            }
            if let Some(address) = self.bad_address {
                return Err(err("trainer_ev_native_address", format!("{address:08X}")));
            }
        }
        Err(err("trainer_ev_native_limit", start))
    }
}

impl Memory for Sandbox<'_> {
    fn r8(&mut self, addr: u32) -> u8 {
        self.byte(addr)
    }
    fn r16(&mut self, addr: u32) -> u16 {
        u16::from_le_bytes([self.byte(addr), self.byte(addr + 1)])
    }
    fn r32(&mut self, addr: u32) -> u32 {
        u32::from_le_bytes([
            self.byte(addr),
            self.byte(addr + 1),
            self.byte(addr + 2),
            self.byte(addr + 3),
        ])
    }
    fn w8(&mut self, addr: u32, value: u8) {
        self.set_byte(addr, value);
    }
    fn w16(&mut self, addr: u32, value: u16) {
        self.put16(addr, value);
    }
    fn w32(&mut self, addr: u32, value: u32) {
        self.put32(addr, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(species: u16, speed: u16) -> PlayerBattleMon {
        PlayerBattleMon {
            species,
            held_item: 0,
            ability_slot: 0,
            nature: 13,
            speed,
            current_hp: 100,
        }
    }

    #[test]
    fn sampled_trainers_and_all_difficulties_remain_bounded() {
        let Ok(path) = std::env::var("GEN3_ULTIMATE_ROM") else {
            return;
        };
        let rom = Rom::open(std::fs::read(path).unwrap()).unwrap();
        let trainers: Vec<_> = rom
            .trainers()
            .unwrap()
            .into_iter()
            .filter(|t| {
                t.party.iter().all(|p| {
                    p.level > 0
                        && p.generation
                            .as_ref()
                            .is_some_and(|g| g.context == "ultimate_template")
                })
            })
            .step_by(20)
            .take(12)
            .collect();
        assert!(trainers.len() >= 8);
        for trainer in trainers {
            for difficulty in 1..=4 {
                let req = TrainerEvRequest {
                    trainer_id: trainer.id,
                    difficulty,
                    player_party: vec![player(6, 150), player(25, 120)],
                    opponent_levels: vec![],
                };
                let result = preview(&rom, &req).unwrap();
                assert_eq!(result.mons.len(), trainer.party.len());
            }
        }
    }

    #[test]
    fn trainer_preview_executes_loaded_rom() {
        let Ok(path) = std::env::var("GEN3_ULTIMATE_ROM") else {
            return;
        };
        let rom = Rom::open(std::fs::read(path).unwrap()).unwrap();
        let trainer = rom
            .trainers()
            .unwrap()
            .into_iter()
            .find(|t| {
                t.party.len() >= 2
                    && t.party.iter().all(|p| {
                        p.level > 0
                            && p.generation
                                .as_ref()
                                .is_some_and(|g| g.context == "ultimate_template")
                    })
            })
            .unwrap();
        let req = TrainerEvRequest {
            trainer_id: trainer.id,
            difficulty: 2,
            player_party: vec![player(6, 150)],
            opponent_levels: vec![],
        };
        let result = preview(&rom, &req).unwrap();
        assert_eq!(result.mons.len(), trainer.party.len());
        assert!(result.mons.iter().any(|m| m.evs.is_some()));
        let casual = preview(
            &rom,
            &TrainerEvRequest {
                difficulty: 1,
                ..req
            },
        )
        .unwrap();
        let lunatic = preview(
            &rom,
            &TrainerEvRequest {
                trainer_id: trainer.id,
                difficulty: 4,
                player_party: vec![player(6, 150)],
                opponent_levels: vec![],
            },
        )
        .unwrap();
        assert!(lunatic.mons.iter().any(|m| m.evs.is_some()));
        for mon in casual.mons.iter().filter(|m| m.evs.is_some()) {
            assert_eq!(mon.evs, Some([0; 6]));
        }
        let mut app = crate::app::App::default();
        app.session = Some(crate::session::Session::new(rom.clone()));
        let response = app
            .dispatch(crate::app::Request {
                command: "trainer_ev_preview".into(),
                payload: serde_json::json!({
                    "trainer_id": trainer.id,
                    "difficulty": 2,
                    "player_party": [{"species": 6, "held_item": 0, "ability_slot": 0,
                        "nature": 13, "speed": 150, "current_hp": 100}],
                    "opponent_levels": []
                }),
            })
            .unwrap();
        assert_eq!(
            response["mons"][0]["evs"],
            serde_json::json!(result.mons[0].evs)
        );
        if let Ok(path) = std::env::var("GEN3_ULTIMATE_SAVE") {
            let mut session = crate::session::Session::new(rom);
            session.load(std::fs::read(path).unwrap(), None).unwrap();
            let party: Vec<PlayerBattleMon> = session
                .snapshot()
                .unwrap()
                .pokemon
                .into_iter()
                .filter(|p| {
                    matches!(p.location, crate::save::Location::Party { .. }) && !p.pokemon.egg
                })
                .map(|p| PlayerBattleMon {
                    species: p.pokemon.species,
                    held_item: p.pokemon.held_item,
                    ability_slot: p.pokemon.ability_slot,
                    nature: p.pokemon.effective_nature,
                    speed: p.pokemon.stats[3],
                    current_hp: p.pokemon.current_hp.unwrap_or(0),
                })
                .collect();
            if !party.is_empty() {
                let from_save = preview(
                    &session.rom,
                    &TrainerEvRequest {
                        trainer_id: trainer.id,
                        difficulty: 3,
                        player_party: party,
                        opponent_levels: vec![],
                    },
                )
                .unwrap();
                assert_eq!(from_save.mons.len(), trainer.party.len());
            }
        }
    }

    #[test]
    fn native_ev_constructor_matches_reference_case() {
        let Ok(path) = std::env::var("GEN3_ULTIMATE_ROM") else {
            return;
        };
        let rom = std::fs::read(path).unwrap();
        let mut mem = Sandbox::new(&rom);
        mem.mon(MON, 6, 0, 0, 13, 150, 100);
        mem.mons.get_mut(&MON).unwrap().fields[13..17].copy_from_slice(&[53, 337, 76, 126]);
        mem.set_byte(0x0202_44e9, 1);
        mem.put32(0x0202_2fec, 0x100);
        mem.put16(0x0203_8bca, 0x483);
        mem.put16(SUMMARY, 100);
        mem.set_byte(SUMMARY + 3, 2);
        mem.native(TRAINER_EV_ROUTINE, [MON, SUMMARY, 0, 31], [252, 100])
            .unwrap();
        assert_eq!(mem.mons[&MON].fields[26..32], [4, 252, 0, 252, 0, 0]);
    }
}
