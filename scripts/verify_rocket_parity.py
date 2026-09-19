#!/usr/bin/env python3
"""Compare expanded reader semantics with native ARM routines (private ROM only).

Uses generated RAM, never writes a ROM/save. Pass world/catalog JSON produced by
`gen3 world ROM` and `gen3 catalog ROM`; do not commit those extracted datasets.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_PC, UC_ARM_REG_LR
from verify_rocket_battle_forms import Native, ROM_MD5, BATTLER, BATTLE, PARTY, fixture


def check(rom, world, catalog):
    assert hashlib.md5(rom).hexdigest() == ROM_MD5
    n = Native(rom)
    storage_forms = 0
    for species in catalog['species']:
        sid = species['id']
        pointer = struct.unpack_from('<I', rom, 0x6193ac + sid * 4)[0]
        if not pointer:
            continue
        rows = []
        for i in range(128):
            row = struct.unpack_from('<4H', rom, pointer - 0x8000000 + i * 8)
            if not row[0]:
                break
            rows.append(row)
        for trigger in (1, 3):
            parameters = {0, 1, *(row[2] for row in rows if row[0] == trigger)}
            for parameter in parameters:
                item = parameter if trigger == 1 else 0
                moves = (parameter, 0, 0, 0) if trigger == 3 else (1, 2, 3, 4)
                expected = 0
                for method, target, value, comparison in rows:
                    if method == trigger and (item == value if trigger == 1 else int(value in moves) != comparison):
                        expected = target
                n.reset()
                n.write(PARTY, fixture(sid, item=item, moves=moves))
                actual = n.call(0x9d210, PARTY, trigger)
                if expected == sid:
                    expected = 0  # Native returns zero when no species change is needed.
                assert actual == expected, ('storage form', sid, trigger, parameter, actual, expected)
                storage_forms += 1
    for selector in range(4096):
        ivs = [((selector >> (2 * i)) & 3) + 28 for i in range(6)]
        n.reset()
        attacker = selector % 4
        n.word(BATTLER + attacker * 0x5c + 0x14, sum(iv << (5*i) for i, iv in enumerate(ivs)))
        n.call(0x53c88, 237, attacker)
        actual = n.read(BATTLE + 0x16, 1)[0] & 63
        expected = 1 + sum((iv & 1) << i for i, iv in enumerate(ivs)) * 15 // 63
        expected += expected > 8
        assert actual == expected, (ivs, actual, expected)
        assert n.call(0x6354c, 237, attacker, (attacker + 1) % 4) == 60
    pictures = 0
    for species in catalog['species']:
        sid = species['id']
        for pid in (0, 255):
            for shiny in (False, True):
                n.reset()
                female = rom[0x54cbe0+sid] != 0 and n.call(0x971b4, sid, pid) == 254
                base = (0x55ba44 if shiny else 0x55716c) if female else (0x558ea4 if shiny else 0x5545cc)
                actual = n.call(0x9c31c, sid, pid if shiny else pid ^ 0x1234, pid)
                assert actual == 0x8000000 + base + sid * 8
                pictures += 1
    compatibility = 0
    for species in (1, 25, 52, 201, 487, 649, 940, 1000, 1394):
        pointer = struct.unpack_from('<I', rom, 0x616090 + species*4)[0]
        values = set()
        for i in range(256):
            value = struct.unpack_from('<H', rom, pointer - 0x8000000 + i*2)[0]
            if not value: break
            values.add(value)
        for move in range(1, 755):
            assert bool(n.call(0x9b80c, species, move)) == (move in values)
            compatibility += 1
    dex = 0
    for number in range(1, 956):
        n.reset(); n.word(0x0300524c, 0x02030000)
        n.call(0xf7c60, number, 2)
        expected = bytearray(120); expected[(number-1)//8] = 1 << ((number-1)%8)
        assert n.read(0x02032ee4, 120) == expected
        assert n.read(0x02032f5c, 120) == bytes(120)
        n.call(0xf7c60, number, 3)
        assert n.read(0x02032f5c, 120) == expected
        dex += 1
    # The actual grass/surf/fishing/smash callers use this map selector.
    headers = {}
    for index in range(443):
        pair = tuple(rom[0xbe8a70+index*20:0xbe8a72+index*20])
        headers.setdefault(pair, index)
    selections = 0
    for pair, first in headers.items():
        for value in (0, 1, 8, 9, 65535):
            n.reset(); n.word(0x0300524c, 0x02030000)
            n.write(0x02030004, bytes(pair))
            n.call(0xd39b4, 0x403e, value)
            expected = first + (value if pair == (51,106) and value <= 8 else 0)
            assert n.call(0xec170) == expected, (pair, value, expected)
            selections += 1
    for encounter in world['encounters']:
        selector = encounter.get('selector')
        if selector:
            assert encounter['map_id'] == '51-106' and selector['variable'] == 0x403e
            assert 0 <= selector['value'] <= 8
    trainers = 0
    formats = set()
    for t in world['trainers']:
        flags = rom[0x586a18 + t['id']*40]
        if t['id'] not in (1, 2, 3, 38, 100, 1000, 2558) and flags in formats: continue
        formats.add(flags)
        n.reset(); n.word(0x0300524c, 0x02030000); n.word(0x03005250, 0x02034000)
        n.word(0x02024bb8, 8)
        n.call(0x4d3f0, 0x02028000, t['id'], 0, instruction_limit=20_000_000)
        for i, mon in enumerate(t['party']):
            address = 0x02028000 + i*100
            get = lambda field: n.call(0x976d0, address, field, 0)
            assert (get(11), get(56), get(12)) == (mon['species'], mon['level'], mon['held_item'])
            expected = mon['generation']
            assert [get(26+j) for j in range(6)] == expected['evs'], (t['id'], i, 'EV')
            assert [get(39+j) for j in range(6)] == expected['ivs'], (t['id'], i, 'IV')
            assert n.call(0x9a334, address, 1) == expected['nature'], (t['id'], i, 'nature')
            actual_moves = [get(13+j) for j in range(4)]
            assert actual_moves == (mon['moves'] + [0]*4)[:4], (t['id'], i, actual_moves, mon['moves'])
            species = next(s for s in catalog['species'] if s['id'] == mon['species'])
            ability = species['abilities'][get(46)]
            assert ability in expected['ability_options'], (t['id'], ability, expected['ability_options'])
            trainers += 1
    # Nature-constrained custom teams do not constrain PID gender/ability bits.
    # Probe every Anya row across fresh RNG seeds, including fixed-slot species.
    random_cases = 0
    random_abilities = set()
    random_genders = set()
    for trainer in world['trainers']:
        if trainer['name'] != '安雅':
            continue
        for seed in range(32):
            n.reset()
            n.word(0x03005240, seed * 0x1234567)
            n.word(0x0300524c, 0x02030000)
            n.word(0x03005250, 0x02034000)
            n.word(0x02024bb8, 8)
            n.call(0x4d3f0, 0x02028000, trainer['id'], 0, instruction_limit=20_000_000)
            for i, mon in enumerate(trainer['party']):
                address = 0x02028000 + i * 100
                options = mon['generation']['ability_options']
                assert len(options) == len(set(options)), (trainer['id'], 'duplicate abilities', options)
                ability = n.call(0x988f0, address)
                assert ability in options
                pid = n.call(0x976d0, address, 0, 0)
                gender = n.call(0x971b4, mon['species'], pid)
                if trainer['id'] == 59 and mon['species'] == 214:
                    random_abilities.add(ability)
                    random_genders.add(gender)
                random_cases += 1
    assert random_abilities == {62, 153} and random_genders == {0, 254}
    # Observe loader arguments at the actual call boundary; skip only the VRAM
    # transfer itself. This checks profile constants against executable code.
    transfers = []
    def record_transfer(cpu, address, size, user):
        if address in (0x080be71c, 0x080be808):
            transfers.append((address, *(cpu.reg_read(reg) for reg in (UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2))))
            cpu.reg_write(UC_ARM_REG_PC, cpu.reg_read(UC_ARM_REG_LR))
    hook = n.cpu.hook_add(UC_HOOK_CODE, record_transfer)
    try:
        n.call(0xbe8f0, 0x087c10d0)
        n.call(0xbe918, 0x087c10d0)
    finally:
        n.cpu.hook_del(hook)
    graphics = catalog['profile']['map_graphics']
    assert [(row[2], row[3]) for row in transfers[:2]] == [(graphics['primary_tiles'], 0), (1024 - graphics['primary_tiles'], graphics['primary_tiles'])]
    primary_banks, secondary_banks = catalog['profile']['map_palette_banks']
    assert [(row[2], row[3]) for row in transfers[2:]] == [(0, primary_banks * 32), (primary_banks * 16, secondary_banks * 32)]
    assert graphics['primary_metatiles'] == 640 and graphics['layers'] == 3
    # The native three-background writer must consume all 12 metatile entries.
    n.reset()
    buffers = (0x02010000, 0x02011000, 0x02012000)
    for pointer, buffer in zip((0x03005264, 0x0300525c, 0x03005260), buffers):
        n.word(pointer, buffer)
    n.write(0x02013000, struct.pack('<12H', *range(101, 113)))
    n.call(0xbfbfc, 0, 0x02013000, 0)
    for layer, buffer in enumerate(buffers):
        actual = [struct.unpack('<H', n.read(buffer + offset, 2))[0] for offset in (0, 2, 64, 66)]
        assert actual == list(range(101 + 4 * layer, 105 + 4 * layer)), ('map layer', layer, actual)
    return dict(storage_form_selections=storage_forms, hidden_power=4096, palettes=pictures, compatibility=compatibility,
                dex_flags=dex, wild_header_selections=selections, anya_seed_pokemon=random_cases, map_background_layers=3, trainer_pokemon=trainers, trainer_formats=sorted(formats))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('rom', 'world', 'catalog'): parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(check(args.rom.read_bytes(), json.loads(args.world.read_text()), json.loads(args.catalog.read_text())), indent=2))
