#!/usr/bin/env python3
"""Verify field EV classification/application in exact ROMs using disposable mGBA RAM.

Uses runtime CLI metadata, independent raw individual fixtures and complete native
routines. No actual save is opened or written. Output is private evidence, never
release content. Menu eligibility, consumption and live context are out of scope.
"""

import argparse
import hashlib
import json
import os
import struct
import subprocess
from pathlib import Path
import verify_breeding as b

CONFIG = {
    "BW": (0x1B7CEC, 0x6BD04, [0x080FDEA1, 0x080FDEBD], 175),
    "DP": (0x1B7CEC, 0x6BD04, [0x080FDEA1, 0x080FDEBD], 175),
    "ROCKET": (0x2064AC, 0x98EB0, [0x081361D5], 303),
    "ULTIMATE": (0x1B7CEC, 0x6BD04, [0x080FDEA1, 0x080FDEBD], 175),
    "MERCURY12": (0x126C68, 0x42414, [0x080A16E1], 175),
}
STAT = {13: 0, 12: 1, 17: 2, 16: 3, 14: 4, 15: 5}


def fixture(key, evs, friendship):
    raw = bytearray(b.parent(key, 25, 42, 1))
    canonical = bytearray(48)
    struct.pack_into("<H", canonical, 0, 25)
    struct.pack_into("<4H", canonical, 12, 1, 2, 3, 4)
    canonical[20:24] = bytes([10] * 4)
    canonical[24:30] = bytes(evs)
    if key == "ROCKET":
        struct.pack_into("<I", canonical, 8, friendship | (26 << 13))
    else:
        canonical[9] = friendship
    struct.pack_into("<I", canonical, 40, 0x12345678)
    if key in ("ULTIMATE", "MERCURY12"):
        raw[32:80] = canonical
        struct.pack_into("<H", raw, 28, sum(struct.unpack("<24H", canonical)) & 65535)
    else:
        raw = bytearray(b.pack(raw, canonical))
    return bytes(raw) + bytes(20)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gen3", type=Path, required=True)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    out = {}
    for key, (classify, apply, handlers, enigma) in CONFIG.items():
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        catalog = json.loads(
            subprocess.check_output([str(args.gen3), "catalog", str(b.ROM_PATH)])
        )
        cpu = b.MGBANative(rom)
        categories = []
        rows = []
        br = b.CONFIG[key]
        for item in catalog["items"]:
            handler = struct.unpack_from("<I", rom, item["offset"] + 28)[0]
            if item["id"] == enigma or handler not in handlers:
                continue
            category = cpu.call(classify, item["id"])
            categories.append(dict(item=item["id"], category=category, handler=handler))
            if category not in STAT:
                continue
            stat = STAT[category]
            cases = []
            for value in [0, 90, 99, 100, 251, 252, 253, 255]:
                evs = [0] * 6
                evs[stat] = value
                cases.append((evs, 0))
            cases += [
                ([255, 255, 0, 0, 0, 0], 254),
                ([255, 255, 1, 0, 0, 0], 255),
                ([10] * 6, 100),
            ]
            for evs, friendship in cases:
                cpu = b.MGBANative(rom)
                cpu.word(br[3], 0x02030000)
                cpu.word(br[4], 0x02034000)
                raw = fixture(key, evs, friendship)
                cpu.write(br[6], raw)
                cpu.write(br[7], b"\x01")
                result = cpu.call(apply, br[6], item["id"], 0, 0)
                after = cpu.read(br[6], 100)
                assert after[:8] == raw[:8]
                canonical_after = (
                    after[32:80]
                    if key in ("ULTIMATE", "MERCURY12")
                    else b.unpack(after)
                )
                assert all(
                    canonical_after[24 + i] == evs[i] for i in range(6) if i != stat
                ), (key, item["id"], "wrong EV axis")
                rows.append(
                    dict(
                        item=item["id"],
                        before=list(raw),
                        after=list(after),
                        no_effect=result != 0,
                    )
                )
        assert cpu.read(0x08000000, len(rom)) == rom
        assert hashlib.md5(b.ROM_PATH.read_bytes()).hexdigest() == b.MD5[key]
        out[key] = dict(md5=b.MD5[key], categories=categories, rows=rows)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(out, indent=2))
        print(
            key,
            len(categories),
            "classified items",
            len(rows),
            "native application cases",
            flush=True,
        )


if __name__ == "__main__":
    main()
