#!/usr/bin/env python3
"""Compare the Rocket family-table reader with the native lookup routine.

Requires a user-supplied exact ROM and Unicorn. Does not read or write saves.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from verify_rocket_battle_forms import Native, ROM_MD5


def check(rom):
    assert hashlib.md5(rom).hexdigest() == ROM_MD5
    native = Native(rom)
    families, lookups, absent = {}, 0, 0
    for species in range(1, 1395):
        pointer, = struct.unpack_from('<I', rom, 0x617B00 + species * 4)
        if not pointer:
            assert native.call(0x9D19C, species, 0) == species
            absent += 1
            continue
        members = []
        for index in range(1395):
            value, = struct.unpack_from('<H', rom, pointer - 0x08000000 + index * 2)
            if value == 0xFFFF: break
            assert 0 < value < 1395
            assert native.call(0x9D19C, species, index) == value
            members.append(value)
            lookups += 1
        else: raise AssertionError(('unterminated family', species))
        families[species] = members
    assert families[9] == families[980] == [9, 980]
    assert 899 not in families
    return dict(table='0x617b00', native_lookup='0x9d19c', species_with_families=len(families),
                unique_families=len({tuple(v) for v in families.values()}),
                native_member_comparisons=lookups, native_identity_controls=absent)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rom', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(check(args.rom.read_bytes()), indent=2))
