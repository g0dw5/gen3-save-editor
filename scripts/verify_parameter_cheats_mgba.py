"""Opt-in exact-ROM encounter/shiny/teleport regression in mGBA.

Same environment as verify_common_cheats_mgba.py. No user saves are loaded.
The CLI's final encrypted codes, not direct patch writes, are under test.
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
from verify_common_cheats_mgba import Probe, PROFILES, mon, unpack

ENTRIES = {
    "BW": dict(wild=0xB4E68, gender=0x6A020, random=0x6F5CC, nature=0x67E90, ability=0x6B6D8, setter=0x84BEC,
               warp=0x84BD8, destination=0x20322E4, header=0x2037318),
    "ROCKET": dict(wild=0xEC2D0, gender=0x971B4, random=0x9D40C, nature=0x95A6C, ability=0x988F0, setter=0xBA498,
                   warp=0xBA484, destination=0x20332A0, header=0x20382D8),
}
ENTRIES["DP"] = ENTRIES["BW"]
ENCOUNTER = "specified-wild-encounter"
SHINY = "shiny-wild-encounters"
TELEPORT = "teleport-to-map"


class ParameterProbe(Probe):
    def __init__(self, library, name, path, temp):
        super().__init__(library, name, path)
        self.entry = ENTRIES[name]
        self.path, self.temp = path, temp
        self.catalog = json.loads(subprocess.check_output([os.environ["GEN3_BIN"], "cheats", str(path)]))
        self.cache = {}

    def lines(self, key, parameters=None):
        cache_key = json.dumps([key, parameters])
        if cache_key not in self.cache:
            args = [os.environ["GEN3_BIN"], "cheat-code", str(self.path), key, "gameshark_v1_v2"]
            if parameters is not None:
                file = self.temp / "parameters.json"
                file.write_text(json.dumps(parameters))
                args.append(str(file))
            code = json.loads(subprocess.check_output(args))
            assert code["parameters"] == parameters
            self.cache[cache_key] = code["lines"]
        return self.cache[cache_key]

    def activate(self, lines):
        ids = [self.c.addgroup("\n".join(lines).encode())]
        assert all(i >= 0 for i in ids)
        return ids

    def seed(self, seed, leader=25, pid=24):
        self.put(self.p["rng"], seed)
        self.put(0x203000A, 0x12345678)
        self.write(self.p["party"], mon(self.rocket, species=leader, pid=pid))
        self.put(self.p["count"], 1, 1)

    def record(self):
        raw = self.read(self.p["enemy"], 100)
        data = unpack(raw)  # Independently checks decrypted native checksum.
        pid, ot = struct.unpack_from("<II", raw)
        return raw, data, pid, (pid >> 16) ^ (pid & 65535) ^ (ot >> 16) ^ (ot & 65535) < 8

    def generated(self):
        shiny = self.lines(SHINY)
        cases = 0
        # Include >255 and the prior failed Sudowoodo species ID; boundary levels.
        for species, level in ((25, 1), (185, 17), (400, 100)):
            encounter = self.lines(ENCOUNTER, dict(kind="encounter", species=species, level=level))
            for on in (False, True):
                self.init()
                self.activate(encounter + (shiny if on else []))
                for seed in range(64):
                    self.seed(seed)
                    self.call(self.entry["wild"], 19, 3)
                    raw, data, pid, is_shiny = self.record()
                    assert struct.unpack_from("<H", data)[0] == species
                    assert raw[84] == level
                    assert not on or is_shiny
                    cases += 1
        # The shiny hook must leave non-wild nature constructors byte-identical.
        for seed in range(32):
            output = []
            for on in (False, True):
                self.init()
                self.seed(seed)
                if on:
                    self.activate(shiny)
                self.put(0x3007A00, 7)  # Native fifth argument: nature.
                self.call(self.entry["nature"], self.p["enemy"], 25, 5, 32)
                output.append(self.read(self.p["enemy"], 100))
            assert output[0] == output[1], seed
        # Find ability carriers from native ROM data, never from a bundled name list.
        carriers = {}
        self.init()
        for s in self.catalog["options"]["species"]:
            self.write(self.p["party"], mon(self.rocket, species=s["id"]))
            ability = self.call(self.entry["ability"], self.p["party"])
            if ability in (28, 56):
                carriers.setdefault(ability, s["id"])
            if len(carriers) == 2:
                break
        assert len(carriers) == 2
        for ability, leader in carriers.items():
            for seed in range(64):
                output = []
                for on in (False, True):
                    self.init()
                    self.seed(seed, leader, pid=24 if seed % 2 else 255)
                    lead_pid = 24 if seed % 2 else 255
                    lead_gender = self.call(self.entry["gender"], leader, lead_pid)
                    roll = self.call(self.entry["random"])
                    self.put(self.p["rng"], seed)
                    if on:
                        self.activate(shiny)
                    self.call(self.entry["wild"], 25, 15)
                    raw, data, pid, is_shiny = self.record()
                    assert not on or is_shiny
                    if ability == 56 and roll % 3 != 0:
                        assert self.call(self.entry["gender"], 25, pid) == (0 if lead_gender == 254 else 254)
                    output.append(pid)
                # Nature is selected before the patched constructor on both paths.
                assert output[0] % 25 == output[1] % 25, (ability, seed, output)
        return dict(species_level_shiny_cases=cases, nonwild_exact_comparisons=32,
                    lead_ability_cases=256, native_ability_carriers=carriers)

    def warps(self):
        maps = [m for m in self.catalog["options"]["maps"] if m["landings"]]
        # Test the native setter -> ApplyCurrentWarp -> header/layout -> coordinates
        # across outdoor, room and cave groups, including a >255 packed map code.
        selected = [maps[i] for i in sorted({0, len(maps)//4, len(maps)//2, len(maps)*3//4, len(maps)-1})]
        cases = []
        for m in selected:
            w = m["landings"][0]
            lines = self.lines(TELEPORT, dict(kind="teleport", map_id=m["id"], warp_id=w["id"]))
            self.init()
            ids = self.activate(lines)
            self.put(0x3007A00, 99)  # fifth argument y; warp must override coords
            self.call(self.entry["setter"], 0, 0, 0, 88)
            assert self.read(self.entry["destination"], 3) == bytes((m["group"], m["number"], w["id"]))
            self.call(self.entry["warp"])
            assert self.read(0x202A004, 3) == bytes((m["group"], m["number"], w["id"]))
            assert struct.unpack("<HH", self.read(0x202A000, 4)) == (w["x"], w["y"]), m
            for i in ids:
                self.c.togglecode(i, False)
            self.call(self.entry["setter"], 0, 0, 0, 1)
            assert self.read(self.entry["destination"], 3) == bytes(3)
            cases.append(dict(map_id=m["id"], warp=w["id"], x=w["x"], y=w["y"]))
        return dict(eligible_maps=len(maps), native_transition_cases=cases)

    def decoding(self):
        self.init()
        original = self.read(0x8000000, 0x2000000)
        reports = {}
        for key, parameters, allowed in (
            (ENCOUNTER, dict(kind="encounter", species=185, level=17), [(0xEC2D4, 8)] if self.rocket else [(0xB4E6C, 8)]),
            (SHINY, None, [(0x95A90, 14), (0x95B7E, 14), (0x1F00000, 72), (0x1F00080, 72)] if self.rocket else [(0x67EB4, 14), (0x67FA2, 14), (0x311200, 72), (0x311280, 72)]),
            (TELEPORT, dict(kind="teleport", map_id=self.catalog["options"]["maps"][0]["id"], warp_id=0), []),
        ):
            if key == TELEPORT:
                m = next(m for m in self.catalog["options"]["maps"] if m["landings"])
                parameters = dict(kind="teleport", map_id=m["id"], warp_id=m["landings"][0]["id"])
                allowed = [(0xBA4BE, 6)] if self.rocket else [(0x84C12, 6)]
            lines = self.lines(key, parameters)
            ids = self.activate(lines)
            patched = self.read(0x8000000, len(original))
            permitted = {i for start, length in allowed for i in range(start, start + length)}
            changed = {i for i, (a, b) in enumerate(zip(original, patched)) if a != b}
            assert changed and changed.issubset(permitted), (key, changed - permitted)
            for turn in range(8):
                on = turn % 2
                for i in ids:
                    self.c.togglecode(i, on)
                self.c.reset()
                assert self.read(0x8000000, len(original)) == (patched if on else original)
            self.c.clearcodes()
            assert self.read(0x8000000, len(original)) == original
            reports[key] = dict(lines=len(lines), changed_bytes=len(changed), full_rom_comparison=True, toggle_reset_cycles=8)
        return reports


def main():
    source, build = Path(os.environ["MGBA_SOURCE"]), Path(os.environ["MGBA_BUILD"])
    library = Path(os.environ.get("MGBA_LIBRARY", str(build / ("libmgba.dylib" if sys.platform == "darwin" else "libmgba.so"))))
    report = {}
    with tempfile.TemporaryDirectory(prefix="gen3-parameter-cheats-") as tmp:
        temp = Path(tmp)
        probe = temp / "probe.so"
        subprocess.run([os.environ.get("CC", "cc"), "-O2", "-shared", "-fPIC", "-DM_CORE_GBA", "-DM_CORE_GB", "-DUSE_DEBUGGERS",
                        "-I" + str(source / "include"), "-I" + str(build / "include"),
                        str(Path(__file__).parent / "native/common_cheat_probe.c"), str(library),
                        "-Wl,-rpath," + str(library.parent), "-o", str(probe)], check=True)
        for name in PROFILES:
            path = Path(os.environ["GEN3_ROM_" + name])
            p = ParameterProbe(probe, name, path, temp)
            report[name] = dict(md5=p.p["md5"], decoding=p.decoding(), generated=p.generated(), warps=p.warps())
            p.c.finish()
            assert hashlib.md5(path.read_bytes()).hexdigest() == p.p["md5"]
            print(name + " verified", file=sys.stderr, flush=True)
    report["scope"] = "Native mGBA fixtures, not all-map gameplay or mobile-emulator validation. No user saves loaded."
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
