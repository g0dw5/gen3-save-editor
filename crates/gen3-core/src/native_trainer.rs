//! Read-only execution of a ROM's party constructor in bounded, isolated GBA RAM.
//! This deliberately exposes a named baseline scenario, not the complete battle setup.
use crate::{err, pokemon, rom::Rom, Result};
use armv4t_emu::{reg, Cpu, Memory, Mode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct NativeTrainerRules {
    pub constructor: u32,
    pub enemy_party: u32,
    pub battle_flags: u32,
    pub save_pointers: [u32; 2],
    pub rng: u32,
    pub instruction_limit: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub trainer_id: u16,
    #[serde(default)]
    pub seed: u32,
}
#[derive(Serialize)]
pub struct Preview {
    pub rom_md5: &'static str,
    pub trainer_id: u16,
    pub seed: u32,
    pub scenario: &'static str,
    pub partial: bool,
    pub mons: Vec<pokemon::Pokemon>,
}

pub fn preview(rom: &Rom, request: &Request) -> Result<Preview> {
    let rules = rom
        .profile
        .native_trainers
        .ok_or_else(|| err("trainer_native_unverified", rom.profile.id))?;
    let trainer = rom
        .trainers()?
        .into_iter()
        .find(|t| t.id == request.trainer_id)
        .ok_or_else(|| err("trainer_id", request.trainer_id))?;
    let mut ram = Sandbox::new(&rom.data);
    ram.w32(rules.save_pointers[0], 0x02030000);
    ram.w32(rules.save_pointers[1], 0x02034000);
    ram.w32(rules.battle_flags, 8 | u32::from(trainer.double_battle));
    ram.w32(rules.rng, request.seed);
    let count = ram.call(
        rules.constructor,
        [rules.enemy_party, u32::from(request.trainer_id), 0, 1],
        [0, 1],
        rules.instruction_limit,
    )? as usize;
    if count > 6 || count != trainer.party.len() {
        return Err(err(
            "trainer_native_count",
            format!("{count}/{}", trainer.party.len()),
        ));
    }
    let mons = (0..count)
        .map(|i| {
            let start = (rules.enemy_party - 0x02000000) as usize + i * 100;
            let raw = ram
                .ewram
                .get(start..start + 100)
                .ok_or_else(|| err("trainer_native_party", start))?;
            pokemon::decode(raw, rom)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Preview {
        rom_md5: rom.profile.md5,
        trainer_id: request.trainer_id,
        seed: request.seed,
        scenario: "ordinary_zero_context",
        partial: true,
        mons,
    })
}

struct Sandbox<'a> {
    rom: &'a [u8],
    ewram: Vec<u8>,
    iwram: Vec<u8>,
    invalid: Option<u32>,
}
impl<'a> Sandbox<'a> {
    fn new(rom: &'a [u8]) -> Self {
        Self {
            rom,
            ewram: vec![0; 0x40000],
            iwram: vec![0; 0x8000],
            invalid: None,
        }
    }
    fn call(&mut self, start: u32, args: [u32; 4], stack: [u32; 2], limit: usize) -> Result<u32> {
        let sp = 0x03007e00;
        self.w32(sp, stack[0]);
        self.w32(sp + 4, stack[1]);
        let mut cpu = Cpu::new();
        cpu.reg_set(Mode::User, reg::CPSR, 0x30);
        cpu.reg_set(Mode::User, reg::PC, start);
        cpu.reg_set(Mode::User, reg::SP, sp);
        cpu.reg_set(Mode::User, reg::LR, 0x0f000001);
        for (i, v) in args.into_iter().enumerate() {
            cpu.reg_set(Mode::User, i as u8, v);
        }
        for _ in 0..limit {
            let pc = cpu.reg_get(Mode::User, reg::PC);
            if pc == 0x0f000000 {
                return Ok(cpu.reg_get(Mode::User, 0));
            }
            if !cpu.step(self) {
                return Err(err("trainer_native_instruction", format!("{pc:08X}")));
            }
            if let Some(a) = self.invalid {
                return Err(err("trainer_native_address", format!("{a:08X}")));
            }
        }
        Err(err("trainer_native_limit", limit))
    }
}
impl Memory for Sandbox<'_> {
    fn r8(&mut self, a: u32) -> u8 {
        let byte = match a {
            0x08000000..=0x09ffffff => self.rom.get((a - 0x08000000) as usize),
            0x02000000..=0x0203ffff => self.ewram.get((a - 0x02000000) as usize),
            0x03000000..=0x03007fff => self.iwram.get((a - 0x03000000) as usize),
            _ => None,
        };
        match byte {
            Some(v) => *v,
            None => {
                self.invalid = Some(a);
                0
            }
        }
    }
    fn r16(&mut self, a: u32) -> u16 {
        u16::from_le_bytes([self.r8(a), self.r8(a.wrapping_add(1))])
    }
    fn r32(&mut self, a: u32) -> u32 {
        u32::from_le_bytes(std::array::from_fn(|i| self.r8(a.wrapping_add(i as u32))))
    }
    fn w8(&mut self, a: u32, v: u8) {
        match a {
            0x02000000..=0x0203ffff => self.ewram[(a - 0x02000000) as usize] = v,
            0x03000000..=0x03007fff => self.iwram[(a - 0x03000000) as usize] = v,
            _ => self.invalid = Some(a),
        }
    }
    fn w16(&mut self, a: u32, v: u16) {
        for (i, b) in v.to_le_bytes().iter().enumerate() {
            self.w8(a.wrapping_add(i as u32), *b);
        }
    }
    fn w32(&mut self, a: u32, v: u32) {
        for (i, b) in v.to_le_bytes().iter().enumerate() {
            self.w8(a.wrapping_add(i as u32), *b);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_memory_rejects_rom_writes_and_unmapped_reads() {
        let bytes = [7];
        let mut ram = Sandbox::new(&bytes);
        ram.w8(0x08000000, 9);
        assert_eq!(bytes, [7]);
        assert_eq!(ram.invalid, Some(0x08000000));
        ram.invalid = None;
        ram.r8(0x04000000);
        assert_eq!(ram.invalid, Some(0x04000000));
    }
    #[test]
    fn native_execution_is_bounded() {
        // Thumb: B .
        let mut ram = Sandbox::new(&[0xfe, 0xe7]);
        assert_eq!(
            ram.call(0x08000000, [0; 4], [0; 2], 32).unwrap_err().code,
            "trainer_native_limit"
        );
    }
}

#[cfg(test)]
mod local_tests {
    use super::*;
    #[test]
    #[ignore = "requires exact private Mercury 1.2 ROM; probe output stays local"]
    fn local_native_trainer_mercury() {
        let rom = Rom::open(std::fs::read(std::env::var("GEN3_ROM_MERCURY12").unwrap()).unwrap())
            .unwrap();
        let original = rom.data.clone();
        let mut probes = vec![];
        for trainer in rom.trainers().unwrap() {
            if trainer.party.is_empty() {
                continue;
            }
            for seed in [0, 0x12345678] {
                let request = Request {
                    trainer_id: trainer.id,
                    seed,
                };
                let result = preview(&rom, &request)
                    .unwrap_or_else(|e| panic!("trainer {} seed {seed}: {e}", trainer.id));
                assert_eq!(result.mons.len(), trainer.party.len());
                assert!(result
                    .mons
                    .iter()
                    .all(|p| p.species > 0 && p.level > 0 && p.ivs.iter().all(|iv| *iv <= 31)));
                probes.push(result);
            }
        }
        assert_eq!(rom.data, original);
        if let Ok(path) = std::env::var("GEN3_NATIVE_TRAINER_PROBES") {
            std::fs::write(path, serde_json::to_vec(&probes).unwrap()).unwrap();
        }
        println!("{} read-only native scenarios", probes.len());
    }
}
