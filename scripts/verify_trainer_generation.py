"""Execute each exact ROM's NPC constructor and compare it with the adapter.

Optional local validation: requires Unicorn and GEN3_ROM_BW / GEN3_ROM_DP.
GEN3_CLI selects a built `gen3` executable. No ROM or generated party is saved.
This validates construction in isolated RAM, not a complete game battle.
"""
import json
import os
import struct
import subprocess
from pathlib import Path

from unicorn import Uc, UC_ARCH_ARM, UC_MODE_THUMB
from unicorn.arm_const import (
    UC_ARM_REG_SP, UC_ARM_REG_LR, UC_ARM_REG_R0, UC_ARM_REG_R1,
    UC_ARM_REG_R2, UC_ARM_REG_PC,
)

ORDERS = [
    'GAEM', 'GAME', 'GEAM', 'GEMA', 'GMAE', 'GMEA',
    'AGEM', 'AGME', 'AEGM', 'AEMG', 'AMGE', 'AMEG',
    'EGAM', 'EGMA', 'EAGM', 'EAMG', 'EMGA', 'EMAG',
    'MGAE', 'MGEA', 'MAGE', 'MAEG', 'MEGA', 'MEAG',
]
TRAINERS = [74, 261, 262, 263, 264, 335, 957, 993, 999, 1000]


def verify(rom, trainer):
    cpu = Uc(UC_ARCH_ARM, UC_MODE_THUMB)
    for address, size in [(0, 0x4000), (0x2000000, 0x40000),
                          (0x3000000, 0x8000), (0x4000000, 0x10000),
                          (0x8000000, 0x2000000)]:
        cpu.mem_map(address, size)
    cpu.mem_write(0x8000000, rom)
    cpu.mem_write(0x3005d8c, struct.pack('<II', 0x2030000, 0x2034000))
    cpu.mem_write(0x2022fec, struct.pack('<I', 8))  # Trainer battle.
    for i, level in enumerate([12, 45, 31, 70, 9, 1]):
        cpu.mem_write(0x2024540 + i * 100, bytes([level]))
    cpu.reg_write(UC_ARM_REG_SP, 0x3007000)
    cpu.reg_write(UC_ARM_REG_LR, 0x8000001)
    cpu.reg_write(UC_ARM_REG_R0, 0x2024744)
    cpu.reg_write(UC_ARM_REG_R1, trainer['id'])
    cpu.reg_write(UC_ARM_REG_R2, 0)
    cpu.emu_start(0x80385e9, 0x8000000, count=30_000_000)
    assert cpu.reg_read(UC_ARM_REG_PC) == 0x8000000, 'instruction budget exhausted'
    assert cpu.reg_read(UC_ARM_REG_R0) == len(trainer['party'])
    raw = bytes(cpu.mem_read(0x2024744, 600))
    for i, expected in enumerate(trainer['party']):
        p = raw[i * 100:(i + 1) * 100]
        pid, ot = struct.unpack_from('<II', p)
        decoded = b''.join(struct.pack('<I', x ^ pid ^ ot)
                           for x in struct.unpack('<12I', p[32:80]))
        assert sum(struct.unpack('<24H', decoded)) & 0xffff == struct.unpack_from('<H', p, 28)[0]
        blocks = {k: decoded[j * 12:j * 12 + 12] for j, k in enumerate(ORDERS[pid % 24])}
        species = struct.unpack_from('<H', blocks['G'])[0]
        iv_word = struct.unpack_from('<I', blocks['M'], 4)[0]
        base = 0x3203cc + species * 28
        ratio = rom[base + 16]
        gender = 'genderless' if ratio == 255 else 'female' if ratio == 254 or pid % 256 < ratio else 'male'
        ability = rom[base + 22 + (iv_word >> 31)]
        actual = {
            'gender': gender, 'nature': pid % 25, 'ability_id': ability,
            'ivs': [(iv_word >> (j * 5)) & 31 for j in range(6)],
            'evs': list(blocks['E'][:6]),
        }
        g = expected['generation']
        for field, value in actual.items():
            assert value == g[field], (trainer['id'], i, field, value, g[field])
        assert species == expected['species']
        assert p[84] == (70 if expected['level_rule'] == 'party_max' else expected['level'])
        if expected['moves_explicit']:
            assert list(struct.unpack('<4H', blocks['A'][:8])) == expected['moves']


if __name__ == '__main__':
    total = 0
    for key in ['GEN3_ROM_BW', 'GEN3_ROM_DP']:
        path = os.environ[key]
        # The CLI rejects unknown MD5/length before executing any ROM code.
        world = json.loads(subprocess.check_output([os.environ.get('GEN3_CLI', 'gen3'), 'world', path]))
        rom = Path(path).read_bytes()
        for trainer in world['trainers']:
            if trainer['id'] in TRAINERS:
                verify(rom, trainer)
                total += len(trainer['party'])
        print(f'{key}: {len(TRAINERS)} trainer parties match the ROM constructor')
    print(f'{total} generated Pokémon verified')
