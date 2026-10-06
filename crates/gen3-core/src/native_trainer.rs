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

/// Bounded read-only ROM execution shared by trainer, teaching and resource queries.
pub(crate) struct Sandbox<'a> {
    rom: &'a [u8],
    ewram: Vec<u8>,
    iwram: Vec<u8>,
    invalid: Option<u32>,
}
impl<'a> Sandbox<'a> {
    pub(crate) fn new(rom: &'a [u8]) -> Self {
        Self {
            rom,
            ewram: vec![0; 0x40000],
            iwram: vec![0; 0x8000],
            invalid: None,
        }
    }
    /// armv4t_emu 0.1.3 masks away the low address bit for Thumb halfword
    /// loads. GBA LDRH instead rotates the aligned value in a 32-bit register;
    /// odd LDRSH sign-extends the addressed byte. Correct only those accesses,
    /// leaving aligned instructions and native function control flow intact.
    fn step_unaligned_thumb_halfword(&mut self, cpu: &mut Cpu) -> bool {
        if cpu.reg_get(Mode::User, reg::CPSR) & 0x20 == 0 {
            return false;
        }
        let pc = cpu.reg_get(Mode::User, reg::PC);
        let instruction = self.r16(pc);
        let (address, signed) = match instruction & 0xf800 {
            0x8800 => {
                let base = ((instruction >> 3) & 7) as u8;
                (
                    cpu.reg_get(Mode::User, base)
                        .wrapping_add(u32::from((instruction >> 6) & 31) * 2),
                    false,
                )
            }
            0x5800 if matches!(instruction & 0xfe00, 0x5a00 | 0x5e00) => {
                let base = ((instruction >> 3) & 7) as u8;
                let offset = ((instruction >> 6) & 7) as u8;
                (
                    cpu.reg_get(Mode::User, base)
                        .wrapping_add(cpu.reg_get(Mode::User, offset)),
                    instruction & 0x0400 != 0,
                )
            }
            _ => return false,
        };
        if address & 1 == 0 {
            return false;
        }
        let value = if signed {
            self.r8(address) as i8 as i32 as u32
        } else {
            u32::from(self.r16(address & !1)).rotate_right(8)
        };
        cpu.reg_set(Mode::User, (instruction & 7) as u8, value);
        cpu.reg_set(Mode::User, reg::PC, pc.wrapping_add(2));
        true
    }
    pub(crate) fn call(
        &mut self,
        start: u32,
        args: [u32; 4],
        stack: [u32; 2],
        limit: usize,
    ) -> Result<u32> {
        Ok(self.observe(start, args, stack, limit, 0x0f000000)?[0])
    }
    /// Stop before a verified instruction and inspect registers without replacing
    /// a native function or changing its control flow. A premature return fails.
    pub(crate) fn observe(
        &mut self,
        start: u32,
        args: [u32; 4],
        stack: [u32; 2],
        limit: usize,
        stop: u32,
    ) -> Result<[u32; 16]> {
        Ok(self.observe_any(start, args, stack, limit, &[stop])?.0)
    }
    /// Observe one of several verified control-flow boundaries before UI effects.
    /// A premature return, invalid access or execution limit never means rejection.
    pub(crate) fn observe_any(
        &mut self,
        start: u32,
        args: [u32; 4],
        stack: [u32; 2],
        limit: usize,
        stops: &[u32],
    ) -> Result<([u32; 16], u32)> {
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
            if stops.contains(&pc) {
                return Ok((
                    std::array::from_fn(|i| cpu.reg_get(Mode::User, i as u8)),
                    pc,
                ));
            }
            if pc == 0x0f000000 {
                return Err(err(
                    "native_observation_missing",
                    if stops.len() == 1 {
                        format!("{:08X}", stops[0])
                    } else {
                        format!("{stops:08X?}")
                    },
                ));
            }
            if !self.step_unaligned_thumb_halfword(&mut cpu) && !cpu.step(self) {
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
    fn observations_preserve_native_execution_and_reject_missing_checkpoints() {
        let code = [7, 0x20, 13, 0x24, 0x70, 0x47]; // MOVS r0,#7; MOVS r4,#13; BX lr
        let mut ram = Sandbox::new(&code);
        let regs = ram
            .observe(0x08000000, [0; 4], [0; 2], 10, 0x08000004)
            .unwrap();
        assert_eq!((regs[0], regs[4], regs[15]), (7, 13, 0x08000004));
        let (regs, boundary) = ram
            .observe_any(0x08000000, [0; 4], [0; 2], 10, &[0x08000004, 0x08000002])
            .unwrap();
        assert_eq!(boundary, 0x08000002);
        assert_eq!((regs[0], regs[4]), (7, 0));
        assert_eq!(
            ram.observe_any(0x08000000, [0; 4], [0; 2], 10, &[0x08000008, 0x0800000a])
                .unwrap_err()
                .code,
            "native_observation_missing"
        );

        assert_eq!(ram.call(0x08000000, [0; 4], [0; 2], 10).unwrap(), 7);
        assert_eq!(
            ram.observe(0x08000000, [0; 4], [0; 2], 10, 0x08000008)
                .unwrap_err()
                .code,
            "native_observation_missing"
        );
        assert_eq!(
            ram.observe(0x08000000, [0; 4], [0; 2], 1, 0x08000004)
                .unwrap_err()
                .code,
            "trainer_native_limit"
        );
        assert_eq!(code, [7, 0x20, 13, 0x24, 0x70, 0x47]);
    }
    #[test]
    fn gba_thumb_odd_halfword_loads_rotate_or_sign_extend_without_changing_flags() {
        for instruction in [0x8808u16, 0x5a88, 0x5e88] {
            let mut rom = instruction.to_le_bytes().to_vec();
            rom.extend(0x4770u16.to_le_bytes());
            let mut ram = Sandbox::new(&rom);
            ram.w8(0x02001000, 0x80);
            ram.w8(0x02001001, 0xff);
            let even = ram
                .call(0x08000000, [0, 0x02001000, 0, 0], [0; 2], 10)
                .unwrap();
            assert_eq!(
                even,
                if instruction == 0x5e88 {
                    0xffffff80
                } else {
                    0xff80
                }
            );
            let odd = ram
                .call(0x08000000, [0, 0x02001001, 0, 0], [0; 2], 10)
                .unwrap();
            assert_eq!(
                odd,
                if instruction == 0x5e88 {
                    0xffffffff
                } else {
                    0x800000ff
                }
            );
            let mut cpu = Cpu::new();
            cpu.reg_set(Mode::User, reg::CPSR, 0xa0000030);
            cpu.reg_set(Mode::User, reg::PC, 0x08000000);
            cpu.reg_set(Mode::User, 1, 0x02001001);
            assert!(ram.step_unaligned_thumb_halfword(&mut cpu));
            assert_eq!(cpu.reg_get(Mode::User, reg::CPSR), 0xa0000030);
            assert_eq!(cpu.reg_get(Mode::User, reg::PC), 0x08000002);
        }
    }
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
