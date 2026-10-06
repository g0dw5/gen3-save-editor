#!/usr/bin/env python3
"""Native input decisions for referenced crown-service menus in private exact ROMs.

Runs complete task creation and native input callback prefixes in disposable RAM,
without hooks, ROM/SAV writes or window-creation/cleanup claims. Menu parameters
are observed through actual script handlers. Generated evidence stays private.
"""

import argparse
import ctypes
import hashlib
import json
import os
import struct
from pathlib import Path
import verify_breeding as b

# Verified engine addresses/layouts, separate from runtime menu content.
RULES = {
    "MERCURY12": dict(
        commands=0x15F9B4,
        menu_entry=0x0809C9B4,
        menu_table=0x3E04B0,
        create=0x0809CC18,
        callback=0x0809CC98,
        tasks=0x03005090,
        keys=0x0300311E,
        cursor=0x0203ADE4,
        delay=0x02039988,
        result=0x020370D0,
        cancel=0x0809CD12,
        selected=0x0809CD28,
        waiting=0x0809CD3C,
        cancel_written=0x0809CD1E,
        menus=[0x7B003D, 0x7B01FB],
        ignore_b_mask=1,
        transform_start=0x0809CAA6,
        transform_stop=0x0809CAAE,
        transform_local=24,
        transform_reg=1,
    ),
    "ULTIMATE": dict(
        commands=0x1DB67C,
        menu_entry=0x080E1E08,
        menu_table=0x1700000,
        create=0x080E1FBC,
        callback=0x080E2058,
        tasks=0x03005E00,
        keys=0x030022EE,
        cursor=0x0203CD90,
        delay=0x02039F90,
        result=0x020375F0,
        cancel=0x080E20D2,
        selected=0x080E20E8,
        waiting=0x080E20FC,
        cancel_written=0x080E20DE,
        menus=[0x181279A],
        ignore_b_mask=255,
        transform_start=0x080E1F9A,
        transform_stop=0x080E1FA2,
        transform_local=0,
        transform_reg=0,
    ),
}


def observe(cpu, start, args, stops):
    regs = (ctypes.c_uint * 16)()
    assert len(args) <= 4 and 1 <= len(stops) <= 2
    result = cpu.c.callchoice(
        start,
        *(list(args) + [0] * (4 - len(args))),
        stops[0],
        stops[1] if len(stops) == 2 else 0,
        regs
    )
    assert 1 <= result <= len(stops), (hex(start), result)
    return list(regs), stops[result - 1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mgba-probe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    b.MGBA_PROBE = args.mgba_probe.resolve()
    output = {}
    for key, r in RULES.items():
        b.ROM_PATH = Path(os.environ["GEN3_ROM_" + key])
        rom = b.ROM_PATH.read_bytes()
        digest = hashlib.sha256(rom).hexdigest()
        assert hashlib.md5(rom).hexdigest() == b.MD5[key]
        entries = []
        rows = []
        for at in r["menus"]:
            assert rom[at] == 0x6F
            entry = struct.unpack_from("<I", rom, r["commands"] + 0x6F * 4)[0] & ~1
            cpu = b.MGBANative(rom)
            cpu.word(0x02010008, 0x08000000 + at + 1)
            regs, _ = observe(cpu, entry, [0x02010000], [r["menu_entry"]])
            params = regs[:4]
            assert params == list(rom[at + 1 : at + 5])
            count = rom[r["menu_table"] + params[2] * 8 + 4]
            assert 0 < count <= 7
            entries.append(dict(command=at, params=params, count=count))
            # Explicit cursor/keypress contexts, not a claim of rendered windows.
            for cursor in range(count):
                for keys in [0, 2, 1, 3]:  # none, B, A, A+B
                    cpu = b.MGBANative(rom)
                    cpu.call(
                        r["create"] - 0x08000000,
                        params[3] & r["ignore_b_mask"],
                        count,
                        0,
                        params[2],
                    )
                    tasks = cpu.read(r["tasks"], 640)
                    active = [i for i in range(16) if tasks[i * 40 + 4]]
                    assert active == [0], active
                    assert struct.unpack_from("<I", tasks, 0)[0] == r["callback"] | 1
                    assert (
                        struct.unpack_from("<H", tasks, 16)[0]
                        == params[3] & r["ignore_b_mask"]
                    )
                    # Run natural no-input task ticks until its startup delay ends.
                    ticks = 0
                    while cpu.read(r["delay"], 1) != bytes(1):
                        cpu.call(r["callback"] - 0x08000000, 0)
                        ticks += 1
                        assert ticks < 20
                    cpu.write(r["cursor"] + 2, bytes([cursor]))
                    cpu.write(r["cursor"] + 11, b"\x01")  # native silent-selection flag
                    cpu.half(r["keys"], keys)
                    cpu.half(r["result"], 255)
                    kind = (
                        "selected"
                        if keys & 1
                        else (
                            "cancel"
                            if keys & 2 and not params[3] & r["ignore_b_mask"]
                            else "waiting"
                        )
                    )
                    stop = r[kind]
                    regs, observed = observe(cpu, r["callback"], [0], [stop])
                    assert observed == stop
                    assert regs[1] == cursor if kind == "selected" else True
                    assert int.from_bytes(cpu.read(r["result"], 2), "little") == 255
                    # Separate native cancellation-result prefix includes sound,
                    # but stops before window destruction and script restart.
                    cancel_result = None
                    if kind == "cancel":
                        _, _ = observe(cpu, r["callback"], [0], [r["cancel_written"]])
                        cancel_result = int.from_bytes(
                            cpu.read(r["result"], 2), "little"
                        )
                        assert cancel_result == 127
                    rows.append(
                        dict(
                            command=at,
                            params=params,
                            count=count,
                            cursor=cursor,
                            keys=keys,
                            kind=kind,
                            stop=stop,
                            register1=regs[1],
                            startup_ticks=ticks,
                            cancel_result=cancel_result,
                        )
                    )
        cancel_paths = []
        for entry in entries:
            if entry["params"][3] & r["ignore_b_mask"]:
                continue
            # Start from the independently observed native B result. Execute
            # actual compare/goto handlers up to the exit dialogue, without
            # replacing branch semantics or claiming dialogue/UI completion.
            cpu = b.MGBANative(rom)
            cpu.half(r["result"], 127)
            at = entry["command"] + 5
            trace = []
            while rom[at] in [0x21, 0x05, 0x06]:
                assert len(trace) < 20
                op = rom[at]
                native = struct.unpack_from("<I", rom, r["commands"] + op * 4)[0]
                cpu.word(0x02010008, 0x08000000 + at + 1)
                cpu.call((native & ~1) - 0x08000000, 0x02010000)
                after = int.from_bytes(cpu.read(0x02010008, 4), "little") - 0x08000000
                trace.append(dict(command=at, opcode=op, next=after))
                at = after
            assert rom[at] == 0x0F
            cancel_paths.append(dict(menu=entry["command"], trace=trace, dialogue=at))
        transforms = []
        cpu = b.MGBANative(rom)
        for flags in range(256):
            # Observe the native flag-transfer instructions in their explicit
            # stack-local context; this slice does not create/render a window.
            cpu.word(0x03007E00 + r["transform_local"], flags)
            regs, _ = observe(cpu, r["transform_start"], [0] * 4, [r["transform_stop"]])
            actual = regs[r["transform_reg"]]
            assert actual == flags & r["ignore_b_mask"]
            transforms.append(dict(flags=flags, actual=actual))
        assert hashlib.sha256(b.ROM_PATH.read_bytes()).hexdigest() == digest
        output[key] = dict(
            md5=b.MD5[key],
            rules=r,
            entries=entries,
            rows=rows,
            cancel_paths=cancel_paths,
            transforms=transforms,
            scope="Native task creation and input prefixes; no rendered windows, party selector, transaction or cleanup replay",
        )
        print(
            key,
            len(entries),
            "menu parameter observations;",
            len(rows),
            "input decisions",
            flush=True,
        )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + "\n")
    b.MGBANative.c.finish()


if __name__ == "__main__":
    main()
