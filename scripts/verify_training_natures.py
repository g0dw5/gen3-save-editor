#!/usr/bin/env python3
"""Verify runtime mint targets and persistent native effects in disposable mGBA RAM.

The Rocket field callback is observed before bag consumption. Complete native
SetMonData/CalculateMonStats routines are also executed without hooks. Menu UI,
item consumption, access and inheritance are outside this bounded verifier.
"""

import argparse
import hashlib
import json
import os
import struct
import subprocess
from pathlib import Path
import verify_breeding as b
import verify_training_items as ev


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gen3", type=Path, required=True)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    b.ROM_PATH = Path(os.environ["GEN3_ROM_ROCKET"])
    rom = b.ROM_PATH.read_bytes()
    assert hashlib.md5(rom).hexdigest() == b.MD5["ROCKET"]
    catalog = json.loads(
        subprocess.check_output([str(args.gen3), "catalog", str(b.ROM_PATH)])
    )
    offers = []
    rows = []
    party = b.CONFIG["ROCKET"][6]
    for item in catalog["items"]:
        if struct.unpack_from("<I", rom, item["offset"] + 28)[0] != 0x08136DDD:
            continue
        cpu = b.MGBANative(rom)
        target = cpu.call(0x10FA0C, item["id"])
        assert target == rom[item["offset"] + 40] and target < 25
        # Observe the real field callback initialization before its UI stage:
        # it reads the selected item and current effective nature into task data.
        raw_initial = ev.fixture("ROCKET", [0] * 6, 70)
        cpu.write(party, raw_initial)
        cpu.half(0x0203DE18, item["id"])
        cpu.write(0x0203DE64 + 9, b"\x00")
        regs = (cpu.ctypes.c_uint * 16)()
        assert cpu.c.calluntil(0x08208350, 0, 0, 0, 0, 0x082083BE, regs)
        assert int.from_bytes(cpu.read(0x030052C8 + 12, 2), "little") == target
        assert cpu.read(party, 100) == raw_initial
        offers.append(dict(item=item["id"], nature=target))
        for permutation in range(24):
            pid = 2400 + permutation
            raw = bytearray(ev.fixture("ROCKET", [252, 0, 4, 0, 0, 252], 177))
            canonical = bytearray(b.unpack(raw))
            species = 1 + permutation % 6
            struct.pack_into("<H", canonical, 0, species)
            growth = rom[0x5B4764 + species * 36 + 21]
            xp = struct.unpack_from(
                "<I", rom, 0x5B3484 + growth * 604 + (5 + permutation) * 4
            )[0]
            struct.pack_into("<I", canonical, 4, xp)
            initial = 26 if permutation % 2 else target
            packed = struct.unpack_from("<I", canonical, 8)[0]
            struct.pack_into("<I", canonical, 8, (packed & ~(31 << 13)) | initial << 13)
            canonical[30:36] = bytes([13, 27, 43, 61, 89, 99])
            struct.pack_into("<H", canonical, 38, 0x1943)
            struct.pack_into("<I", canonical, 44, 0x12345678)
            struct.pack_into("<II", raw, 0, pid, 0x12345678)
            raw = bytearray(b.pack(raw, canonical))
            cpu = b.MGBANative(rom)
            cpu.word(b.CONFIG["ROCKET"][3], 0x02030000)
            cpu.word(b.CONFIG["ROCKET"][4], 0x02034000)
            cpu.write(party, raw)
            cpu.call(0x967B4, party)
            baseline = bytearray(cpu.read(party, 100))
            max_hp = struct.unpack_from("<H", baseline, 88)[0]
            for hp in [0, max_hp // 2, max_hp]:
                before = bytearray(baseline)
                struct.pack_into("<I", before, 80, 8)
                struct.pack_into("<H", before, 86, hp)
                cpu = b.MGBANative(rom)
                cpu.word(b.CONFIG["ROCKET"][3], 0x02030000)
                cpu.word(b.CONFIG["ROCKET"][4], 0x02034000)
                cpu.write(party, before)
                cpu.write(0x02038000, bytes([target]))
                cpu.call(0x97E2C, party, 89, 0x02038000)
                cpu.call(0x967B4, party)
                applied = cpu.read(party, 100)
                effective = cpu.call(0x9A334, party, 1)
                # Read the original effective nature independently before testing
                # the field callback's equality rejection.
                cpu.write(party, before)
                current_nature = cpu.call(0x9A334, party, 1)
                no_effect = current_nature == target
                after = bytes(before) if no_effect else applied
                assert after[:28] == before[:28] and after[80:84] == before[80:84]
                cafter = bytearray(b.unpack(after))
                original = bytearray(b.unpack(before))
                cafter[9] = (cafter[9] & 31) | (original[9] & 224)
                cafter[10] = (cafter[10] & 252) | (original[10] & 3)
                assert cafter == original, "Unrelated persistent bytes changed"
                # Full native field callback's confirmed-apply state executes the
                # same setters before its bag-removal/UI stage. No function hooks.
                cpu = b.MGBANative(rom)
                cpu.word(b.CONFIG["ROCKET"][3], 0x02030000)
                cpu.word(b.CONFIG["ROCKET"][4], 0x02034000)
                cpu.write(party, before)
                cpu.half(0x0203DE18, item["id"])
                cpu.half(0x030052C8, 5)
                cpu.half(0x030052C8 + 6, 0)
                cpu.half(0x030052C8 + 12, target)
                regs = (cpu.ctypes.c_uint * 16)()
                assert cpu.c.calluntil(0x08208160, 0, 0, 0, 0, 0x08208320, regs)
                assert cpu.read(party, 100) == applied
                assert effective == target
                cpu.half(0x030052C8, 0)
                cpu.half(0x030052C8 + 4, current_nature)
                assert cpu.c.calluntil(
                    0x08208160,
                    0,
                    0,
                    0,
                    0,
                    0x082081B0 if no_effect else 0x082081E8,
                    regs,
                )
                rows.append(
                    dict(
                        item=item["id"],
                        nature=target,
                        no_effect=no_effect,
                        before=list(before),
                        after=list(after),
                        confirmed_stage=list(applied),
                    )
                )
    assert len(offers) == 21 and len(rows) == 1512
    assert cpu.read(0x08000000, len(rom)) == rom
    assert hashlib.md5(b.ROM_PATH.read_bytes()).hexdigest() == b.MD5["ROCKET"]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(dict(md5=b.MD5["ROCKET"], offers=offers, rows=rows), indent=2)
    )
    print(
        len(offers),
        "runtime mint targets;",
        len(rows),
        "byte-preserving native stage cases",
    )


if __name__ == "__main__":
    main()
