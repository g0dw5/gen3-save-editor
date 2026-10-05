#!/usr/bin/env python3
"""Complete ordinary production steps and exhaustive native roll arithmetic.

Five exact private ROMs, disposable synthetic RAM, mGBA ARM7 only. No user SAV
is opened, no native function is replaced, and no ROM write API is available.
Service access, live daycare state and complete hatching are separate evidence.
"""

import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
import verify_breeding as breeding


def fresh(rom):
    cpu = breeding.MGBANative(rom)
    # Each independent scenario resets board timing/CPU state as well as RAM.
    # Bulk instruction observations must not carry peripheral time into a new call.
    cpu.c.resetfixture()
    cpu.write(0x02000000, bytes(0x40000))
    cpu.write(0x03000000, bytes(0x8000))
    return cpu


# Native addresses/layouts, not extracted content or a charm/item-name catalog.
CONFIG = {
    "BW": (0x70AC4, 0x70B2C, 0x70B1E, 0x02039DD8, 0x310EA2, 0, "immediate", 0xD6724),
    "DP": (0x70AC4, 0x70B2C, 0x70B1E, 0x02039DD8, 0x310EA2, 0, "immediate", 0xD6724),
    "ULTIMATE": (0x70AC4, 0x70B2C, 0x70B1E, 0x02039DD8, None, 0, None, None),
    "ROCKET": (0x9EAAC, 0x9EB1C, 0x9EB0E, 0x0203ADDC, 0x9F35C, 0, "word", 0x10EAE0),
    "MERCURY12": (
        0x462C4,
        0x4632C,
        0x4631E,
        0x0203988C,
        0x1D1BF0C,
        1,
        "immediate",
        0x99F40,
    ),
}
POCKET = dict(
    BW=0xD7590, DP=0xD7590, ULTIMATE=0xD7590, ROCKET=0x10F958, MERCURY12=0x9A9D8
)
KEY_OFFSET = dict(BW=0xAC, DP=0xAC, ULTIMATE=0xAC, ROCKET=0xAC, MERCURY12=0xF20)


def observe(cpu, start, stop, *args):
    regs = (cpu.ctypes.c_uint * 16)()
    assert cpu.c.calluntil(
        0x08000000 + start,
        *(list(args) + [0] * (4 - len(args))),
        0x08000000 + stop,
        regs
    ), hex(start)
    return list(regs)


def verify(name):
    breeding.ROM_PATH = Path(os.environ["GEN3_ROM_" + name])
    rom = breeding.ROM_PATH.read_bytes()
    assert hashlib.md5(rom).hexdigest() == breeding.MD5[name]
    step, cmp, roll, descriptors, item_at, shift, operand, check = CONFIG[name]
    _, compat, off, sb1, sb2, rng, party, count, width = breeding.CONFIG[name]
    item = (
        None
        if item_at is None
        else (
            (struct.unpack_from("<H", rom, item_at)[0] & 255) << shift
            if operand == "immediate"
            else struct.unpack_from("<I", rom, item_at)[0]
        )
    )
    cpu = fresh(rom)
    rolls = (cpu.ctypes.c_uint * 65536)()
    assert cpu.c.allrolls(0x08000000 + roll, 0x08000000 + cmp, rolls)
    # Independent ROM arithmetic is recorded for every possible 16-bit draw.
    native_rolls = list(rolls)
    rows = []
    gates = []
    pairs = [
        ((25, 0, 1), (25, 255, 1)),
        ((25, 0, 1), (25, 255, 2)),
        ((25, 0, 1), (133, 255, 1)),
        ((25, 0, 1), (133, 255, 2)),
        ((25, 0, 1), (25, 0, 2)),
    ]
    for pair in pairs:
        parents = [breeding.parent(name, *p) for p in pair]
        for key in [0, 0x87654321]:
            for present in ([False, True] if item else [False]):
                for draw in [0, 32768, 65535]:
                    cpu = fresh(rom)
                    cpu.word(sb1, 0x02030000)
                    cpu.word(sb2, 0x02034000)
                    cpu.word(0x02034000 + KEY_OFFSET[name], key)
                    if item:
                        category = cpu.call(POCKET[name], item)
                        desc = descriptors + (category - 1) * 8
                        cpu.word(desc, 0x02010000)
                        cpu.half(desc + 4, 1)
                        cpu.half(0x02010000, item if present else 0)
                        cpu.half(
                            0x02010002,
                            (
                                (int(present) ^ (key & 65535))
                                if name != "MERCURY12"
                                else int(present)
                            ),
                        )
                        actual_present = cpu.call(check, item, 1) != 0
                        assert actual_present == present
                    daycare = 0x02030000 + off
                    for i, raw in enumerate(parents):
                        cpu.write(daycare + i * 140, raw)
                        cpu.word(daycare + i * 140 + 136, 254)
                    base = cpu.call(compat, daycare)
                    # The first draw is forced by the inverse native LCG, without
                    # replacing the RNG function. The step still calls it normally.
                    seed = (
                        ((draw << 16) - 0x6073) * pow(0x41C64E6D, -1, 1 << 32)
                    ) & 0xFFFFFFFF
                    cpu.word(rng, seed)
                    regs = observe(cpu, step, cmp, daycare)
                    assert regs[0] == native_rolls[draw], (name, draw, regs[0])
                    threshold = regs[4]
                    numerator = sum(value < threshold for value in native_rolls)
                    # Replay the COMPLETE production function, including pending
                    # personality generation and its native RNG/item branches.
                    inputs = [
                        (0x02000000, cpu.read(0x02000000, 0x40000)),
                        (0x03000000, cpu.read(0x03000000, 0x8000)),
                    ]
                    cpu = fresh(rom)
                    for address, data in inputs:
                        cpu.write(address, data)
                    cpu.word(daycare + 136, 254)
                    cpu.word(daycare + 276, 254)
                    cpu.word(rng, seed)
                    cpu.call(step, daycare)
                    pending = int.from_bytes(cpu.read(daycare + 280, width), "little")
                    # Mercury replaced personality generation with a persistent
                    # egg-available flag. Its legacy 16-bit personality stays zero.
                    flag = (
                        None
                        if name != "MERCURY12"
                        else struct.unpack_from("<I", rom, 0x1D1BB9C)[0]
                    )
                    available = (
                        bool(pending) if flag is None else bool(cpu.call(0x6E6D0, flag))
                    )
                    assert available == (regs[0] < threshold), (
                        name,
                        pair,
                        present,
                        draw,
                        pending,
                        threshold,
                    )
                    assert [
                        cpu.read(daycare + i * 140, 80) for i in range(2)
                    ] == parents
                    rows.append(
                        dict(
                            parents=[list(v) for v in parents],
                            key=key,
                            modifier_present=present,
                            seed=seed,
                            base=base,
                            threshold=threshold,
                            roll=regs[0],
                            numerator=numerator,
                            denominator=65536,
                            pending_after=pending,
                            available_flag=flag,
                            passed=available,
                        )
                    )
    # Non-boundary/pending gates and exactly one ordinary 256-step interval.
    for steps, pending in [(0, 0), (253, 0), (255, 0), (256, 0), (254, 123), (510, 0)]:
        cpu = fresh(rom)
        cpu.word(sb1, 0x02030000)
        cpu.word(sb2, 0x02034000)
        daycare = 0x02030000 + off
        for i, p in enumerate(pairs[1]):
            cpu.write(daycare + i * 140, breeding.parent(name, *p))
            cpu.word(daycare + i * 140 + 136, steps)
        cpu.write(daycare + 280, pending.to_bytes(width, "little"))
        cpu.word(rng, 0)
        cpu.call(step, daycare)
        after = int.from_bytes(cpu.read(daycare + 280, width), "little")
        state = int.from_bytes(cpu.read(rng, 4), "little")
        assert (state != 0) == (steps == 510 and pending == 0), (
            name,
            steps,
            pending,
            state,
        )
        if pending:
            assert after == pending
        gates.append(
            dict(parent_steps=steps, pending=pending, after=after, rng_after=state)
        )
    assert cpu.read(0x08000000, len(rom)) == rom
    assert hashlib.md5(breeding.ROM_PATH.read_bytes()).hexdigest() == breeding.MD5[name]
    return dict(
        md5=breeding.MD5[name],
        engine="mGBA ARM7",
        modifier_item=item,
        rolls=native_rolls,
        rows=rows,
        gates=gates,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    breeding.MGBA_PROBE = args.mgba_probe.resolve()
    result = {}
    for name in CONFIG:
        result[name] = verify(name)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result), encoding="utf-8")
        print(
            name,
            len(result[name]["rows"]),
            "complete production steps; 65536 native rolls; gates verified",
            flush=True,
        )
