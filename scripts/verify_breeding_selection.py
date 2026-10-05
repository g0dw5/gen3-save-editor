#!/usr/bin/env python3
"""Native offspring selection checkpoints vs complete ordinary egg constructions.

Five exact private ROMs and prior complete native vectors, disposable RAM only.
No user SAV is opened, ROM writes or native function replacements are used.
"""

import argparse, hashlib, json, os, struct
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--mgba-probe", type=Path, required=True)
parser.add_argument(
    "--baseline",
    type=Path,
    required=True,
    help="Complete native vectors from verify_breeding.py",
)
parser.add_argument("--output", type=Path, required=True)
args = parser.parse_args()
b.MGBA_PROBE = args.mgba_probe.resolve()
v = json.loads(args.baseline.read_text())
checks = {
    "BW": 0x80708E8,
    "DP": 0x80708E8,
    "ROCKET": 0x809E8D0,
    "ULTIMATE": 0x80708E8,
    "MERCURY12": 0x9D1B68C,
}
result = {}
for key, stop in checks.items():
    b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
    rom = b.ROM_PATH.read_bytes()
    assert hashlib.md5(rom).hexdigest() == b.MD5[key] == v[key]["md5"]
    entry, compat, offset, sb1, sb2, rng, party, count, width = b.CONFIG[key]
    rows = []
    for row in v[key]["rows"]:
        cpu = fresh(rom)
        cpu.word(sb1, 0x2030000)
        cpu.word(sb2, 0x2034000)
        daycare = 0x2030000 + offset
        for i, p in enumerate(row["parents"]):
            cpu.write(daycare + i * 140, p)
        cpu.word(rng, row["seed"])
        cpu.write(daycare + 280, int(row["offspring_pid"]).to_bytes(width, "little"))
        score = cpu.call(compat, daycare)
        species = None
        if score:
            regs = (cpu.ctypes.c_uint * 16)()
            assert cpu.c.calluntil(0x8000000 + entry, daycare, 0, 0, 0, stop, regs)
            species = regs[1]
            raw = bytes(row["child_raw"])
            canonical = (
                raw[32:80] if key in ("ULTIMATE", "MERCURY12") else b.unpack(raw)
            )
            if isinstance(canonical, tuple):
                canonical = canonical[1]
            assert species == struct.unpack_from("<H", canonical)[0], (
                key,
                species,
                canonical[:2],
            )
        for i, parent in enumerate(row["parents"]):
            assert cpu.read(daycare + i * 140, 80) == bytes(parent)
        rows.append(
            dict(
                parents=row["parents"],
                seed=row["seed"],
                offspring_pid=row["offspring_pid"],
                compatibility=score,
                species=species,
            )
        )
    assert cpu.read(0x08000000, len(rom)) == rom
    assert b.ROM_PATH.read_bytes() == rom
    result[key] = dict(md5=b.MD5[key], engine="mGBA ARM7", checkpoint=stop, rows=rows)
    print(
        key,
        len(rows),
        "compatibility scenarios;",
        sum(row["species"] is not None for row in rows),
        "selection checkpoints match complete constructions",
        flush=True,
    )
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(result))
