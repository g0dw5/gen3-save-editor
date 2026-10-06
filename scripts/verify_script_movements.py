#!/usr/bin/env python3
"""Native actor-action dispatch and wait lifecycle in five exact ROMs.

Complete, unmodified ARM7 routines and an end-only native task, synthetic RAM.
No SAV inputs, ROM writes or native hooks. This does not simulate arbitrary
movement bodies, overworld collisions, map entry or NPC current positions.
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
from verify_standard_dialogues import bl, word, VARIABLE_POINTER

MAIN, TRAINER, MOVEMENT = 0x02030000, 0x02028000, 0x02018000


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists() and args.output.suffix == ".json"
    inputs = [Path(os.environ["GEN3_ROM_" + key]).resolve() for key in CONFIG]
    hashes = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    assert args.output.resolve() not in inputs
    b.MGBA_PROBE = args.mgba_probe.resolve()
    result = {}
    try:
        for key, (table, sb1, sb2, *_) in CONFIG.items():
            b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key]).resolve()
            rom = b.ROM_PATH.read_bytes()
            assert hashlib.md5(rom).hexdigest() == b.MD5[key]
            handlers = [
                (word(rom, table + op * 4) & ~1) - 0x08000000
                for op in range(0x4F, 0x53)
            ]
            apply = bl(rom, handlers[0] + 0x2A)
            lookup = bl(rom, apply + 0x14)
            find = bl(rom, bl(rom, lookup + 0x10) + 0x18)
            objects = word(rom, find + 0x38)
            init = bl(rom, apply + 0x30)
            task = (word(rom, init + 0x3C) & ~1) - 0x08000000
            tasks = word(rom, init + 0x40) - 8
            callback = handlers[2] - 0x28
            globals_ = [word(rom, handlers[2] + off) for off in [0x40, 0x44, 0x4C]]
            assert 0x02000000 <= objects <= 0x02040000 - 16 * 36
            assert 0x03000000 <= tasks <= 0x03008000 - 16 * 40
            rows = []
            for apply_op in [0x4F, 0x50]:
                for wait_op in [0x51, 0x52]:
                    for present in [False, True]:
                        for operand, value in [
                            (0, 0),
                            (7, 7),
                            (255, 255),
                            (256, 256),
                            (0x8000, 7),
                            (0x8000, 65535),
                            (0x800D, 0),
                            (0x4000, 7),
                        ]:
                            for group, number in [(3, 5), (128, 255)]:
                                cpu = fresh(rom)
                                cpu.word(sb1, MAIN)
                                cpu.word(sb2, TRAINER)
                                cpu.write(MAIN, bytes([0xA5]) * 0x3D68)
                                cpu.write(TRAINER, bytes([0x5A]) * 0xE1C)
                                cpu.write(MAIN + 4, bytes([group, number]))
                                for i in range(16):
                                    cpu.half(
                                        cpu.call(VARIABLE_POINTER[key], 0x8000 + i),
                                        0x1000 + i,
                                    )
                                if operand >= 0x4000:
                                    cpu.half(
                                        cpu.call(VARIABLE_POINTER[key], operand), value
                                    )
                                if present:
                                    cpu.write(
                                        objects,
                                        bytes([1])
                                        + bytes(7)
                                        + bytes([value & 255, number, group])
                                        + bytes(25),
                                    )
                                cpu.write(MOVEMENT, b"\xfe")
                                cpu.write(CONTEXT + 2, b"\x02")
                                before = [
                                    cpu.read(MAIN, 0x3D68),
                                    cpu.read(TRAINER, 0xE1C),
                                ]
                                before_vars = [
                                    cpu.read(
                                        cpu.call(VARIABLE_POINTER[key], 0x8000 + i), 2
                                    )
                                    for i in range(16)
                                ]

                                def preserve():
                                    assert [
                                        cpu.read(MAIN, 0x3D68),
                                        cpu.read(TRAINER, 0xE1C),
                                    ] == before
                                    assert [
                                        cpu.read(
                                            cpu.call(VARIABLE_POINTER[key], 0x8000 + i),
                                            2,
                                        )
                                        for i in range(16)
                                    ] == before_vars
                                    assert cpu.read(CONTEXT + 2, 1) == b"\x02"

                                cpu.word(CONTEXT + 8, OPERANDS)
                                cpu.write(
                                    OPERANDS,
                                    struct.pack("<HI", operand, MOVEMENT)
                                    + (
                                        bytes([group, number])
                                        if apply_op == 0x50
                                        else b""
                                    ),
                                )
                                assert cpu.call(handlers[apply_op - 0x4F], CONTEXT) == 0
                                assert word(cpu.read(CONTEXT, 16), 8) == OPERANDS + (
                                    8 if apply_op == 0x50 else 6
                                )
                                preserve()
                                cpu.word(CONTEXT + 8, OPERANDS)
                                cpu.write(
                                    OPERANDS,
                                    struct.pack("<H", operand)
                                    + (
                                        bytes([group, number])
                                        if wait_op == 0x52
                                        else b""
                                    ),
                                )
                                assert cpu.call(handlers[wait_op - 0x4F], CONTEXT) == 1
                                assert word(cpu.read(CONTEXT, 16), 8) == OPERANDS + (
                                    4 if wait_op == 0x52 else 2
                                )
                                assert (
                                    word(cpu.read(CONTEXT, 16), 4)
                                    == 0x08000000 + callback + 1
                                )
                                assert cpu.read(CONTEXT + 1, 1) == b"\x02"
                                waiting = cpu.call(callback)
                                assert waiting in [0, 1]
                                preserve()
                                # The real native task consumes its end marker.
                                # No branch/task function is replaced or skipped.
                                task_active = cpu.read(tasks + 4, 1) != b"\x00"
                                if task_active:
                                    cpu.call(task, 0)
                                finished = cpu.call(callback)
                                assert finished == 1
                                preserve()
                                rows.append(
                                    dict(
                                        apply_opcode=apply_op,
                                        wait_opcode=wait_op,
                                        operand=operand,
                                        resolved=value,
                                        present=present,
                                        map_id=f"{group}-{number}",
                                        waiting=waiting,
                                        finished=finished,
                                        task_active=task_active,
                                    )
                                )
            result[key] = dict(
                md5=b.MD5[key],
                rom_sha256=hashes[b.ROM_PATH],
                handlers=handlers,
                callback=callback,
                task=task,
                objects=objects,
                tasks=tasks,
                wait_globals=globals_,
                rows=rows,
            )
            print(
                f"{key}: {len(rows)} complete native apply/wait/end-task scenarios; PC, callback, cache, all special variables and SaveBlock1/2 checked",
                flush=True,
            )
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result) + "\n")
    finally:
        if b.MGBANative.c is not None:
            b.MGBANative.c.finish()
        for path, digest in hashes.items():
            assert hashlib.sha256(path.read_bytes()).hexdigest() == digest


if __name__ == "__main__":
    main()
