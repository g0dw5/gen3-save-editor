#!/usr/bin/env python3
"""Observe complete item-evolution selectors in five exact private ROMs.

Runtime table inputs, synthetic level-50 party records and isolated mGBA RAM.
No SAV, ROM mutation, native hooks or replacement functions. Mode 2/3 observations
are not full item consumption, eligibility, animation or evolution completion.
The output is private evidence and must not be included in release inputs.
"""

import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh

# Native entries, table layouts and species exceptions, not extracted contents.
CONFIG = {
    "BW": (0x6D098, 0x32531C, 40, 412, 0x31F72C, 404, 0x3203CC, 28),
    "DP": (0x6D098, 0x32531C, 40, 412, 0x31F72C, 404, 0x3203CC, 28),
    "ROCKET": (0x9A37C, 0x5F96D4, 80, 1395, 0x5B3484, 604, 0x5B4764, 36),
    "ULTIMATE": (0x6D098, 0xF387C0, 40, 1200, 0x31F72C, 404, 0xF186E0, 28),
    "MERCURY133": (0x42EC4, 0x1788F5A, 128, 1554, 0x1e0b258, 1024, 0x176DFBC, 28),
}
OVERRIDES = {
    "BW": {133: (0x1196300, 7)},
    "DP": {133: (0x1196300, 7)},
    "ULTIMATE": {133: (0x1F0B760, 10)},
}


def table_rows(rom, key, species):
    _, table, stride, *_ = CONFIG[key]
    start, count = OVERRIDES.get(key, {}).get(
        species, (table + species * stride, stride // 8)
    )
    return [
        dict(
            offset=start + i * 8,
            method=(
                rom[start + i * 8]
                if key == "ULTIMATE"
                else struct.unpack_from("<H", rom, start + i * 8)[0]
            ),
            parameter=struct.unpack_from("<H", rom, start + i * 8 + 2)[0],
            target=struct.unpack_from("<H", rom, start + i * 8 + 4)[0],
            auxiliary=struct.unpack_from("<H", rom, start + i * 8 + 6)[0],
        )
        for i in range(count)
    ]


def record(rom, key, species, pid):
    _, _, _, _, exp, exp_stride, stats, stat_stride = CONFIG[key]
    raw = bytearray(b.parent(key, species, pid, 1) + bytes(20))
    c = bytearray(b.unpack(raw) if key in ("BW", "DP", "ROCKET") else raw[32:80])
    growth = rom[stats + species * stat_stride + (21 if stat_stride == 36 else 19)]
    struct.pack_into(
        "<I", c, 4, struct.unpack_from("<I", rom, exp + growth * exp_stride + 50 * 4)[0]
    )
    c[9] = 220
    if key in ("BW", "DP", "ROCKET"):
        raw = bytearray(b.pack(raw, c))
    else:
        raw[32:80] = c
        struct.pack_into("<H", raw, 28, sum(struct.unpack("<24H", c)) & 65535)
    raw = raw[:100]
    raw[84] = 50
    struct.pack_into("<7H", raw, 86, *([100] * 7))
    return bytes(raw)


def verify(key):
    b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key]).resolve()
    rom = b.ROM_PATH.read_bytes()
    sha = hashlib.sha256(rom).hexdigest()
    assert hashlib.md5(rom).hexdigest() == b.MD5[key]
    hook = None
    if key in ("BW", "DP"):
        target = (struct.unpack_from("<I", rom, 0x6D3A8)[0] & ~1) - 0x08000000
        assert target == 0x1196250
        base = struct.unpack_from("<I", rom, target + 0x1C)[0] - 0x08000000
        assert base + 133 * 40 == OVERRIDES[key][133][0]
        assert struct.unpack_from("<H", rom, target + 0x12)[0] == 0x2A06
        legacy = CONFIG[key][1] + 133 * 40
        assert rom[legacy : legacy + 40] == rom[base + 133 * 40 : base + 133 * 40 + 40]
        hook = dict(
            entry=0x6D3A4,
            target=target,
            biased_table_base=base,
            rows=7,
            legacy_prefix_equal=True,
        )
    observations, source_count = [], 0
    for species in range(1, CONFIG[key][3]):
        rows = table_rows(rom, key, species)
        items = sorted(
            {r["parameter"] for r in rows if r["method"] == 7 and r["parameter"]}
        )
        if not items:
            continue
        source_count += 1
        for pid in (42, 255):
            raw = record(rom, key, species, pid)
            for mode in (2, 3):
                for item in [0, *items]:
                    cpu = fresh(rom)
                    _, _, _, sb1, sb2, rng, _, _, _ = b.CONFIG[key]
                    cpu.word(sb1, 0x02030000)
                    cpu.word(sb2, 0x02028000)
                    cpu.word(rng, 42)
                    cpu.write(0x02020000, raw)
                    result = cpu.call(CONFIG[key][0], 0x02020000, mode, item, 0)
                    after = cpu.read(0x02020000, len(raw))
                    if key in ("BW", "DP"):
                        expected = next(
                            (
                                r["target"]
                                for r in rows
                                if r["method"] == 7 and r["parameter"] == item
                            ),
                            0,
                        )
                        assert result == expected, (
                            key,
                            species,
                            pid,
                            mode,
                            item,
                            result,
                            expected,
                        )
                        assert after == raw
                    observations.append(
                        dict(
                            species=species,
                            pid=pid,
                            mode=mode,
                            item=item,
                            target=result,
                            rows=rows,
                            changed_bytes=[
                                i for i, (a, z) in enumerate(zip(raw, after)) if a != z
                            ],
                        )
                    )
        (
            print(f"{key}: native selectors through species {species}", flush=True)
            if source_count % 50 == 0
            else None
        )
    gender_rows = []
    if key == "MERCURY133":
        instruction = struct.unpack_from("<H", rom, 0x1D2A006)[0]
        assert instruction & 0xFF00 == 0x2A00
        item = instruction & 255
        getter = (struct.unpack_from("<I", rom, 0x1D2A578)[0] & ~1) - 0x08000000
        for species in range(1, CONFIG[key][3]):
            rows = [
                r
                for r in table_rows(rom, key, species)
                if r["method"] == 7 and r["parameter"] == item
            ]
            if not rows:
                continue
            for pid in range(256):
                raw = record(rom, key, species, pid)
                for mode in (2, 3):
                    cpu = fresh(rom)
                    cpu.word(b.CONFIG[key][3], 0x02030000)
                    cpu.word(b.CONFIG[key][4], 0x02028000)
                    cpu.word(b.CONFIG[key][5], 42)
                    cpu.write(0x02020000, raw)
                    gender = cpu.call(getter, 0x02020000)
                    target = cpu.call(CONFIG[key][0], 0x02020000, mode, item, 0)
                    expected = next(
                        (r["target"] for r in rows if r["auxiliary"] == gender), 0
                    )
                    assert target == expected, (
                        species,
                        pid,
                        mode,
                        gender,
                        target,
                        expected,
                    )
                    assert cpu.read(0x02020000, len(raw)) == raw
                    gender_rows.append(
                        dict(
                            species=species,
                            pid=pid,
                            mode=mode,
                            item=item,
                            gender=gender,
                            target=target,
                        )
                    )
        print(
            f"{key}: {len(gender_rows)} additional full native gender/selector pairs",
            flush=True,
        )
    assert observations
    assert cpu.read(0x08000000, len(rom)) == rom
    assert hashlib.sha256(b.ROM_PATH.read_bytes()).hexdigest() == sha
    print(
        f"{key}: {source_count} item-row species, {len(observations)} complete native calls",
        flush=True,
    )
    return dict(
        md5=b.MD5[key],
        sha256=sha,
        hook=hook,
        item_species=source_count,
        rows=observations,
        gender_rows=gender_rows,
        scenario="level 50, friendship 220, held item 0, PIDs 42/255, mode 2/3, zero map/weather/party RAM; no SAV",
        scope="item-selector outputs only; not full contextual eligibility, consumption, animation or completion",
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert args.output.resolve() not in [
        Path(os.environ["GEN3_ROM_" + k]).resolve() for k in CONFIG
    ]
    b.MGBA_PROBE = args.mgba_probe.resolve()
    result = {key: verify(key) for key in CONFIG}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    b.MGBANative.c.finish()


if __name__ == "__main__":
    main()
