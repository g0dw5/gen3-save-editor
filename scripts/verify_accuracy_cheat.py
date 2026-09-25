"""Run the exact Ultimate Emerald 5.5 accuracy branches with and without the cheat.

Requires GEN3_ROM_ULTIMATE and the Unicorn Python package. No input is written.
The two unrelated callees are controlled; the original ROM branch instructions run.
"""

import hashlib
import os
from pathlib import Path

from unicorn import UC_ARCH_ARM, UC_HOOK_CODE, UC_MODE_THUMB, Uc
from unicorn.arm_const import (
    UC_ARM_REG_LR,
    UC_ARM_REG_PC,
    UC_ARM_REG_R0,
    UC_ARM_REG_R1,
    UC_ARM_REG_R4,
    UC_ARM_REG_R5,
    UC_ARM_REG_SP,
)

ROM_MD5 = "17ce9785b33319b3dbda9a5d37c57ec1"
rom = Path(os.environ["GEN3_ROM_ULTIMATE"]).read_bytes()
assert hashlib.md5(rom).hexdigest() == ROM_MD5
assert rom[0x1D48CBC:0x1D48CBE] == bytes.fromhex("05d5")
assert rom[0x1D48CCC:0x1D48CCE] == bytes.fromhex("0ad4")


def run(patched, difficulty, trainer, target, base=80):
    cpu = Uc(UC_ARCH_ARM, UC_MODE_THUMB)
    cpu.mem_map(0x08000000, 0x02000000)
    cpu.mem_write(0x08000000, rom)
    if patched:
        cpu.mem_write(0x09D48CBC, bytes.fromhex("05d4"))
        cpu.mem_write(0x09D48CCC, bytes.fromhex("0ad5"))
    cpu.mem_map(0x02000000, 0x40000)
    cpu.mem_map(0x03000000, 0x10000)
    cpu.mem_write(0x02022FEC, bytes([8 if trainer else 0]))
    cpu.reg_write(UC_ARM_REG_R4, base)
    cpu.reg_write(UC_ARM_REG_R5, target)
    cpu.reg_write(UC_ARM_REG_SP, 0x03007F00)

    def hook(uc, address, _size, _unused):
        if address == 0x09D48E8C:  # Save variable 0x409B.
            uc.reg_write(UC_ARM_REG_R0, difficulty)
            uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))
        elif address == 0x09D5E294:  # ROM's verified relative-percent helper.
            value = uc.reg_read(UC_ARM_REG_R0)
            percent = uc.reg_read(UC_ARM_REG_R1)
            uc.reg_write(UC_ARM_REG_R0, value * (100 + percent) // 100)
            uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))

    cpu.hook_add(UC_HOOK_CODE, hook)
    cpu.emu_start(0x09D48CB3, 0x09D48CE4, count=100)
    return cpu.reg_read(UC_ARM_REG_R4)


cases = 0
for patched in (False, True):
    for difficulty in (1, 2, 3, 4):
        for trainer in (False, True):
            for target in (0, 1, 2, 3):
                result = run(patched, difficulty, trainer, target)
                bonus = difficulty == 1 and target % 2 == int(patched)
                bonus |= trainer and difficulty == 4 and target % 2 != int(patched)
                assert result == (96 if bonus else 80), (
                    patched, difficulty, trainer, target, result
                )
                cases += 1
for base in (50, 90, 100):
    assert run(True, 1, True, 1, base) == base * 120 // 100
    assert run(True, 4, True, 0, base) == base * 120 // 100
    cases += 2

print({
    "native_branch_cases": cases,
    "casual_player_attack": run(True, 1, True, 1),
    "casual_opponent_attack": run(True, 1, True, 0),
    "lunatic_player_attack": run(True, 4, True, 1),
    "lunatic_opponent_attack": run(True, 4, True, 0),
    "lunatic_wild_attack": run(True, 4, False, 0),
})
