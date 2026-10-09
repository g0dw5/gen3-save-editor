#!/usr/bin/env python3
"""Native script-warp operands in five exact private ROMs, independent mGBA ARM7.

Immediate handlers run unchanged up to their destination setter. Complete
setwarp calls verify its RAM record and false return. This does not run map
loading, prove script access, or treat destination setters as transitions.
Only disposable RAM is written; no SAV is opened and no ROM is patched.
"""

import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh, observe
from verify_event_effects import CONFIG, CONTEXT, OPERANDS

# Verified engine addresses, not a destination or map-content catalog.
RULES = {
    "BW": (0x84BEC, 0x020322E4, [0x39, 0x3A, 0x3B, 0x3D, 0xD1, 0xD7]),
    "DP": (0x84BEC, 0x020322E4, [0x39, 0x3A, 0x3B, 0x3D, 0xD1, 0xD7]),
    "ROCKET": (0xBA498, 0x020332A0, [0x39, 0x3A, 0x3B, 0x3D, 0xD1, 0xD7]),
    "ULTIMATE": (0x84BEC, 0x020322E4, [0x39, 0x3A, 0x3B, 0x3D, 0xD1, 0xD7]),
    "MERCURY133": (0x5538C, 0x02031DBC, [0x39, 0x3A, 0x3B, 0x3D, 0xD1]),
}


def signed_byte(value):
    value &= 255
    return value - 256 if value >= 128 else value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    inputs = [Path(os.environ["GEN3_ROM_" + key]) for key in RULES]
    assert all(args.output.resolve() != p.resolve() for p in inputs)
    results = {}
    for key, (setter, destination, immediate) in RULES.items():
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        digest = hashlib.sha256(rom).hexdigest()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        table, sb1, sb2, _, variables = CONFIG[key]
        handlers = {
            op: struct.unpack_from("<I", rom, table + op * 4)[0] & ~1
            for op in immediate + [0x3E]
        }
        rows = []
        scenarios = [
            (v, w, False)
            for v, w in [(0, 1), (127, 128), (255, 256), (511, 0x3FFF), (0x3FFF, 128)]
        ]
        scenarios += [
            (v, w, True) for v, w in [(0, 1), (127, 128), (255, 256), (511, 65535)]
        ]
        for op, handler in handlers.items():
            for group, number, warp in [(1, 2, 255), (127, 128, 254), (255, 255, 0)]:
                for x, y, variable in scenarios:
                    cpu = fresh(rom)
                    cpu.word(sb1, 0x02030000)
                    cpu.word(sb2, 0x02038000)
                    cpu.word(CONTEXT + 8, OPERANDS)
                    if variable:
                        first, _, base = variables[0]
                        cpu.half(base + 2, x)
                        cpu.half(base + 4, y)
                        operands = struct.pack(
                            "<BBBHH", group, number, warp, first + 1, first + 2
                        )
                    else:
                        operands = struct.pack("<BBBHH", group, number, warp, x, y)
                    cpu.write(OPERANDS, operands)
                    before = bytes([0xA5]) * 12
                    cpu.write(destination - 2, before)
                    expected = [signed_byte(v) for v in [group, number, warp, x, y]]
                    if op == 0x3E:
                        assert cpu.call(handler - 0x08000000, CONTEXT) == 0
                        after = bytearray(before)
                        after[2:5] = bytes([group, number, warp])
                        struct.pack_into("<hh", after, 6, expected[3], expected[4])
                        assert cpu.read(destination - 2, 12) == after, (
                            key,
                            operands.hex(),
                        )
                    else:
                        regs = observe(cpu, handler - 0x08000000, setter, CONTEXT)
                        arguments = [
                            v if v < 0x80000000 else v - 0x100000000 for v in regs[:4]
                        ]
                        fifth = struct.unpack("<i", cpu.read(regs[13], 4))[0]
                        assert arguments + [fifth] == expected, (
                            key,
                            op,
                            expected,
                            arguments,
                            fifth,
                        )
                        assert cpu.read(destination - 2, 12) == before
                    assert cpu.read(CONTEXT + 8, 4) == struct.pack("<I", OPERANDS + 7)
                    rows.append(
                        dict(
                            opcode=op,
                            operands=list(operands),
                            variables=variable,
                            expected_arguments=expected,
                            resolved=[x, y],
                            transition=op != 0x3E,
                        )
                    )
        assert cpu.read(0x08000000, len(rom)) == rom
        assert hashlib.sha256(b.ROM_PATH.read_bytes()).hexdigest() == digest
        results[key] = dict(
            md5=b.MD5[key],
            sha256=digest,
            handlers=handlers,
            setter=setter,
            destination=destination,
            rows=rows,
            coverage="Native operand prefixes and complete setwarp only; no access or full map-load proof",
        )
        print(key, len(rows), "native warp cases; ROM unchanged", flush=True)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(results))


if __name__ == "__main__":
    main()
