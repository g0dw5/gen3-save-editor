#!/usr/bin/env python3
"""Bounded native text buffers and preservation of script/SAV RAM in five ROMs.

Runs complete unmodified ARM7 routines, without hooks, save inputs or ROM writes.
Runtime profile metadata supplies table bounds, not an extracted content catalog.
Outputs belong in private analysis directories only.
"""

import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh
from verify_event_effects import CONFIG, CONTEXT, OPERANDS
from verify_standard_dialogues import VARIABLE_POINTER, word

OPS = [0x7D, 0x80, 0x82, 0x83, 0x85]
LITERALS = [0x38, 0x30, 0x38, 0x40, 0x24]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--profiles", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert args.output.suffix == ".json" and not args.output.exists()
    inputs = [Path(os.environ["GEN3_ROM_" + k]) for k in CONFIG]
    hashes = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    assert args.output.resolve() not in [p.resolve() for p in inputs]
    profiles = {p["md5"]: p for p in json.loads(args.profiles.read_text())}
    b.MGBA_PROBE = args.mgba_probe.resolve()
    result = {}
    try:
        for key, (commands, sb1, sb2, _, _) in CONFIG.items():
            b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
            rom = b.ROM_PATH.read_bytes()
            assert hashlib.md5(rom).hexdigest() == b.MD5[key]
            profile = profiles[b.MD5[key]]
            handlers = {
                op: (word(rom, commands + op * 4) & ~1) - 0x08000000 for op in OPS
            }
            tables = {
                op: word(rom, handlers[op] + literal) - 0x08000000
                for op, literal in zip(OPS, LITERALS)
            }
            destinations = {
                op: list(struct.unpack_from("<3I", rom, tables[op])) for op in OPS
            }
            assert all(value == destinations[OPS[0]] for value in destinations.values())
            cpu = fresh(rom)
            cpu.word(sb1, 0x02030000)
            cpu.word(sb2, 0x02028000)
            pointers = [cpu.call(VARIABLE_POINTER[key], 0x8000 + i) for i in range(16)]
            for i, p in enumerate(pointers):
                cpu.half(p, 0x1000 + i)
            rows, skipped = [], []

            def bounded_string(at):
                data = rom[at : at + 14]
                return len(data) == 14 and 255 in data

            cases = []
            for op, catalog in [(0x7D, "species"), (0x80, "items"), (0x82, "moves")]:
                count = profile[catalog]["count"]
                # Every item exercises the formatter's item-specific paths. Names
                # and moves sample edges and interior entries from the native table.
                ids = (
                    range(count)
                    if op == 0x80
                    else sorted(
                        {
                            0,
                            1,
                            2,
                            count - 1,
                            count // 2,
                            *range(0, count, max(1, count // 32)),
                        }
                    )
                )
                for value in ids:
                    if op in [0x7D, 0x82]:
                        stride = 11 if op == 0x7D else 13
                        assert (
                            struct.unpack_from("<H", rom, handlers[op] + 0x22)[0]
                            == 0x2100 + stride
                        )
                        source = (
                            word(rom, handlers[op] + 0x3C) - 0x08000000 + value * stride
                        )
                    else:
                        source = (
                            profile["items"]["offset"]
                            + value * profile["items"]["stride"]
                        )
                    if not bounded_string(source):
                        skipped.append(
                            dict(opcode=op, value=value, reason="bounded_source")
                        )
                        continue
                    cases.append((op, value, 0, False, None))
                    if value in [0, 1, 2, count - 1, count // 2]:
                        cases += [
                            (op, value, slot, variable, None)
                            for slot in range(3)
                            for variable in [False, True]
                        ]
            for value in [
                0,
                1,
                9,
                10,
                99,
                100,
                999,
                1000,
                9999,
                10000,
                16383,
                16384,
                65535,
            ]:
                for slot in range(3):
                    cases.append((0x83, value, slot, True, None))
                    if value < 0x4000:
                        cases.append((0x83, value, slot, False, None))
            for value in [0, 1, 2, profile["species"]["count"] - 1]:
                source = (
                    profile["species"]["offset"] + value * profile["species"]["stride"]
                )
                if bounded_string(source):
                    for slot in range(3):
                        cases.append((0x85, value, slot, False, source))

            for op, value, slot, variable, source in cases:
                for i, p in enumerate(pointers):
                    cpu.half(p, 0x1000 + i)
                if variable:
                    cpu.half(pointers[0], value)
                operand = 0x8000 if variable else value
                payload = bytes([slot]) + (
                    struct.pack("<I", 0x08000000 + source)
                    if op == 0x85
                    else struct.pack("<H", operand)
                )
                cpu.write(CONTEXT, bytes([0x55]) * 128)
                cpu.word(CONTEXT + 8, OPERANDS)
                cpu.write(OPERANDS, payload)
                before = cpu.read(0x02000000, 0x40000)
                iwram = cpu.read(0x03000000, 0x7000)
                assert cpu.call(handlers[op], CONTEXT) == 0
                assert cpu.read(0x03000000, 0x7000) == iwram
                after = cpu.read(0x02000000, 0x40000)
                expected = bytearray(before)
                destination = destinations[op][slot]
                start = destination - 0x02000000
                data = after[start : start + 32]
                assert 255 in data, (key, op, value, "unterminated output")
                output = data[: data.index(255) + 1]
                assert len(output) <= 14, (
                    key,
                    op,
                    value,
                    "formatter exceeds supported bound",
                    output.hex(),
                )
                expected[start : start + len(output)] = output
                at = CONTEXT + 8 - 0x02000000
                expected[at : at + 4] = struct.pack("<I", OPERANDS + len(payload))
                assert after == bytes(expected), (
                    key,
                    op,
                    value,
                    "unexpected write outside destination/context",
                )
                rows.append(
                    dict(
                        opcode=op,
                        value=value,
                        slot=slot,
                        variable=variable,
                        operand=operand,
                        source=source,
                        output=output.hex(),
                        context_dependent=(
                            key == "MERCURY133"
                            and op == 0x80
                            and value
                            == (struct.unpack_from("<H", rom, 0x99E98)[0] & 255)
                        ),
                    )
                )
            result[key] = dict(
                md5=b.MD5[key],
                sha256=hashes[b.ROM_PATH],
                handlers=handlers,
                tables=tables,
                destinations=destinations[OPS[0]],
                rows=rows,
                skipped=skipped,
            )
            print(
                key,
                len(rows),
                "complete native buffers",
                len(skipped),
                "bounded sources omitted",
                flush=True,
            )
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2) + "\n")
    finally:
        if b.MGBANative.c is not None:
            b.MGBANative.c.finish()
        for path, digest in hashes.items():
            assert hashlib.sha256(path.read_bytes()).hexdigest() == digest


if __name__ == "__main__":
    main()
