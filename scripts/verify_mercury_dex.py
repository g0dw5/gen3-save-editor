#!/usr/bin/env python3
"""Mercury 1.2 native split Dex flags and read-only battery projection.

Complete, unmodified ARM7 getters; synthetic RAM and optional booted SAV RAM.
No source ROM/SAV writes, patching, native hooks or derived-ROM outputs.
"""

import argparse
import hashlib
import json
import os
import struct
import tempfile
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh
from verify_save_roundtrip import Frames


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--frame-probe", type=Path, required=True)
    parser.add_argument("--boot", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists() and args.output.suffix == ".json"
    rompath = Path(os.environ["GEN3_ROM_MERCURY12"]).resolve()
    savepath = Path(os.environ["GEN3_SAVE_MERCURY12"]).resolve()
    assert args.output.resolve() not in [rompath, savepath]
    hashes = {
        p: hashlib.sha256(p.read_bytes()).hexdigest() for p in [rompath, savepath]
    }
    rom = rompath.read_bytes()
    assert hashlib.md5(rom).hexdigest() == b.MD5["MERCURY12"]
    word = lambda at: struct.unpack_from("<I", rom, at)[0]
    half = lambda at: struct.unpack_from("<H", rom, at)[0]
    assert rom[0x88E74:0x88E78] == bytes.fromhex("004a1047")
    wrapper = (word(0x88E78) & ~1) - 0x08000000
    assert rom[wrapper : wrapper + 4] == bytes.fromhex("074a1047")
    native = (word(wrapper + 0x20) & ~1) - 0x08000000
    assert native == 0x1D6680C
    first = (-word(native + 0xE4)) & 0xFFFFFFFF
    assert half(native + 10) & 0xFF00 == 0x2B00
    count = (half(native + 10) & 255) + 1
    assert half(native + 0x7A) == 0x24E2 and half(native + 0x82) == 0x00A4
    prefix = (0xE2 << 2) + 1
    sb1 = word(native + 0xE8)
    marker_at, seen, marker, clear_end, owned, ordinary_owned = [
        word(native + at) for at in [0xEC, 0xF0, 0xF4, 0xF8, 0xFC, 0x100]
    ]
    ordinary_seen = 0xC4 << 2  # MOVS/LSLS in both native ordinary branches
    assert first == prefix + 1 and (
        sb1,
        marker_at,
        seen,
        owned,
        marker,
        clear_end,
        ordinary_owned,
    ) == (0x03005008, 0x40A, 0x40C, 0x41C, 0x11DE, 0x42C, 0x38D)
    assert word(0x1D669E8) == first + count
    b.ROM_PATH = rompath
    b.MGBA_PROBE = args.mgba_probe.resolve()
    rows = []
    try:
        # Boot the actual battery via normal Continue key inputs, then clone its
        # native RAM into an independent CPU. The source battery is never saved.
        frames = Frames(args.frame_probe)
        scratch = tempfile.TemporaryDirectory(prefix="gen3-dex-boot-")
        flash = Path(scratch.name) / "battery.sav"
        source = savepath.read_bytes()
        assert len(source) in [0x20000, 0x20010]
        flash.write_bytes(source[:0x20000])
        frames.start(rompath, flash)
        try:
            for step in json.loads(args.boot.read_text()):
                frames.c.frames(step["frames"], step.get("keys", 0))
            ram = frames.read(0x02000000, 0x40000)
            iwram = frames.read(0x03000000, 0x8000)
            loaded_sb1 = struct.unpack_from("<I", iwram, sb1 - 0x03000000)[0]
            assert 0x02000000 <= loaded_sb1 <= 0x02040000 - 0x3D68
        finally:
            frames.c.finish()
            scratch.cleanup()
        cpu = fresh(rom)
        cpu.write(0x02000000, ram)
        cpu.write(0x03000000, iwram)
        real = []
        for number in range(1, first + count):
            real.append(
                dict(
                    number=number,
                    seen=bool(cpu.call(0x88E74, number, 0)),
                    owned=bool(cpu.call(0x88E74, number, 1)),
                )
            )
        # Deliberate initialization may change the independent RAM, not the SAV.
        real_initialized = (
            struct.unpack_from("<H", ram, loaded_sb1 - 0x02000000 + marker_at)[0]
            == marker
        )
        base = 0x02028000
        bank = dict(
            first=first,
            count=count,
            seen=seen,
            owned=owned,
            initialization=[marker_at, marker],
        )
        for pattern in [0x00, 0xFF, 0xA5, 0x5A]:
            for initialized in [True, False]:
                cpu = fresh(rom)
                cpu.word(sb1, base)
                block = bytearray([pattern] * 0x3D68)
                for start, length in [
                    (ordinary_seen, (prefix + 7) // 8),
                    (seen, (count + 7) // 8),
                ]:
                    block[start : start + length] = bytes([pattern]) * length
                for start, length in [
                    (ordinary_owned, (prefix + 7) // 8),
                    (owned, (count + 7) // 8),
                ]:
                    block[start : start + length] = bytes([pattern ^ 255]) * length
                struct.pack_into(
                    "<H", block, marker_at, marker if initialized else marker ^ 1
                )
                cpu.write(base, block)
                for number in range(1, first + count):
                    before = cpu.read(base, len(block))
                    seen_value = cpu.call(0x88E74, number, 0)
                    owned_value = cpu.call(0x88E74, number, 1)
                    if number < first:
                        i = number - 1
                        expected_seen = bool(pattern & (1 << (i % 8)))
                        expected_owned = bool((pattern ^ 255) & (1 << (i % 8)))
                    else:
                        i = number - first
                        expected_seen = bool(initialized and pattern & (1 << (i % 8)))
                        expected_owned = bool(
                            initialized and (pattern ^ 255) & (1 << (i % 8))
                        )
                    assert (bool(seen_value), bool(owned_value)) == (
                        expected_seen,
                        expected_owned,
                    ), (pattern, initialized, number)
                    if number == first and not initialized:
                        after = bytearray(before)
                        struct.pack_into("<H", after, marker_at, marker)
                        after[seen:clear_end] = bytes(clear_end - seen)
                    else:
                        after = before
                    assert cpu.read(base, len(block)) == after
                    rows.append(
                        dict(
                            pattern=pattern,
                            initialized=initialized,
                            number=number,
                            seen=bool(seen_value),
                            owned=bool(owned_value),
                        )
                    )
                for invalid in [0, first + count, 65535]:
                    before = cpu.read(base, len(block))
                    assert (
                        cpu.call(0x88E74, invalid, 0) == 0
                        and cpu.call(0x88E74, invalid, 1) == 0
                    )
                    assert cpu.read(base, len(block)) == before
        result = dict(
            md5=b.MD5["MERCURY12"],
            rom_sha256=hashes[rompath],
            save_sha256=hashes[savepath],
            native=native,
            prefix=prefix,
            banks=[
                dict(
                    first=1,
                    count=prefix,
                    seen=ordinary_seen,
                    owned=ordinary_owned,
                    initialization=None,
                ),
                bank,
            ],
            rows=rows,
            real=real,
            real_initialized=real_initialized,
            loaded_sb1=loaded_sb1,
        )
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result) + "\n")
        print(
            "Mercury:",
            len(rows) * 2,
            "complete synthetic getters,",
            len(real) * 2,
            "booted-save getters; exact split banks and initialization verified",
            flush=True,
        )
    finally:
        if b.MGBANative.c is not None:
            b.MGBANative.c.finish()
        for path, digest in hashes.items():
            assert hashlib.sha256(path.read_bytes()).hexdigest() == digest


if __name__ == "__main__":
    main()
