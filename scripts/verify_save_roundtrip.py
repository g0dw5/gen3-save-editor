#!/usr/bin/env python3
"""Full-frame edit/load/in-game-save/re-read tests with private exact-ROM fixtures.

The scenario supplies real key-input sequences for its current game location.
No cheats, RAM writes, native function replacement or savestate loading. Source
ROM/SAV files are read-only; all edits and captures go to a fresh output folder.
Successful snapshots certify the tested fields/scenario, not every game edit.
"""

import argparse
import copy
import ctypes
import hashlib
import json
import re
import struct
import subprocess
import zlib
from pathlib import Path
from verify_breeding import MD5
from verify_breeding import CONFIG as NATIVE
from verify_breeding_production import CONFIG as PRODUCTION
from verify_breeding_production import KEY_OFFSET

# Descriptor order is a native engine layout, distinct from an item's ROM
# category or the editor's display order. Exact-ROM booted pointers/capacities
# were observed independently; no names, items or extracted catalogs live here.
EMERALD_POCKETS = ("items", "balls", "tmhm", "berries", "key_items")
POCKET_ORDER = {
    "BW": EMERALD_POCKETS,
    "DP": EMERALD_POCKETS,
    "ULTIMATE": EMERALD_POCKETS,
    "ROCKET": (
        "items",
        "medicine",
        "balls",
        "battle_items",
        "berries",
        "special_items",
        "tmhm",
        "key_items",
    ),
    "MERCURY133": ("items", "key_items", "balls", "tmhm", "berries"),
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inspect(cli, rom, save):
    return json.loads(
        subprocess.check_output([str(cli), "inspect", str(rom), str(save)])
    )


def logical(data, snapshot, layout, ids):
    start = snapshot["active_slot"] * 14 * 4096
    blocks = {}
    for index in range(14):
        block = data[start + index * 4096 : start + (index + 1) * 4096]
        section = struct.unpack_from("<H", block, 0xFF4)[0]
        assert section < 14 and section not in blocks
        blocks[section] = block[: layout["sizes"][section]]
    return b"".join(blocks[i] for i in ids)


def loaded_party(probe, save, snapshot, layout, ram):
    count = sum(p["location"]["kind"] == "party" for p in snapshot["pokemon"])
    assert probe.read(ram["count"], 1) == bytes([count]), "native party count differs"
    first = logical(save.read_bytes(), snapshot, layout, [1])
    expected = first[layout["party"] : layout["party"] + count * 100]
    assert (
        probe.read(ram["party"], len(expected)) == expected
    ), "native party bytes differ"
    return count


def loaded_storage(probe, save, snapshot, layout, ram):
    pointer = int.from_bytes(probe.read(ram["storage_pointer"], 4), "little")
    expected = logical(save.read_bytes(), snapshot, layout, range(5, 14))
    assert 0x02000000 <= pointer and pointer + len(expected) <= 0x02040000
    assert (
        probe.read(pointer, len(expected)) == expected
    ), "native box storage bytes differ"
    if layout.get("compressed_boxes"):
        data = save.read_bytes()
        # Compare every native box pointer, including RAM outside storage.
        # The table is the game's actual GetCompressedMonPtr input, not an
        # independently invented contiguous 80-byte layout.
        pointers = struct.unpack("<25I", probe.read(0x9DE4A7C, 100))
        blocks = {"Trainer": logical(data, snapshot, layout, [0]),
                  "Main": logical(data, snapshot, layout, range(1, 5)),
                  "Storage": expected}
        start = snapshot["active_slot"] * 14 * 4096
        sections = {struct.unpack_from("<H", data, start+i*4096+0xff4)[0]: start+i*4096 for i in range(14)}
        extensions = b"".join(data[sections[i]+size:sections[i]+0xff0] for i,size in enumerate(layout["sizes"]))
        extensions += b"".join(data[i*4096:i*4096+0xff0] for i in layout["extension_sectors"])
        blocks["Extensions"] = extensions
        compact = layout["compressed_boxes"]
        assert compact["record_size"] == 58
        for region in compact["regions"]:
            for i in range(region["count"]):
                box = region["first"] + i
                offset = region["offset"] + i * 30 * 58
                assert probe.read(pointers[box], 30*58) == blocks[region["block"]][offset:offset+30*58], f"native compact box {box+1} differs"


def loaded_inventory(probe, snapshot, layout, key):
    """Use native pocket descriptors, not the editor's file offsets for bags."""
    descriptors = PRODUCTION[key][3]
    sb1 = int.from_bytes(probe.read(NATIVE[key][3], 4), "little")
    sb2 = int.from_bytes(probe.read(NATIVE[key][4], 4), "little")
    assert 0x02000000 <= sb1 < 0x02040000 and 0x02000000 <= sb2 < 0x02040000
    security = int.from_bytes(probe.read(sb2 + KEY_OFFSET[key], 2), "little")
    total = 0
    for pocket in layout["pockets"]:
        if pocket["category"] == 0:
            pointer, count = sb1 + pocket["offset"], pocket["count"]
        else:
            index = POCKET_ORDER[key].index(pocket["id"])
            data = probe.read(descriptors + index * 8, 8)
            pointer, count = struct.unpack("<II", data)
            assert count == pocket["count"], (pocket["id"], count)
        assert 0x02000000 <= pointer and pointer + count * 4 <= 0x02040000
        native = list(struct.iter_unpack("<HH", probe.read(pointer, count * 4)))
        expected = [p for p in snapshot["bag"] if p["pocket"] == pocket["id"]]
        assert len(expected) == count
        for i, (item, quantity) in enumerate(native):
            quantity ^= security if pocket["encrypted"] else 0
            # Empty slots carry no visible quantity even if unused bytes are dirty.
            if item == 0:
                quantity = 0
            assert (item, quantity) == (expected[i]["item"], expected[i]["quantity"]), (
                pocket["id"],
                i,
                item,
                quantity,
                expected[i],
            )
        total += count
    return total


def unchanged_rom_bus(probe, rom):
    expected = rom.read_bytes()
    actual = probe.read(0x08000000, len(expected))
    # mGBA represents GPIO data/direction/control in ROM-mapped bytes C4..C9.
    # Native RTC writes legitimately change these six emulator hardware bytes.
    assert actual[:0xC4] == expected[:0xC4] and actual[0xCA:] == expected[0xCA:]
    return actual[0xC4:0xCA].hex()


def negative_controls(probe, save, snapshot, layout, ram, key):
    """An unrelated RAM region or incorrect quantity must not count as a pass."""
    wrong_party = dict(ram, party=ram["party"] + 1)
    wrong_storage = dict(ram, storage_pointer=ram["storage_pointer"] + 1)
    wrong_bag = copy.deepcopy(snapshot)
    entry = next(row for row in wrong_bag["bag"] if row["item"])
    entry["quantity"] += 1
    checks = [
        lambda: loaded_party(probe, save, snapshot, layout, wrong_party),
        lambda: loaded_storage(probe, save, snapshot, layout, wrong_storage),
        lambda: loaded_inventory(probe, wrong_bag, layout, key),
    ]
    for check in checks:
        try:
            check()
        except AssertionError:
            continue
        raise AssertionError("incorrect native expectation was accepted")
    return len(checks)


class Frames:
    def __init__(self, library):
        self.c = ctypes.CDLL(str(library.resolve()))
        for name in ["start", "load_battery", "export_battery", "screenshot"]:
            getattr(self.c, name).argtypes = [ctypes.c_char_p]
        self.c.frames.argtypes = [ctypes.c_uint, ctypes.c_uint]
        self.c.readbytes.argtypes = [ctypes.c_uint, ctypes.c_void_p, ctypes.c_uint]
        for name in ["finish", "reset", "frames", "readbytes"]:
            getattr(self.c, name).restype = None

    def start(self, rom, save):
        assert self.c.start(str(rom).encode()) == 1
        assert self.c.load_battery(str(save).encode()) == 1
        self.c.reset()

    def read(self, address, size):
        data = ctypes.create_string_buffer(size)
        self.c.readbytes(address, data, size)
        return data.raw

    def capture(self, output, name):
        assert re.fullmatch(r"[A-Za-z0-9_-]+", name)
        raw = output / (name + ".rgba")
        assert self.c.screenshot(str(raw).encode()) == 1
        pixels = raw.read_bytes()
        assert len(pixels) == 240 * 160 * 4 and set(pixels[3::4]) == {255}

        def chunk(kind, data):
            return (
                struct.pack(">I", len(data))
                + kind
                + data
                + struct.pack(">I", zlib.crc32(kind + data))
            )

        scanlines = b"".join(
            b"\0" + pixels[i * 960 : (i + 1) * 960] for i in range(160)
        )
        png = (
            b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">2I5B", 240, 160, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(scanlines))
            + chunk(b"IEND", b"")
        )
        with (output / (name + ".png")).open("xb") as file:
            file.write(png)

    def steps(self, steps, output):
        for row in steps:
            frames, keys = row["frames"], row.get("keys", 0)
            assert 0 <= frames <= 10000 and 0 <= keys <= 1023
            self.c.frames(frames, keys)
            if "capture" in row:
                self.capture(output, row["capture"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", type=Path, required=True)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--scenario", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    scenario = json.loads(args.scenario.read_text())
    profiles = {
        p["md5"]: p
        for p in json.loads(subprocess.check_output([str(args.cli), "profiles"]))
    }
    args.output.mkdir(parents=True, exist_ok=False)
    result = {}
    (args.output / "scenario.json").write_text(json.dumps(scenario, indent=2))
    (args.output / "tooling.json").write_text(
        json.dumps(
            dict(
                probe_sha256=digest(args.probe),
                cli_sha256=digest(args.cli),
                source_revision=subprocess.check_output(
                    ["git", "rev-parse", "HEAD"], text=True
                ).strip(),
            ),
            indent=2,
        )
    )
    for key, case in scenario.items():
        assert key in MD5
        rom, source = Path(case["rom"]).resolve(), Path(case["save"]).resolve()
        assert hashlib.md5(rom.read_bytes()).hexdigest() == MD5[key]
        layout = profiles[MD5[key]]["save"]
        original = {str(p): digest(p) for p in [rom, source]}
        try:
            output = args.output / key
            output.mkdir()
            before = inspect(args.cli, rom, source)
            actions = output / "actions.json"
            actions.write_text(json.dumps(case["actions"]))
            edited = output / "edited.sav"
            with (output / "edit-report.json").open("w") as report:
                subprocess.run(
                    [
                        str(args.cli),
                        "patch-save",
                        str(rom),
                        str(source),
                        str(actions),
                        str(edited),
                    ],
                    check=True,
                    stdout=report,
                )
            expected = inspect(args.cli, rom, edited)
            for label, data in [("before", before), ("edited", expected)]:
                (output / (label + ".json")).write_text(json.dumps(data))
            probe = Frames(args.probe)
            try:
                probe.start(rom, edited)
                probe.steps(case["boot"], output)
                probe.capture(output, "loaded")
                loaded_party(probe, edited, expected, layout, case["ram"])
                loaded_storage(probe, edited, expected, layout, case["ram"])
                slots = loaded_inventory(probe, expected, layout, key)
                rejected = negative_controls(
                    probe, edited, expected, layout, case["ram"], key
                )
                probe.steps(case["save_steps"], output)
                probe.capture(output, "saved")
                saved = output / "resaved.sav"
                assert probe.c.export_battery(str(saved).encode()) == 1
                actual = inspect(args.cli, rom, saved)
                (output / "resaved.json").write_text(json.dumps(actual))
                assert (
                    actual["counter"] == (expected["counter"] + 1) & 0xFFFFFFFF
                ), "unexpected native save count"
                assert (
                    actual["pokemon"] == expected["pokemon"]
                ), "individual fields changed"
                assert actual["bag"] == expected["bag"], "inventory changed"
                saved_hash = digest(saved)
                assert (
                    probe.c.export_battery(str(saved).encode()) == 0
                ), "export overwrote a file"
                assert digest(saved) == saved_hash
                probe.c.finish()
                probe.start(rom, saved)
                probe.steps(case["reload"], output)
                probe.capture(output, "reloaded")
                loaded_party(probe, saved, actual, layout, case["ram"])
                loaded_storage(probe, saved, actual, layout, case["ram"])
                assert loaded_inventory(probe, actual, layout, key) == slots
                gpio = unchanged_rom_bus(probe, rom)
                final = output / "reloaded.sav"
                assert probe.c.export_battery(str(final).encode()) == 1
                assert (
                    final.read_bytes() == saved.read_bytes()
                ), "reload changed flash without saving"
            finally:
                probe.c.finish()
            for label, data in [
                ("before", before),
                ("edited", expected),
                ("resaved", actual),
            ]:
                (output / (label + ".json")).write_text(json.dumps(data))
            result[key] = dict(
                md5=MD5[key],
                input_hashes=original,
                edited_sha256=digest(edited),
                resaved_sha256=digest(saved),
                counter=[expected["counter"], actual["counter"]],
                actions=case["actions"],
                individual_snapshot_equal=True,
                bag_snapshot_equal=True,
                native_inventory_slots=slots,
                rejected_negative_controls=rejected,
                existing_export_preserved=True,
                gpio_bus_bytes=gpio,
                scope="Normal-key native save/reload for this fixture; no all-edit or legitimacy claim",
            )
            print(
                key,
                "native save/reload preserves tested individual and inventory fields",
                flush=True,
            )
            (args.output / "report.json").write_text(json.dumps(result, indent=2))
        finally:
            assert all(
                digest(Path(p)) == value for p, value in original.items()
            ), "source file changed"


if __name__ == "__main__":
    main()
