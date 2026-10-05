#!/usr/bin/env python3
"""Compare ordinary NPC trade generation with exact native routines.

Requires five GEN3_ROM_* paths and Unicorn. Writes only synthetic emulated RAM;
JSON on stdout is private parity evidence, never release content. The offered
party member's native level getter is injected. This checks quote/generation,
not NPC access, trade animation/evolution, receipt flags or special-mode output.
"""
import hashlib
import json
import os
import struct
import sys
from pathlib import Path

from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_LR, UC_ARM_REG_PC, UC_ARM_REG_R0, UC_ARM_REG_R1
from verify_event_state import PROFILES
from verify_pickup_receipts import RULES, SAVE, TRAINER_SAVE
from verify_rocket_battle_forms import Native

# Engine dispatch/layout addresses only. Read all offer content from each ROM.
CONFIG = {
    'BW': (0x1DBA64, 0xFF, 4, 0x6A518, 0x03005D90, 0x03005D80),
    'DP': (0x1DBA64, 0xFF, 4, 0x6A518, 0x03005D90, 0x03005D80),
    'ULTIMATE': (0x1DBA64, 0xFF, 8, 0x6A518, 0x03005D90, 0x03005D80),
    'ROCKET': (0x22B620, 0xFF, 4, 0x976D0, 0x03005250, 0x03005240),
    'MERCURY12': (0x15FD60, 0xFC, 9, 0x3FBE8, 0x0300500C, 0x03005000),
}


def entry(rom, table, index):
    return (struct.unpack_from('<I', rom, table + index * 4)[0] & ~1) - 0x08000000


def setup(key, rom, index, seed):
    _, _, _, _, trainer_pointer, rng = CONFIG[key]
    native = Native(rom)
    native.word(RULES[key].save_pointer, SAVE)
    native.word(trainer_pointer, TRAINER_SAVE)
    native.write(TRAINER_SAVE, b'\xff' * 8)
    native.word(rng, seed)
    for variable, value in [(0x8004, index), (0x8005, 0)]:
        native.half(native.call(RULES[key].get_variable_pointer, variable), value)
    return native


def verify(key, rom):
    assert hashlib.md5(rom).hexdigest() == PROFILES[key][0]
    specials, info, count, getter, _, _ = CONFIG[key]
    rows = []
    for index in range(count):
        cases = []
        for level in [1, 50, 100]:
            for seed in [1, 0x1234ABCD]:
                native = setup(key, rom, index, seed)
                requested = native.call(entry(rom, specials, info))
                # Ordinary constructors obtain level from the selected player slot.
                # Record their party destination from native SetMonData calls.
                destinations = []
                setter = {'BW': 0x6ACAC, 'DP': 0x6ACAC, 'ULTIMATE': 0x6ACAC,
                          'ROCKET': 0x97E2C, 'MERCURY12': 0x4037C}[key]

                def hook(cpu, address, size, unused):
                    if address == 0x08000000 + getter and cpu.reg_read(UC_ARM_REG_R1) == 56:
                        cpu.reg_write(UC_ARM_REG_R0, level)
                        cpu.reg_write(UC_ARM_REG_PC, cpu.reg_read(UC_ARM_REG_LR))
                    if address == 0x08000000 + setter:
                        destinations.append(cpu.reg_read(UC_ARM_REG_R0))

                handle = native.cpu.hook_add(UC_HOOK_CODE, hook)
                native.call(entry(rom, specials, info + 1))
                native.cpu.hook_del(handle)
                assert destinations and len(set(destinations)) == 1, (key, index, destinations)
                mon = destinations[0]
                actual = {name: native.call(getter, mon, field, 0)
                          for name, field in [('species', 11), ('level', 56), ('held_item', 12)]}
                assert actual['level'] == level, (key, index, actual)
                assert native.read(0x08000000, len(rom)) == rom
                cases.append(dict(seed=seed, offered_level=level, **actual))
        assert len({(c['species'], c['held_item']) for c in cases}) == 1
        rows.append(dict(index=index, requested_species=requested, cases=cases))
    alternate = None
    if key == 'MERCURY12':
        native = setup(key, rom, 0, 1)
        native.call(RULES[key].set_flag, 0x15F8)
        assert native.call(PROFILES[key][1], 0x15F8) == 1
        reached = []

        def hook(cpu, address, size, unused):
            if address == 0x09D0DDE8:
                reached.append(address)
                cpu.reg_write(UC_ARM_REG_PC, 0x08000000)

        native.cpu.hook_add(UC_HOOK_CODE, hook)
        native.call(entry(rom, specials, info + 1))
        assert reached == [0x09D0DDE8]
        assert native.read(0x08000000, len(rom)) == rom
        alternate = dict(flag=0x15F8, different_constructor=True, output_verified=False)
    print(key, len(rows), 'offer records;', len(rows) * 6,
          'native generation cases; special outputs remain unverified', file=sys.stderr)
    return dict(md5=PROFILES[key][0], rows=rows, alternate=alternate)


def main():
    results = {}
    for key in CONFIG:
        path = Path(os.environ['GEN3_ROM_' + key])
        rom = path.read_bytes()
        before = hashlib.sha256(rom).digest()
        results[key] = verify(key, rom)
        assert hashlib.sha256(path.read_bytes()).digest() == before
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
