"""Opt-in exact-ROM / independent mGBA decoder validation.

Required: GEN3_BIN, GEN3_ROM_ULTIMATE, GEN3_ROM_BW, MGBA_SOURCE,
MGBA_BUILD. Optional MGBA_LIBRARY (defaults to build/libmgba.dylib on macOS,
build/libmgba.so elsewhere), CC. No save is loaded. Compilation/output stays
in a temporary directory. A private JSON report may be redirected by the caller.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    source, build = Path(os.environ["MGBA_SOURCE"]), Path(os.environ["MGBA_BUILD"])
    library = Path(os.environ.get("MGBA_LIBRARY", str(build / ("libmgba.dylib" if sys.platform == "darwin" else "libmgba.so"))))
    ultimate, bw = Path(os.environ["GEN3_ROM_ULTIMATE"]), Path(os.environ["GEN3_ROM_BW"])
    originals = {p: p.read_bytes() for p in (ultimate, bw)}
    assert hashlib.md5(originals[ultimate]).hexdigest() == "17ce9785b33319b3dbda9a5d37c57ec1"
    assert hashlib.md5(originals[bw]).hexdigest() == "0d9b129f7dd76895f79bb47ad7dec2fe"
    cli = os.environ["GEN3_BIN"]
    generated = json.loads(subprocess.check_output([cli, "cheat-code", str(ultimate), "disable-input-peeking", "gameshark_v1_v2"]))
    species = json.loads(subprocess.check_output([cli, "species", str(bw), "185"]))
    with tempfile.TemporaryDirectory(prefix="gen3-cheat-probe-") as temp:
        binary = str(Path(temp) / "probe")
        subprocess.run([os.environ.get("CC", "cc"), "-O2", "-DM_CORE_GBA", "-DM_CORE_GB", "-DUSE_DEBUGGERS",
                        "-I" + str(source / "include"), "-I" + str(build / "include"),
                        str(Path(__file__).parent / "native/cheat_probe.c"), str(library),
                        "-Wl,-rpath," + str(library.parent), "-o", binary], check=True)
        result = {
            "ultimate_generated_codes": generated,
            "ultimate_mgba": json.loads(subprocess.check_output([binary, str(ultimate), "ultimate", *generated["lines"]])),
            "old_sudowoodo_mgba": json.loads(subprocess.check_output([binary, str(bw), "old-sudowoodo"])),
            "bw_species_185": species["species"]["name"],
            "scope": "Native instruction / decoder probe; not an ordinary overworld encounter or mobile emulator replay.",
        }
    assert all(p.read_bytes() == data for p, data in originals.items())
    result["source_files_unchanged"] = True
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
