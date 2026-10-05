#!/usr/bin/env python3
"""Read-only native tutor selector/menu parity for five exact ROMs.

Synthetic RAM only. JSON stdout is private evidence, not release content.
Menu setup is intercepted before graphics/UI work; this does not execute teaching,
payment, party selection, one-time receipt or current-map access.
"""
import hashlib
import json
import os
import struct
import sys
from pathlib import Path
from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_PC, UC_ARM_REG_LR, UC_ARM_REG_R0, UC_ARM_REG_R2, UC_ARM_REG_SP
from verify_event_state import PROFILES
from verify_pickup_receipts import RULES
from verify_rocket_battle_forms import Native, fixture

# Native entry/layout metadata only, no names or move catalogs.
CONFIG = {
    'BW': (0x1DBA64, 0x1DD, 0x1B2360, 32, 0x1B0038),
    'DP': (0x1DBA64, 0x1DD, 0x1B2360, 32, 0x1B0038),
    'ULTIMATE': (0x1DBA64, 0x1DD, 0x1B2360, 127, 0x1B0038),
    'ROCKET': (0x22B620, 0x1DD, None, 0, 0x1FC6E4),
    'MERCURY12': (0x15FD60, 0x18D, 0x120BA8, 154, 0x11EA44),
}


def verify(key, rom):
    assert hashlib.md5(rom).hexdigest() == PROFILES[key][0]
    table, special, getter, count, menu = CONFIG[key]
    entry = (struct.unpack_from('<I', rom, table + special * 4)[0] & ~1) - 0x08000000
    native = Native(rom)
    rows = [dict(parameter=i, move_id=native.call(getter, i)) for i in range(count)] if getter else []
    menu_cases = []
    for parameter in ([0, count - 1, count, 256] if getter else [1, 85, 600]):
        address = native.call(RULES[key].get_variable_pointer, 0x8005)
        native.half(address, parameter)
        actions = []

        def hook(cpu, pc, size, user_data):
            if pc == 0x08000000 + menu:
                actions.append((cpu.reg_read(UC_ARM_REG_R2), struct.unpack("<I", native.read(cpu.reg_read(UC_ARM_REG_SP), 4))[0]))
                cpu.reg_write(UC_ARM_REG_R0, 0)
                cpu.reg_write(UC_ARM_REG_PC, cpu.reg_read(UC_ARM_REG_LR))

        handle = native.cpu.hook_add(UC_HOOK_CODE, hook)
        native.call(entry)
        native.cpu.hook_del(handle)
        assert actions, (key, parameter)
        assert actions[0][0] == 12
        # Mercury changes the party-selection callback/message, not tutor action.
        expected_message = 127 if key == 'MERCURY12' and parameter >= count else 4
        assert actions[0][1] == expected_message, (key, parameter, actions)
        menu_cases.append(dict(parameter=parameter, action=actions[0][0], message=actions[0][1]))
    direct = []
    if key == 'ROCKET':
        # Native compatibility consumes the move ID itself, not a stock tutor index.
        # Both compatibility and knows-move routines execute; raw record is synthetic.
        native.write(0x02020000, fixture(1))
        pointer = struct.unpack_from('<I', rom, 0x616090 + 4)[0] - 0x08000000
        moves = []
        for i in range(256):
            move = struct.unpack_from('<H', rom, pointer + i * 2)[0]
            if not move: break
            moves.append(move)
        for move in moves:
            result = native.call(0x1FED28, 0x02020000, move)
            assert result in (0, 2), (move, result)
            direct.append(dict(parameter=move, result=result))
    assert native.read(0x08000000, len(rom)) == rom
    return dict(md5=PROFILES[key][0], entry=entry, rows=rows, menu_cases=menu_cases, direct_compatibility=direct)


def main():
    results = {}
    for key in CONFIG:
        path = Path(os.environ['GEN3_ROM_' + key]); rom = path.read_bytes()
        results[key] = verify(key, rom)
        assert hashlib.md5(path.read_bytes()).hexdigest() == PROFILES[key][0]
        print(f'{key}: {len(results[key]["rows"])} native lookups, menu selector and ROM immutability verified', file=sys.stderr)
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
