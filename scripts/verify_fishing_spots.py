"""Compare fishing coordinates against both exact ROMs' original Thumb code.

Requires Unicorn, GEN3_ROM_BW, GEN3_ROM_DP and GEN3_CLI. Uses generated, rotated
save banks with different seeds; no user save, copyrighted fixture or ROM edit.
Only the player's facing tile is supplied by a hook. RNG, water classification,
section numbering and special encounter selection execute unmodified ROM code.
"""
import hashlib
import json
import os
import struct
import subprocess
import tempfile
from pathlib import Path

from unicorn import Uc, UC_ARCH_ARM, UC_MODE_THUMB, UC_HOOK_CODE, UC_PROT_READ, UC_PROT_EXEC
from unicorn.arm_const import UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_SP, UC_ARM_REG_LR, UC_ARM_REG_PC
from verify_encounter_selection import PROFILES

SIZES = [3884, 3968, 3968, 3968, 3848] + [3968] * 8 + [2000]


def save_bytes(seed):
    data = bytearray(0x20000)
    for bank in range(2):
        main = bytearray(sum(SIZES[1:5]))
        struct.pack_into('<H', main, 0x2e6a, seed if bank else seed ^ 0xffff)
        for physical in range(14):
            section = (physical + 4) % 14
            start = bank * 14 * 4096 + physical * 4096
            if 1 <= section <= 4:
                offset = sum(SIZES[1:section])
                data[start:start + SIZES[section]] = main[offset:offset + SIZES[section]]
            total = sum(struct.unpack_from('<I', data, o)[0]
                        for o in range(start, start + SIZES[section], 4)) & 0xffffffff
            checksum = ((total >> 16) + (total & 0xffff)) & 0xffff
            struct.pack_into('<HHII', data, start + 0xff4, section, checksum, 0x08012025, 10 + bank)
    return bytes(data)


class NativeFishing:
    def __init__(self, rom, map_row):
        self.cpu = c = Uc(UC_ARCH_ARM, UC_MODE_THUMB)
        for address, size in [(0x8000000, 0x2000000), (0x2000000, 0x40000), (0x3000000, 0x8000)]:
            c.mem_map(address, size)
        c.mem_write(0x8000000, rom)
        c.mem_protect(0x8000000, 0x2000000, UC_PROT_READ | UC_PROT_EXEC)
        c.mem_write(0x3005d8c, struct.pack('<I', 0x2030000))
        c.mem_write(0x2030004, bytes([0, 34]))
        c.mem_write(0x2037318, rom[map_row['header']:map_row['header'] + 28])
        width, height = map_row['width'], map_row['height']
        block = struct.unpack_from('<I', rom, map_row['layout'] + 12)[0] - 0x8000000
        grid = bytearray((width + 15) * (height + 14) * 2)
        for y in range(height):
            start = ((y + 7) * (width + 15) + 7) * 2
            grid[start:start + width * 2] = rom[block + y * width * 2:block + (y + 1) * width * 2]
        grid_header = struct.unpack_from('<I', rom, 0x88250)[0]
        c.mem_write(grid_header, struct.pack('<III', width + 15, height + 14, 0x2000000))
        c.mem_write(0x2000000, bytes(grid))
        self.front = (0, 0)
        c.hook_add(UC_HOOK_CODE, self.facing_tile, begin=0x808ba68, end=0x808ba68)
        self.water = [(x, y) for y in range(height) for x in range(width)
                      if self.call(0x89660, self.call(0x882bc, x + 7, y + 7))]

    def facing_tile(self, cpu, *_):
        cpu.mem_write(cpu.reg_read(UC_ARM_REG_R0), struct.pack('<H', self.front[0] + 7))
        cpu.mem_write(cpu.reg_read(UC_ARM_REG_R1), struct.pack('<H', self.front[1] + 7))
        cpu.reg_write(UC_ARM_REG_PC, cpu.reg_read(UC_ARM_REG_LR))

    def call(self, offset, *args):
        for reg, value in zip([UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2], args):
            self.cpu.reg_write(reg, value)
        self.cpu.reg_write(UC_ARM_REG_SP, 0x3007000)
        self.cpu.reg_write(UC_ARM_REG_LR, 0x8001001)
        self.cpu.emu_start(0x8000001 + offset, 0x8001000, count=1_000_000)
        assert self.cpu.reg_read(UC_ARM_REG_PC) == 0x8001000, 'instruction budget exceeded'
        return self.cpu.reg_read(UC_ARM_REG_R0)

    def check(self, point, roll):
        self.front = point
        # Seed the real Random() so its next output is the requested roll.
        rng = (((roll << 16) - 0x6073) * pow(0x41c64e6d, -1, 1 << 32)) & 0xffffffff
        self.cpu.mem_write(0x3005d80, struct.pack('<I', rng))
        return self.call(0xb4984)


def main():
    cli = os.environ['GEN3_CLI']
    for env, md5 in PROFILES.items():
        path = Path(os.environ[env]); rom = path.read_bytes()
        assert hashlib.md5(rom).hexdigest() == md5
        world = json.loads(subprocess.check_output([cli, 'world', str(path)]))
        native = NativeFishing(rom, next(m for m in world['maps'] if m['id'] == '0-34'))
        no_save = json.loads(subprocess.check_output([cli, 'fishing-spots', str(path)]))
        assert no_save['seed'] is None and no_save['spots'] == []
        assert (no_save['species'], no_save['min_level'], no_save['max_level'], no_save['percent']) == (328, 20, 25, 50)
        with tempfile.TemporaryDirectory() as tmp:
            save = Path(tmp) / 'generated.sav'
            for seed in [0, 1, 0x1234, 0xffff]:
                before = save_bytes(seed); save.write_bytes(before)
                report = json.loads(subprocess.check_output([cli, 'fishing-spots', str(path), str(save)]))
                assert save.read_bytes() == before
                assert report['seed'] == seed  # Newer bank, despite physical sector rotation.
                native.cpu.mem_write(0x2030000 + 0x2e6a, struct.pack('<H', seed))
                expected = {point for point in native.water if native.check(point, 0)}
                actual = {(p['x'], p['y']) for p in report['spots']}
                assert actual == expected, (env, seed, actual, expected)
                for point in actual:
                    assert sum(native.check(point, roll) for roll in range(100)) == 50
                print(f'{env}: seed {seed:#06x}, {len(actual)} tiles; native coordinates and 50/100 rolls match', flush=True)
        assert path.read_bytes() == rom


if __name__ == '__main__':
    main()
