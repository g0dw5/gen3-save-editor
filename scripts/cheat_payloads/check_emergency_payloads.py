#!/usr/bin/env python3
"""Rebuild and compare both authored emergency cheat payloads.

Requires Clang with its ARM target and LLD. Never reads or writes a ROM or save.
The checked-in binary is compared byte-for-byte; this script does not update it.
"""

from pathlib import Path
import struct
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
EXPECTED = HERE.parents[1] / "crates/gen3-core/src/cheats"


def section(data: bytes, name: bytes) -> bytes:
    if data[:4] != b"\x7fELF" or data[4:6] != b"\x01\x01":
        raise ValueError("expected little-endian ELF32")
    table = struct.unpack_from("<I", data, 0x20)[0]
    entry_size, count, names_index = struct.unpack_from("<HHH", data, 0x2E)
    if entry_size != 40 or names_index >= count:
        raise ValueError("unexpected ELF section table")

    def header(index: int) -> tuple[int, ...]:
        return struct.unpack_from("<10I", data, table + index * entry_size)

    names_header = header(names_index)
    names = data[names_header[4]:names_header[4] + names_header[5]]
    for index in range(count):
        fields = header(index)
        start = fields[0]
        if names[start:].split(b"\0", 1)[0] == name:
            return data[fields[4]:fields[4] + fields[5]]
    raise ValueError(f"missing {name!r} section")


def main() -> None:
    for variant, defines in (("ultimate", []), ("rocket", ["-DGEN3_EMERGENCY_ROCKET"]), ("mercury", ["-DGEN3_EMERGENCY_MERCURY"])):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "emergency.elf"
            subprocess.run([
                "clang", "--target=armv4t-none-eabi", "-mthumb", "-Oz",
                "-ffreestanding", "-fno-builtin", "-nostdlib", "-fuse-ld=lld",
                f"-Wl,-T,{HERE / 'emergency.ld'}", *defines,
                str(HERE / "emergency.c"), "-o", str(output),
            ], check=True)
            actual = section(output.read_bytes(), b".payload")
        path = EXPECTED / f"emergency_{variant}.bin"
        expected = path.read_bytes()
        if actual != expected:
            raise SystemExit(f"{variant} payload mismatch: built {len(actual)} bytes, checked in {len(expected)} bytes")
        print(f"{variant} emergency payload matches checked-in binary: {len(actual)} bytes")


if __name__ == "__main__":
    main()
