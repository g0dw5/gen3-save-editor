#!/usr/bin/env python3
"""Opt-in mGBA regression for the final Ultimate emergency-heal cheat codes.

Set GEN3_BIN, GEN3_ROM_ULTIMATE, MGBA_SOURCE, MGBA_BUILD,
GEN3_EMERGENCY_STATE_SINGLE and GEN3_EMERGENCY_STATE_DOUBLE. The two states
must be at the player's battle command menu. No input file is modified.
"""

import ctypes
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile


MD5 = "17ce9785b33319b3dbda9a5d37c57ec1"
ROOT = Path(__file__).resolve().parents[1]
PAYLOAD = ROOT / "crates/gen3-core/src/cheats/emergency_ultimate.bin"
PARTY = 0x020244EC
MONS = 0x02024084


def main():
    rom = Path(os.environ["GEN3_ROM_ULTIMATE"])
    states = [Path(os.environ[f"GEN3_EMERGENCY_STATE_{mode}"]) for mode in ("SINGLE", "DOUBLE")]
    originals = {path: hashlib.sha256(path.read_bytes()).digest() for path in (rom, *states)}
    data = rom.read_bytes()
    assert hashlib.md5(data).hexdigest() == MD5
    codes = json.loads(subprocess.check_output([
        os.environ["GEN3_BIN"], "cheat-code", str(rom), "emergency-battle-heal",
        "gameshark_v1_v2",
    ]))["lines"]
    payload = PAYLOAD.read_bytes()
    assert len(payload) == 328 and len(codes) == 168
    assert data[0x39F30:0x39F34] == struct.pack("<I", 0x03005D04)
    assert data[0x1FFF200:0x1FFF204 + len(payload)] == bytes([0xFF]) * (4 + len(payload))
    source = Path(os.environ["MGBA_SOURCE"])
    build = Path(os.environ["MGBA_BUILD"])
    library = Path(os.environ.get("MGBA_LIBRARY", build / ("libmgba.dylib" if sys.platform == "darwin" else "libmgba.so")))
    with tempfile.TemporaryDirectory(prefix="gen3-emergency-cheat-") as temp:
        probe = Path(temp) / "probe.so"
        subprocess.run([
            os.environ.get("CC", "cc"), "-O2", "-shared", "-fPIC",
            "-DM_CORE_GBA", "-DM_CORE_GB", "-DUSE_DEBUGGERS",
            "-I" + str(source / "include"), "-I" + str(build / "include"),
            str(ROOT / "scripts/native/storage_cheat_probe.c"), str(library),
            "-Wl,-rpath," + str(library.parent), "-o", str(probe),
        ], check=True)
        c = ctypes.CDLL(str(probe))
        c.readmem.restype = ctypes.c_uint
        assert c.start(str(rom).encode())
        try:
            read = lambda address, width=4: c.readmem(address, width)
            write = lambda address, value, width=4: c.writemem(address, value, width)
            def block(address, length):
                result = ctypes.create_string_buffer(length)
                c.readbytes(address, result, length)
                return result.raw

            before = block(0x08000000, len(data))
            group = c.addgroup("\n".join(codes).encode())
            assert group >= 0
            expected = bytearray(before)
            struct.pack_into("<I", expected, 0x39F30, 0x09FFF200)
            struct.pack_into("<I", expected, 0x1FFF200, 0x09FFF205)
            expected[0x1FFF204:0x1FFF204 + len(payload)] = payload
            assert block(0x08000000, len(data)) == expected, "final codes changed unexpected ROM bytes"

            gameplay = {}
            state_header = None
            for mode, state, active in zip(("single", "double"), states, ((0,), (0, 2))):
                assert c.load_state(str(state).encode())
                # mGBA state loading restores three cartridge-header bytes from
                # the state itself, independent of the cheat (0xC4/0xC6/0xC8).
                header = tuple(read(0x08000000 + i, 1) for i in (0xC4, 0xC6, 0xC8))
                assert state_header is None or header == state_header
                state_header = header
                assert read(0x03005D04) == 0x0803BE75, f"{mode}: not at command menu"
                battler_count = read(0x0202406C, 1)
                enemy_before = [block(MONS + b * 0x58, 0x58) for b in (1, 3) if b < battler_count]
                party_before = [block(PARTY + slot * 100, 100) for slot in range(6)]
                for slot in range(6):
                    write(PARTY + slot * 100 + 86, 5 if slot in (0, 1) else 0, 2)
                    write(PARTY + slot * 100 + 80, 0x40)
                for b in active:
                    address = MONS + b * 0x58
                    write(address + 0x28, 5, 2)
                    write(address + 0x4C, 0x40)
                    write(address + 0x24, 0, 1)
                c.frames(5, 0x304)  # L + R + SELECT, single press.
                c.frames(60, 0)
                hp = [read(PARTY + slot * 100 + 86, 2) for slot in range(6)]
                maximum = [read(PARTY + slot * 100 + 88, 2) for slot in range(6)]
                assert hp == maximum and all(read(PARTY + slot * 100 + 80) == 0 for slot in range(6))
                assert [block(MONS + b * 0x58, 0x58) for b in (1, 3) if b < battler_count] == enemy_before
                for b in active:
                    address = MONS + b * 0x58
                    assert read(address + 0x28, 2) == read(address + 0x2C, 2)
                    assert read(address + 0x4C) == 0 and read(address + 0x24, 1) > 0
                for slot in range(6):
                    old = party_before[slot]
                    new = block(PARTY + slot * 100, 100)
                    assert all(index in range(52, 56) or index in range(80, 84) or index in range(86, 88)
                               for index, (a, b) in enumerate(zip(old, new)) if a != b)
                gameplay[mode] = {"party_hp": hp, "active_battlers": list(active)}
                c.togglecode(group, 0)
                assert read(0x08039F30) == 0x03005D04
                assert read(0x09FFF200) == 0xFFFFFFFF
                c.togglecode(group, 1)

            guard_results = {}
            for case in ("no_combo", "wrong_phase", "link", "safari", "multi", "partner", "max_hp_mismatch", "disabled", "valid"):
                assert c.load_state(str(states[0]).encode())
                c.togglecode(group, 1)
                write(PARTY + 86, 5, 2)
                write(MONS + 0x28, 5, 2)
                if case == "wrong_phase":
                    write(0x03005D04, 0x0803D819)
                if case == "link":
                    write(0x02022FEC, read(0x02022FEC) | 2)
                if case == "safari":
                    write(0x02022FEC, read(0x02022FEC) | 0x80)
                if case == "multi":
                    write(0x02022FEC, read(0x02022FEC) | 0x40)
                if case == "partner":
                    write(0x02022FEC, read(0x02022FEC) | 0x400000)
                if case == "max_hp_mismatch":
                    write(MONS + 0x2C, 100, 2)
                if case == "disabled":
                    c.togglecode(group, 0)
                c.frames(5, 0 if case == "no_combo" else 0x304)
                c.frames(60, 0)
                hp = read(PARTY + 86, 2)
                assert (hp == read(PARTY + 88, 2)) == (case == "valid"), (case, hp)
                guard_results[case] = hp
            c.togglecode(group, 0)
            c.reset()
            final_rom = block(0x08000000, len(data))
            differences = [i for i, (a, b) in enumerate(zip(before, final_rom)) if a != b]
            assert set(differences) <= {0xC4, 0xC6, 0xC8}, f"ROM not restored at offsets {differences[:16]}"
            assert tuple(final_rom[i] for i in (0xC4, 0xC6, 0xC8)) == state_header
            print(json.dumps({"lines": len(codes), "rom_diff_exact": True,
                              "gameplay": gameplay, "guards": guard_results,
                              "patch_restored": True}, ensure_ascii=False, indent=2))
        finally:
            c.finish()
    assert all(hashlib.sha256(path.read_bytes()).digest() == digest for path, digest in originals.items())


if __name__ == "__main__":
    main()
