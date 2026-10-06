#!/usr/bin/env python3
"""Native field presentation effects in five exact private ROMs, mGBA ARM7.

Complete unmodified dispatch/return, message/wait/close, delay and yes/no routines
run in disposable synthetic RAM. Callback choices use native key/menu/task RAM;
no native function is replaced, no ROM is patched and no user SAV is opened.
This proves script-variable preservation, not visible dialogue or NPC access.
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

# Verified engine helper addresses/literal layouts, not standard-script content.
VARIABLE_POINTER = dict(
    BW=0x9D648, DP=0x9D648, ULTIMATE=0x9D648, ROCKET=0xD3930, MERCURY12=0x6E454
)
CALLBACK_LITERAL = dict(BW=0x28, DP=0x28, ULTIMATE=0x28, ROCKET=0x28, MERCURY12=0x18)
OPS = [0x08, 0x09, 0x03, 0x66, 0x67, 0x6E, 0x28, 0x68]


def word(rom, at):
    return struct.unpack_from("<I", rom, at)[0]


def bl(rom, at):
    first, second = struct.unpack_from("<HH", rom, at)
    assert first & 0xF800 == 0xF000 and second & 0xF800 == 0xF800
    high = first & 0x7FF
    if high & 0x400:
        high -= 0x800
    return at + 4 + (high << 12) + ((second & 0x7FF) << 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert args.output.suffix.lower() == ".json", "write private evidence to JSON only"
    b.MGBA_PROBE = args.mgba_probe.resolve()
    assert all(
        args.output.resolve() != Path(os.environ["GEN3_ROM_" + key]).resolve()
        for key in CONFIG
    )
    results = {}
    for key, (table, sb1, sb2, flags, variables) in CONFIG.items():
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        digest = hashlib.sha256(rom).hexdigest()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        handlers = {op: (word(rom, table + op * 4) & ~1) - 0x08000000 for op in OPS}
        tables = {}
        for op in [8, 9]:
            first = word(rom, handlers[op] + 0x28) - 0x08000000
            end = word(rom, handlers[op] + 0x2C) - 0x08000000
            if key == "ULTIMATE" and op == 9:
                hook = end & ~1
                assert hook == 0x14A332C
                instruction = struct.unpack_from("<H", rom, hook + 6)[0]
                assert instruction & 0xFF00 == 0x2800
                end = first + ((instruction & 255) + 1) * 4
            assert 0 <= first <= end < len(rom) and (end - first) % 4 == 0
            tables[op] = [first, end]

        def setup():
            cpu = fresh(rom)
            cpu.word(sb1, 0x02030000)
            cpu.word(sb2, 0x02038000)
            pointers = [cpu.call(VARIABLE_POINTER[key], 0x8000 + i) for i in range(16)]
            for i, address in enumerate(pointers):
                cpu.half(address, 0x1000 + i)
            cpu.word(CONTEXT + 8, OPERANDS)
            cpu.write(CONTEXT + 2, b"\2")
            return cpu, pointers

        def values(cpu, pointers):
            return [int.from_bytes(cpu.read(p, 2), "little") for p in pointers]

        def persistent(cpu):
            ranges = [(base, (count + 7) // 8) for _, count, base in flags]
            ranges += [(base, count * 2) for _, count, base in variables]
            return [cpu.read(base, length) for base, length in ranges]

        dispatch = []
        cpu, pointers = setup()
        initial = values(cpu, pointers)
        for op in [8, 9]:
            first, end = tables[op]
            count = (end - first) // 4
            for index in range(256):
                cpu.write(CONTEXT, bytes(0x80))
                cpu.word(CONTEXT + 8, OPERANDS)
                cpu.write(OPERANDS, bytes([index]))
                assert cpu.call(handlers[op], CONTEXT) == 0
                valid = index < count
                expected = word(rom, first + index * 4) if valid else OPERANDS + 1
                assert cpu.read(CONTEXT + 8, 4) == struct.pack("<I", expected)
                assert cpu.read(CONTEXT, 1) == bytes([int(op == 9 and valid)])
                if op == 9 and valid:
                    assert cpu.read(CONTEXT + 12, 4) == struct.pack("<I", OPERANDS + 1)
                    assert cpu.call(handlers[3], CONTEXT) == 0
                    assert cpu.read(CONTEXT + 8, 4) == struct.pack("<I", OPERANDS + 1)
                    assert cpu.read(CONTEXT, 1) == b"\0"
                assert values(cpu, pointers) == initial
                dispatch.append(dict(opcode=op, index=index, valid=valid))

        presentation = []
        for op in [0x67, 0x66, 0x68, 0x6E, 0x28]:
            cpu, pointers = setup()
            initial = values(cpu, pointers)
            persistent_before = persistent(cpu)
            cpu.word(CONTEXT + 0x64, 0x02016000)
            cpu.write(0x02016000, b"\xff")
            cpu.write(OPERANDS, bytes([20, 8]) if op == 0x6E else bytes(4))
            if op == 0x28:
                cpu.half(OPERANDS, 30)
            assert cpu.call(handlers[op], CONTEXT) == int(op in [0x66, 0x6E, 0x28])
            expected = initial[:]
            if op == 0x6E:
                expected[13] = 255
            assert values(cpu, pointers) == expected, (key, op)
            assert cpu.read(CONTEXT + 2, 1) == b"\2", (key, op, "cached comparison")
            assert persistent(cpu) == persistent_before, (key, op, "persistent")
            if op == 0x28:
                callback = word(rom, handlers[op] + 0x20) & ~1
                for tick in range(30):
                    assert cpu.call(callback - 0x08000000, CONTEXT) == int(tick == 29)
                    assert values(cpu, pointers) == initial
            presentation.append(dict(opcode=op, before=initial, after=expected))

        helper = bl(rom, handlers[0x6E] + 0x12)
        callback = (word(rom, helper + CALLBACK_LITERAL[key]) & ~1) - 0x08000000
        tasks = word(rom, callback + 0x20)
        input_clear = bl(rom, callback + 0x24)
        input_inner = bl(rom, input_clear + 2)
        menu, main_state = [word(rom, input_inner + off) for off in [0x28, 0x2C]]
        choices = []
        for keys, cursor, result in [(0, 0, None), (1, 0, 1), (1, 1, 0), (2, 0, 0)]:
            cpu, pointers = setup()
            initial = values(cpu, pointers)
            persistent_before = persistent(cpu)
            cpu.word(tasks, 0x08000001 + callback)
            cpu.write(tasks + 4, b"\1")
            cpu.half(tasks + 12, 6)
            # Native window sentinel, cursor and skip-menu-sound flag. Actual
            # callback/input/cleanup functions run without skipping instructions.
            cpu.write(menu, bytes([255, 0, cursor]))
            cpu.write(menu + 11, b"\1")
            cpu.half(main_state + 0x2E, keys)
            cpu.call(callback, 0)
            expected = initial[:]
            if result is not None:
                expected[13] = result
            assert values(cpu, pointers) == expected, (key, keys, cursor)
            assert cpu.read(CONTEXT + 2, 1) == b"\2", (
                key,
                keys,
                cursor,
                "cached comparison",
            )
            assert persistent(cpu) == persistent_before, (
                key,
                keys,
                cursor,
                "persistent",
            )
            choices.append(
                dict(keys=keys, cursor=cursor, before=initial, after=expected)
            )

        cpu, pointers = setup()
        initial = values(cpu, pointers)
        cpu.word(tasks, 0x08000001 + callback)
        cpu.write(tasks + 4, b"\1")
        cpu.write(OPERANDS, bytes([20, 8]))
        assert cpu.call(handlers[0x6E], CONTEXT) == 0
        assert values(cpu, pointers) == initial
        assert cpu.read(0x08000000, len(rom)) == rom
        assert hashlib.sha256(b.ROM_PATH.read_bytes()).hexdigest() == digest
        results[key] = dict(
            md5=b.MD5[key],
            sha256=digest,
            handlers=handlers,
            standard_tables=tables,
            dispatch=dispatch,
            presentation=presentation,
            choices=choices,
            existing_menu_preserved=True,
            scope="Complete native field effects in synthetic RAM; no visible dialogue, NPC access or map-load proof",
        )
        print(
            key,
            "512 dispatches, 5 presentation calls, 30 delay ticks, 4 choices; ROM unchanged",
            flush=True,
        )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(results))


if __name__ == "__main__":
    main()
