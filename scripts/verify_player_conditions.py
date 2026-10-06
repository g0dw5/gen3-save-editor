#!/usr/bin/env python3
"""Player-gender script reads/copies/branches in five exact ROMs, native mGBA.

Complete unmodified functions run with disposable synthetic RAM. No user SAV,
ROM patch, native replacement or inferred binary gender restriction is involved.
The output is private evidence, not a bundled script/content catalog.
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    assert args.output.suffix == ".json" and not args.output.exists()
    inputs = [Path(os.environ["GEN3_ROM_" + key]) for key in CONFIG]
    assert args.output.resolve() not in [p.resolve() for p in inputs]
    original = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    b.MGBA_PROBE = args.mgba_probe.resolve()
    report = {}
    try:
        for key, (table, sb1, sb2, _, _) in CONFIG.items():
            b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
            rom = b.ROM_PATH.read_bytes()
            assert hashlib.md5(rom).hexdigest() == b.MD5[key]
            handlers = {
                op: (word(rom, table + op * 4) & ~1) - 0x08000000
                for op in [0xA0, 0x19, 0x1A, 0x21, 0x22, 6, 7]
            }
            cpu = fresh(rom)
            cpu.word(sb1, 0x02030000)
            cpu.word(sb2, 0x02028000)
            pointers = [cpu.call(VARIABLE_POINTER[key], 0x8000 + i) for i in range(16)]
            assert word(rom, handlers[0xA0] + 0x10) == pointers[13]
            assert word(rom, handlers[0xA0] + 0x14) == sb2
            reads, branches, copies = [], [], []

            def variables():
                return [int.from_bytes(cpu.read(p, 2), "little") for p in pointers]

            for gender in range(256):
                for i, p in enumerate(pointers):
                    cpu.half(p, 0x3100 + i)
                context = bytearray((i * 29 + 17) & 255 for i in range(128))
                struct.pack_into("<I", context, 8, OPERANDS)
                cpu.write(CONTEXT, context)
                cpu.write(0x02028008, bytes([gender]))
                persistent = cpu.read(0x02030000, 0x4000) + cpu.read(0x02028000, 0x1000)
                before = variables()
                assert cpu.call(handlers[0xA0], CONTEXT) == 0
                expected = before[:]
                expected[13] = gender
                assert variables() == expected
                assert cpu.read(CONTEXT, 128) == bytes(context)
                assert persistent == cpu.read(0x02030000, 0x4000) + cpu.read(
                    0x02028000, 0x1000
                )
                reads.append(dict(gender=gender, result=expected[13]))

            for gender in [0, 1, 2, 255]:
                cpu.write(0x02028008, bytes([gender]))
                assert cpu.call(handlers[0xA0], CONTEXT) == 0
                for op in [0x19, 0x1A]:
                    before = variables()
                    cpu.word(CONTEXT + 8, OPERANDS)
                    cpu.write(OPERANDS, struct.pack("<HH", 0x8000, 0x800D))
                    assert cpu.call(handlers[op], CONTEXT) == 0
                    expected = before[:]
                    expected[0] = gender
                    assert variables() == expected
                    copies.append(dict(opcode=op, gender=gender, result=variables()[0]))
                for rhs in [0, 1, 2, 255, 65535]:
                    for reversed_ in [False, True]:
                        cpu.half(pointers[1], rhs)
                        for selector in range(6):
                            for branch_op in [6, 7]:
                                cpu.write(CONTEXT, bytes(128))
                                cpu.word(CONTEXT + 8, OPERANDS)
                                cpu.write(
                                    OPERANDS,
                                    struct.pack(
                                        "<HH",
                                        0x8001 if reversed_ else 0x8000,
                                        0x8000 if reversed_ else rhs,
                                    ),
                                )
                                assert (
                                    cpu.call(
                                        handlers[0x22 if reversed_ else 0x21], CONTEXT
                                    )
                                    == 0
                                )
                                native_comparison = cpu.read(CONTEXT + 2, 1)[0]
                                cpu.word(CONTEXT + 8, OPERANDS)
                                cpu.write(
                                    OPERANDS,
                                    struct.pack("<BI", selector, OPERANDS + 0x100),
                                )
                                assert cpu.call(handlers[branch_op], CONTEXT) == 0
                                selected = (
                                    word(cpu.read(CONTEXT, 128), 8) == OPERANDS + 0x100
                                )
                                assert word(cpu.read(CONTEXT, 128), 8) == (
                                    OPERANDS + 0x100 if selected else OPERANDS + 5
                                )
                                if branch_op == 7:
                                    assert cpu.read(CONTEXT, 1) == bytes(
                                        [int(selected)]
                                    )
                                    if selected:
                                        assert (
                                            word(cpu.read(CONTEXT, 128), 12)
                                            == OPERANDS + 5
                                        )
                                branches.append(
                                    dict(
                                        gender=gender,
                                        rhs=rhs,
                                        reversed=reversed_,
                                        selector=selector,
                                        opcode=branch_op,
                                        selected=selected,
                                        comparison=native_comparison,
                                    )
                                )
            report[key] = dict(
                md5=b.MD5[key],
                sha256=original[b.ROM_PATH],
                handlers=handlers,
                result_pointer=pointers[13],
                save_pointer=sb2,
                reads=reads,
                copies=copies,
                branches=branches,
            )
            print(
                f"{key}: {len(reads)} reads, {len(copies)} copies, {len(branches)} complete native branches",
                flush=True,
            )
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n")
    finally:
        if b.MGBANative.c is not None:
            b.MGBANative.c.finish()
        for path, digest in original.items():
            assert hashlib.sha256(path.read_bytes()).hexdigest() == digest


if __name__ == "__main__":
    main()
