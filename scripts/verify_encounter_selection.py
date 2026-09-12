"""Execute the supported ROMs' encounter selectors in isolated ARM memory.

Requires Unicorn and user-supplied GEN3_ROM_BW / GEN3_ROM_DP. The ROMs and saves
are never modified. Exhaustive slot rolls and bounded map-header lookup tests
also reject memory reads outside the selectors' verified inputs. This checks
selection, not a full emulator playthrough or all scripted battle conditions.
"""

import hashlib
import os
import struct
from collections import Counter
from pathlib import Path

from unicorn import Uc, UC_ARCH_ARM, UC_HOOK_MEM_READ, UC_MODE_THUMB, UC_PROT_READ, UC_PROT_EXEC
from unicorn.arm_const import UC_ARM_REG_LR, UC_ARM_REG_PC, UC_ARM_REG_R0, UC_ARM_REG_SP

PROFILES = {
    "GEN3_ROM_BW": "0d9b129f7dd76895f79bb47ad7dec2fe",
    "GEN3_ROM_DP": "cb2940215f4dafb1bef133c3af379f44",
}
ROM_BASE = 0x08000000
WILD = 0xEA2D34
SAVE = 0x02030000
SAVE_POINTER = 0x03005D8C
RNG = 0x03005D80
RNG_CALLS = 0x020249C0
STACK = (0x03006000, 0x03007000)
RETURN = 0x08001000
ALTERING_CAVE = (24, 106)
ALTERING_VAR = SAVE + 0x139C + (0x403E - 0x4000) * 2
LAND_WEIGHTS = [20, 20, 10, 10, 10, 10, 5, 5, 4, 4, 1, 1]
WATER_WEIGHTS = [60, 30, 5, 4, 1]
ROD_WEIGHTS = [[70, 30], [60, 20, 20], [40, 40, 15, 4, 1]]


def u32(data, offset):
    return struct.unpack_from("<I", data, offset)[0]


def headers(rom):
    result = []
    for index in range(600):
        offset = WILD + index * 20
        group, number = rom[offset:offset + 2]
        if group == 255:
            return result
        tables = [u32(rom, offset + field) for field in (4, 8, 12, 16)]
        for pointer in tables:
            assert not pointer or ROM_BASE <= pointer < ROM_BASE + len(rom) - 8
        result.append(((group, number), tables))
    raise AssertionError("wild header terminator missing")


class SelectorCPU:
    def __init__(self, rom):
        self.cpu = Uc(UC_ARCH_ARM, UC_MODE_THUMB)
        for address, size in [(0x02000000, 0x40000), (0x03000000, 0x8000),
                              (ROM_BASE, 0x2000000)]:
            self.cpu.mem_map(address, size)
        self.cpu.mem_write(ROM_BASE, rom)
        self.cpu.mem_protect(ROM_BASE, 0x2000000, UC_PROT_READ | UC_PROT_EXEC)
        self.cpu.mem_write(SAVE_POINTER, struct.pack("<I", SAVE))
        self.inputs = []
        self.reads = set()
        self.unexpected = []
        self.cpu.hook_add(UC_HOOK_MEM_READ, self.read)

    def read(self, cpu, access, address, size, value, user_data):
        # ROM literals and the stack are expected. GPIO/RTC registers at the
        # beginning of cartridge space are deliberately outside this range.
        allowed = [(ROM_BASE + 0x1000, ROM_BASE + 0x2000000), STACK] + self.inputs
        if not any(start <= address and address + size <= end for start, end in allowed):
            self.unexpected.append((cpu.reg_read(UC_ARM_REG_PC), address, size))
            cpu.emu_stop()
        self.reads.add((address, size))

    def call(self, offset, argument=0, inputs=()):
        self.inputs = list(inputs)
        self.reads.clear()
        self.unexpected.clear()
        self.cpu.reg_write(UC_ARM_REG_R0, argument)
        self.cpu.reg_write(UC_ARM_REG_SP, STACK[1])
        self.cpu.reg_write(UC_ARM_REG_LR, RETURN | 1)
        self.cpu.emu_start(ROM_BASE + offset + 1, RETURN, count=100_000)
        assert not self.unexpected, f"unexpected input read: {self.unexpected}"
        assert self.cpu.reg_read(UC_ARM_REG_PC) == RETURN, "instruction budget exhausted"
        return self.cpu.reg_read(UC_ARM_REG_R0)

    def roll(self, offset, value, argument=0):
        # Choose a seed whose next *unmodified ROM* Random() result is value.
        seed = (((value << 16) - 0x6073) * pow(0x41C64E6D, -1, 1 << 32)) & 0xFFFFFFFF
        self.cpu.mem_write(RNG, struct.pack("<I", seed))
        self.cpu.mem_write(RNG_CALLS, bytes(4))
        return self.call(offset, argument, [(RNG, RNG + 4), (RNG_CALLS, RNG_CALLS + 4)])


def verify(rom):
    # Fail if the engine audited in the accompanying research note changes.
    assert hashlib.sha256(rom[0xB4880:0xB5B3C]).hexdigest() == (
        "2e71a4b6ff571679e4165d5e2061c85f253ae02e0067a12328b0dae6fdcfc698"
    )
    records = headers(rom)
    first = {}
    for index, (map_id, _) in enumerate(records):
        first.setdefault(map_id, index)
    engine = SelectorCPU(rom)
    map_inputs = [(SAVE_POINTER, SAVE_POINTER + 4), (SAVE + 4, SAVE + 6)]
    for map_id, expected in first.items():
        engine.cpu.mem_write(SAVE + 4, bytes(map_id))
        engine.cpu.mem_write(ALTERING_VAR, bytes(2))
        inputs = map_inputs + ([(ALTERING_VAR, ALTERING_VAR + 2)] if map_id == ALTERING_CAVE else [])
        assert engine.call(0xB4CF8, inputs=inputs) == expected, map_id
    engine.cpu.mem_write(SAVE + 4, bytes([127, 127]))
    assert engine.call(0xB4CF8, inputs=map_inputs) == 0xFFFF

    cave_checks = 0
    if ALTERING_CAVE in first:
        engine.cpu.mem_write(SAVE + 4, bytes(ALTERING_CAVE))
        for value in [*range(10), 65535]:
            engine.cpu.mem_write(ALTERING_VAR, struct.pack("<H", value))
            actual = engine.call(0xB4CF8, inputs=map_inputs + [(ALTERING_VAR, ALTERING_VAR + 2)])
            assert actual == first[ALTERING_CAVE] + (value if value <= 8 else 0)
            cave_checks += 1

    land = [engine.roll(0xB4AC8, value) for value in range(100)]
    water = [engine.roll(0xB4B84, value) for value in range(100)]
    assert Counter(land) == Counter(dict(enumerate(LAND_WEIGHTS)))
    assert Counter(water) == Counter(dict(enumerate(WATER_WEIGHTS)))
    for rod, start in enumerate([0, 2, 5]):
        actual = Counter(engine.roll(0xB4BD8, value, rod) for value in range(100))
        assert actual == Counter({start + i: weight for i, weight in enumerate(ROD_WEIGHTS[rod])})

    # Decode the ROM's actual Safari rows using the executed slot selector.
    expected = {(26, 1): {(185, 31): 1}, (26, 13): {(192, 36): 4, (192, 39): 1}}
    for map_id, targets in expected.items():
        table = records[first[map_id]][1][0] - ROM_BASE
        entries = u32(rom, table + 4) - ROM_BASE
        actual = Counter()
        for slot in land:
            low, high, species = struct.unpack_from("<BBH", rom, entries + slot * 4)
            if species in (185, 192):
                assert low == high
                actual[(species, low)] += 1
        assert actual == Counter(targets), (map_id, actual)
    return len(records), len(first) + 1, cave_checks


if __name__ == "__main__":
    supplied = 0
    for key, md5 in PROFILES.items():
        if key not in os.environ:
            continue
        rom = Path(os.environ[key]).read_bytes()
        assert hashlib.md5(rom).hexdigest() == md5, f"unrecognized ROM: {key}"
        count, maps, caves = verify(rom)
        print(f"{key}: {count} headers; {maps} map lookups; {caves} cave-variable cases; "
              "500 slot rolls; no reads outside verified non-clock inputs")
        supplied += 1
    if not supplied:
        raise SystemExit("Set GEN3_ROM_BW and/or GEN3_ROM_DP to user-supplied ROM paths.")
