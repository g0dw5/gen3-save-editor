#!/usr/bin/env python3
"""Compare runtime journal visibility with Mercury 1.2's native DF7C40.

Opt-in private vectors from adventure::journal::tests::exact_rom_journal_native_vectors.
Uses isolated mGBA RAM; source files stay read-only. No quest catalog is bundled.
"""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path



def verify(rom_path, vectors, probe):
    rom = rom_path.read_bytes()
    assert hashlib.md5(rom).hexdigest() == 'f323df1792ac68462a34b42fe8571533'
    c = ctypes.CDLL(str(probe.resolve()))
    c.callfunc.restype = ctypes.c_uint
    assert c.start(str(rom_path.resolve()).encode())
    def write(a, b): c.writebytes(a, bytes(b), len(b))
    comparisons = 0
    unknown = 0
    for case in vectors:
        write(0x02000000, bytes(0x40000))
        write(0x03000000, bytes(0x8000))
        c.writemem(0x03005008, 0x02028000, 4)
        write(0x02028000, bytes(case['main']))
        write(0x0203b174, bytes(case['extensions']))
        for page in case['pages']:
            actual = bool(c.callfunc(0x08df7c40, page['quest'], 0x08000000 + page['page'], 0, 0))
            if page['expected'] is None:
                unknown += 1
            else:
                assert actual == page['expected'], (page, actual)
                comparisons += 1
    c.finish()
    return dict(native_visibility_comparisons=comparisons, unresolved=unknown,
                source='Native quest journal, not stage order or bag holdings')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--rom', type=Path, required=True)
    p.add_argument('--vectors', type=Path, required=True)
    p.add_argument('--probe', type=Path, required=True)
    a = p.parse_args()
    print(json.dumps(verify(a.rom, json.loads(a.vectors.read_text()), a.probe), indent=2))
