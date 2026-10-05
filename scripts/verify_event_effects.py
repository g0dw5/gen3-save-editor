#!/usr/bin/env python3
"""Persistent event-write commands in five exact ROMs, independent mGBA ARM7.

Synthetic script contexts/RAM only. No native replacements, ROM writes or SAV
inputs. Output is private evidence, never a bundled event/text/task catalog.
"""

import argparse, hashlib, json, os, struct
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh

# Verified engine dispatch/layout configuration, not extracted game content.
CONFIG = {
    "BW": (
        0x1DB67C,
        0x03005D8C,
        0x03005D90,
        [(1, 0x3FFF, 0x02031270)],
        [(0x4000, 0x100, 0x0203139C)],
    ),
    "DP": (
        0x1DB67C,
        0x03005D8C,
        0x03005D90,
        [(1, 0x3FFF, 0x02031270)],
        [(0x4000, 0x100, 0x0203139C)],
    ),
    "ROCKET": (
        0x22B218,
        0x0300524C,
        0x03005250,
        [(1, 0x3FFF, 0x02031CA8)],
        [(0x4000, 0x100, 0x02031F6C)],
    ),
    "ULTIMATE": (
        0x1DB67C,
        0x03005D8C,
        0x03005D90,
        [
            (1, 0x3FFF, 0x02031270),
            (0x4000, 0x1A0, 0x02030988),
            (0x41A0, 0x1A0, 0x02033B24),
            (0x4340, 0x1A0, 0x0203805C),
            (0x44E0, 0x1A0, 0x02038028),
        ],
        [(0x4000, 0x100, 0x0203139C)],
    ),
    "MERCURY12": (
        0x15F9B4,
        0x03005008,
        0x0300500C,
        [(1, 0x8FF, 0x02030EE0), (0x900, 0x1000, 0x0203B174)],
        [(0x4000, 0x100, 0x02031000), (0x5000, 0x200, 0x0203B374)],
    ),
}
CONTEXT, OPERANDS = 0x02010000, 0x02011000


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    inputs = [Path(os.environ["GEN3_ROM_" + key]) for key in CONFIG]
    assert all(args.output.resolve() != p.resolve() for p in inputs)
    result = {}
    for key, (commands, sb1, sb2, flags, variables) in CONFIG.items():
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        handlers = {
            op: struct.unpack_from("<I", rom, commands + op * 4)[0] & ~1
            for op in [0x16, 0x17, 0x18, 0x19, 0x1A, 0x29, 0x2A, 0x0F]
        }
        rows = []

        def setup():
            cpu = fresh(rom)
            cpu.word(sb1, 0x02030000)
            cpu.word(sb2, 0x02038000)
            cpu.word(CONTEXT + 8, OPERANDS)
            return cpu

        def call(cpu, op, operands):
            cpu.write(OPERANDS, operands)
            assert cpu.call(handlers[op] - 0x08000000, CONTEXT) == 0
            assert int.from_bytes(cpu.read(CONTEXT + 8, 4), "little") == OPERANDS + len(
                operands
            )

        for first, count, base in flags:
            # The initial first=1 range still indexes from flag zero's native base.
            bias = 0 if first == 1 else first
            for dst in sorted({first, first + 1, first + count - 1}):
                addr = base + (dst - bias) // 8
                mask = 1 << ((dst - bias) % 8)
                for op in [0x29, 0x2A]:
                    for before in [0x55, 0xAA]:
                        cpu = setup()
                        cpu.write(addr - 1, bytes([0xC3, before, 0x3C]))
                        call(cpu, op, struct.pack("<H", dst))
                        expected = before | mask if op == 0x29 else before & ~mask
                        assert cpu.read(addr - 1, 3) == bytes([0xC3, expected, 0x3C]), (
                            key,
                            op,
                            dst,
                        )
                        rows.append(
                            dict(
                                opcode=op,
                                destination=dst,
                                result=1 if op == 0x29 else 0,
                            )
                        )
        for first, count, base in variables:
            for dst in [first, first + count - 1]:
                src = first + 1
                addr = base + (dst - first) * 2
                srcaddr = base + 2
                for op in [0x16, 0x17, 0x18, 0x19, 0x1A]:
                    for before, operand in [
                        (0, 1),
                        (65535, 1),
                        (1, 65535),
                        (12345, 0x8010),
                    ]:
                        cpu = setup()
                        cpu.half(addr, before)
                        source_value = 54321
                        if op == 0x19 or (op in [0x18, 0x1A] and operand >= 0x4000):
                            operand = src
                            cpu.half(srcaddr, source_value)
                        if op in [0x19, 0x1A] and operand >= 0x4000:
                            expected = source_value
                        elif op == 0x17:
                            expected = (before + operand) & 65535
                        elif op == 0x18:
                            expected = (
                                before
                                - (source_value if operand >= 0x4000 else operand)
                            ) & 65535
                        else:
                            expected = operand
                        before_block = cpu.read(base, count * 2)
                        call(cpu, op, struct.pack("<HH", dst, operand))
                        after = bytearray(before_block)
                        struct.pack_into("<H", after, (dst - first) * 2, expected)
                        assert cpu.read(base, count * 2) == bytes(after), (
                            key,
                            op,
                            hex(dst),
                            expected,
                        )
                        rows.append(
                            dict(
                                opcode=op,
                                destination=dst,
                                operand=operand,
                                before=before,
                                source_value=source_value,
                                result=expected,
                            )
                        )
        # Loadpointer bank 0 is the ordinary message fallback context. This is
        # a pointer reference, not proof that a particular dialogue is displayed.
        cpu = setup()
        call(cpu, 0x0F, b"\0" + struct.pack("<I", 0x08000100))
        assert cpu.read(CONTEXT + 0x64, 4) == struct.pack("<I", 0x08000100)
        assert cpu.read(0x08000000, len(rom)) == rom
        assert b.ROM_PATH.read_bytes() == rom
        result[key] = dict(
            md5=b.MD5[key],
            engine="mGBA ARM7",
            handlers=handlers,
            rows=rows,
            text_context=True,
        )
        print(
            key,
            len(rows),
            "native persistent command cases; neighbors and ROM unchanged",
            flush=True,
        )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result))


if __name__ == "__main__":
    main()
