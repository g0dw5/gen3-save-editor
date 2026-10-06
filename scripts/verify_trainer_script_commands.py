#!/usr/bin/env python3
"""Native trainer-command operand boundaries in isolated mGBA RAM.

No ROM writes or user SAV inputs. Emit private context/continuation evidence,
not a trainer catalog. Uses the existing readonly native probe.
"""

import argparse, hashlib, json, os, struct
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh
from verify_event_effects import CONFIG, CONTEXT, OPERANDS

# Verified engine RAM fields: return-to-field script and opponent A/B.
FIELDS = {
    "BW": (0x02038BEC, 0x02038BCA, 0x02038BCC),
    "DP": (0x02038BEC, 0x02038BCA, 0x02038BCC),
    "ROCKET": (0x02039BCC, 0x02039BAA, 0x02039BAC),
    "ULTIMATE": (0x02038BEC, 0x02038BCA, 0x02038BCC),
    "MERCURY12": (0x020386C4, 0x020386AE, 0x020386B0),
}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--mgba-probe", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    args = p.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    result = {}
    for key, (table, sb1, sb2, flags, variables) in CONFIG.items():
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        assert args.output.resolve() != b.ROM_PATH.resolve()
        handler = struct.unpack_from("<I", rom, table + 0x5C * 4)[0] & ~1
        continuation, opponent_a, opponent_b = FIELDS[key]
        rows = []
        for typ in range(17):
            for trainer, context in [(1, 0), (0x123, 0), (0x4321, 0)] + (
                [(0x123, 1), (0x123, 2)] if key == "MERCURY12" else []
            ):
                cpu = fresh(rom)
                cpu.word(sb1, 0x02030000)
                cpu.word(sb2, 0x02038000)
                cpu.word(CONTEXT + 8, OPERANDS)
                if key == "MERCURY12":
                    cpu.write(0x03000F28, bytes([context]))
                data = bytearray(32)
                data[0] = typ
                struct.pack_into("<H", data, 1, trainer)
                for offset in [5, 9, 13, 17, 21]:
                    struct.pack_into("<I", data, offset, 0x02012000 + offset)
                cpu.write(OPERANDS, data)
                before = cpu.read(0x08000000, len(rom))
                returned = cpu.call(handler - 0x08000000, CONTEXT)
                saved_pc = int.from_bytes(cpu.read(continuation, 4), "little")
                length = (
                    saved_pc - OPERANDS + 1
                    if OPERANDS < saved_pc < OPERANDS + 32
                    else None
                )
                rows.append(
                    dict(
                        type=typ,
                        input_trainer=trainer,
                        context=context,
                        operand_length=length,
                        opponent_a=int.from_bytes(cpu.read(opponent_a, 2), "little"),
                        opponent_b=int.from_bytes(cpu.read(opponent_b, 2), "little"),
                        context_pc=int.from_bytes(cpu.read(CONTEXT + 8, 4), "little"),
                        return_value=returned,
                    )
                )
                assert cpu.read(0x08000000, len(rom)) == before == rom
        assert b.ROM_PATH.read_bytes() == rom
        result[key] = dict(md5=b.MD5[key], handler=handler - 0x08000000, rows=rows)
        print(key, len(rows), "native command scenarios; input unchanged", flush=True)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
