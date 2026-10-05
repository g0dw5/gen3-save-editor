#!/usr/bin/env python3
"""Exact-ROM native holdings checks using synthetic RAM, never ROM/SAV writes.

Requires Unicorn and the five GEN3_ROM_* files. Bag descriptors and encrypted
wallets are injected into RAM; native item-category, decryption, duplicate-slot,
quantity-width, comparison and ignore-operand code executes without bypasses.
Ordinary-bag probes use a zero map header and unset temporary flags. Separate
facility-flag probes show why SAV-only ordinary inventory is insufficient there.
This does not simulate complete map loading, spending, rewards or receipt state.
JSON includes no names/artwork/ROM bytes. Keep private parity output outside Git.
"""
import hashlib
import json
import os
import struct
from pathlib import Path
from verify_pickup_receipts import RULES, CONTEXT, SAVE, TRAINER_SAVE
from verify_rocket_battle_forms import Native

# Verified native addresses and engine semantics, not extracted game content.
CONFIG = {
    'BW': (0x99A6C, 0x9B4C0, 0xD7590, 0x02039DD8, 0x03005D90, 0xAC, 0x490, True),
    'DP': (0x99A6C, 0x9B4C0, 0xD7590, 0x02039DD8, 0x03005D90, 0xAC, 0x490, True),
    'ULTIMATE': (0x99A6C, 0x9B4C0, 0xD7590, 0x02039DD8, 0x03005D90, 0xAC, 0x490, True),
    'ROCKET': (0xCF95C, 0xD1420, 0x10F958, 0x0203ADDC, 0x03005250, 0xAC, 0x490, True),
    'MERCURY12': (0x6A6E4, 0x6C18C, 0x9A9D8, 0x0203988C, 0x0300500C, 0xF20, 0x290, False),
}
SANITIZERS = {"BW": 0xD745C, "DP": 0xD745C, "ROCKET": 0x10F820, "ULTIMATE": 0xD745C, "MERCURY12": 0x9A8A4}
ITEM_LAYOUT = {
    "BW": (0x5839A0,377), "DP": (0x5839A0,377), "ROCKET": (0xC3D558,923),
    "ULTIMATE": (0xFC2C7C,800), "MERCURY12": (0x7C7E00,750),
}
OPERANDS, SLOTS = 0x02008000, 0x02009000
KEY = 0xDEAD4321

def verify(name):
    rom = Path(os.environ['GEN3_ROM_' + name]).read_bytes()
    rules = RULES[name]
    item_code, money_code, pocket_code, descriptors, sb2, key_offset, money_offset, encrypted = CONFIG[name]
    for op, expected in [(0x47, item_code), (0x92, money_code)]:
        assert struct.unpack_from('<I', rom, rules.commands + op * 4)[0] == 0x08000001 + expected
    cpu = Native(rom)
    cpu.word(rules.save_pointer, SAVE)
    cpu.word(sb2, TRAINER_SAVE)
    cpu.word(TRAINER_SAVE + key_offset, KEY)
    # Find a live ordinary-item category through the current native getter.
    table, count = ITEM_LAYOUT[name]
    categories = []
    sanitized = []
    for i in range(count):
        category = cpu.call(pocket_code, i)
        index = cpu.call(SANITIZERS[name], i)
        assert index < count
        assert category == rom[table + index * 44 + 26], (name, i, index, category)
        categories.append(category)
        if i != index: sanitized.append(dict(item=i,index=index,category=category))
    item = next(i for i in range(1, count) if categories[i] == 1)
    descriptor = descriptors
    result = cpu.call(rules.get_variable_pointer, 0x800D)
    param = cpu.call(rules.get_variable_pointer, 0x8005)
    rows = []
    for quantities in [[], [0], [1], [1, 2], [100, 200], [65535, 1]]:
        cpu.write(SLOTS, bytes(32))
        for index, quantity in enumerate(quantities):
            cpu.half(SLOTS + index * 4, item)
            cpu.half(SLOTS + index * 4 + 2, quantity ^ ((KEY & 0xFFFF) if encrypted else 0))
        cpu.word(descriptor, SLOTS)
        cpu.half(descriptor + 4, len(quantities))
        for requested in [0, 1, 2, 3, 255, 256, 257, 65535]:
            cpu.half(param, requested)
            cpu.write(OPERANDS, struct.pack('<HH', item, 0x8005))
            cpu.word(CONTEXT + 8, OPERANDS)
            cpu.half(result, 7)
            cpu.call(item_code, CONTEXT)
            actual = struct.unpack('<H', cpu.read(result, 2))[0]
            effective = requested if name == 'MERCURY12' else requested & 255
            total = (quantities[0] if quantities else 0) if name == 'MERCURY12' else sum(quantities)
            assert actual == int(bool(quantities) and total >= effective), (name, quantities, requested, actual)
            rows.append(dict(kind='bag_item', item=item, quantities=quantities, requested=requested,
                             value=effective, result=actual))
    # Native alternate-bag selection can differ from the ordinary inventory.
    alternate = []
    if name != 'MERCURY12':
        cpu.half(SLOTS, item); cpu.half(SLOTS + 2, 10 ^ (KEY & 0xFFFF))
        cpu.half(descriptor + 4, 1)
        for enabled in [False, True]:
            if enabled: cpu.call(rules.set_flag, 0x4004)
            cpu.write(OPERANDS, struct.pack('<HH', item, 1)); cpu.word(CONTEXT + 8, OPERANDS)
            cpu.call(item_code, CONTEXT)
            alternate.append(dict(flag=0x4004, enabled=enabled,
                                  result=struct.unpack('<H', cpu.read(result, 2))[0]))
        assert [v['result'] for v in alternate] == [1, 0]
    for wallet in [0, 500, 70000, 999999, 0xFFFFFFFF]:
        cpu.word(SAVE + money_offset, wallet ^ KEY)
        for required in [0, 1, 500, 70000, 1000000, 0xFFFFFFFF]:
            for ignore in [0, 1, 255]:
                cpu.half(result, 7)
                cpu.write(OPERANDS, struct.pack('<IB', required, ignore)); cpu.word(CONTEXT + 8, OPERANDS)
                cpu.call(money_code, CONTEXT)
                actual = struct.unpack('<H', cpu.read(result, 2))[0]
                assert actual == (7 if ignore else int(wallet >= required)), (name, wallet, required, ignore, actual)
                rows.append(dict(kind='money', wallet=wallet, value=required, ignore=ignore, result=actual))
    comparisons = []
    compare_code = (struct.unpack_from('<I', rom, rules.commands + 0x21 * 4)[0] - 0x08000001)
    branch_code = (struct.unpack_from('<I', rom, rules.commands + 0x06 * 4)[0] - 0x08000001)
    alias_code = (struct.unpack_from('<I', rom, rules.commands + 0x19 * 4)[0] - 0x08000001)
    for boolean in [0, 1]:
        cpu.half(result, boolean)
        cpu.write(OPERANDS, struct.pack('<HH', 0x8004, 0x800D)); cpu.word(CONTEXT + 8, OPERANDS)
        cpu.call(alias_code, CONTEXT)
        for rhs in [0, 1, 2, 65535]:
            for comparison in range(6):
                cpu.write(OPERANDS, struct.pack('<HH', 0x8004, rhs)); cpu.word(CONTEXT + 8, OPERANDS)
                cpu.call(compare_code, CONTEXT)
                cpu.write(OPERANDS, struct.pack('<BI', comparison, 0x0200A000)); cpu.word(CONTEXT + 8, OPERANDS)
                cpu.call(branch_code, CONTEXT)
                taken = struct.unpack('<I', cpu.read(CONTEXT + 8, 4))[0] == 0x0200A000
                expected = [boolean < rhs, boolean == rhs, boolean > rhs, boolean <= rhs, boolean >= rhs, boolean != rhs][comparison]
                assert taken == expected
                comparisons.append(dict(boolean=boolean, rhs=rhs, comparison=comparison, taken=taken))
    return dict(md5=hashlib.md5(rom).hexdigest(), item=item, vectors=rows,
                comparisons=comparisons, categories=categories, sanitized=sanitized, alternate_bag=alternate)

if __name__ == '__main__':
    print(json.dumps({name: verify(name) for name in CONFIG}, indent=2))
