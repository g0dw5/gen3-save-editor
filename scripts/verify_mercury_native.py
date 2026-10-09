#!/usr/bin/env python3
"""Compare Mercury adapters with exact native getters, setters and stat routines.

Generate local probes with mercury::tests::exact_rom* and GEN3_MERCURY_PROBES.
Requires Unicorn. No ROM or save is written; only synthetic emulator RAM changes.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

from verify_rocket_battle_forms import Native, PARTY

FINGERPRINTS = {
    "5ffb1cbd5c28cda9b987b3b445da68e0": "mercury-fc-1.33",
}


def check(rom, probes):
    md5 = hashlib.md5(rom).hexdigest()
    assert md5 in FINGERPRINTS, "unsupported exact ROM"
    native = Native(rom)
    fields = {
        "species": 11, "held_item": 12, "experience": 25,
        "friendship": 32, "ball": 38, "met_level": 36,
        "origin_game": 37, "ot_gender": 49,
    }
    setters = {"ball": 38, "friendship": 32, "experience": 25, "pp_ups": 21}
    reads = writes = stats = 0
    for case in probes:
        raw = bytes(case["after"])
        pokemon = case["pokemon"]
        native.write(PARTY, raw)
        for name, field in fields.items():
            actual = native.call(0x3FBE8, PARTY, field, 0)
            assert actual == pokemon[name], (name, actual, pokemon[name], pokemon["pid"])
            reads += 1
        assert native.call(0x40D38, PARTY) == pokemon["ability_id"]
        for i in range(6):
            assert native.call(0x3FBE8, PARTY, 39 + i, 0) == pokemon["ivs"][i]
            reads += 1
        for i in range(4):
            assert native.call(0x3FBE8, PARTY, 13 + i, 0) == pokemon["moves"][i]
            assert native.call(0x3FBE8, PARTY, 17 + i, 0) == pokemon["pps"][i]
            reads += 2
        assert native.read(PARTY, len(raw)) == raw, "getters changed record bytes"
        native.call(0x3E47C, PARTY)
        actual = list(struct.unpack("<6H", native.read(PARTY + 88, 12)))
        assert actual == pokemon["stats"], ("stats", pokemon["species"], actual, pokemon["stats"])
        stats += 1
        active = [(name, value) for name, value in case["patch"].items()
                  if value is not None and name in setters]
        if not active:
            continue
        assert len(active) == 1
        name, value = active[0]
        if name == "pp_ups":
            value = sum(v << (i * 2) for i, v in enumerate(value))
        native.write(PARTY, bytes(case["before"]))
        native.word(0x0203F000, value)
        native.call(0x4037C, PARTY, setters[name], 0x0203F000)
        # A native data setter doesn't recalculate party stats or refill PP.
        expected = bytearray(raw)
        expected[80:100] = bytes(case["before"])[80:100]
        if name == "pp_ups":
            expected[52:56] = bytes(case["before"])[52:56]
        assert native.read(PARTY, len(raw)) == bytes(expected), ("setter", name, pokemon["pid"])
        writes += 1
    return {"profile": FINGERPRINTS[md5], "records": len(probes),
            "native_getters": reads, "native_setter_byte_comparisons": writes,
            "native_stat_comparisons": stats}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rom", type=Path, required=True)
    parser.add_argument("--probes", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(check(args.rom.read_bytes(), json.loads(args.probes.read_text())), indent=2))
