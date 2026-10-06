#!/usr/bin/env python3
"""Mercury 1.2 crown field/level/payment evidence in disposable mGBA RAM.

Complete native calls, no hooks, source SAV or ROM writes. Synthetic valid
individuals use runtime species/experience data. Payment commands are tested
separately from menus; this does not certify the entire NPC transaction/access.
Keep generated vectors private, outside release inputs.
"""

import argparse
import hashlib
import json
import os
import struct
import subprocess
from pathlib import Path
import verify_breeding as b
import verify_training_abilities as abilities

PARTY, SELECTED = 0x02024284, 0x020370C0
CONTEXT, SLOTS, DESCRIPTORS = 0x02010000, 0x02009000, 0x0203988C
COMMANDS = 0x15F9B4


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gen3", type=Path, required=True)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    b.ROM_PATH = Path(os.environ["GEN3_ROM_MERCURY12"])
    b.MGBA_PROBE = args.mgba_probe.resolve()
    rom = b.ROM_PATH.read_bytes()
    digest = hashlib.sha256(rom).hexdigest()
    assert hashlib.md5(rom).hexdigest() == b.MD5["MERCURY12"]
    catalog = json.loads(
        subprocess.check_output([str(args.gen3), "catalog", str(b.ROM_PATH)])
    )
    cpu = b.MGBANative(rom)
    records = {s["id"]: s for s in catalog["species"]}
    rows, levels = [], []
    # These are explicit fixture identities, not a shipped acquisition catalogue.
    for species in [1, 25, 137]:
        for level in [49, 50, 100]:
            for pid in [42, 43]:
                for ability in [0, 2]:
                    raw = abilities.raw_individual(
                        "MERCURY12",
                        species,
                        ability,
                        pid=pid,
                        level=level,
                        rom=rom,
                        species_record=records[species],
                    )
                    cpu.write(PARTY, raw)
                    cpu.call(0x3E47C, PARTY)
                    raw = bytearray(cpu.read(PARTY, 100))
                    maximum = struct.unpack_from("<H", raw, 88)[0]
                    struct.pack_into(
                        "<H", raw, 86, 0 if pid == 42 else max(1, maximum - 1)
                    )
                    for slot in range(6):
                        before_party = bytes(raw) * 6
                        cpu.write(PARTY, before_party)
                        cpu.half(SELECTED, slot)
                        cpu.call(0x96A730)
                        actual = struct.unpack("<H", cpu.read(SELECTED + 4, 2))[0]
                        assert actual == level
                        assert cpu.read(PARTY, 600) == before_party
                        accepted = []
                        for check_at in [0x7B07B0, 0x7B099B]:
                            for op, at in [(0x21, check_at), (0x06, check_at + 5)]:
                                entry = struct.unpack_from(
                                    "<I", rom, COMMANDS + op * 4
                                )[0]
                                cpu.word(CONTEXT + 8, 0x08000000 + at + 1)
                                cpu.call(entry - 0x08000001, CONTEXT)
                            target = struct.unpack("<I", cpu.read(CONTEXT + 8, 4))[0]
                            ok = target == 0x08000000 + check_at + 11
                            assert ok == (level >= 50)
                            accepted.append(ok)
                        levels.append(
                            dict(
                                slot=slot,
                                before=list(raw),
                                actual=actual,
                                accepted=accepted,
                            )
                        )
                        for selector in range(7):
                            cpu.write(PARTY, before_party)
                            cpu.half(SELECTED - 2, 0)  # party, not alternate PC wrapper
                            cpu.half(SELECTED, slot)
                            cpu.half(SELECTED + 2, selector)
                            cpu.half(SELECTED + 4, 31)
                            cpu.call(0x1D5C1D8)
                            after_party = cpu.read(PARTY, 600)
                            if selector == 6:
                                # Actual gold script performs six complete single-stat
                                # calls, rather than using diagnostic selector 6.
                                cpu.write(PARTY, before_party)
                                for stat in range(6):
                                    cpu.half(SELECTED + 2, stat)
                                    cpu.half(SELECTED + 4, 31)
                                    cpu.call(0x1D5C1D8)
                                assert (
                                    cpu.read(PARTY, 600) == after_party
                                ), "gold sequence differs from all-stat helper"
                            at = slot * 100
                            after = after_party[at : at + 100]
                            assert after_party[:at] == before_party[:at]
                            assert after_party[at + 100 :] == before_party[at + 100 :]
                            bits = struct.unpack_from("<I", raw, 72)[0]
                            expected = bits
                            for stat in range(6):
                                if selector == 6 or selector == stat:
                                    expected = (expected & ~(31 << (stat * 5))) | (
                                        31 << (stat * 5)
                                    )
                            assert struct.unpack_from("<I", after, 72)[0] == expected
                            assert (
                                after[:72] == raw[:72]
                            )  # PID, trainer, identity, moves, EVs, origin
                            assert (
                                after[76:86] == raw[76:86]
                            )  # ribbons, status, level, mail
                            assert (
                                after[75] & 192 == raw[75] & 192
                            )  # egg/hidden ability
                            rows.append(
                                dict(
                                    slot=slot,
                                    selector=selector,
                                    before=list(raw),
                                    after=list(after),
                                )
                            )
    # Invalid selected slots must not touch any individual.
    invalid = []
    for slot in [6, 7, 255, 65535]:
        cpu.write(PARTY, before_party)
        cpu.half(SELECTED, slot)
        cpu.half(SELECTED + 2, 6)
        cpu.half(SELECTED + 4, 31)
        cpu.call(0x1D5C1D8)
        assert cpu.read(PARTY, 600) == before_party
        cpu.call(0x96A730)
        assert cpu.read(SELECTED + 4, 2) == bytes(2)
        invalid.append(slot)
    # Native script item checks/removals: independent ROM operands and bag data.
    cpu = b.MGBANative(rom)
    cpu.word(0x03005008, 0x02020000)
    cpu.word(0x0300500C, 0x02030000)
    result = cpu.call(0x6E454, 0x800D)
    payments = []
    for branch, check_at, pay_at in [
        ("gold", 0x7B0097, 0x7B0116),
        ("silver", 0x7B00C0, 0x7B019C),
    ]:
        assert rom[check_at] == 0x47 and rom[pay_at] == 0x45
        required, quantity = struct.unpack_from("<HH", rom, check_at + 1)
        removed, removed_quantity = struct.unpack_from("<HH", rom, pay_at + 1)
        assert cpu.call(0x9A9D8, required) == cpu.call(0x9A9D8, removed) == 1
        for silver in [0, 1, 2, 65535]:
            for gold in [0, 1, 2, 65535]:
                cpu.write(SLOTS, struct.pack("<4H", 639, silver, 640, gold) + bytes(32))
                cpu.word(DESCRIPTORS, SLOTS)
                cpu.half(DESCRIPTORS + 4, 10)
                observed = []
                for op, at in [(0x47, check_at), (0x45, pay_at)]:
                    entry = struct.unpack_from("<I", rom, COMMANDS + op * 4)[0]
                    cpu.word(CONTEXT + 8, 0x08000000 + at + 1)
                    cpu.half(result, 7)
                    cpu.call(entry - 0x08000001, CONTEXT)
                    observed.append(struct.unpack("<H", cpu.read(result, 2))[0])
                slots = list(struct.unpack("<20H", cpu.read(SLOTS, 40)))
                bag = {slots[i]: slots[i + 1] for i in range(0, 20, 2) if slots[i]}
                assert observed == [
                    int((gold if branch == "gold" else silver) >= quantity),
                    int(gold >= removed_quantity),
                ]
                assert bag.get(639, 0) == silver
                assert bag.get(640, 0) == max(0, gold - removed_quantity)
                payments.append(
                    dict(
                        branch=branch,
                        silver=silver,
                        gold=gold,
                        required=required,
                        removed=removed,
                        check_result=observed[0],
                        remove_result=observed[1],
                        bag=bag,
                    )
                )
    assert hashlib.sha256(b.ROM_PATH.read_bytes()).hexdigest() == digest
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(
            dict(
                rom_md5=b.MD5["MERCURY12"],
                levels=levels,
                rows=rows,
                invalid_slots=invalid,
                payments=payments,
            ),
            indent=2,
        )
    )
    print(
        f"{len(levels)} level, {len(rows)} IV/stat, {len(payments)} payment cases; ROM unchanged"
    )
    cpu.c.finish()


if __name__ == "__main__":
    main()
