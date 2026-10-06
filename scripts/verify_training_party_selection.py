#!/usr/bin/env python3
"""Native party-only crown selection and script return in private exact ROMs.

Observes actual launchers up to InitPartyMenu and native input callbacks up to
close-menu entry, plus Ultimate's return prefix and script branches. No window
rendering, cursor reachability, complete transaction, ROM/SAV writes or hooks.
Synthetic egg/HP scenarios test selection alone, not training eligibility.
"""

import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
import verify_breeding as b
import verify_training_items as ev
from verify_training_service_menus import observe

RULES = {
    "MERCURY12": dict(
        commands=0x15F9B4,
        specials=0x15FD60,
        selection_commands=[0x7B00B7, 0x7B00E0],
        cancel_checks=[0x7B07A0, 0x7B098B],
        entry=0x080BF8FC,
        launch=0x080BF97C,
        tasks=0x03005090,
        fade=0x02037AB8,
        init=0x0811EA44,
        init_prefix=0x081283A8,
        init_argument=3,
        menu=0x0203B0A0,
        input=0x0811FB28,
        close=0x0811FA78,
        waiting=0x0811FB98,
        keys=0x0300311E,
        selected=0x020370C0,
        buffer=None,
        buffer_stop=None,
    ),
    "ULTIMATE": dict(
        commands=0x1DB67C,
        specials=0x1DBA64,
        selection_commands=[0x1812776],
        cancel_checks=[0x181277A],
        entry=0x081B94B0,
        launch=0x081B94D0,
        tasks=0x03005E00,
        fade=0x02037FD4,
        init=0x081B0038,
        init_prefix=0x081B94EC,
        init_argument=0,
        menu=0x0203CEC8,
        input=0x081B1370,
        close=0x081B12C0,
        waiting=0x081B13E0,
        keys=0x030022EE,
        selected=0x020375E0,
        buffer=0x081B9390,
        buffer_stop=0x081B93A6,
    ),
}


def route(cpu, rom, r, start, value):
    cpu.half(r["selected"], value)
    at = start
    trace = []
    while rom[at] in [0x21, 0x05, 0x06]:
        assert len(trace) < 20
        op = rom[at]
        handler = struct.unpack_from("<I", rom, r["commands"] + op * 4)[0] & ~1
        cpu.word(0x02010008, 0x08000000 + at + 1)
        cpu.call(handler - 0x08000000, 0x02010000)
        after = int.from_bytes(cpu.read(0x02010008, 4), "little") - 0x08000000
        trace.append(dict(command=at, opcode=op, next=after))
        at = after
    return dict(start=start, value=value, trace=trace, destination=at)


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
        for at in r["selection_commands"]:
            assert rom[at] == 0x25
            special = struct.unpack_from("<H", rom, at + 1)[0]
            target = struct.unpack_from("<I", rom, r["specials"] + special * 4)[0] & ~1
            assert target == r["entry"]
            cpu = b.MGBANative(rom)
            handler = struct.unpack_from("<I", rom, r["commands"] + 0x25 * 4)[0] & ~1
            cpu.word(0x02010008, 0x08000000 + at + 1)
            observe(cpu, handler, [0x02010000], [target])
            cpu.call(target - 0x08000000)
            assert cpu.read(r["tasks"] + 4, 1) == b"\x01"
            assert int.from_bytes(cpu.read(r["tasks"], 4), "little") == r["launch"] | 1
            # Fade completion is explicit context; no rendered-screen claim.
            cpu.write(r["fade"] + 7, bytes(1))
            regs, _ = observe(cpu, r["launch"], [0], [r["init"]])
            assert regs[:4] == [3, 0, 11, 0]
            callbacks = list(struct.unpack("<3I", cpu.read(regs[13], 12)))
            assert callbacks[0] == 0 and callbacks[1] == r["input"] | 1
            if r["buffer"]:
                assert callbacks[2] == r["buffer"] | 1
            entries.append(
                dict(
                    command=at,
                    special=special,
                    native=target,
                    init_args=regs[:4],
                    callbacks=callbacks,
                )
            )
        rows = []
        for slot in range(6):
            for egg in [False, True]:
                for hp, status in [(0, 0), (20, 0), (20, 8)]:
                    for keys in [0, 1, 2, 3]:
                        cpu = b.MGBANative(rom)
                        cpu.word(b.CONFIG[key][3], 0x02030000)
                        cpu.word(b.CONFIG[key][4], 0x02034000)
                        raw = bytearray(ev.fixture(key, [0] * 6, 70))
                        raw[84] = 50 if key == "MERCURY12" else 100
                        struct.pack_into("<IH", raw, 80, status, hp)
                        bits = struct.unpack_from("<I", raw, 72)[0]
                        struct.pack_into("<I", raw, 72, bits | (int(egg) << 30))
                        struct.pack_into(
                            "<H",
                            raw,
                            28,
                            sum(struct.unpack_from("<24H", raw, 32)) & 65535,
                        )
                        party = bytes(raw) * 6
                        cpu.write(b.CONFIG[key][6], party)
                        cpu.write(b.CONFIG[key][7], bytes([6]))
                        cpu.write(r["menu"] + 8, bytes([3, slot, 0, 11]))
                        cpu.half(r["selected"], 88)
                        cpu.half(r["keys"], keys)
                        decision = (
                            "selected"
                            if keys == 1
                            else "cancelled" if keys == 2 else "waiting"
                        )
                        stop = r["waiting"] if decision == "waiting" else r["close"]
                        observe(cpu, r["input"], [0], [stop])
                        selection = int.from_bytes(cpu.read(r["selected"], 2), "little")
                        menu_slot = cpu.read(r["menu"] + 9, 1)[0]
                        assert menu_slot == (7 if decision == "cancelled" else slot)
                        assert selection == (
                            7
                            if decision == "cancelled"
                            else (
                                slot
                                if decision == "selected" and not r["buffer"]
                                else 88
                            )
                        )
                        result = selection
                        if r["buffer"] and decision != "waiting":
                            observe(cpu, r["buffer"], [], [r["buffer_stop"]])
                            result = int.from_bytes(
                                cpu.read(r["selected"], 2), "little"
                            )
                            assert result == (255 if decision == "cancelled" else slot)
                        assert cpu.read(b.CONFIG[key][6], 600) == party
                        rows.append(
                            dict(
                                slot=slot,
                                egg=egg,
                                hp=hp,
                                status=status,
                                keys=keys,
                                decision=decision,
                                stop=stop,
                                raw=list(raw),
                                menu_slot=menu_slot,
                                selection=selection,
                                result=result,
                            )
                        )
        normalized = []
        if r["buffer"]:
            cpu = b.MGBANative(rom)
            for slot in range(256):
                cpu.write(r["menu"] + 9, bytes([slot]))
                observe(cpu, r["buffer"], [], [r["buffer_stop"]])
                result = int.from_bytes(cpu.read(r["selected"], 2), "little")
                assert result == (slot if slot <= 5 else 255)
                normalized.append(dict(slot=slot, result=result))
        paths = []
        for at in r["cancel_checks"]:
            for result in list(range(6)) + [7 if not r["buffer"] else 255]:
                cpu = b.MGBANative(rom)
                paths.append(route(cpu, rom, r, at, result))
        assert hashlib.sha256(b.ROM_PATH.read_bytes()).hexdigest() == digest
        output[key] = dict(
            md5=b.MD5[key],
            rules=r,
            entries=entries,
            rows=rows,
            normalized=normalized,
            paths=paths,
        )
        print(
            f"{key}: {len(entries)} native launchers, {len(rows)} input/state cases, {len(normalized)} return prefixes, {len(paths)} script routes; ROM unchanged",
            flush=True,
        )
    args.output.write_text(json.dumps(output, indent=2) + "\n")
    b.MGBANative.c.finish()


if __name__ == "__main__":
    main()
