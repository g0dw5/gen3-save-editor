#!/usr/bin/env python3
"""Opt-in exact-ROM mGBA battle regression for the shared emergency recipe.

Requires GEN3_BIN, MGBA_SOURCE, MGBA_BUILD and GEN3_ROM_{BW,DP,ROCKET,ULTIMATE}.
Battle-menu states live under .local/analysis; input ROMs and states stay read-only.
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

ROOT = Path(__file__).resolve().parents[1]
STATES = ROOT / ".local/analysis/emergency-cross-20260926"
PROFILES = {
    "BW": ("0d9b129f7dd76895f79bb47ad7dec2fe", "BW-menu-4.state", 0x39f30, 0x03005d04, 0, "ultimate", 0x03005d04, 0x0803be75, 0x02022fec, 0x020244ec, 0x02024084, 0x58, 0x28, 0x2c, 0x4c, 0x24),
    "DP": ("cb2940215f4dafb1bef133c3af379f44", "DP-battle-menu.state", 0x39f30, 0x03005d04, 0, "ultimate", 0x03005d04, 0x0803be75, 0x02022fec, 0x020244ec, 0x02024084, 0x58, 0x28, 0x2c, 0x4c, 0x24),
    "ROCKET": ("59c658a1081f542086de1060bb65f0b3", "ROCKET-menu-2.state", 0x4ee70, 0x030051b4, 0xff, "rocket", 0x030051b4, 0x08050d51, 0x02024bb8, 0x02025170, 0x02024c50, 0x5c, 0x2a, 0x2e, 0x50, 0x25),
    "ULTIMATE": ("17ce9785b33319b3dbda9a5d37c57ec1", str(ROOT / ".local/analysis/emergency-heal-20260926/root.state"), 0x39f30, 0x03005d04, 0xff, "ultimate", 0x03005d04, 0x0803be75, 0x02022fec, 0x020244ec, 0x02024084, 0x58, 0x28, 0x2c, 0x4c, 0x24),
}


def main():
    source = Path(os.environ["MGBA_SOURCE"])
    build = Path(os.environ["MGBA_BUILD"])
    library = Path(os.environ.get("MGBA_LIBRARY", build / ("libmgba.dylib" if sys.platform == "darwin" else "libmgba.so")))
    report = {}
    with tempfile.TemporaryDirectory(prefix="gen3-emergency-cross-") as temp:
        probe = Path(temp) / "probe.dylib"
        subprocess.run([os.environ.get("CC", "cc"), "-O2", "-shared", "-fPIC", "-DM_CORE_GBA", "-DM_CORE_GB", "-DUSE_DEBUGGERS", "-I"+str(source/"include"), "-I"+str(build/"include"), str(ROOT/"scripts/native/storage_cheat_probe.c"), str(library), "-Wl,-rpath,"+str(library.parent), "-o", str(probe)], check=True)
        c = ctypes.CDLL(str(probe))
        c.readmem.restype = ctypes.c_uint
        for name, v in PROFILES.items():
            md5, state_name, hook, original, cave, payload_name, mainfn, phase, flags, party, mons, stride, hp_off, max_off, status_off, pp_off = v
            rom = Path(os.environ[f"GEN3_ROM_{name}"])
            state = Path(state_name) if Path(state_name).is_absolute() else STATES/state_name
            inputs = {p: hashlib.sha256(p.read_bytes()).digest() for p in (rom, state)}
            data = rom.read_bytes()
            assert hashlib.md5(data).hexdigest() == md5, name
            payload = (ROOT/f"crates/gen3-core/src/cheats/emergency_{payload_name}.bin").read_bytes()
            assert data[hook:hook+4] == struct.pack("<I", original)
            assert data[0x1fff200:0x1fff204+len(payload)] == bytes([cave])*(4+len(payload))
            codes = json.loads(subprocess.check_output([os.environ["GEN3_BIN"], "cheat-code", str(rom), "emergency-battle-heal", "gameshark_v1_v2"]))["lines"]
            assert len(codes) == 4+len(payload)//2, (name, len(codes))
            assert c.start(str(rom).encode()), name
            try:
                read = lambda a, w=4: c.readmem(a, w)
                write = lambda a, n, w=4: c.writemem(a, n, w)
                def block(a, n):
                    buf = ctypes.create_string_buffer(n)
                    c.readbytes(a, buf, n)
                    return buf.raw
                before = block(0x08000000, len(data))
                group = c.addgroup("\n".join(codes).encode())
                assert group >= 0
                expected = bytearray(before)
                struct.pack_into("<I", expected, hook, 0x09fff200)
                struct.pack_into("<I", expected, 0x1fff200, 0x09fff205)
                expected[0x1fff204:0x1fff204+len(payload)] = payload
                assert block(0x08000000, len(data)) == expected, f"{name}: extraneous patch bytes"
                assert c.load_state(str(state).encode()), name
                assert read(mainfn) == phase, f"{name}: state is not command phase"
                enemy = block(mons+stride, stride)
                party_before = [block(party+slot*100, 100) for slot in range(6)]
                maximum = read(party+88, 2)
                assert maximum > 5 and read(mons+max_off, 2) == maximum
                for slot in range(6):
                    write(party+slot*100+86, 5 if slot == 0 else 0, 2)
                    write(party+slot*100+80, 0x40)
                write(mons+hp_off, 5, 2)
                write(mons+status_off, 0x40)
                write(mons+pp_off, 0, 1)
                c.frames(5, 0x304)
                c.frames(60, 0)
                assert read(party+86, 2) == maximum, (name, "party", read(party+86, 2), maximum)
                assert read(mons+hp_off, 2) == maximum, (name, "active", read(mons+hp_off, 2), maximum)
                assert read(party+80) == 0 and read(mons+status_off) == 0
                assert read(mons+pp_off, 1) > 0
                assert block(mons+stride, stride) == enemy, (name, "opponent changed")
                for slot, old in enumerate(party_before):
                    new = block(party+slot*100, 100)
                    assert all(index in range(52,56) or index in range(80,84) or index in range(86,88)
                               for index, (a,b) in enumerate(zip(old,new)) if a != b), (name,slot,"unrelated party byte changed")
                c.togglecode(group, 0)
                assert read(0x08000000+hook) == original
                assert read(0x09fff200) == cave*0x01010101
                report[name] = {"lines": len(codes), "party_hp": maximum, "active_hp": maximum, "opponent_unchanged": True, "patch_restored": True}
                # The same code must reject battles where it cannot safely map active forms.
                for guard in ("no_combo", "link", "safari", "multi", "partner", "max_mismatch", "disabled"):
                    assert c.load_state(str(state).encode())
                    c.togglecode(group, 1)
                    write(party+86, 5, 2)
                    write(mons+hp_off, 5, 2)
                    if guard in ("link", "safari", "multi", "partner"):
                        write(flags, read(flags) | {"link":2,"safari":0x80,"multi":0x40,"partner":0x400000}[guard])
                    if guard == "max_mismatch": write(mons+max_off, maximum+1, 2)
                    if guard == "disabled": c.togglecode(group, 0)
                    c.frames(5, 0 if guard == "no_combo" else 0x304)
                    c.frames(60, 0)
                    assert read(party+86, 2) == 5, (name, guard, read(party+86, 2))
                c.togglecode(group, 0)
            finally:
                c.finish()
            assert all(hashlib.sha256(p.read_bytes()).digest() == digest for p, digest in inputs.items())
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
