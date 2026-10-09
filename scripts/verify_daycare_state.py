#!/usr/bin/env python3
"""Native saved ordinary daycare reads and next-check phases across five ROMs.

Exact private ROMs, synthetic RAM only. No user save is opened or written. This
proves ordinary field-step checkpoints, not time until an egg or custom access.
"""

import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
import verify_breeding as breeding
from verify_breeding_production import fresh, CONFIG as PRODUCTION

CONFIG = {
    "BW": (0x6A674, 0x70BF0, 0x70CB0),
    "DP": (0x6A674, 0x70BF0, 0x70CB0),
    "ULTIMATE": (0x6A674, 0x70BF0, 0x70CB0),
    "ROCKET": (0x977D0, 0x9EBE0, 0x9ECA0),
    "MERCURY133": (0x3FD44, 0x463FC, 0x464B4),
}


def verify(key):
    breeding.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
    rom = breeding.ROM_PATH.read_bytes()
    assert hashlib.md5(rom).hexdigest() == breeding.MD5[key]
    getter, available, state = CONFIG[key]
    _, compat, offset, sb1, sb2, rng, party, count, width = breeding.CONFIG[key]
    rows, phases = [], []
    flag = struct.unpack_from("<I", rom, 0x1d1c4a4)[0] if key == "MERCURY133" else None
    counters = [0, 1, 253, 254, 255, 256, 0xFFFFFFFE, 0xFFFFFFFF]
    for mask in range(4):
        for pending in [0, 24, 65535]:
            for flag_set in ([False, True] if flag is not None else [False]):
                for steps in counters:
                    cpu = fresh(rom)
                    cpu.word(sb1, 0x02030000)
                    cpu.word(sb2, 0x02034000)
                    daycare = 0x02030000 + offset
                    parents = [
                        (
                            breeding.parent(key, 25, 0 if i == 0 else 255, i + 1)
                            if mask & (1 << i)
                            else bytes(80)
                        )
                        for i in range(2)
                    ]
                    for i, raw in enumerate(parents):
                        cpu.write(daycare + 140 * i, raw)
                        cpu.word(daycare + 140 * i + 136, steps)
                    cpu.write(daycare + 280, pending.to_bytes(width, "little"))
                    if flag_set:
                        # The native setter address is read from the patched generator.
                        setter = (
                            struct.unpack_from("<I", rom, 0x1D1C494)[0] & ~1
                        ) - 0x08000000
                        cpu.call(setter, flag)
                    before = cpu.read(0x02030000, 0x8000)
                    presence = [
                        cpu.call(getter, daycare + 140 * i, 5, 0) for i in range(2)
                    ]
                    actual_available = cpu.call(available, daycare)
                    actual_state = cpu.call(state)
                    compatibility = cpu.call(compat, daycare) if mask == 3 else None
                    assert presence == [int(bool(mask & (1 << i))) for i in range(2)]
                    assert actual_available == int(
                        flag_set if flag is not None else pending != 0
                    )
                    expected_state = (
                        1 if actual_available else (sum(presence) + 1 if mask else 0)
                    )
                    assert actual_state == expected_state
                    assert cpu.read(0x02030000, 0x8000) == before
                    rows.append(
                        dict(
                            parents=[list(p) for p in parents],
                            steps=steps,
                            pending=pending,
                            flag=flag,
                            flag_set=flag_set,
                            presence=presence,
                            available=actual_available,
                            native_state=actual_state,
                            compatibility=compatibility,
                        )
                    )
    # Replay sequential complete ordinary steps up to the FIRST native comparison.
    # No gate/Random function is replaced. The last call stops before comparison;
    # previous calls return normally, preserving counter phase and RAM state.
    step, comparison = PRODUCTION[key][:2]
    for steps in counters:
        cpu = fresh(rom)
        cpu.word(sb1, 0x02030000)
        cpu.word(sb2, 0x02034000)
        daycare = 0x02030000 + offset
        for i in range(2):
            cpu.write(
                daycare + 140 * i, breeding.parent(key, 25, 0 if i == 0 else 255, i + 1)
            )
            cpu.word(daycare + 140 * i + 136, steps)
        regs = (cpu.ctypes.c_uint * 16)()
        reached = None
        for n in range(1, 257):
            if cpu.c.calluntil(
                0x08000000 + step, daycare, 0, 0, 0, 0x08000000 + comparison, regs
            ):
                reached = n
                break
        assert reached is not None, (key, steps)
        phases.append(dict(steps=steps, next_check=reached))
    assert cpu.read(0x08000000, len(rom)) == rom
    assert breeding.ROM_PATH.read_bytes() == rom
    return dict(md5=breeding.MD5[key], engine="mGBA ARM7", rows=rows, phases=phases)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    breeding.MGBA_PROBE = args.mgba_probe.resolve()
    result = {}
    for key in CONFIG:
        result[key] = verify(key)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result), encoding="utf-8")
        print(
            key,
            len(result[key]["rows"]),
            "saved states; 8 sequential native phases",
            flush=True,
        )
