"""Opt-in common-cheat regression against exact private ROMs and mGBA 0.10.5.

Requires GEN3_BIN, GEN3_ROM_BW/DP/ROCKET/ULTIMATE, MGBA_SOURCE, MGBA_BUILD; optional
MGBA_LIBRARY and CC. No saves are loaded. Uses disposable synthetic RAM and
complete native functions, not mocks. Output is a JSON verification report.
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

NO_ENCOUNTERS = "disable-walking-encounters"
CATCH = "guaranteed-wild-catch"
HATCH = "faster-egg-hatching"
BREED = "guaranteed-compatible-daycare-egg"
# Independent test oracle: reverse-engineered entry points and RAM layouts.
BW = dict(md5="0d9b129f7dd76895f79bb47ad7dec2fe", party=0x20244EC,
          count=0x20244E9, enemy=0x2024744, sb1p=0x3005D8C, sb2p=0x3005D90,
          day=0x3030, hatch=0x70BD0, compat=0x70D4C, rng=0x3005D80,
          special=0x20375E0, walk=0xB5288, catch=0x56300, get=0x6A518,
          cycles=0x7228C, wild=0xEA2D34, charm=56, pocket=0xD7590,
          bag=0x2039DD8, has_item=0xD6724,
          patches={NO_ENCOUNTERS: (0xB52A2, 0xD100, 0x46C0),
                   CATCH: (0x56566, 0xD92F, 0x46C0), HATCH: (0x70B46, 0xD13B, 0x46C0)})
PROFILES = {
    "BW": BW,
    "DP": dict(BW, md5="cb2940215f4dafb1bef133c3af379f44"),
    "ROCKET": dict(md5="59c658a1081f542086de1060bb65f0b3", party=0x2025170,
                   count=0x202516D, enemy=0x20253C8, sb1p=0x300524C, sb2p=0x3005250,
                   day=0x297C, hatch=0x9EBC0, compat=0x9ED3C, rng=0x3005240,
                   special=0x20385A0, walk=0xEC7D0, catch=0x806B0, get=0x976D0,
                   cycles=0xA624C, wild=0xBE8A70, charm=846, pocket=0x10F958,
                   bag=0x203ADDC, has_item=0x10EAE0,
                   patches={NO_ENCOUNTERS: (0xEC7EC, 0xD100, 0x46C0),
                            CATCH: (0x80BB4, 0xD948, 0x46C0), HATCH: (0x9EB36, 0xD13B, 0x46C0),
                            BREED: (0x9EB1C, 0x4284, 0x2C00)}),
}
PROFILES["ULTIMATE"] = dict(BW, md5="17ce9785b33319b3dbda9a5d37c57ec1", wild=0xE17D50)
ORDERS = ("GAEM", "GAME", "GEAM", "GEMA", "GMAE", "GMEA",
          "AGEM", "AGME", "AEGM", "AEMG", "AMGE", "AMEG",
          "EGAM", "EGMA", "EAGM", "EAMG", "EMGA", "EMAG",
          "MGAE", "MGEA", "MAGE", "MAEG", "MEGA", "MEAG")


def unpack(raw):
    pid, ot = struct.unpack_from("<II", raw)
    data = struct.pack("<12I", *(v ^ pid ^ ot for v in struct.unpack_from("<12I", raw, 32)))
    chunks = {ch: data[i * 12:(i + 1) * 12] for i, ch in enumerate(ORDERS[pid % 24])}
    q = b"".join(chunks[ch] for ch in "GAEM")
    assert sum(struct.unpack("<24H", q)) & 0xFFFF == struct.unpack_from("<H", raw, 28)[0]
    return q


def pack(raw, q):
    raw = bytearray(raw)
    pid, ot = struct.unpack_from("<II", raw)
    data = b"".join(q["GAEM".index(ch) * 12:("GAEM".index(ch) + 1) * 12] for ch in ORDERS[pid % 24])
    raw[32:80] = struct.pack("<12I", *(v ^ pid ^ ot for v in struct.unpack("<12I", data)))
    struct.pack_into("<H", raw, 28, sum(struct.unpack("<24H", q)) & 0xFFFF)
    return bytes(raw)


def mon(rocket, species=25, egg=False, cycles=5, pid=24, ot=0x12345678, bad=False):
    raw = bytearray(100)
    struct.pack_into("<II", raw, 0, pid, ot)
    raw[18] = 2 | (0x10 if rocket else 0) | (0x20 if rocket and egg else 0) | (8 if rocket and bad else 0)
    if not rocket:
        raw[19] = 2 | (4 if egg else 0) | (1 if bad else 0)
    raw[84] = 5
    struct.pack_into("<HH", raw, 86, 20, 20)
    q = bytearray(48)
    struct.pack_into("<HHI", q, 0, species, 0, 125)
    q[8 if rocket else 9] = cycles
    if rocket:
        struct.pack_into("<H", q, 9, (26 << 5) | 1)
    else:
        struct.pack_into("<H", q, 38, 0x2000)
    struct.pack_into("<I", q, 40, 0x3FFFFFFF | (0x40000000 if egg else 0))
    return pack(raw, q)


class Probe:
    def __init__(self, library, name, path):
        self.c = ctypes.CDLL(str(library))
        self.c.readmem.restype = ctypes.c_uint
        self.p = PROFILES[name]
        self.rocket = name == "ROCKET"
        self.ultimate = name == "ULTIMATE"
        self.data = path.read_bytes()
        assert hashlib.md5(self.data).hexdigest() == self.p["md5"]
        cli = os.environ["GEN3_BIN"]
        self.codes = {key: json.loads(subprocess.check_output(
            [cli, "cheat-code", str(path), key, "gameshark_v1_v2"]))["lines"] for key in self.p["patches"]}
        catalog = json.loads(subprocess.check_output([cli, "cheats", str(path)]))
        assert set(self.codes).issubset({e["id"] for e in catalog["entries"]})
        assert self.c.start(str(path).encode())

    def mon(self, rocket, **kwargs):
        raw = mon(rocket, **kwargs)
        return raw[:32] + unpack(raw) + raw[80:] if self.ultimate else raw

    def unpack(self, raw):
        return raw[32:80] if self.ultimate else unpack(raw)

    def pack(self, raw, q):
        return raw[:32] + bytes(q) + raw[80:] if self.ultimate else pack(raw, q)

    def read(self, a, n):
        b = ctypes.create_string_buffer(n)
        self.c.readbytes(a, b, n)
        return b.raw

    def write(self, a, b):
        self.c.writebytes(a, bytes(b), len(b))

    def put(self, a, value, width=4):
        self.c.writemem(a, value, width)

    def get(self, a, width=4):
        return self.c.readmem(a, width)

    def call(self, fn, *args):
        result = self.c.callfunc(0x8000000 + fn, *(list(args) + [0] * (4 - len(args))))
        assert result != -1, hex(fn)
        return result

    def init(self):
        self.c.clearcodes()
        self.c.reset()
        for off, before, _ in self.p["patches"].values():
            assert self.get(0x8000000 + off, 2) == before
        self.write(0x2000000, bytes(0x40000))
        self.write(0x3000000, bytes(0x8000))
        self.put(self.p["sb1p"], 0x202A000)
        self.put(self.p["sb2p"], 0x2030000)
        self.put(self.p["rng"], 42)

    def enable(self, key):
        return [self.c.addcode(line.encode()) for line in self.codes[key]]

    def charm(self):
        p = self.p
        pocket = self.call(p["pocket"], p["charm"])
        self.put(p["bag"] + (pocket - 1) * 8, 0x2037000)
        self.put(p["bag"] + (pocket - 1) * 8 + 4, 1, 1)
        self.put(0x2037000, p["charm"], 2)
        self.put(0x2037002, 1, 2)
        assert self.call(p["has_item"], p["charm"], 1) == 1

    def decoder(self):
        result = {}
        for key, (off, before, after) in self.p["patches"].items():
            self.init()
            original = self.read(0x8000000, 0x2000000)
            assert struct.unpack_from("<H", original, off)[0] == before
            ids = self.enable(key)
            assert len(ids) == 1 and ids[0] >= 0
            expected = bytearray(original)
            struct.pack_into("<H", expected, off, after)
            assert self.read(0x8000000, len(expected)) == expected
            for i in range(24):
                on = i % 2
                self.c.togglecode(ids[0], on)
                self.c.reset()
                assert self.get(0x8000000 + off, 2) == (after if on else before)
            self.c.togglecode(ids[0], 0)
            assert self.read(0x8000000, len(original)) == original
            result[key] = dict(lines=self.codes[key], full_rom_diff=True, toggle_reset_checks=24)
        self.init()
        original = self.read(0x8000000, 0x2000000)
        expected = bytearray(original)
        for key, (off, _, after) in self.p["patches"].items():
            self.enable(key)
            struct.pack_into("<H", expected, off, after)
        assert self.read(0x8000000, len(expected)) == expected
        self.c.clearcodes()
        assert self.read(0x8000000, len(original)) == original
        result["all_recipes_combined_rom_diff_and_restore"] = True
        return result

    def hatch(self):
        p, r = self.p, self.rocket
        cases = 0
        for pid in range(24):
            for cycles in (0, 1, 2, 5, 255):
                for egg, bad in ((True, False), (False, False), (True, True)):
                    self.init()
                    raw = self.mon(r, egg=egg, cycles=cycles, pid=pid, bad=bad)
                    self.write(p["party"], raw)
                    self.put(p["count"], 1, 1)
                    before = self.read(p["party"], 100)
                    assert self.call(p["hatch"]) == 0
                    assert self.read(p["party"], 100) == before  # No regular checkpoint yet.
                    ids = self.enable(HATCH)
                    result = self.call(p["hatch"])
                    expected = bytearray(self.unpack(raw))
                    expected[8 if r else 9] = max(0, cycles - 1) if egg and not bad else cycles
                    assert self.read(p["party"], 100) == self.pack(raw, expected), (pid, cycles, egg, bad)
                    assert result == int(egg and not bad and cycles == 0)
                    self.c.togglecode(ids[0], 0)
                    after = self.read(p["party"], 100)
                    assert self.call(p["hatch"]) == 0
                    assert self.read(p["party"], 100) == after
                    cases += 1
        # Derive a native ability carrier from runtime species data by evaluating
        # the ROM's own cycle bonus. No names or ability catalog are bundled.
        carrier = None
        for species in range(1, 420):
            self.init()
            self.write(p["party"], self.mon(r, species=species))
            self.put(p["count"], 1, 1)
            if self.call(p["cycles"]) == 2:
                carrier = species
                break
        assert carrier
        for cycles in (1, 2, 5):
            self.init()
            raw = self.mon(r, egg=True, cycles=cycles)
            helper = self.mon(r, species=carrier)
            self.write(p["party"], raw + helper)
            self.put(p["count"], 2, 1)
            self.enable(HATCH)
            assert self.call(p["hatch"]) == 0
            q = bytearray(self.unpack(raw))
            q[8 if r else 9] = max(0, cycles - 2)
            assert self.read(p["party"], 200) == self.pack(raw, q) + helper
        return dict(permutation_boundary_skip_restore_cases=cases, ability_carrier_species=carrier, ability_cases=3)

    def catch(self):
        p, r = self.p, self.rocket
        scripts = (0x8525524, 0x852558D, 0x85255BB) if r else (0x82DBD84, 0x82DBDD4, 0x82DBE02)
        records = []
        for ball in ((1, 2, 3) if r else (4, 3, 2)):
            for flags in (0, 8, 512):
                for seed in (0, 1, 42, 999999, 0xFFFFFFFF):
                    pair = []
                    for on in (False, True):
                        self.init()
                        self.write(p["party"], mon(r))
                        self.put(p["count"], 1, 1)
                        raw = self.mon(r, species=150, pid=seed)
                        self.write(p["enemy"], raw)
                        self.put(0x2024C42 if r else 0x202406C, 0x03020100)
                        self.put(p["rng"], seed)
                        battle = (0x2024C50 + 92) if r else (0x2024084 + 88)
                        self.put(battle, 150, 2)
                        self.put(battle + (0x2A if r else 0x28), 100, 2)
                        self.put(battle + (0x2E if r else 0x2C), 100, 2)
                        self.put(0x2024DE4 if r else 0x2024208, ball, 2)
                        self.put(0x2024BB8 if r else 0x2022FEC, flags)
                        self.put(0x20250EC if r else 0x202439C, 0x2036000)
                        if r:
                            self.put(0x2025120, 0x2035000)
                            self.put(0x2035008, 0x2035100)
                        if on:
                            self.enable(CATCH)
                        self.call(p["catch"])
                        script = self.get(0x2024DF0 if r else 0x2024214)
                        after = self.read(p["enemy"], 100)
                        if flags == 0 and on:
                            assert script == scripts[0]
                            assert self.call(p["get"], p["enemy"], 38) == ({4:0,3:1,2:3}[ball] if self.ultimate else ball)
                            q = bytearray(self.unpack(raw))
                            if r:
                                q[9] = (q[9] & 0xE0) | ball
                            elif p['md5'] == '17ce9785b33319b3dbda9a5d37c57ec1':
                                # Independently observed native ball item -> stored index.
                                q[39] = (q[39] & ~0x7c) | ({4:0,3:1,2:3}[ball] << 2)
                            else:
                                value = struct.unpack_from("<H", q, 38)[0]
                                struct.pack_into("<H", q, 38, (value & ~0x7800) | (ball << 11))
                            assert after == self.pack(raw, q), (ball, seed)
                        elif flags == 8:
                            assert script == scripts[2] and after == raw
                        pair.append((script, after))
                    if flags:
                        assert pair[0] == pair[1]  # Trainer and tutorial branches unchanged.
                    records.append(dict(ball=ball, flags=flags, seed=seed, before=hex(pair[0][0]), after=hex(pair[1][0])))
        assert any(x["before"] != x["after"] for x in records if x["flags"] == 0)
        return dict(cases=len(records) * 2, outcomes=records)

    def walk(self):
        p = self.p
        table = p["wild"]
        rec = next(self.data[o:o + 20] for o in range(table, table + 6000, 20)
                   if struct.unpack_from("<I", self.data, o + 4)[0] and self.data[o] != 255)
        counts = []
        for on in (False, True):
            hits = 0
            for seed in range(32):
                self.init()
                self.write(p["party"], self.mon(self.rocket))
                self.put(p["count"], 1, 1)
                self.put(p["rng"], seed)
                self.write(0x202A004, rec[:2])
                if on:
                    self.enable(NO_ENCOUNTERS)
                hits += bool(self.call(p["walk"], 2, 2))
            counts.append(hits)
        assert counts[0] > 0 and counts[1] == 0
        return dict(seeds=32, map=list(rec[:2]), unpatched=counts[0], patched=counts[1])

    def breed(self):
        if not self.rocket:
            return {"published": False, "reason": "Different Oval Charm semantics; no compatible-pair guarantee."}
        p, d = self.p, 0x202A000 + self.p["day"]
        results = []
        for charm in (False, True):
            for case in ("compatible", "incompatible", "empty", "pending", "early"):
                counts = []
                for on in (False, True):
                    produced = 0
                    for seed in range(32):
                        self.init()
                        first = mon(True, species=150 if case == "incompatible" else 25, pid=0)[:80]
                        second = mon(True, species=150 if case == "incompatible" else 25, pid=255)[:80]
                        self.write(d, first)
                        if case != "empty":
                            self.write(d + 140, second)
                        if charm:
                            self.charm()
                        self.put(d + 0x114, 0 if case == "early" else 254)
                        self.put(d + 0x118, 12345 if case == "pending" else 0)
                        self.put(p["rng"], (seed * 0x9E3779B9) & 0xFFFFFFFF)
                        if on:
                            self.enable(BREED)
                        self.call(p["hatch"])
                        pending = self.get(d + 0x118)
                        assert self.read(d, 80) == first
                        if case != "empty":
                            assert self.read(d + 140, 80) == second
                        if case == "pending":
                            assert pending == 12345
                        elif case != "compatible":
                            assert pending == 0, (charm, case, seed)
                        produced += bool(pending)
                    counts.append(produced)
                if case == "compatible":
                    assert 0 < counts[0] < 32 and counts[1] == 32, (charm, case, counts)
                results.append(dict(charm=charm, case=case, unpatched=counts[0], patched=counts[1]))
        return dict(cases=640, results=results)


def main():
    source, build = Path(os.environ["MGBA_SOURCE"]), Path(os.environ["MGBA_BUILD"])
    library = Path(os.environ.get("MGBA_LIBRARY", str(build / ("libmgba.dylib" if sys.platform == "darwin" else "libmgba.so"))))
    report = {}
    with tempfile.TemporaryDirectory(prefix="gen3-common-cheats-") as temp:
        probe = Path(temp) / "probe.so"
        subprocess.run([os.environ.get("CC", "cc"), "-O2", "-shared", "-fPIC", "-DM_CORE_GBA", "-DM_CORE_GB", "-DUSE_DEBUGGERS",
                        "-I" + str(source / "include"), "-I" + str(build / "include"),
                        str(Path(__file__).parent / "native/common_cheat_probe.c"), str(library),
                        "-Wl,-rpath," + str(library.parent), "-o", str(probe)], check=True)
        for name in PROFILES:
            path = Path(os.environ["GEN3_ROM_" + name])
            native = Probe(probe, name, path)
            report[name] = dict(md5=native.p["md5"], decoder=native.decoder(), hatch=native.hatch(),
                                catch=native.catch(), walk=native.walk(), breeding=native.breed())
            native.c.finish()
            assert path.read_bytes() == native.data
            report[name]["source_rom_unchanged"] = True
    report["scope"] = "Complete native-function fixtures in mGBA; no user save loaded, not an entire-story or mobile gameplay test."
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
