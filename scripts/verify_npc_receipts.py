#!/usr/bin/env python3
"""Native NPC receipt parity for runtime-qualified reward scripts.

Requires Unicorn and five GEN3_ROM_* inputs; optionally GEN3_CLI. No SAV is
opened, no input is written. JSON vectors go to stdout and must remain private.
Bag insertion is injected as success/failure. Native command dispatch, calls,
comparisons, standard-script selection and flag writes execute the loaded ROM.
Presentation/waits and documented presentation specials are bypassed: this is
receipt-protocol verification, not a complete NPC interaction simulation.
"""
import hashlib
import json
import os
import struct
import subprocess
import sys
from pathlib import Path

from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_LR, UC_ARM_REG_PC, UC_ARM_REG_R0
from verify_pickup_receipts import RULES, EXECUTE, BYPASS, CONTEXT, SAVE, TRAINER_SAVE
from verify_event_state import PROFILES
from verify_rocket_battle_forms import Native

ADD_BAG = {'BW': 0xD6928, 'DP': 0xD6928, 'ULTIMATE': 0xD6928,
           'ROCKET': 0x10ED14, 'MERCURY12': 0x9A084}
# Mercury's standard gift formats text and creates/destroys item icon sprites.
# These helpers do not determine VAR_RESULT or the caller's receipt flag.
MERCURY_PRESENTATION = {0x9A824, 0x1D5DEB0, 0x1D5DEE0}
PRESENTATION = BYPASS | {0xD4: 6, 0x6E: 3}
NATIVE_OPS = EXECUTE | {0x44, 0x17, 0x18, 0x00, 0x01}


def desired(c):
    def compare(a):
        b = c['value']
        return [a < b, a == b, a > b, a <= b, a >= b, a != b][c['comparison']] == c['taken']
    for v in [0, c['value'], min(c['value'] + 1, 65535), max(c['value'] - 1, 0)]:
        if compare(v):
            return v
    raise AssertionError(c)


def verify(key, rom, world):
    rules = RULES[key]
    cpu = Native(rom)
    fits, sets = False, []
    def hook(uc, address, _size, _data):
        if address == 0x08000000 + rules.set_flag:
            sets.append(uc.reg_read(UC_ARM_REG_R0))
        if address == 0x08000000 + ADD_BAG[key]:
            uc.reg_write(UC_ARM_REG_R0, int(fits))
            uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))
        if key == 'MERCURY12' and address - 0x08000000 in MERCURY_PRESENTATION:
            uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))
    cpu.cpu.hook_add(UC_HOOK_CODE, hook)
    rows, cases = [], 0
    for report in world['map_events']:
        for marker in report['markers']:
            for reward in marker['rewards']:
                evidence = reward.get('receipt')
                if not evidence:
                    continue
                flag = evidence['flag']
                outcomes = []
                for already in [False, True]:
                    for fits in [False, True]:
                        sets.clear()
                        cpu.write(0x02000000, bytes(0x40000))
                        cpu.write(0x03000000, bytes(0x8000))
                        cpu.word(rules.save_pointer, SAVE)
                        cpu.word(0x03005D90, TRAINER_SAVE)
                        for guard in reward['conditions']:
                            value = desired(guard)
                            if guard['kind'] == 'flag':
                                if value:
                                    cpu.call(rules.set_flag, guard['id'])
                            elif guard['kind'] == 'variable':
                                cpu.half(cpu.call(rules.get_variable_pointer, guard['id']), value)
                        if already:
                            cpu.call(rules.set_flag, flag)
                        sets.clear()
                        cpu.word(CONTEXT + 8, 0x08000000 + marker['script'])
                        trace, awarded = [], False
                        for _ in range(600):
                            pc = struct.unpack('<I', cpu.read(CONTEXT + 8, 4))[0]
                            if pc == 0:
                                break
                            op = cpu.read(pc, 1)[0]
                            trace.append((pc, op))
                            if op == 2:
                                break
                            if pc == 0x08000000 + reward['offset']:
                                awarded = True
                            cpu.word(CONTEXT + 8, pc + 1)
                            if op in NATIVE_OPS or (key == 'MERCURY12' and op == 0xC7):
                                entry = struct.unpack_from('<I', rom, rules.commands + 4 * op)[0]
                                cpu.call(entry - 0x08000001, CONTEXT)
                            elif op in PRESENTATION:
                                cpu.word(CONTEXT + 8, pc + PRESENTATION[op])
                                if op == 0x26:
                                    variable = struct.unpack('<H', cpu.read(pc + 1, 2))[0]
                                    cpu.half(cpu.call(rules.get_variable_pointer, variable), 0)
                                elif op == 0x6E:
                                    cpu.half(cpu.call(rules.get_variable_pointer, 0x800D), 1)
                            else:
                                raise AssertionError((key, report['map_id'], hex(pc), hex(op), trace))
                        else:
                            raise AssertionError(('bounded native execution', key, trace))
                        got = cpu.call(PROFILES[key][1], flag)
                        assert got == int(already or (fits and awarded)), (key, report['map_id'], reward, already, fits, got, trace)
                        assert (flag in sets) == (not already and fits and awarded), (key, flag, fits, sets)
                        if already:
                            assert not awarded, (key, flag, trace)
                        outcomes.append(awarded)
                        cases += 1
                assert outcomes[:2] == [True, True], (key, report['map_id'], reward, outcomes)
                rows.append({'map_id': report['map_id'], 'marker_offset': marker['offset'],
                             'award_offset': reward['offset'], 'receipt_flag': flag})
    assert rows, key
    assert cpu.read(0x08000000, len(rom)) == rom
    return {'md5': hashlib.md5(rom).hexdigest(), 'cases': cases, 'rows': rows,
            'rom_memory_unchanged': True,
            'scope': 'native receipt control flow; injected bag outcomes; presentation bypassed'}


def main():
    cli = os.environ.get('GEN3_CLI', 'target/debug/gen3')
    results = {}
    for key in PROFILES:
        path = Path(os.environ['GEN3_ROM_' + key])
        rom = path.read_bytes()
        digest = hashlib.sha256(rom).digest()
        assert hashlib.md5(rom).hexdigest() == PROFILES[key][0]
        world = json.loads(subprocess.check_output([cli, 'world', str(path)]))
        results[key] = verify(key, rom, world)
        assert hashlib.sha256(path.read_bytes()).digest() == digest
        print(key, results[key]['cases'], 'native NPC receipt cases; input unchanged', file=sys.stderr)
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
