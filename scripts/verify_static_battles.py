#!/usr/bin/env python3
"""Complete native fixed-encounter setup in five exact private ROMs.

Synthetic RAM only, no hooks/replaced functions, no SAV opened, no ROM writes.
This verifies command inputs and generated opponent fields, not battle-start
access, capture permissions, story progress or receipt. Keep vectors private.
"""

import argparse
import hashlib
import itertools
import json
import os
import struct
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh
from verify_event_effects import CONFIG, CONTEXT, OPERANDS

# Engine interfaces, never species names, encounters or extracted game catalogs.
READERS = {
    "BW": (0x6A518, 0x02024744),
    "DP": (0x6A518, 0x02024744),
    "ROCKET": (0x976D0, 0x020253C8),
    "ULTIMATE": (0x6A518, 0x02024744),
    "MERCURY12": (0x3FBE8, 0x0202402C),
}


def operands(key, first, second, level, held, double, variable):
    species = 0x8004 if variable else first
    if key == "ROCKET":
        return struct.pack(
            "<HBHHBH",
            species,
            level,
            held,
            second if double else 0,
            level + 1,
            held + 1,
        )
    if key == "MERCURY12" and double:
        return (
            struct.pack("<HBHB", 0xFFFF, 173, 0xBEEF, 201)
            + struct.pack("<HBHB", species, level, held, 229)
            + struct.pack("<HBH", 0x8005 if variable else second, level + 1, held + 1)
        )
    return struct.pack("<HBH", species, level, held)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--mgba-probe", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    args = p.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    inputs = [Path(os.environ["GEN3_ROM_" + k]).resolve() for k in READERS]
    assert args.output.resolve() not in inputs
    result = {}
    for key, (getter, enemy) in READERS.items():
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        digest = hashlib.sha256(rom).hexdigest()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        handler = (
            struct.unpack_from("<I", rom, CONFIG[key][0] + 0xB6 * 4)[0] & ~1
        ) - 0x08000000
        rows = []
        try:
            modes = [(False, False)]
            if key in ["ROCKET", "MERCURY12"]:
                modes.append((True, False))
            if key == "MERCURY12":
                modes.extend([(False, True), (True, True)])
            for (double, variable), pair, level, held, seed in itertools.product(
                modes,
                [(25, 26), (133, 133), (1, 2)],
                [1, 21, 99],
                [0, 13, 255],
                [42, 0x1234ABCD],
            ):
                cpu = fresh(rom)
                cpu.word(CONFIG[key][1], 0x02030000)
                cpu.word(CONFIG[key][2], 0x02028000)
                cpu.word(b.CONFIG[key][5], seed)
                cpu.word(CONTEXT + 8, OPERANDS)
                var_pointer = (
                    struct.unpack_from("<I", rom, CONFIG[key][0] + 0x16 * 4)[0] & ~1
                ) - 0x08000000
                # Use the native setvar command for the two temporary inputs.
                for var, value in [(0x8004, pair[0]), (0x8005, pair[1])]:
                    cpu.write(OPERANDS, struct.pack("<HH", var, value))
                    cpu.word(CONTEXT + 8, OPERANDS)
                    assert cpu.call(var_pointer, CONTEXT) == 0
                data = operands(key, *pair, level, held, double, variable)
                cpu.write(OPERANDS, data)
                cpu.word(CONTEXT + 8, OPERANDS)
                party = b.CONFIG[key][6]
                before_party = cpu.read(party, 600)
                regs = (cpu.ctypes.c_uint * 16)()
                completed = cpu.c.calluntil(
                    0x08000000 + handler, CONTEXT, 0, 0, 0, 0, regs
                )
                assert completed, (
                    key,
                    double,
                    variable,
                    pair,
                    level,
                    held,
                    seed,
                    [hex(v) for v in regs],
                )
                value = regs[0]
                assert value == 0
                assert cpu.read(party, 600) == before_party, (
                    key,
                    "player party changed",
                )
                consumed = struct.unpack("<I", cpu.read(CONTEXT + 8, 4))[0] - OPERANDS
                assert consumed == len(data), (key, double, consumed, len(data))
                raw = cpu.read(enemy, 600)
                mons = []
                for slot in range(6):
                    at = enemy + 100 * slot
                    species = cpu.call(getter, at, 11, 0)
                    if species:
                        mons.append(
                            dict(
                                slot=slot,
                                species=species,
                                level=cpu.call(getter, at, 56, 0),
                                held_item=cpu.call(getter, at, 12, 0),
                            )
                        )
                expected = [dict(slot=0, species=pair[0], level=level, held_item=held)]
                if double:
                    expected.append(
                        dict(
                            slot=3 if key == "ROCKET" else 1,
                            species=pair[1],
                            level=level + 1,
                            held_item=held + 1,
                        )
                    )
                assert mons == expected, (key, double, variable, mons, expected)
                assert cpu.read(enemy, 600) == raw, (key, "getters changed opponents")
                rows.append(
                    dict(
                        operands=list(data),
                        double=double,
                        variable=variable,
                        resolved=list(pair),
                        seed=seed,
                        consumed=consumed,
                        mons=mons,
                    )
                )
            result[key] = dict(
                md5=b.MD5[key], sha256=digest, handler=handler, rows=rows
            )
            print(key, len(rows), "complete native setup calls passed", flush=True)
        finally:
            assert hashlib.sha256(b.ROM_PATH.read_bytes()).hexdigest() == digest
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
