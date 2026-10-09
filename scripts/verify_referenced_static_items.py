#!/usr/bin/env python3
"""Run referenced fixed-encounter commands from five exact private ROMs.

CLI finds referenced sources at runtime; original ROM operands execute unchanged
in isolated mGBA RAM. Resolved species variables are explicit query scenarios.
No SAV opened, no function hooks/replacements or ROM writes. Setup fields are
not full encounter startup, access, capture, theft or receipt evidence.
"""

import argparse, hashlib, json, os, struct, subprocess
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh
from verify_event_effects import CONFIG, CONTEXT, OPERANDS
from verify_static_battles import READERS


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--gen3", type=Path, required=True)
    p.add_argument("--mgba-probe", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    args = p.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    inputs = [Path(os.environ["GEN3_ROM_" + k]).resolve() for k in READERS]
    assert args.output.resolve() not in inputs
    result = {}
    for key, (getter, enemy) in READERS.items():
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        sha = hashlib.sha256(rom).hexdigest()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        try:
            world = json.loads(
                subprocess.check_output([str(args.gen3), "world", str(b.ROM_PATH)])
            )
            sources = [
                s
                for report in world["map_events"]
                for marker in report["markers"]
                for s in marker["pokemon"]
            ]
            sources.extend(
                s for report in world["map_events"] for s in report["unplaced_pokemon"]
            )
            static = [s for s in sources if s["method"] == "static"]
            unique = {
                (s["offset"], s["species"], s["level"], s["held_item"], s["member"]): s
                for s in static
            }
            rows = []
            skipped = []
            handler = (
                struct.unpack_from("<I", rom, CONFIG[key][0] + 0xB6 * 4)[0] & ~1
            ) - 0x08000000
            setvar = (
                struct.unpack_from("<I", rom, CONFIG[key][0] + 0x16 * 4)[0] & ~1
            ) - 0x08000000
            for source in unique.values():
                pc = source["offset"]
                if (
                    source["level"] is None
                    or source["held_item"] is None
                    or rom[pc] != 0xB6
                ):
                    skipped.append(
                        dict(
                            source=source,
                            reason="Unknown fields or nonordinary command",
                        )
                    )
                    continue
                prefix = struct.unpack_from("<H", rom, pc + 1)[0]
                paired = (key == "MERCURY133" and prefix == 65535) or (
                    key == "ROCKET" and struct.unpack_from("<H", rom, pc + 6)[0] != 0
                )
                length = (
                    18
                    if key == "MERCURY133" and paired
                    else 11 if key == "ROCKET" else 6
                )
                members = source["battle_members"] if paired else [source]
                if any(
                    m["species"] is None or m["level"] is None or m["held_item"] is None
                    for m in members
                ):
                    skipped.append(dict(source=source, reason="Unknown paired member"))
                    continue
                for seed in [42, 0x1234ABCD]:
                    cpu = fresh(rom)
                    cpu.word(CONFIG[key][1], 0x02030000)
                    cpu.word(CONFIG[key][2], 0x02028000)
                    cpu.word(b.CONFIG[key][5], seed)
                    assignments = {}
                    if key == "MERCURY133":
                        offsets = [7, 13] if paired else [1]
                        for offset, member in zip(offsets, members):
                            var = struct.unpack_from("<H", rom, pc + offset)[0]
                            if var >= 0x4000:
                                assert (
                                    var not in assignments
                                    or assignments[var] == member["species"]
                                )
                                assignments[var] = member["species"]
                        for var, value in assignments.items():
                            cpu.write(OPERANDS, struct.pack("<HH", var, value))
                            cpu.word(CONTEXT + 8, OPERANDS)
                            assert cpu.call(setvar, CONTEXT) == 0
                    start = 0x08000000 + pc + 1
                    cpu.word(CONTEXT + 8, start)
                    player = b.CONFIG[key][6]
                    before = cpu.read(player, 600)
                    regs = (cpu.ctypes.c_uint * 16)()
                    assert cpu.c.calluntil(
                        0x08000000 + handler, CONTEXT, 0, 0, 0, 0, regs
                    ), (key, pc, source, [hex(v) for v in regs])
                    assert regs[0] == 0
                    assert cpu.read(player, 600) == before
                    assert (
                        struct.unpack("<I", cpu.read(CONTEXT + 8, 4))[0]
                        == start + length - 1
                    )
                    raw = cpu.read(enemy, 600)
                    mons = []
                    for i, member in enumerate(members):
                        slot = 3 if key == "ROCKET" and i == 1 else i
                        at = enemy + slot * 100
                        actual = dict(
                            species=cpu.call(getter, at, 11, 0),
                            level=cpu.call(getter, at, 56, 0),
                            held_item=cpu.call(getter, at, 12, 0),
                            slot=slot,
                        )
                        expected = {
                            field: member[field]
                            for field in ["species", "level", "held_item"]
                        }
                        assert {k: actual[k] for k in expected} == expected, (
                            key,
                            pc,
                            seed,
                            expected,
                            actual,
                        )
                        mons.append(actual)
                    assert cpu.read(enemy, 600) == raw
                    rows.append(
                        dict(
                            source=source,
                            seed=seed,
                            assignments=assignments,
                            mons=mons,
                            length=length,
                        )
                    )
            result[key] = dict(
                md5=b.MD5[key],
                sha256=sha,
                handler=handler,
                reference_rows=len(static),
                unique_inputs=len(unique),
                held_reference_rows=sum(bool(s["held_item"]) for s in static),
                rows=rows,
                skipped=skipped,
            )
            print(
                key,
                len(static),
                "references,",
                len(unique),
                "unique inputs,",
                len(rows),
                "native calls,",
                len(skipped),
                "unresolved",
                flush=True,
            )
            assert cpu.read(0x08000000, len(rom)) == rom
        finally:
            assert hashlib.sha256(b.ROM_PATH.read_bytes()).hexdigest() == sha
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
