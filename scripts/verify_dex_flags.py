#!/usr/bin/env python3
"""Exact-ROM native Dex read checks, including mirrored and dependent records.

Runs complete unmodified ARM7 getters on disposable synthetic RAM. No SAV is
opened and no ROM bytes are written. Allocation bounds are adapter flag ranges,
not counts of obtainable species. Mercury has a separate split-bank verifier.
"""

import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh

# Native routines, SaveBlock pointers and layouts only; no extracted catalogs.
CONFIG = {
    "BW": (
        0xC0664,
        0x03005D8C,
        0x03005D90,
        False,
        416,
        0,
        0x5C,
        0x28,
        [0x988, 0x3B24],
        True,
    ),
    "DP": (
        0xC0664,
        0x03005D8C,
        0x03005D90,
        False,
        416,
        0,
        0x5C,
        0x28,
        [0x988, 0x3B24],
        True,
    ),
    "ROCKET": (
        0xF7C60,
        0x0300524C,
        0x03005250,
        True,
        955,
        0,
        0x2EE4,
        0x2F5C,
        [],
        False,
    ),
    "ULTIMATE": (0xC0664, 0x03005D8C, 0x03005D90, True, 905, 1, 0x560, 0x5D8, [], True),
}
MAIN, TRAINER = 0x02020000, 0x02028000


def verify(key):
    path = Path(os.environ["GEN3_ROM_" + key]).resolve()
    rom = path.read_bytes()
    digest = hashlib.sha256(rom).hexdigest()
    assert hashlib.md5(rom).hexdigest() == b.MD5[key]
    getter, sb1, sb2, main_block, count, bias, seen_at, owned_at, mirrors, requires = (
        CONFIG[key]
    )
    word = lambda at: struct.unpack_from("<I", rom, at)[0]
    if key in ["BW", "DP"]:
        assert rom[getter : getter + 2] == bytes.fromhex("f0b5")
        assert [word(at) for at in [0xC06EC, 0xC06F0, 0xC06F4, 0xC06F8]] == [
            sb2,
            sb1,
            *mirrors,
        ]
    elif key == "ROCKET":
        assert rom[getter : getter + 2] == bytes.fromhex("70b5")
        assert [word(at) for at in [0xF7CA4, 0xF7CA8, 0xF7CC4, 0xF7CC8]] == [
            sb1,
            seen_at,
            sb1,
            owned_at,
        ]
    else:
        assert rom[getter : getter + 4] == bytes.fromhex("004b1847")
        assert word(getter + 4) == 0x09257951
        assert word(0x12579D0) == sb1
        masks = word(0x12579D4) - 0x08000000
        assert struct.unpack_from("<8I", rom, masks) == tuple(1 << i for i in range(8))
    b.ROM_PATH = path
    rows = []
    fields = [(main_block, owned_at), (main_block, seen_at)] + [
        (True, at) for at in mirrors
    ]
    try:
        for state in range(1 << len(fields)):
            for order in [[0, 1], [1, 0]]:
                cpu = fresh(rom)
                cpu.word(sb1, MAIN)
                cpu.word(sb2, TRAINER)
                for number in range(1, count + 1):
                    background = 0xA5 if number % 2 else 0x5A
                    main = bytearray([background] * 0x3D68)
                    trainer = bytearray([background ^ 255] * 0xE1C)
                    index = number - 1 + bias
                    byte, mask = index // 8, 1 << (index % 8)
                    for bit, (in_main, offset) in enumerate(fields):
                        block = main if in_main else trainer
                        block[offset + byte] = (block[offset + byte] & ~mask) | (
                            mask if state & (1 << bit) else 0
                        )
                    cpu.write(MAIN, main)
                    cpu.write(TRAINER, trainer)
                    before_iwram = cpu.read(0x03000000, 0x7D00)
                    expected_main, expected_trainer = bytearray(main), bytearray(
                        trainer
                    )
                    expected_blocks = [expected_trainer, expected_main]
                    actual = {}
                    for operation in order:
                        block = expected_blocks[int(main_block)]
                        raw_seen = bool(block[seen_at + byte] & mask)
                        raw_owned = bool(block[owned_at + byte] & mask)
                        valid_seen = raw_seen and all(
                            expected_main[at + byte] & mask for at in mirrors
                        )
                        expected = (
                            valid_seen
                            if operation == 0
                            else raw_owned and (not requires or valid_seen)
                        )
                        value = cpu.call(getter, number, operation)
                        assert value in [0, 1] and bool(value) == expected, (
                            key,
                            state,
                            number,
                            operation,
                            value,
                            expected,
                        )
                        actual[operation] = bool(value)
                        # Only BW/DP clear bad positive records in native RAM.
                        # The editor must project these reads, never do these writes.
                        if mirrors and (
                            (operation == 0 and raw_seen and not valid_seen)
                            or (operation == 1 and raw_owned and not valid_seen)
                        ):
                            block[seen_at + byte] &= ~mask
                            for at in mirrors:
                                expected_main[at + byte] &= ~mask
                            if operation == 1:
                                block[owned_at + byte] &= ~mask
                        assert cpu.read(MAIN, len(main)) == expected_main
                        assert cpu.read(TRAINER, len(trainer)) == expected_trainer
                        assert cpu.read(0x03000000, 0x7D00) == before_iwram
                    rows.append(
                        dict(
                            number=number,
                            state=state,
                            order=order,
                            seen=actual[0],
                            owned=actual[1],
                        )
                    )
        print(
            f"{key}: {len(rows)*2} complete native getters; independent flags, both query orders and native invalidation verified",
            flush=True,
        )
        return dict(
            md5=b.MD5[key],
            rom_sha256=digest,
            count=count,
            bit_bias=bias,
            main_block=main_block,
            seen=seen_at,
            owned=owned_at,
            mirrors=mirrors,
            owned_requires_seen=requires,
            getter=getter,
            rows=rows,
        )
    finally:
        assert hashlib.sha256(path.read_bytes()).hexdigest() == digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists() and args.output.suffix == ".json"
    inputs = [Path(os.environ["GEN3_ROM_" + key]).resolve() for key in CONFIG]
    assert args.output.resolve() not in inputs
    b.MGBA_PROBE = args.mgba_probe.resolve()
    try:
        result = {key: verify(key) for key in CONFIG}
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result) + "\n")
    finally:
        if b.MGBANative.c is not None:
            b.MGBANative.c.finish()


if __name__ == "__main__":
    main()
