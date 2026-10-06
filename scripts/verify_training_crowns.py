#!/usr/bin/env python3
"""Observe Ultimate's referenced NPC crown helpers in disposable mGBA RAM.

Independent complete native calls, no ROM writes, SAV input or function hooks.
Private output covers byte selection/mutation, not complete menu/payment/access.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import verify_breeding as b
import verify_training_items as ev


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gen3", type=Path, required=True)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    import subprocess

    b.MGBA_PROBE = args.mgba_probe.resolve()
    b.ROM_PATH = Path(os.environ["GEN3_ROM_ULTIMATE"])
    rom = b.ROM_PATH.read_bytes()
    digest = hashlib.sha256(rom).hexdigest()
    assert hashlib.md5(rom).hexdigest() == b.MD5["ULTIMATE"]
    report = json.loads(
        subprocess.check_output([str(args.gen3), "training-services", str(b.ROM_PATH)])
    )
    assert len(report["services"]) == 1
    service = report["services"][0]
    rules = service["evidence"]
    cpu = b.MGBANative(rom)
    party = b.CONFIG["ULTIMATE"][6]
    selected = rules["selected_individual"]
    base = bytearray(ev.fixture("ULTIMATE", [252, 0, 4, 0, 0, 252], 177))
    base[84] = 100
    individuals = bytes(base) * 6
    levels = []
    for slot in range(6):
        for level in [0, 1, 50, 99, 100, 101, 127, 255]:
            raw = bytearray(individuals)
            raw[slot * 100 + 84] = level
            cpu.write(party, raw)
            cpu.half(selected, slot)
            cpu.half(selected + 2, 84)
            cpu.call(rules["level_reader"] - 0x08000000)
            actual = int.from_bytes(cpu.read(selected + 2, 2), "little")
            assert actual == level
            assert cpu.read(party, 600) == raw
            levels.append(dict(slot=slot, level=level, actual=actual))
    rows = []
    changed = unchanged = 0
    for slot in range(6):
        for header in range(256):
            for choice in service["choices"]:
                raw = bytearray(individuals)
                raw[slot * 100 + 30] = header
                cpu.write(party, raw)
                cpu.half(selected, slot)
                cpu.half(selected + 2, 126 if choice["menu_index"] == 0 else 1)
                if choice["menu_index"]:
                    cpu.write(rules["selected_choice"], bytes([choice["menu_index"]]))
                    cpu.call(rules["selection_mask"] - 0x08000000)
                mask = int.from_bytes(cpu.read(selected + 2, 2), "little")
                assert mask == choice["mask"]
                cpu.call(rules["mark"] - 0x08000000)
                expected = bytearray(raw)
                expected[slot * 100 + 30] |= mask
                after = cpu.read(party, 600)
                assert after == expected, "unrelated party bytes changed"
                previous, current = (
                    cpu.read(selected + 4, 1)[0],
                    cpu.read(selected + 6, 1)[0],
                )
                assert previous == header and current == (header | mask)
                changed += current != previous
                unchanged += current == previous
                rows.append(
                    dict(
                        slot=slot,
                        header=header,
                        menu_index=choice["menu_index"],
                        mask=mask,
                        previous=previous,
                        current=current,
                    )
                )
    assert hashlib.sha256(b.ROM_PATH.read_bytes()).hexdigest() == digest
    args.output.write_text(
        json.dumps(
            dict(
                rom_md5=report["rom_md5"],
                levels=levels,
                rows=rows,
                changed=changed,
                unchanged=unchanged,
            ),
            indent=2,
        )
    )
    print(
        f"Native crown helpers: {len(levels)} byte-reader cases, {len(rows)} complete mutation cases; {changed} changed, {unchanged} unchanged; ROM unchanged"
    )
    cpu.c.finish()


if __name__ == "__main__":
    main()
