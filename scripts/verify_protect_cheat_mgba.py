#!/usr/bin/env python3
"""Opt-in mGBA regression for the final four-line native Protect codes.

Set GEN3_BIN, MGBA_SOURCE, MGBA_BUILD and GEN3_ROM_{BW,DP,ROCKET,ULTIMATE,MERCURY133}.
Private battle-menu states live in ignored .local/analysis; no input is modified.
"""
import ctypes
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
STATES = ROOT / ".local/analysis"
PROFILES = {
    "MERCURY133": ("5ffb1cbd5c28cda9b987b3b445da68e0", "mercury-133-20261009/battle-menu.state", 0x03004f84, 0x02023e8c, 16, 0x02023be4, 0x58, 0x28, 0x24),
    "BW": ("0d9b129f7dd76895f79bb47ad7dec2fe", "emergency-cross-20260926/BW-menu-4.state", 0x03005d04, 0x0202433c, 16, 0x02024084, 0x58, 0x28, 0x24),
    "DP": ("cb2940215f4dafb1bef133c3af379f44", "emergency-cross-20260926/DP-battle-menu.state", 0x03005d04, 0x0202433c, 16, 0x02024084, 0x58, 0x28, 0x24),
    "ROCKET": ("59c658a1081f542086de1060bb65f0b3", "emergency-cross-20260926/ROCKET-menu-2.state", 0x030051b4, 0x02024f6c, 20, 0x02024c50, 0x5c, 0x2a, 0x25),
    "ULTIMATE": ("17ce9785b33319b3dbda9a5d37c57ec1", "emergency-heal-20260926/root.state", 0x03005d04, 0x0202433c, 16, 0x02024084, 0x58, 0x28, 0x24),
}


def main():
    source = Path(os.environ["MGBA_SOURCE"])
    build = Path(os.environ["MGBA_BUILD"])
    library = Path(os.environ.get("MGBA_LIBRARY", build / ("libmgba.dylib" if sys.platform == "darwin" else "libmgba.so")))
    results = {}
    with tempfile.TemporaryDirectory(prefix="gen3-protect-cheat-") as temp:
        probe = Path(temp) / "probe.dylib"
        subprocess.run([os.environ.get("CC", "cc"), "-O2", "-shared", "-fPIC", "-DM_CORE_GBA", "-DM_CORE_GB", "-DUSE_DEBUGGERS", "-I"+str(source/"include"), "-I"+str(build/"include"), str(ROOT/"scripts/native/storage_cheat_probe.c"), str(library), "-Wl,-rpath,"+str(library.parent), "-o", str(probe)], check=True)
        c = ctypes.CDLL(str(probe))
        c.readmem.restype = ctypes.c_uint
        for name, (md5, state_name, callback, protect, stride, mons, mon_stride, hp_off, pp_off) in PROFILES.items():
            rom = Path(os.environ[f"GEN3_ROM_{name}"])
            state = STATES / state_name
            double = STATES / "emergency-heal-20260926/double-command.state" if name == "ULTIMATE" else None
            hashes = {p: hashlib.sha256(p.read_bytes()).digest() for p in (rom, state, *([double] if double else []))}
            assert hashlib.md5(rom.read_bytes()).hexdigest() == md5
            lines = json.loads(subprocess.check_output([os.environ["GEN3_BIN"], "cheat-code", str(rom), "persistent-player-protect", "codebreaker"]))["lines"]
            assert lines == [f"{0xa0000000|callback:08X} 0000", f"{0x20000000|protect:08X} 0001", f"{0xa0000000|callback:08X} 0000", f"{0x20000000|protect+2*stride:08X} 0001"]
            assert c.start(str(rom).encode())
            try:
                read = lambda a, w=4: c.readmem(a, w)
                write = lambda a, v, w=4: c.writemem(a, v, w)
                group = c.addgroup_codebreaker("\n".join(lines).encode())
                assert group >= 0
                # No active battle: no writes to either ProtectStruct.
                c.frames(300, 0)  # Let boot-time EWRAM initialization finish.
                assert read(callback) == 0, (name, "title callback")
                write(protect, 2, 1)
                write(protect+2*stride, 4, 1)
                c.frames(1, 0)
                assert (read(protect,1), read(protect+2*stride,1)) == (2,4)
                # In battle, OR only bit zero and preserve unrelated flags.
                assert c.load_state(str(state).encode())
                assert read(callback) != 0
                write(protect, 2, 1)
                write(protect+2*stride, 4, 1)
                write(protect+stride, 8, 1)
                c.frames(1, 0)
                assert (read(protect,1), read(protect+2*stride,1), read(protect+stride,1)) == (3,5,8)
                if double:
                    assert c.load_state(str(double).encode())
                    for battler in range(4):
                        write(protect+battler*stride, 0, 1)
                    c.frames(1, 0)
                    assert [read(protect+battler*stride,1) for battler in range(4)] == [1,0,1,0]
                c.togglecode(group, 0)
                write(protect, 2, 1)
                c.frames(1, 0)
                assert read(protect,1) == 2
                battle = {}
                for enabled in (False, True):
                    c.togglecode(group, int(enabled))
                    assert c.load_state(str(state).encode())
                    enemy = mons+mon_stride
                    # Both sides can act without fainting: Splash vs Tackle.
                    write(mons+0x0c, 150, 2)
                    write(mons+pp_off, 30, 1)
                    for move in range(4):
                        write(enemy+0x0c+move*2, 33, 2)
                        write(enemy+pp_off+move, 30, 1)
                    before = read(mons+hp_off, 2)
                    if name in ("BW", "DP"):
                        # These fixtures need text prompts advanced between actions.
                        for _ in range(30):
                            c.frames(3, 1)
                            c.frames(30, 0)
                    else:
                        c.frames(1,1)
                        c.frames(20,0)
                        c.frames(1,1)
                        c.frames(1000,0)
                    battle["on" if enabled else "off"] = (before, read(mons+hp_off,2))
                assert battle["off"][1] < battle["off"][0], (name, battle)
                assert battle["on"][1] == battle["on"][0], (name, battle)
                results[name] = {"lines": len(lines), "without_protect": battle["off"], "with_protect": battle["on"], "other_flags_preserved": True, "overworld_inert": True}
                c.togglecode(group, 0)
            finally:
                c.finish()
            assert all(hashlib.sha256(p.read_bytes()).digest() == digest for p, digest in hashes.items())
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
