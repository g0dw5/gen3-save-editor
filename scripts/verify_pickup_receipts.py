#!/usr/bin/env python3
"""Native ordinary item-ball receipt probes for five exact ROM fingerprints.

Set GEN3_ROM_* and optionally GEN3_CLI (default target/debug/gen3). Requires
Unicorn. All writes affect synthetic emulated RAM only. JSON parity vectors go to
stdout; keep them private. No SAV is opened and no input file is written.

Native handlers execute variable assignment, comparison, call/return, standard
script selection, object/template lookup and FlagSet/Get. Bag-space outcomes are
injected. Presentation, waits and non-receipt specials are bypassed; success
stops immediately after native object removal, before bag insertion/bookkeeping.
This proves the receipt protocol, not a complete field interaction simulation.
"""

import hashlib
import json
import os
import re
import struct
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_LR, UC_ARM_REG_PC, UC_ARM_REG_R0

from verify_event_state import PROFILES
from verify_rocket_battle_forms import Native


@dataclass(frozen=True)
class Rules:
    commands: int
    get_variable_pointer: int
    set_flag: int
    save_pointer: int
    map_header: int
    objects: int
    saved_templates: int
    bag_space: int
    despawn: int


EMERALD = Rules(0x1DB67C, 0x9D648, 0x9D740, 0x03005D8C, 0x02037318,
                0x02037350, 0xC70, 0xD6800, 0x8D8B0)
RULES = {
    'BW': EMERALD,
    'DP': EMERALD,
    'ULTIMATE': EMERALD,
    'ROCKET': Rules(0x22B218, 0xD3930, 0xD3A6C, 0x0300524C, 0x020382D8,
                    0x02038314, 0x16A8, 0x10EBF0, 0xC392C),
    'MERCURY133': Rules(0x15F9B4, 0x6E454, 0x6E680, 0x03005008, 0x02036DFC,
                       0x02036E38, 0x8E0, 0x9A000, 0x5E4B4),
}
# These only advance the script pointer over presentation/housekeeping operands.
BYPASS = {0x6A: 1, 0x5A: 1, 0x30: 1, 0x31: 3, 0x32: 1, 0x80: 4,
          0x67: 5, 0x0F: 6, 0x25: 3, 0x26: 5, 0x66: 1, 0x6C: 1,
          0x6D: 1, 0xC7: 2, 0xE2: 6, 0x84: 4}
EXECUTE = {3, 4, 5, 6, 7, 8, 9, 0x16, 0x19, 0x1A, 0x21, 0x22,
           0x28, 0x29, 0x2A, 0x2B, 0x46, 0x48, 0x53}
CONTEXT, SAVE, TRAINER_SAVE = 0x02010000, 0x02020000, 0x02030000


def ordinary(rom, root):
    # Independent candidate selection; protocol assertions below execute ROM code.
    return re.fullmatch(rb'[\x16\x1a]\x00\x80..[\x16\x1a]\x01\x80..\x09\x01\x02',
                        rom[root:root + 13], re.DOTALL) is not None and all(
        0 < struct.unpack_from('<H', rom, root + offset)[0] < 0x4000
        for offset in [3, 8])


def verify(key, rom, world):
    rules = RULES[key]
    cpu = Native(rom)
    fits = False
    sets = []

    def hook(uc, address, _size, _data):
        if address == 0x08000000 + rules.set_flag:
            sets.append(uc.reg_read(UC_ARM_REG_R0))
        if address in [0x08000000 + rules.bag_space, 0x08000000 + rules.despawn]:
            uc.reg_write(UC_ARM_REG_R0, int(fits))
            uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))

    cpu.cpu.hook_add(UC_HOOK_CODE, hook)
    reports = {report['map_id']: report for report in world['map_events']}
    rows = []
    for map in world['maps']:
        for marker in reports[map['id']]['markers']:
            root = marker['script']
            if (marker['local_id'] is None or marker['kind'] != 'pickup'
                    or not marker['flag'] or root is None or not ordinary(rom, root)):
                continue
            for fits in [False, True]:
                sets.clear()
                cpu.write(0x02000000, bytes(0x40000))
                cpu.write(0x03000000, bytes(0x8000))
                cpu.word(rules.save_pointer, SAVE)
                cpu.word(0x03005D90, TRAINER_SAVE)
                cpu.write(rules.map_header, rom[map['header']:map['header'] + 28])
                cpu.write(SAVE + 4, bytes([map['group'], map['number']]))
                events = map['events']
                count = rom[events]
                templates = struct.unpack_from('<I', rom, events + 4)[0] - 0x08000000
                cpu.write(SAVE + rules.saved_templates, rom[templates:templates + count * 24])
                actor = bytearray(36)
                actor[0] = 1
                actor[8:11] = bytes([marker['local_id'], map['number'], map['group']])
                cpu.write(rules.objects, actor)
                cpu.half(cpu.call(rules.get_variable_pointer, 0x800F), marker['local_id'])
                cpu.word(CONTEXT + 8, 0x08000000 + root)
                flag = marker['flag']
                assert cpu.call(PROFILES[key][1], flag) == 0
                trace = []
                for _ in range(300):
                    pc = struct.unpack('<I', cpu.read(CONTEXT + 8, 4))[0]
                    opcode = cpu.read(pc, 1)[0]
                    trace.append((pc, opcode))
                    if opcode == 2:
                        break
                    cpu.word(CONTEXT + 8, pc + 1)
                    if opcode in EXECUTE or (key == 'MERCURY133' and opcode in [0x25, 0xC7]):
                        entry = struct.unpack_from('<I', rom, rules.commands + 4 * opcode)[0]
                        cpu.call(entry - 0x08000001, CONTEXT)
                    elif opcode in BYPASS:
                        cpu.word(CONTEXT + 8, pc + BYPASS[opcode])
                        if opcode == 0x26:
                            variable = struct.unpack('<H', cpu.read(pc + 1, 2))[0]
                            cpu.half(cpu.call(rules.get_variable_pointer, variable), 0)
                    else:
                        raise AssertionError((key, map['id'], hex(pc), hex(opcode), trace))
                    if fits and opcode == 0x53:
                        break
                else:
                    raise AssertionError(('script instruction bound', key, map['id'], trace))
                assert sets == ([flag] if fits else []), (key, map['id'], flag, fits, sets)
                assert cpu.call(PROFILES[key][1], flag) == int(fits)
                if fits:
                    next_pc = struct.unpack('<I', cpu.read(CONTEXT + 8, 4))[0]
                    # Mercury inserts temporary-variable/text housekeeping before additem.
                    while cpu.read(next_pc, 1)[0] in [0x28, 0x25]:
                        next_pc += 3
                    assert cpu.read(next_pc, 1) == b'\x44'
                    for variable, offset in [(0x8004, 3), (0x8005, 8)]:
                        actual = cpu.read(cpu.call(rules.get_variable_pointer, variable), 2)
                        assert actual == rom[root + offset:root + offset + 2]
            rows.append({'map_id': map['id'], 'marker_offset': marker['offset'],
                         'receipt_flag': flag})
    assert cpu.read(0x08000000, len(rom)) == rom
    return {'md5': hashlib.md5(rom).hexdigest(), 'cases': len(rows) * 2,
            'rows': rows, 'rom_memory_unchanged': True,
            'scope': 'synthetic bag-space outcomes, native receipt handlers; presentation bypassed'}


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
        print(key, results[key]['cases'], 'native receipt cases; input unchanged', file=sys.stderr)
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
