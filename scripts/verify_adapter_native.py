#!/usr/bin/env python3
"""Compare Rust adapter probe JSON with the exact ROM's own getters/setters.

Generate JSON with local_rocket_adapter_regression and GEN3_ADAPTER_PROBES.
Inputs stay local. No ROM or save is ever written. Requires Unicorn.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
from verify_rocket_battle_forms import Native, PARTY, ROM_MD5


def check(rom, probes):
    assert hashlib.md5(rom).hexdigest() == ROM_MD5
    native = Native(rom)
    getter_fields = {'species': 11, 'held_item': 12, 'experience': 25,
                     'friendship': 32, 'ball': 38, 'ability_slot': 46,
                     'nature_override': 89, 'language': 3, 'markings': 8}
    setter_fields = {'experience': 25, 'friendship': 32, 'ball': 38,
                     'ability_slot': 46, 'nature_override': 89, 'pp_ups': 21}
    reads = writes = 0
    for case in probes:
        raw = bytes(case['after'])
        p = case['pokemon']
        native.reset()
        native.write(PARTY, raw.ljust(100, b'\0'))
        for name, field in getter_fields.items():
            expected = p[name]
            assert native.call(0x976D0, PARTY, field, 0) == expected, (name, expected)
            reads += 1
        for i in range(4):
            assert native.call(0x976D0, PARTY, 13 + i, 0) == p['moves'][i]
            assert native.call(0x976D0, PARTY, 17 + i, 0) == p['pps'][i]
            reads += 2
        assert native.call(0x976D0, PARTY, 21, 0) == sum(v << (i * 2) for i, v in enumerate(p['pp_ups']))
        assert native.call(0x9A334, PARTY, 1) == p['effective_nature']
        reads += 2
        # Getters must leave every original byte unchanged.
        assert native.read(PARTY, len(raw)) == raw
        native.call(0x967B4, PARTY)
        assert list(struct.unpack('<6H', native.read(PARTY + 88, 12))) == p['stats'], ('stats', p['species'], p['pid'])
        assert native.call(0x976D0, PARTY, 56, 0) == p['level']
        if 'patch' not in case:
            continue
        patch = case['patch']
        active = [(name, value) for name, value in patch.items() if value is not None and name in setter_fields]
        assert len(active) == 1
        name, value = active[0]
        if name == 'pp_ups': value = sum(v << (i * 2) for i, v in enumerate(value))
        native.write(PARTY, bytes(case['before']).ljust(100, b'\0'))
        native.word(0x0203F000, value)
        native.call(0x97E2C, PARTY, setter_fields[name], 0x0203F000)
        assert native.read(PARTY, len(raw)) == raw, ('setter mismatch', name, p['pid'])
        writes += 1
    return {'records': len(probes), 'native_getter_comparisons': reads, 'native_setter_byte_comparisons': writes, 'native_stat_comparisons': len(probes)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rom', type=Path, required=True)
    parser.add_argument('--probes', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(check(args.rom.read_bytes(), json.loads(args.probes.read_text())), indent=2))


if __name__ == '__main__':
    main()
