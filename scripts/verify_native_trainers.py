#!/usr/bin/env python3
"""Compare isolated Mercury party scenarios with an independent Unicorn CPU.

Generate private probes using local_native_trainer_mercury. No user saves are
opened or changed. The exact fingerprint and bounded scenario are mandatory.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
from verify_rocket_battle_forms import Native


def check(rom, probes):
    assert hashlib.md5(rom).hexdigest() == '5ffb1cbd5c28cda9b987b3b445da68e0'
    native = Native(rom)
    getters = mons = 0
    for case in probes:
        assert case['scenario'] == 'ordinary_zero_context' and case['partial']
        native.write(0x02000000, bytes(0x40000))
        native.write(0x03000000, bytes(0x8000))
        native.word(0x03005008, 0x02030000)
        native.word(0x0300500c, 0x02034000)
        native.word(0x03005000, case['seed'])
        header = 0x23eac8 + case['trainer_id'] * 40
        native.word(0x02022b4c, 8 | int(rom[header+24] != 0))
        count = native.call(0x1d0ba44, 0x0202402c, case['trainer_id'], 0, 1, 0, 1)
        assert count == len(case['mons']), case['trainer_id']
        for i, mon in enumerate(case['mons']):
            address = 0x0202402c + i * 100
            raw = native.read(address, 100)
            assert struct.unpack_from('<I', raw)[0] == mon['pid']
            assert raw[84] == mon['level']
            assert list(struct.unpack_from('<6H', raw, 88)) == mon['stats']
            for key, field in [('species',11), ('held_item',12), ('experience',25)]:
                assert native.call(0x3fbe8, address, field, 0) == mon[key], (case['trainer_id'],i,key)
                getters += 1
            for key, first, count in [('moves',13,4), ('ivs',39,6), ('evs',26,6)]:
                for j in range(count):
                    assert native.call(0x3fbe8, address, first+j, 0) == mon[key][j], (case['trainer_id'],i,key,j)
                    getters += 1
            assert native.call(0x40d38,address) == mon['ability_id']
            assert native.read(address,100) == raw, 'query changed generated record'
            mons += 1
    return dict(scenarios=len(probes), mons=mons, native_getters=getters,
                scope='ordinary zero context; excludes full battle setup and facilities')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rom', type=Path, required=True)
    parser.add_argument('--probes', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(check(args.rom.read_bytes(), json.loads(args.probes.read_text())), indent=2))
