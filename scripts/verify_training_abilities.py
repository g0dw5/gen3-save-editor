#!/usr/bin/env python3
"""Observe exact-ROM ability field paths before UI/consumption, in disposable RAM.

No function hooks, no source save, no ROM writes. Private vectors are independent
of the core runner and must not be committed or bundled.
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

SLOT_RULES = {
    0x081361F1: (0x203398, 0x2033FA, 0x20318C, 0x203238, 0x2031FA, 0x20336A),
    0x0813620D: (0x203648, 0x2036AC, 0x203430, 0x2034E8, 0x2034AC, 0x20361A),
}


def raw_individual(
    key, species, slot, pid=42, shiny=False, level=1, rom=None, species_record=None
):
    raw = bytearray(ev.fixture(key, [252, 0, 4, 0, 0, 252], 177))
    canonical = bytearray(b.unpack(raw) if key == "ROCKET" else raw[32:80])
    if key == "MERCURY12":
        pid = (pid & ~1) | (1 if slot == 1 else 0)
        canonical[43] = (canonical[43] & 127) | (128 if slot == 2 else 0)
    elif key == "ULTIMATE":
        canonical[43] = (canonical[43] & 127) | ((slot & 1) << 7)
        raw[30] = 0x56 | int(slot >= 2)
        raw[31] = 0x86
    else:
        canonical[47] = (canonical[47] & 252) | slot
    ot = pid if shiny else 1
    struct.pack_into("<II", raw, 0, pid, ot)
    struct.pack_into("<H", canonical, 0, species)
    if rom is not None:
        base, stride = {
            "ROCKET": (0x5B3484, 604),
            "MERCURY12": (0x1E052C8, 1024),
            "ULTIMATE": (0x31F72C, 404),
        }[key]
        xp = struct.unpack_from(
            "<I", rom, base + species_record["growth"] * stride + level * 4
        )[0]
        struct.pack_into("<I", canonical, 4, xp)
    canonical[30:36] = bytes([13, 27, 43, 61, 89, 99])
    struct.pack_into("<H", canonical, 38, 0x1943)
    if key == "ROCKET":
        raw = bytearray(b.pack(raw, canonical))
    else:
        raw[32:80] = canonical
        struct.pack_into("<H", raw, 28, sum(struct.unpack("<24H", canonical)) & 65535)
    struct.pack_into("<I", raw, 80, 8)
    if key == "ULTIMATE" and rom is not None:
        raw[84] = level
        struct.pack_into("<H6H", raw, 86, 75, 125, 63, 70, 64, 62, 66)
    return bytes(raw)


def script_stage(cpu, item, stop, regs):
    cpu.half(0x020375E0, 0)
    cpu.half(0x0203CE7C, item)
    decision = cpu.c.callchoice(0x09F00DD0, 0, 0, 0, 0, stop, 0x09F00E3E, regs)
    assert decision in (1, 2), "script prefix did not reach a verified boundary"
    return decision == 1, cpu.read(regs[6], 1)[0] if decision == 1 else 0


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--gen3", type=Path, required=True)
    p.add_argument("--mgba-probe", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    args = p.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    result = {}
    for key in ["ROCKET", "MERCURY12", "ULTIMATE"]:
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        cat = json.loads(
            subprocess.check_output([str(args.gen3), "catalog", str(b.ROM_PATH)])
        )
        cpu = b.MGBANative(rom)
        party = b.CONFIG[key][6]
        regs = (cpu.ctypes.c_uint * 16)()
        offers = []
        guards = []
        rows = []
        for item in cat["items"]:
            handler = struct.unpack_from("<I", rom, item["offset"] + 28)[0]
            if (
                handler
                not in {
                    "ROCKET": SLOT_RULES,
                    "MERCURY12": [0x09D58019],
                    "ULTIMATE": [0x08F7F111],
                }[key]
            ):
                continue
            offer = dict(item=item["id"], handler=handler)
            if key == "ULTIMATE":
                offer["variant"] = cpu.call(0xD7644, item["id"])
                assert offer["variant"] in [1, 2]
            offers.append(offer)
            for s in cat["species"]:
                for slot in range(4 if key == "ULTIMATE" else 3):
                    before = raw_individual(key, s["id"], slot)
                    cpu.write(party, before)
                    if key == "ROCKET":
                        init, stop, callback, yes, no, applied = SLOT_RULES[handler]
                        assert cpu.c.calluntil(
                            0x08000000 + init, 0, 0, 0, 0, 0x08000000 + stop, regs
                        )
                        target = struct.unpack("<H", cpu.read(0x030052C8 + 4, 2))[0]
                        decision = cpu.c.callchoice(
                            0x08000000 + callback,
                            0,
                            0,
                            0,
                            0,
                            0x08000000 + yes,
                            0x08000000 + no,
                            regs,
                        )
                        assert decision in (1, 2)
                        accepted = decision == 1
                    elif key == "ULTIMATE":
                        cpu.word(b.CONFIG[key][5], 42)
                        accepted, target = script_stage(
                            cpu, item["id"], 0x09F00E20, regs
                        )
                    else:
                        cpu.half(0x0203AD30, item["id"])
                        assert cpu.c.calluntil(0x09D561E0, 0, 0, 0, 0, 0x09D56246, regs)
                        target = regs[4]
                        accepted = target != 0
                    assert cpu.read(party, 100) == before
                    guards.append(
                        dict(
                            item=item["id"],
                            before=list(before),
                            target=target,
                            accepted=accepted,
                        )
                    )
            # Rich persistent cases are separate from the complete species guard sweep.
            selected = [
                s
                for s in cat["species"]
                if s["abilities"][1] and s["abilities"][0] != s["abilities"][1]
            ][:6]
            selected += [s for s in cat["species"] if s["id"] in [25, 132, 201, 327]]
            if key == "ULTIMATE":
                selected += [s for s in cat["species"] if s["id"] == 150]
            for s in selected:
                for slot in range(4 if key == "ULTIMATE" else 3):
                    for pid, seed, shiny in [
                        (2400 + i, seed, i % 2 == 0)
                        for i, seed in enumerate(
                            [1, 42, 0x12345678, 65535, 0xFFFFFFFF, 0]
                        )
                    ]:
                        cpu = b.MGBANative(rom)
                        cpu.word(b.CONFIG[key][3], 0x02030000)
                        cpu.word(b.CONFIG[key][4], 0x02034000)
                        before = raw_individual(
                            key, s["id"], slot, pid, shiny, 30, rom, s
                        )
                        cpu.write(party, before)
                        if key != "ULTIMATE":
                            cpu.call(0x967B4 if key == "ROCKET" else 0x3E47C, party)
                        before = cpu.read(party, 100)
                        cpu.word(b.CONFIG[key][5], seed)
                        if key == "ROCKET":
                            init, stop, callback, yes, no, applied = SLOT_RULES[handler]
                            assert cpu.c.calluntil(
                                0x08000000 + init, 0, 0, 0, 0, 0x08000000 + stop, regs
                            )
                            target = struct.unpack("<H", cpu.read(0x030052C8 + 4, 2))[0]
                            decision = cpu.c.callchoice(
                                0x08000000 + callback,
                                0,
                                0,
                                0,
                                0,
                                0x08000000 + yes,
                                0x08000000 + no,
                                regs,
                            )
                            assert decision in (
                                1,
                                2,
                            ), "native guard did not reach a verified boundary"
                            accepted = decision == 1
                            if accepted:
                                cpu.half(0x030052C8, 5)
                                assert cpu.c.calluntil(
                                    0x08000000 + callback,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0x08000000 + applied,
                                    regs,
                                )
                        elif key == "ULTIMATE":
                            # One complete prefix, including native random selection.
                            accepted, target = script_stage(
                                cpu, item["id"], 0x09F00E2C, regs
                            )
                        else:
                            cpu.half(0x0203AD30, item["id"])
                            assert cpu.c.calluntil(
                                0x09D561E0, 0, 0, 0, 0, 0x09D56246, regs
                            )
                            target = regs[4]
                            accepted = target != 0
                            if accepted:
                                assert cpu.c.calluntil(
                                    0x09D55FB4, 0, 0, 0, 0, 0x09D56006, regs
                                )
                        after = cpu.read(party, 100)
                        if not accepted:
                            assert after == before
                        header = bytearray(after[:32])
                        if key == "ULTIMATE":
                            header[30] = (header[30] & 254) | (before[30] & 1)
                        assert (
                            header[4:28] == before[4:28]
                            and header[30:32] == before[30:32]
                        )
                        c0 = bytearray(
                            b.unpack(before) if key == "ROCKET" else before[32:80]
                        )
                        c1 = bytearray(
                            b.unpack(after) if key == "ROCKET" else after[32:80]
                        )
                        if key == "ROCKET":
                            c1[47] = (c1[47] & 252) | (c0[47] & 3)
                            assert after[:4] == before[:4]
                        elif key == "ULTIMATE":
                            c1[43] = (c1[43] & 127) | (c0[43] & 128)
                            assert after[:4] == before[:4]
                        else:
                            c1[43] = (c1[43] & 127) | (c0[43] & 128)
                            assert (
                                int.from_bytes(before[:4], "little") % 25
                                == int.from_bytes(after[:4], "little") % 25
                            )
                        assert c1 == c0, "unrelated persistent fields changed"
                        rows.append(
                            dict(
                                item=item["id"],
                                before=list(before),
                                after=list(after),
                                seed=seed,
                                rng_after=int.from_bytes(
                                    cpu.read(b.CONFIG[key][5], 4), "little"
                                ),
                                accepted=accepted,
                                target=target,
                            )
                        )
        assert cpu.read(0x08000000, len(rom)) == rom
        assert hashlib.md5(b.ROM_PATH.read_bytes()).hexdigest() == b.MD5[key]
        result[key] = dict(md5=b.MD5[key], offers=offers, guards=guards, rows=rows)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2))
        print(
            key,
            len(offers),
            "runtime offers;",
            len(guards),
            "native guards;",
            len(rows),
            "persistent scenarios",
            flush=True,
        )


if __name__ == "__main__":
    main()
