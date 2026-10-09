#!/usr/bin/env python3
"""Native category/quantity checks using booted exact-ROM pocket descriptors.

Private scenarios come from verify_save_roundtrip.py. Inputs are read-only;
full-frame boot RAM is copied to a separate ARM7 probe, then only disposable RAM
is changed. No native hooks or replacements and no ROM/SAV write or export.
"""

import argparse
import hashlib
import json
import struct
from pathlib import Path
import verify_breeding as breeding
from verify_save_roundtrip import Frames, digest, POCKET_ORDER, NATIVE, PRODUCTION
from verify_breeding_production import KEY_OFFSET

COMMANDS = dict(
    BW=0x99A6C, DP=0x99A6C, ROCKET=0xCF95C, ULTIMATE=0x99A6C, MERCURY133=0x6A6E4
)
GET_POCKET = dict(
    BW=0xD7590, DP=0xD7590, ROCKET=0x10F958, ULTIMATE=0xD7590, MERCURY133=0x9A9D8
)
GET_VARIABLE = dict(
    BW=0x9D648, DP=0x9D648, ROCKET=0xD3930, ULTIMATE=0x9D648, MERCURY133=0x6E454
)
CONTEXT, OPERANDS = 0x02001000, 0x02002000


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--frame-probe", type=Path, required=True)
    parser.add_argument("--native-probe", type=Path, required=True)
    parser.add_argument("--scenario", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    breeding.MGBA_PROBE = args.native_probe
    rows = {}
    for key, case in json.loads(args.scenario.read_text()).items():
        rom, source = Path(case["rom"]).resolve(), Path(case["save"]).resolve()
        hashes = [digest(rom), digest(source)]
        cpu = None
        try:
            assert hashlib.md5(rom.read_bytes()).hexdigest() == breeding.MD5[key]
            frame = Frames(args.frame_probe)
            try:
                frame.start(rom, source)
                # Scenarios used here contain no captures or exports.
                assert not any("capture" in row for row in case["boot"])
                frame.steps(case["boot"], args.output.parent)
                inputs = [
                    (0x02000000, frame.read(0x02000000, 0x40000)),
                    (0x03000000, frame.read(0x03000000, 0x8000)),
                ]
                descriptors = [
                    struct.unpack("<II", frame.read(PRODUCTION[key][3] + i * 8, 8))
                    for i in range(len(POCKET_ORDER[key]))
                ]
                security_pointer = int.from_bytes(
                    frame.read(NATIVE[key][4], 4), "little"
                )
                security = int.from_bytes(
                    frame.read(security_pointer + KEY_OFFSET[key], 2), "little"
                )
            finally:
                frame.c.finish()
            breeding.ROM_PATH = rom
            cpu = breeding.MGBANative(rom.read_bytes())
            candidates = {}
            # Getter, rather than bundled item/category data, selects representatives.
            count = {
                "BW": 377,
                "DP": 377,
                "ROCKET": 923,
                "ULTIMATE": 800,
                "MERCURY133": 750,
            }[key]
            for item in range(1, count):
                category = cpu.call(GET_POCKET[key], item)
                if 1 <= category <= len(descriptors):
                    candidates.setdefault(category, item)
            assert set(candidates) == set(range(1, len(descriptors) + 1))
            pockets = []
            for index, (pointer, capacity) in enumerate(descriptors):
                assert 0x02000000 <= pointer and pointer + capacity * 4 <= 0x02040000
                category, item = index + 1, candidates[index + 1]
                vectors = []
                for quantities in [[], [0], [1], [7], [1, 6], [65535, 1]]:
                    for requested in [0, 1, 7, 8, 256, 65535]:
                        cpu.c.resetfixture()
                        for address, data in inputs:
                            cpu.write(address, data)
                        cpu.write(pointer, bytes(capacity * 4))
                        for slot, quantity in enumerate(quantities):
                            cpu.write(
                                pointer + slot * 4,
                                struct.pack(
                                    "<HH",
                                    item,
                                    quantity ^ (security if key != "MERCURY133" else 0),
                                ),
                            )
                        parameter_pointer = cpu.call(GET_VARIABLE[key], 0x8005)
                        cpu.half(parameter_pointer, requested)
                        cpu.write(OPERANDS, struct.pack("<HH", item, 0x8005))
                        cpu.word(CONTEXT + 8, OPERANDS)
                        cpu.call(COMMANDS[key], CONTEXT)
                        # Special variables are read through the loaded native pointer table.
                        result_pointer = cpu.call(GET_VARIABLE[key], 0x800D)
                        result = int.from_bytes(cpu.read(result_pointer, 2), "little")
                        value = requested if key == "MERCURY133" else requested & 255
                        quantity = (
                            (quantities[0] if quantities else 0)
                            if key == "MERCURY133"
                            else sum(quantities)
                        )
                        assert result == int(bool(quantities) and quantity >= value), (
                            key,
                            index,
                            quantities,
                            requested,
                            result,
                        )
                        vectors.append(
                            dict(
                                quantities=quantities,
                                requested=requested,
                                value=value,
                                result=result,
                            )
                        )
                pockets.append(
                    dict(
                        id=POCKET_ORDER[key][index],
                        category=category,
                        capacity=capacity,
                        item=item,
                        vectors=vectors,
                    )
                )
            rows[key] = dict(
                md5=breeding.MD5[key], pockets=pockets, input_hashes=hashes
            )
            print(key, len(pockets), "booted native pockets checked", flush=True)
        finally:
            if cpu is not None:
                cpu.c.finish()
                breeding.MGBANative.current = None
            assert hashes == [digest(rom), digest(source)], "input changed"

    args.output.write_text(json.dumps(rows, indent=2))


if __name__ == "__main__":
    main()
