#!/usr/bin/env python3
"""Run exact-ROM form/state probes without opening or writing a save.

Requires Python 3 and Unicorn. All Pokémon and battle RAM are synthetic.
This is a research harness, not a save adapter or full battle simulation.
"""

import argparse
import hashlib
import json
import struct
from pathlib import Path

from unicorn import Uc, UC_ARCH_ARM, UC_MODE_THUMB
from unicorn.arm_const import (
    UC_ARM_REG_LR,
    UC_ARM_REG_PC,
    UC_ARM_REG_R0,
    UC_ARM_REG_R1,
    UC_ARM_REG_R2,
    UC_ARM_REG_R3,
    UC_ARM_REG_SP,
)


ROM_MD5 = "59c658a1081f542086de1060bb65f0b3"
PARTY = 0x02025170
BATTLE = 0x0202E000
BATTLER = 0x02024C50
ORDERS = (
    "GAEM", "GAME", "GEAM", "GEMA", "GMAE", "GMEA",
    "AGEM", "AGME", "AEGM", "AEMG", "AMGE", "AMEG",
    "EGAM", "EGMA", "EAGM", "EAMG", "EMGA", "EMAG",
    "MGAE", "MGEA", "MAGE", "MAEG", "MEGA", "MEAG",
)


def unpack(raw):
    pid, ot = struct.unpack_from("<II", raw)
    data = struct.pack("<12I", *(v ^ pid ^ ot for v in struct.unpack_from("<12I", raw, 32)))
    chunks = {ch: data[i * 12:(i + 1) * 12] for i, ch in enumerate(ORDERS[pid % 24])}
    return b"".join(chunks[ch] for ch in "GAEM")


def pack(raw, canonical):
    raw = bytearray(raw)
    pid, ot = struct.unpack_from("<II", raw)
    data = b"".join(
        canonical["GAEM".index(ch) * 12:("GAEM".index(ch) + 1) * 12]
        for ch in ORDERS[pid % 24]
    )
    raw[32:80] = struct.pack("<12I", *(v ^ pid ^ ot for v in struct.unpack("<12I", data)))
    struct.pack_into("<H", raw, 28, sum(struct.unpack("<24H", canonical)) & 0xFFFF)
    return bytes(raw)


def fixture(species, item=0, pid=24, moves=(1, 2, 3, 4)):
    raw = bytearray(100)
    struct.pack_into("<II", raw, 0, pid, 0x12345678)
    # This engine packs language and hasSpecies into byte 18.
    raw[18] = 2 | 0x10
    raw[84] = 50
    struct.pack_into("<HH", raw, 86, 100, 100)
    canonical = bytearray(48)
    struct.pack_into("<HHI", canonical, 0, species, item, 125000)
    # Ball 1 and the native "use PID nature" sentinel 26 share this word.
    struct.pack_into("<I", canonical, 8, 70 | (1 << 8) | (26 << 13))
    struct.pack_into("<4H", canonical, 12, *moves)
    canonical[20:24] = bytes((10, 10, 10, 10))
    struct.pack_into("<I", canonical, 40, 0x3FFFFFFF)
    return pack(raw, canonical)


class Native:
    def __init__(self, rom):
        self.rom = rom
        self.cpu = Uc(UC_ARCH_ARM, UC_MODE_THUMB)
        for address, size in ((0x08000000, 0x02000000), (0x02000000, 0x40000), (0x03000000, 0x8000)):
            self.cpu.mem_map(address, size)
        self.write(0x08000000, rom)

    def write(self, address, data):
        self.cpu.mem_write(address, bytes(data))

    def read(self, address, size):
        return bytes(self.cpu.mem_read(address, size))

    def half(self, address, value):
        self.write(address, struct.pack("<H", value))

    def word(self, address, value):
        self.write(address, struct.pack("<I", value))

    def call(self, offset, *args):
        registers = (UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3)
        for register, value in zip(registers, args):
            self.cpu.reg_write(register, value)
        self.cpu.reg_write(UC_ARM_REG_SP, 0x03007000)
        for i, value in enumerate(args[4:]):
            self.word(0x03007000 + i * 4, value)
        self.cpu.reg_write(UC_ARM_REG_LR, 0x08000001)
        self.cpu.emu_start(0x08000001 + offset, 0x08000000, count=1_000_000)
        assert self.cpu.reg_read(UC_ARM_REG_PC) == 0x08000000, f"unfinished native call {offset:#x}"
        return self.cpu.reg_read(UC_ARM_REG_R0)

    def reset(self):
        self.write(0x02000000, bytes(0x40000))
        self.word(0x020250EC, BATTLE)
        self.word(0x020250F8, 0x0202C000)
        self.write(0x02024C42, bytes((0, 1, 2, 3)))


def check(rom):
    native = Native(rom)
    results = {}
    associations = []
    for species in range(1, 1395):
        for slot in range(10):
            method, item, target, _ = struct.unpack_from("<4H", rom, 0x5F96D4 + species * 80 + slot * 8)
            if method in (0xFFFF, 0xFFFE, 0xFFFD):
                associations.append((species, method, item, target))
    assert len(associations) == 59

    for pid in range(24):
        for source, method, item, target in associations:
            native.reset()
            raw = fixture(target, item if method != 0xFFFE else 0, pid)
            native.write(PARTY, raw)
            flag = 0x2C2 if method == 0xFFFD else 0x2B1
            expected = native.call(0x9D19C, target, 0) if method == 0xFFFD else source
            native.write(BATTLE + flag, b"\1")
            native.half(BATTLE + 0x2C0, source)
            native.call(0x678E0, 0)
            after = native.read(PARTY, 100)
            canonical = unpack(after)
            assert int.from_bytes(canonical[:2], "little") == expected
            assert canonical[2:] == unpack(raw)[2:]
            assert after[:28] == raw[:28] and after[30:32] == raw[30:32]
            assert sum(struct.unpack("<24H", canonical)) & 0xFFFF == int.from_bytes(after[28:30], "little")
            assert native.read(BATTLE + flag, 1) == b"\0"

            # Merely storing a Mega ID does not create its restoration metadata.
            native.reset()
            native.write(PARTY, raw)
            native.call(0x678E0, 0)
            assert native.read(PARTY, 100) == raw
    results["special_restore_cases_all_pid_orders"] = 59 * 24
    results["missing_battle_metadata_controls"] = 59 * 24

    for i in range(16):
        source, target, on_switch = struct.unpack_from("<3H", rom, 0x5AA6A4 + i * 6)
        for side in range(2):
            for switching in range(2):
                native.reset()
                address = PARTY if side == 0 else 0x020253C8
                raw = fixture(source)
                native.write(address, raw)
                native.call(0x679BC, 0, side, switching)
                canonical = unpack(native.read(address, 100))
                assert int.from_bytes(canonical[:2], "little") == (target if not switching or on_switch else source)
                assert canonical[2:] == unpack(raw)[2:]
    results["other_form_reset_cases"] = 64

    for i in range(38):
        species, item, move, zmove = struct.unpack_from("<4H", rom, 0x5AA704 + i * 8)
        assert native.call(0x68F58, move, species, item) == zmove
        native.reset()
        raw = fixture(species, item, moves=(move, 1, 2, 3))
        native.write(PARTY, raw)
        native.half(BATTLER, species)
        native.half(BATTLER + 0x30, item)
        assert native.call(0x68950, 0, move) == 1
        assert int.from_bytes(native.read(BATTLE + 0x2DA, 2), "little") == zmove
        native.call(0x688D4, 0, move)
        assert int.from_bytes(native.read(BATTLE + 0x2E2, 2), "little") == zmove
        assert int.from_bytes(native.read(BATTLE + 0x2EA, 2), "little") == move
        assert native.read(PARTY, 100) == raw
    results["signature_z_eligibility_and_queue"] = 38

    for item in range(367, 385):
        move_type = rom[0xC3D558 + item * 44 + 40]
        move = next(i for i in range(1, 755) if rom[0x5ACD5C + i * 20 + 4] == move_type and rom[0x5ACD5C + i * 20 + 16] != 2)
        native.reset()
        native.half(BATTLER, 25)
        native.half(BATTLER + 0x30, item)
        assert native.call(0x68950, 0, move) == 1
        assert int.from_bytes(native.read(BATTLE + 0x2DA, 2), "little") == 755 + move_type - (move_type > 9)
    results["type_z_eligibility"] = 18

    for state in ("used", "mega_used", "wrong_item", "wrong_type", "frontier"):
        native.reset()
        native.half(BATTLER, 25)
        native.half(BATTLER + 0x30, 370)
        if state == "used":
            native.write(BATTLE + 0x2DD, b"\1")
        elif state == "mega_used":
            native.write(0x0202C03B, b"\1")
        elif state == "wrong_item":
            native.half(BATTLER + 0x30, 0)
        elif state == "wrong_type":
            native.half(BATTLER + 0x30, 368)
        else:
            native.word(0x02024BB8, 0x10000)
        assert native.call(0x68950, 0, 85) == 0, state
    results["z_negative_controls"] = 5

    for move in range(850):
        assert native.call(0x688B8, move) == int(755 <= move <= 789)
    results["z_range_cases"] = 850

    # The known anti-Dynamax effect dispatches to the common exit, not a check.
    assert struct.unpack_from("<I", rom, 0x635CC + (356 - 7) * 4)[0] == 0x08064280
    dynamax_cases = 0
    for move in range(1, 755):
        effect, power = struct.unpack_from("<HH", rom, 0x5ACD5C + move * 20)
        if effect == 356:
            native.reset()
            assert native.call(0x6354C, move, 0, 1) == power
            dynamax_cases += 1
    results["anti_dynamax_moves_without_target_check"] = dynamax_cases

    native.reset()
    native.write(PARTY, fixture(487))
    native.half(0x0202F800, 413)
    native.call(0x97E2C, PARTY, 12, 0x0202F800)
    assert native.call(0x976D0, PARTY, 11) == 487
    assert native.call(0x9D210, PARTY, 1, 0) == 1143
    results["held_item_setter_requires_separate_form_transition"] = True

    for value in (0, 3, 10, 24, 26):
        raw = fixture(330)
        canonical = bytearray(unpack(raw))
        word = int.from_bytes(canonical[8:12], "little")
        canonical[8:12] = ((word & ~(31 << 13)) | (value << 13)).to_bytes(4, "little")
        native.write(PARTY, pack(raw, canonical))
        assert native.call(0x976D0, PARTY, 89) == value
        assert native.call(0x9A334, PARTY, 0) == 24
        assert native.call(0x9A334, PARTY, 1) == (24 if value == 26 else value)
    results["effective_nature_controls"] = 5

    fields = (
        (25, 12345, 4 * 8, 23),
        (21, 0x69, 7 * 8, 8),
        (32, 120, 8 * 8, 8),
        (38, 17, 9 * 8, 5),
        (89, 3, 9 * 8 + 5, 5),
        (46, 2, 47 * 8, 2),
    )
    for pid in range(24):
        for field, value, start, width in fields:
            native.reset()
            raw = fixture(330, pid=pid)
            canonical = bytearray((i * 17 + 0xA5) & 255 for i in range(48))
            struct.pack_into("<H", canonical, 0, 330)
            raw = pack(raw, canonical)
            native.write(PARTY, raw)
            native.word(0x0202F800, value)
            native.call(0x97E2C, PARTY, field, 0x0202F800)
            after = native.read(PARTY, 100)
            mask = ((1 << width) - 1) << start
            expected = (int.from_bytes(canonical, "little") & ~mask) | (value << start)
            assert int.from_bytes(unpack(after), "little") == expected
            assert native.call(0x976D0, PARTY, field) == value
            assert after[:28] == raw[:28] and after[30:32] == raw[30:32]
    results["packed_field_native_setter_cases"] = len(fields) * 24
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rom", required=True, type=Path)
    args = parser.parse_args()
    data = args.rom.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if hashlib.md5(data).hexdigest() != ROM_MD5:
        parser.error("unsupported ROM fingerprint; do not reuse these offsets for another build")
    results = check(data)
    assert hashlib.sha256(args.rom.read_bytes()).hexdigest() == digest
    print(json.dumps({"rom_md5": ROM_MD5, "checks": results, "source_unchanged": True}, indent=2))


if __name__ == "__main__":
    main()
