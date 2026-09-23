"""Exact-ROM portable-PC tests using final encrypted codes in mGBA.

Required: GEN3_BIN, GEN3_ROM_BW/DP/ROCKET/ULTIMATE, MGBA_SOURCE, MGBA_BUILD.
Optional full-frame tests require GEN3_STORAGE_STATE_<name> and
GEN3_STORAGE_SAVE_<name>: raw mGBA states at a freely walkable outdoor tile
(with space below) and matching battery saves. Inputs are read into memory,
never attached as writable saves. Synthetic party/boxes replace RAM fixtures.
GEN3_STORAGE_OUTPUT retains reports/screenshots/exported test saves locally.
No ROM, save, state or screenshot is a repository/release input.
"""
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import zlib
from verify_common_cheats_mgba import Probe, PROFILES, unpack

CHEAT = "portable-pokemon-storage"


class StorageProbe(Probe):
    def __init__(self, library, name, path):
        super().__init__(library, name, path)
        self.name = name
        self.path = path
        self.codes_pc = json.loads(subprocess.check_output([
            os.environ["GEN3_BIN"], "cheat-code", str(path), CHEAT,
            "gameshark_v1_v2"]))["lines"]
        self.entry = 0x1F9B8C if self.rocket else 0x1AD520
        self.compare = 0x1F9BD4 if self.rocket else 0x1AD568
        self.literal = 0x1F9C44 if self.rocket else 0x1AD5D8
        self.script = 0x1FFF100 if self.ultimate else 0x1F00100 if self.rocket else 0x311300
        self.header = 0x20382D8 if self.rocket else 0x2037318
        self.storage_pointer = 0x3005254 if self.rocket else 0x3005D94

    def enable_pc(self):
        result = self.c.addgroup("\n".join(self.codes_pc).encode())
        assert result >= 0
        return result

    def decoder(self):
        self.init()
        original = self.read(0x8000000, 0x2000000)
        expected = bytearray(original)
        struct.pack_into("<H", expected, self.compare, 0x4280)
        struct.pack_into("<I", expected, self.literal, 0x8000000 + self.script)
        expected[self.script:self.script + 8] = bytes.fromhex("6a253f00276c0200")
        assert len(self.codes_pc) == 7
        code = self.enable_pc()
        assert self.read(0x8000000, len(original)) == expected
        for i in range(12):
            self.c.togglecode(code, i % 2)
            self.c.reset()
            assert self.read(0x8000000 + self.script, 8) == (expected if i % 2 else original)[self.script:self.script + 8]
        self.c.togglecode(code, False)
        assert self.read(0x8000000, len(original)) == original
        return dict(lines=7, full_rom_diff=True, toggle_reset_checks=12)

    def guards(self):
        # Values come from each native eligibility routine, not UI map labels.
        union = 0x3C34 if self.rocket else 0x3C19
        partner = 0x0F35 if self.rocket else 0x0F1A
        cases = [(union, 0, 0), (0, 0x169, 0), (0, 0x17A, 0),
                 (0, 0x15F, 0), (0, 0x160, 0), (0, 0x166, 0),
                 (0, 0x167, 0), (partner, 0, 2)]
        for on in (False, True):
            for location, layout, var in cases:
                self.init()
                if on:
                    self.enable_pc()
                self.put(0x202A004, location, 2)
                self.put(self.header + 0x12, layout, 2)
                ptr = self.call(0xD3930 if self.rocket else 0x9D648, 0x40CE)
                self.put(ptr, var, 2)
                self.put(0x202A496, 259, 2)
                before = self.read(0x202A000, 0x4000)
                assert self.call(self.entry) == 0, (on, location, layout)
                assert self.read(0x202A000, 0x4000) == before
        # Both registered and unregistered paths use the script without changing
        # the registration, party or storage records. The original script returns
        # for an unregistered item after the complete code group is disabled.
        for registered in (0, 259, 65535):
            self.init()
            self.put(0x202A496, registered, 2)
            before = self.read(0x202A000, 0x4000)
            code = self.enable_pc()
            assert self.call(self.entry) == 1
            assert self.get(0x3000E48) == 0x8000000 + self.script
            assert self.read(0x202A000, 0x4000) == before
            self.c.togglecode(code, False)
            self.put(0x202A496, 0, 2)
            assert self.call(self.entry) == 1
            original = struct.unpack_from("<I", self.data, self.literal)[0]
            assert self.get(0x3000E48) == original
        return dict(blocked_native_cases=16, registration_cases=3)

    def press(self, *keys):
        for key in keys:
            self.c.frames(2, key)
            self.c.frames(160, 0)

    def screenshot(self, output, label):
        raw = output / f"{self.name}-{label}.rgba"
        self.c.screenshot(str(raw).encode())
        pixels = raw.read_bytes()
        assert len(pixels) == 240 * 160 * 4
        assert set(pixels[3::4]) == {255}, "Screenshots must be fully opaque"
        # PNG stores the native-size, unmodified RGB frame with explicit opacity.
        def chunk(kind, data):
            return (struct.pack(">I", len(data)) + kind + data
                    + struct.pack(">I", zlib.crc32(kind + data)))
        scanlines = b"".join(b"\0" + pixels[y * 960:(y + 1) * 960] for y in range(160))
        raw.with_suffix(".png").write_bytes(b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", 240, 160, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(scanlines)) + chunk(b"IEND", b""))

    def sb1(self):
        return self.get(self.p["sb1p"])

    def boxes(self):
        # Native exit/save reallocates SaveBlocks; never reuse an old pointer.
        return self.get(self.storage_pointer)

    def snapshot(self):
        count = self.get(self.p["count"], 1)
        party = [self.read(self.p["party"] + 100 * i, 80) for i in range(count)]
        boxes = [self.read(self.boxes() + 4 + 80 * i, 80) for i in range(420)]
        for raw in party + [r for r in boxes if any(r)]:
            self.unpack(raw)  # Native codec: plaintext or checked encrypted record.
        return party, boxes

    def gameplay(self, state, save, output):
        self.c.clearcodes()
        assert self.c.load_battery(str(save).encode())
        self.c.reset()
        assert self.c.load_state(str(state).encode())
        self.c.frames(10, 0)
        self.put(self.boxes(), 0, 1)
        self.write(self.boxes() + 4, bytes(420 * 80))
        for i, species in enumerate((25, 185)):
            self.call(0xEC2D0 if self.rocket else 0xB4E68, species, 17)
            self.write(self.p["party"] + 100 * i, self.read(self.p["enemy"], 100))
        self.put(self.p["count"], 2, 1)
        self.write(self.p["party"] + 200, bytes(400))
        first, second = self.snapshot()[0]
        registration = self.read(self.sb1() + 0x496, 2)
        location = self.read(self.sb1(), 12)
        code = self.enable_pc()
        self.press(4)  # SELECT, from normal overworld input.
        self.screenshot(output, "pc-menu")
        self.press(128, 128, 1)  # Open Organize, then exit without changing data.
        self.screenshot(output, "organize")
        self.press(2, 128, 1, 2)
        assert self.snapshot() == ([first, second], [bytes(80)] * 420)
        # Deposit first Pokémon, choose box 1, leave storage and its outer menu.
        self.press(4, 128, 1, 1, 1, 1, 2, 128, 1, 2)
        deposited_party, deposited_boxes = self.snapshot()
        assert deposited_party == [second]
        assert deposited_boxes == [first] + [bytes(80)] * 419
        assert self.read(self.sb1(), 12) == location
        assert self.read(self.sb1() + 0x496, 2) == registration
        # Move the boxed record to the adjacent slot through Organize's native
        # pick-up/place menus, then exit. No direct record write is used here.
        self.press(4, 128, 128, 1, 1, 1, 16, 1, 1)
        self.screenshot(output, "moved")
        self.press(2, 128, 1, 2)
        deposited_boxes = [bytes(80), first] + [bytes(80)] * 418
        assert self.snapshot() == ([second], deposited_boxes)
        # Normal in-game save through START. The private BW/DP fixture has eight
        # menu rows; Rocket's earlier-progress fixture has six (Save at index 3).
        # The test never invokes a raw save routine.
        self.press(8, *([128] * int(os.environ.get("GEN3_STORAGE_SAVE_ROW_" + self.name, 3 if self.rocket else 5))), 1, 1, 1)
        self.c.frames(360, 0)
        exported = output / f"{self.name}-saved-test.sav"
        assert self.c.export_battery(str(exported).encode())
        self.screenshot(output, "saved")
        # Use a fresh core (no residual RAM or cheat set) and the exported battery
        # save, then boot through title/Continue rather than loading a savestate.
        self.c.togglecode(code, False)
        self.c.finish()
        assert self.c.start(str(self.path).encode())
        assert self.c.load_battery(str(exported).encode())
        self.c.reset()
        self.c.frames(600, 0)
        self.screenshot(output, "boot")
        self.press(8, 1, 1, 1)
        self.c.frames(360, 0)
        self.screenshot(output, "continued")
        reloaded_party, reloaded_boxes = self.snapshot()
        assert reloaded_party == deposited_party, (self.name, "reload party", len(reloaded_party),
            [[(i, a, b) for i, (a, b) in enumerate(zip(actual, expected)) if a != b]
             for actual, expected in zip(reloaded_party, deposited_party)])
        assert reloaded_boxes == deposited_boxes, (self.name, "reload boxes",
            [i for i, (a, b) in enumerate(zip(reloaded_boxes, deposited_boxes)) if a != b])
        assert self.read(self.sb1() + 0x496, 2) == registration
        # Re-enable and withdraw the exact record via the native PC.
        code = self.enable_pc()
        self.press(4, 1)
        self.screenshot(output, "withdraw-open")
        self.press(16)
        self.screenshot(output, "withdraw-selection")
        self.press(1, 1, 2, 128, 1, 2)
        self.screenshot(output, "withdraw-result")
        current_party, current_boxes = self.snapshot()
        assert current_party == [second, first], (self.name, "withdraw", len(current_party),
            [[(i, a, b) for i, (a, b) in enumerate(zip(actual, expected)) if a != b]
             for actual, expected in zip(current_party, [second, first])])
        assert current_boxes == [bytes(80)] * 420, (self.name, "boxes", [i for i, b in enumerate(current_boxes) if any(b)])
        assert self.read(self.sb1(), 12) == location
        # Repeated opening/cancelling neither duplicates Pokémon nor changes data.
        for _ in range(3):
            self.press(4, 2)
            assert self.snapshot() == ([second, first], [bytes(80)] * 420)
        self.c.togglecode(code, False)
        self.c.frames(24, 128)
        self.c.frames(160, 0)
        assert self.read(self.sb1(), 4) != location[:4]  # Player controls resumed.
        return dict(organize_open_exit=True, exact_deposit_withdraw=True, exact_slot_move=True,
                    native_records_preserved=True, save_reset_reload=True,
                    reopen_cancel_cycles=3, registration_preserved=True,
                    exit_walk=True)


def main():
    source, build = Path(os.environ["MGBA_SOURCE"]), Path(os.environ["MGBA_BUILD"])
    library = Path(os.environ.get("MGBA_LIBRARY", str(build / ("libmgba.dylib" if sys.platform == "darwin" else "libmgba.so"))))
    with tempfile.TemporaryDirectory(prefix="gen3-storage-cheat-") as temp:
        output = Path(os.environ.get("GEN3_STORAGE_OUTPUT", temp))
        output.mkdir(parents=True, exist_ok=True)
        binary = Path(temp) / "probe.so"
        subprocess.run([os.environ.get("CC", "cc"), "-O2", "-shared", "-fPIC", "-DM_CORE_GBA", "-DM_CORE_GB", "-DUSE_DEBUGGERS",
                        "-I" + str(source / "include"), "-I" + str(build / "include"),
                        str(Path(__file__).parent / "native/storage_cheat_probe.c"), str(library),
                        "-Wl,-rpath," + str(library.parent), "-o", str(binary)], check=True)
        report = {}
        for name in PROFILES:
            path = Path(os.environ["GEN3_ROM_" + name])
            native = StorageProbe(binary, name, path)
            report[name] = dict(md5=native.p["md5"], decoder=native.decoder(), guards=native.guards())
            state, save = os.environ.get("GEN3_STORAGE_STATE_" + name), os.environ.get("GEN3_STORAGE_SAVE_" + name)
            if state and save:
                state, save = Path(state), Path(save)
                originals = {p: hashlib.sha256(p.read_bytes()).digest() for p in (state, save)}
                report[name]["gameplay"] = native.gameplay(state, save, output)
                assert all(hashlib.sha256(p.read_bytes()).digest() == digest for p, digest in originals.items())
            else:
                report[name]["gameplay"] = "not run; optional private fixtures not supplied"
            native.c.finish()
            assert path.read_bytes() == native.data
            report[name]["source_files_unchanged"] = True
            print(name + " passed", file=sys.stderr, flush=True)
            (output / "verification-progress.json").write_text(json.dumps(report, indent=2))
        print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
