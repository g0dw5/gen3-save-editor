#!/usr/bin/env python3
"""Execute exact-ROM RTC validation/difference in disposable mGBA RAM.

Hardware input is explicit, never host time. Source ROMs and user saves are not
written. Private vectors are native scenario evidence, not a game-time catalog.
"""

import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
import verify_breeding as b
from verify_breeding_production import fresh

# Native validation, difference, full getter, SB2 pointer, local RAM.
CONFIG = {
    "BW": (0x2F2FC, 0x2F504, 0x2F588, 0x03005D90, 0x03005CF8),
    "DP": (0x2F2FC, 0x2F504, 0x2F588, 0x03005D90, 0x03005CF8),
    "ULTIMATE": (0x2F2FC, 0x2F504, 0x2F588, 0x03005D90, 0x03005CF8),
    "ROCKET": (0x441A0, 0x443A8, 0x4442C, 0x03005250, 0x030051A8),
}
RTC, RESULT, OFFSET, SB2 = 0x02010000, 0x02010100, 0x02010200, 0x02038000


def raw_rtc(date):
    year, month, day, hour, minute, second = date
    bcd = lambda v: (v // 10) * 16 + v % 10
    return bytes(
        [
            bcd(year - 2000),
            bcd(month),
            bcd(day),
            0,
            bcd(hour),
            bcd(minute),
            bcd(second),
            0x40,
            0,
            0,
            0,
            0,
        ]
    )


def native_time(raw):
    days, hour, minute, second = struct.unpack_from("<hbbb", raw)
    return dict(days=days, hour=hour, minute=minute, second=second)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--mgba-probe", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    args = p.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    output = {}
    dates = [
        (2000, 1, 1),
        (2000, 2, 29),
        (2001, 3, 1),
        (2004, 2, 29),
        (2026, 10, 6),
        (2099, 12, 31),
    ]
    offsets = [
        (0, 0, 0, 0),
        (1, 23, 59, 59),
        (-1, 5, 30, 10),
        (2000, 12, 0, 0),
        (32767, 0, 0, 0),
        (-32768, 0, 0, 0),
    ]
    for key, (check, diff, getter, sb2ptr, local) in CONFIG.items():
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        assert args.output.resolve() != b.ROM_PATH.resolve()
        rows = []
        cpu = fresh(rom)
        for y, m, d in dates:
            for h in range(24):
                for off in offsets:
                    date = (y, m, d, h, (h * 7) % 60, (h * 11) % 60)
                    cpu.write(RTC, raw_rtc(date))
                    cpu.write(OFFSET, struct.pack("<hbbb", *off) + b"\x00")
                    assert cpu.call(check, RTC) == 0
                    before = cpu.read(OFFSET, 6)
                    cpu.write(RESULT, bytes(6))
                    cpu.call(diff, RTC, RESULT, OFFSET)
                    result = native_time(cpu.read(RESULT, 6))
                    assert cpu.read(OFFSET, 6) == before and cpu.read(
                        RTC, 12
                    ) == raw_rtc(date)
                    assert (
                        0 <= result["hour"] < 24
                        and 0 <= result["minute"] < 60
                        and 0 <= result["second"] < 60
                    )
                    rows.append(dict(date=date, offset=off, result=result))
        # Full getter: native RTC error path reads its ROM dummy, then real SB2 +0x98.
        dummy = []
        for off in offsets:
            cpu = fresh(rom)
            cpu.word(sb2ptr, SB2)
            cpu.half(0x03000DBC, 0x10)
            raw = struct.pack("<hbbb", *off) + b"\0"
            cpu.write(SB2 + 0x98, raw)
            before = cpu.read(SB2, 0x100)
            cpu.call(getter)
            assert cpu.read(SB2, 0x100) == before
            dummy.append(dict(offset=off, result=native_time(cpu.read(local, 6))))
        assert cpu.read(0x08000000, len(rom)) == rom and b.ROM_PATH.read_bytes() == rom
        output[key] = dict(
            md5=b.MD5[key],
            validation=check,
            difference=diff,
            getter=getter,
            rows=rows,
            dummy=dummy,
        )
        print(
            key,
            len(rows),
            "RTC scenarios;",
            len(dummy),
            "full native error-clock cases",
            flush=True,
        )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + "\n")


if __name__ == "__main__":
    main()
