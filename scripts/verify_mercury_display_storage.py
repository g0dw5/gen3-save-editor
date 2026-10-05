#!/usr/bin/env python3
"""Verify Mercury 1.2 names, expanded bag and metatile hook in native Thumb code.

Supply an exact private ROM. Optional dev-bridge JSON and loaded emulator EWRAM
cross-check current SAV observations. Only synthetic RAM is written; no ROM/SAV
is patched. Requires Unicorn (uv run --with unicorn python ...).
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_LR, UC_ARM_REG_PC
from verify_rocket_battle_forms import Native

MD5 = "f323df1792ac68462a34b42fe8571533"
BASE = 0x0203B174
POCKETS = [("items", 0x9AC, 450), ("key_items", 0x10B4, 75),
           ("balls", 0x11E0, 50), ("tmhm", 0x12A8, 128), ("berries", 0x14A8, 75)]


def terminated(data):
    # ROM text terminators here follow complete encoded character bytes.
    return data[:data.index(255) + 1]


def verify(rom, snapshot=None, ram=None, iwram=None):
    assert hashlib.md5(rom).hexdigest() == MD5, "wrong ROM fingerprint"
    native = Native(rom)
    native.call(0x1D40C70, 0, 0, 0x0203988C)
    for i, (_, offset, count) in enumerate(POCKETS):
        assert struct.unpack("<II", native.read(0x0203988C + i * 8, 8)) == (BASE + offset, count)
    # Quantity is plaintext, including with a nonzero FireRed security key.
    native.word(0x0300500C, 0x02025000)
    native.word(0x02025F20, 0xDEAD4321)
    for count in [0, 1, 6, 99, 999, 65535]:
        native.call(0x99DDC, 0x0203BB22, count)
        assert native.read(0x0203BB22, 2) == struct.pack("<H", count)
        assert native.call(0x99DD8, 0x0203BB22) == count
    # Independently execute the native save-tail hook with patterned extension RAM.
    pattern = bytes((i * 29 + 7) & 255 for i in range(0xEC4))
    native.write(BASE, pattern)
    fast_pointer = struct.unpack_from("<I", rom, 0x1D5AE48)[0]
    native.word(fast_pointer, 0x02010000)
    position = 0
    for section, start in [(0, 0xF24), (4, 0xD98), (13, 0x450)]:
        native.write(0x02010000, bytes([0xA5]) * 0x1000)
        native.half(0x02010FF4, section)
        before = native.read(0x02010000, 0x1000)
        native.call(0x1D5AE08)
        n = 0xFF0 - start
        expected = bytearray(before)
        expected[start:0xFF0] = pattern[position:position + n]
        assert native.read(0x02010000, 0x1000) == expected
        position += n
    assert position == 0xEC4
    # Keep the optional ship-name special case out of these static-name vectors.
    native.word(0x03005008, 0x02025000)
    names = 0
    invalid_names = 0
    for section in range(88, 253):
        if section == 94:
            continue
        ptr = struct.unpack_from("<I", rom, 0xC2B000 + (section - 88) * 4)[0] - 0x08000000
        if not 0 <= ptr < len(rom) - 80:
            invalid_names += 1
            continue
        native.call(0xC4D78, 0x02020000, section, 0)
        assert terminated(native.read(0x02020000, 80)) == terminated(rom[ptr:ptr + 80]), section
        names += 1
    # Record the three BG buffers before the final tilemap-upload helper.
    def skip_upload(cpu, address, _size, _data):
        if address == 0x080F67A4:
            cpu.reg_write(UC_ARM_REG_PC, cpu.reg_read(UC_ARM_REG_LR))
    handle = native.cpu.hook_add(UC_HOOK_CODE, skip_upload)
    targets = [0x02000000, 0x02001000, 0x02002000]  # BG3, BG2, BG1
    globals_ = [0x0300501C, 0x03005014, 0x03005018]
    for address, target in zip(globals_, targets):
        native.word(address, target)
    native.write(0x02003000, struct.pack("<12H", *range(101, 113)))
    vectors = {
        0: [[0x3014]*4, [101,102,103,104], [105,106,107,108]],
        1: [[0x3014]*4, [101,102,103,104], [105,106,107,108]],
        2: [[101,102,103,104], [105,106,107,108], [0]*4],
        3: [[101,102,103,104], [105,106,107,108], [109,110,111,112]],
        4: [[101,102,103,104], [0]*4, [105,106,107,108]],
    }
    for kind, expected in vectors.items():
        for target in targets:
            native.write(target, bytes(0x100))
        native.call(0x5A9B4, kind, 0x02003000, 0)
        actual = [[int.from_bytes(native.read(target + off, 2), "little")
                   for off in [0, 2, 64, 66]] for target in targets]
        assert actual == expected, (kind, actual)
    native.cpu.hook_del(handle)
    inventory_reads = 0
    if snapshot is not None and ram is not None:
        native.write(0x02000000, ram)
        for name, offset, count in POCKETS:
            entries = [e for e in snapshot["bag"] if e["pocket"] == name]
            assert len(entries) == count
            for entry in entries:
                address = BASE + offset + entry["slot"] * 4
                assert int.from_bytes(native.read(address, 2), "little") == entry["item"]
                assert native.call(0x99DD8, address + 2) == entry["quantity"]
                inventory_reads += 1
        sb1 = struct.unpack_from("<I", iwram, 0x5008)[0]
        for stored in snapshot["pokemon"]:
            if stored["location"]["kind"] == "party":
                raw = native.read(sb1 + 0x38 + stored["location"]["slot"] * 100, 100)
                native.write(0x02022000, raw)
                assert native.call(0x3FBE8, 0x02022000, 35, 0) == stored["pokemon"]["met_location"]
    return {"rom_md5": MD5, "native_name_vectors": names, "invalid_name_slots": invalid_names,
            "native_pocket_descriptors": len(POCKETS), "native_plain_quantity_vectors": 6,
            "native_save_tail_vectors": 3, "native_metatile_layer_vectors": len(vectors),
            "loaded_inventory_slots": inventory_reads}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rom", type=Path, required=True)
    parser.add_argument("--snapshot", type=Path)
    parser.add_argument("--ram", type=Path)
    parser.add_argument("--iwram", type=Path)
    args = parser.parse_args()
    assert bool(args.snapshot) == bool(args.ram) == bool(args.iwram), "supply snapshot, RAM and IWRAM together"
    print(json.dumps(verify(args.rom.read_bytes(),
          json.loads(args.snapshot.read_text()) if args.snapshot else None,
          args.ram.read_bytes() if args.ram else None,
          args.iwram.read_bytes() if args.iwram else None), indent=2))
