#!/usr/bin/env python3
"""Opt-in Mercury 1.33 common cheat tests using final codes and native mGBA calls.

Requires GEN3_BIN, GEN3_ROM_MERCURY133 and --probe compiled from the public
native/storage_cheat_probe.c harness. Default routine tests do not load a SAV. Optional PC tests read a supplied
battery/state fixture into memory only; exports go to a local test directory.
Battle tests live in the shared cross-ROM runners.
"""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import tempfile
from verify_common_cheats_mgba import Probe, mon, unpack, BREED, HATCH, CATCH, NO_ENCOUNTERS

ENCOUNTER = 'specified-wild-encounter'
SHINY = 'shiny-wild-encounters'
TELEPORT = 'teleport-to-map'
PC = 'portable-pokemon-storage'
EMERGENCY = 'emergency-battle-heal'


class MercuryProbe(Probe):
    def __init__(self, library, path, temp):
        self.c = ctypes.CDLL(str(library.resolve()))
        self.c.readmem.restype = self.c.callfunc.restype = ctypes.c_uint
        self.p = dict(md5='5ffb1cbd5c28cda9b987b3b445da68e0', party=0x2024284,
                      count=0x2024029, enemy=0x202402c, sb1p=0x3005008,
                      sb2p=0x300500c, rng=0x3005000, day=0x2f80,
                      hatch=0x463b8, compat=0x4654c, get=0x3fbe8, egg_width=2,
                      egg_flag=0x266, flag_get=0x6e6d0, flag_set=0x6e680)
        self.rocket = False
        self.ultimate = False
        self.path, self.temp = path, temp
        self.data = path.read_bytes()
        assert hashlib.md5(self.data).hexdigest() == self.p['md5']
        self.catalog = json.loads(subprocess.check_output([os.environ['GEN3_BIN'], 'cheats', str(path)]))
        self.reference = json.loads(subprocess.check_output([os.environ['GEN3_BIN'], 'catalog', str(path)]))
        self.codes = {}
        assert len(self.catalog['entries']) == 10
        assert self.c.start(str(path).encode())

    def lines(self, key, parameters=None):
        cache = json.dumps([key, parameters])
        if cache not in self.codes:
            args = [os.environ['GEN3_BIN'], 'cheat-code', str(self.path), key, 'gameshark_v1_v2']
            if parameters:
                p = self.temp / 'parameters.json'
                p.write_text(json.dumps(parameters))
                args.append(str(p))
            self.codes[cache] = json.loads(subprocess.check_output(args))['lines']
        return self.codes[cache]

    def enable(self, key, parameters=None):
        result = self.c.addgroup('\n'.join(self.lines(key, parameters)).encode())
        assert result >= 0
        return [result]

    def init(self):
        self.c.clearcodes()
        self.c.reset()
        self.write(0x2000000, bytes(0x40000))
        self.write(0x3000000, bytes(0x8000))
        self.put(self.p['sb1p'], 0x202a000)
        self.put(self.p['sb2p'], 0x2030000)
        self.put(self.p['rng'], 42)
        self.put(0x203000a, 0x12345678)

    def mon(self, rocket=False, **kwargs):
        raw = mon(False, **kwargs)
        return raw[:32] + unpack(raw) + raw[80:]

    def unpack(self, raw): return raw[32:80]
    def pack(self, raw, q): return raw[:32] + bytes(q) + raw[80:]

    def charm(self):
        item = 0x1d6  # native helper 1D1C7FC compares this item
        pocket = self.call(0x9a9d8, item)
        self.put(0x203988c + (pocket - 1)*8, 0x2037000)
        self.put(0x203988c + (pocket - 1)*8 + 4, 1, 2)
        self.put(0x2037000, item, 2)
        self.put(0x2037002, 1, 2)
        assert self.call(0x99f40, item, 1) == 1

    def decoder(self):
        self.init()
        original = self.read(0x8000000, len(self.data))
        reports = {}
        maps = [m for m in self.catalog['options']['maps'] if m['landings']]
        params = {ENCOUNTER: dict(kind='encounter',species=185,level=17),
                  TELEPORT: dict(kind='teleport', map_id=maps[0]['id'], warp_id=maps[0]['landings'][0]['id'])}
        spans = {NO_ENCOUNTERS: [(0x1d6cd7e,2)], CATCH: [(0x1d0f2da,2)],
                 HATCH: [(0x46346,2)], BREED: [(0x4632c,2)],
                 PC: [(0x1d522ce,2),(0x1d523ac,4),(0x13fd200,8)],
                 ENCOUNTER: [(0x1d6ba80,8),(0x13fd280,24)],
                 SHINY: [(0x3ddbc,14),(0x3de44,14),(0x3deaa,14),(0x13fd000,72),(0x13fd080,72),(0x13fd100,72)],
                 TELEPORT: [(0x553b2,6)], EMERGENCY: [(0x12424,4),(0x13fd400,328)]}
        for key, allowed in spans.items():
            ids = self.enable(key, params.get(key))
            patched = self.read(0x8000000, len(original))
            changes = {i for i,(a,b) in enumerate(zip(original,patched)) if a != b}
            permitted = {i for start,length in allowed for i in range(start,start+length)}
            assert changes and changes.issubset(permitted), (key, changes - permitted)
            for cycle in range(8):
                self.c.togglecode(ids[0], cycle % 2)
                self.c.reset()
                assert self.read(0x8000000, len(original)) == (patched if cycle % 2 else original)
            self.c.clearcodes()
            assert self.read(0x8000000, len(original)) == original
            reports[key] = dict(lines=len(self.lines(key, params.get(key))),changed_bytes=len(changes),restore_cycles=8)
        return reports

    def hatch(self):
        count = 0
        for pid in range(24):
            for cycles in [0,1,2,5,255]:
                for egg,bad in [(True,False),(False,False),(True,True)]:
                    self.init()
                    raw = self.mon(egg=egg,bad=bad,cycles=cycles,pid=pid)
                    self.write(self.p['party'], raw)
                    self.put(self.p['count'],1,1)
                    assert self.call(self.p['hatch']) == 0
                    assert self.read(self.p['party'],100) == raw
                    self.enable(HATCH)
                    result = self.call(self.p['hatch'])
                    q = bytearray(self.unpack(raw))
                    q[9] = max(0,cycles-1) if egg and not bad else cycles
                    assert self.read(self.p['party'],100) == self.pack(raw,q), (pid,cycles,egg,bad)
                    assert result == int(egg and not bad and cycles <= 1), (pid,cycles,egg,bad,result)
                    count += 1
        return dict(boundary_bad_egg_permutation_cases=count)

    def generated(self):
        count = 0
        for species,level in [(25,1),(185,17),(400,100)]:
            for shiny in [False,True]:
                for slot in [0,1]:
                    self.init()
                    self.enable(ENCOUNTER,dict(kind='encounter',species=species,level=level))
                    if shiny: self.enable(SHINY)
                    for seed in range(64):
                        self.put(self.p['rng'],seed)
                        self.call(0x829fc,19,3,0,int(slot == 0))
                        raw=self.read(self.p['enemy']+slot*100,100)
                        pid,ot=struct.unpack_from('<II',raw)
                        assert self.call(self.p['get'],self.p['enemy']+slot*100,11,0)==species
                        assert raw[84]==level
                        assert not shiny or ((pid>>16)^(pid&65535)^(ot>>16)^(ot&65535))<8
                        count += 1
        # Calling the constructor outside an ordinary wild encounter is unchanged.
        for seed in range(32):
            results=[]
            for on in [False,True]:
                self.init();self.put(self.p['rng'],seed)
                if on:self.enable(SHINY)
                self.put(0x3007a00,7)
                assert self.call(0x3dd98,self.p['enemy'],25,5,32) != 0xffffffff
                results.append(self.read(self.p['enemy'],100))
            assert results[0]==results[1], seed
        # Native lead-ability paths must retain nature and requested gender.
        carriers={}
        self.init()
        for s in self.catalog['options']['species']:
            self.write(self.p['party'],self.mon(species=s['id']))
            ability=self.call(0x40d38,self.p['party'])
            if ability in (28,56):carriers.setdefault(ability,s['id'])
            if len(carriers)==2:break
        assert len(carriers)==2
        for ability,leader in carriers.items():
            for seed in range(64):
                results=[]
                for on in [False,True]:
                    self.init();self.put(self.p['rng'],seed)
                    self.write(self.p['party'],self.mon(species=leader,pid=24 if seed%2 else 254))
                    assert self.call(0x40d38,self.p['party']) == ability
                    self.put(self.p['count'],1,1)
                    if on:self.enable(SHINY)
                    self.call(0x829fc,25,15,0,1)
                    raw=self.read(self.p['enemy'],100);pid,ot=struct.unpack_from('<II',raw)
                    assert not on or ((pid>>16)^(pid&65535)^(ot>>16)^(ot&65535))<8
                    results.append((pid%25,self.call(0x3f78c,25,pid)))
                assert results[0][0] == results[1][0], (ability,seed,results)
                charm_active = ((seed * 0x41c64e6d + 0x6073) >> 16 & 65535) % 3 != 0
                if ability == 56 and charm_active:
                    assert results[0][1] == results[1][1], (ability,seed,results)
        return dict(species_level_shiny_slot_cases=count,nonwild_exact_cases=32,lead_ability_cases=256)

    def warps(self):
        maps=[m for m in self.catalog['options']['maps'] if m['landings']]
        cases=[]
        for index in sorted({0,len(maps)//4,len(maps)//2,len(maps)*3//4,len(maps)-1}):
            m=maps[index];w=m['landings'][0]
            self.init();ids=self.enable(TELEPORT,dict(kind='teleport',map_id=m['id'],warp_id=w['id']))
            self.put(0x3007a00,99)
            self.call(0x5538c,0,0,0,88)
            assert self.read(0x2031dbc,3)==bytes([m['group'],m['number'],w['id']])
            self.call(0x55378)
            assert self.read(0x202a004,3)==bytes([m['group'],m['number'],w['id']])
            assert struct.unpack('<HH',self.read(0x202a000,4))==(w['x'],w['y'])
            self.c.togglecode(ids[0],0);self.call(0x5538c,0,0,0,1)
            assert self.read(0x2031dbc,3)==bytes(3)
            cases.append(m['id'])
        return dict(eligible_maps=len(maps),native_setter_transition_cases=cases)

    def capture(self):
        cases=[]
        for ball in [2,3,4]:
            for flags in [0,0x200,0x8000]:
                for seed in [0,1,42,999999,0xffffffff]:
                    outputs=[]
                    for on in [False,True]:
                        self.init();raw=self.mon(species=150,pid=seed)
                        self.write(self.p['enemy'],raw);self.write(self.p['party'],self.mon())
                        self.put(self.p['count'],1,1);self.put(self.p['rng'],seed)
                        self.put(0x2023bcc,2,1);self.put(0x2023bd6,0x03020100)
                        self.put(0x2022b4c,flags);self.put(0x2023d68,ball,2)
                        self.put(0x203e038,0x2034000)
                        self.put(0x2023fe8,0x2035000)
                        enemy=0x2023be4+0x58
                        self.put(enemy,150,2);self.put(enemy+0x28,100,2);self.put(enemy+0x2c,100,2)
                        self.put(enemy+0x2a,100,1)
                        if on:self.enable(CATCH)
                        self.call(0x1d0f09c)
                        output=self.read(self.p['enemy'],100)
                        script=self.get(0x2023d74)
                        if flags==0 and on:
                            assert script==0x09c8e2c5,hex(script)
                            stored = next(b['value'] for b in self.reference['editor_rules']['ball_options'] if b['item'] == ball)
                            assert self.call(self.p['get'],self.p['enemy'],38,0)==stored
                            changed={i for i,(a,b) in enumerate(zip(raw,output)) if a!=b}
                            assert changed.issubset({42}),changed
                        outputs.append((script,output))
                    if flags:assert outputs[0]==outputs[1],(flags,ball,seed)
                    cases.append(dict(ball=ball,flags=flags,seed=seed,scripts=[hex(s[0]) for s in outputs]))
        assert any(c['scripts'][0]!=c['scripts'][1] for c in cases if not c['flags'])
        return dict(native_throw_cases=len(cases)*2,outcomes=cases)

    def walk(self):
        # Early-exit boundary only; full native map tests require --walk-state.
        for mode in [2, 3, 4, 5]:
            self.init()
            self.enable(NO_ENCOUNTERS)
            assert self.call(0x1d6cd4e, mode, mode) == 0
        return dict(native_early_exit_movement_cases=4)

    def walk_in_state(self, state):
        original = hashlib.sha256(state.read_bytes()).hexdigest()
        self.c.clearcodes()
        self.c.reset()
        assert self.c.load_state(str(state).encode())
        # Native SetWarp/Apply load the map's actual header/encounter context.
        self.put(0x3007a00, 0)
        self.call(0x5538c, 3, 79, 0, 0)
        self.call(0x55378)
        route = self.temp / "route.state"
        assert self.c.export_state(str(route).encode())
        self.enable(NO_ENCOUNTERS)
        result = {}
        for tile in (0x01000002, 0x01000003, 0x01000010):
            hits = []
            for enabled in (False, True):
                count = 0
                for seed in range(64):
                    assert self.c.load_state(str(route).encode())
                    self.c.togglecode(0, enabled)
                    self.put(self.p['rng'], seed * 0x9e3779b9 & 0xffffffff)
                    count += bool(self.call(0x1d6cd4e, tile, tile & 0x3ff))
                hits.append(count)
            assert hits[0] > 0 and hits[1] == 0, (tile, hits)
            result[hex(tile)] = hits
        self.c.clearcodes()
        assert hashlib.sha256(state.read_bytes()).hexdigest() == original
        return dict(native_map_cases=384, off_on_encounters=result)

    def compact_vectors(self, output):
        vectors = []
        for species in (25, 185, 400, 1, 6):
            for code in range(32):
                for parity in (0, 1):
                    self.init()
                    self.call(0x829fc, species, 17, 0, 1)
                    raw = bytearray(self.read(self.p['enemy'], 80))
                    struct.pack_into('<I', raw, 0, 0x12345678 + parity)
                    struct.pack_into('<H', raw, 30, 0xa500 | code)
                    raw[40:43] = bytes((code * 7 & 255, code, parity))
                    for i, move in enumerate((0, 1, 999, 1014)):
                        struct.pack_into('<H', raw, 44 + i * 2, move)
                    raw[52:56] = bytes((1, 2, 3, 4))
                    raw[56:68] = bytes(range(100, 112))
                    raw[76:80] = bytes((1, 2, 3, 4))
                    self.write(0x2038000, raw)
                    self.write(0x2038100, bytes([0xa5] * 80))
                    assert self.call(0x1d5a630, 0x2038000, 0x2038100) != 0xffffffff
                    compact = self.read(0x2038100, 58)
                    assert self.read(0x203813a, 22) == bytes([0xa5] * 22)
                    assert self.call(0x1d5a404, 0x2038200, 0x2038100) != 0xffffffff
                    expanded = self.read(0x2038200, 80)
                    vectors.append(dict(raw=list(raw), compact=list(compact), expanded=list(expanded)))
        output.write_text(json.dumps(vectors))
        return dict(native_converter_vectors=len(vectors))


class MercuryStorageProbe(MercuryProbe):
    """The game's 25 compact boxes, expanded by its own converter for comparison."""
    total_slots = 750

    def __init__(self, library, path, temp):
        from verify_storage_cheats_mgba import StorageProbe
        super().__init__(library, path, temp)
        self.name = "MERCURY133"
        self.storage_pointer = 0x3005010
        self.codes_pc = self.lines(PC)
        # Reuse the native SELECT/save/reboot/withdraw workflow, not an 80-byte
        # assumption about this game's physical PC storage.
        for name in ("enable_pc", "screenshot", "sb1", "boxes", "snapshot", "gameplay"):
            setattr(self, name, getattr(StorageProbe, name).__get__(self))

    def pointers(self):
        return [self.get(0x9DE4A7C + i * 4) for i in range(25)]

    def clear_boxes(self):
        for pointer in self.pointers():
            self.write(pointer, bytes(30 * 58))

    def box_records(self):
        result = []
        # Preserve scratch RAM across independent native calls.
        scratch = 0x0203F000
        old = self.read(scratch, 80)
        try:
            for pointer in self.pointers():
                for slot in range(30):
                    record = pointer + slot * 58
                    if not self.get(record + 28, 2):
                        result.append(bytes(80))
                    else:
                        assert self.call(0x1d5a404, scratch, record) != 0xffffffff
                        result.append(self.read(scratch, 80))
        finally:
            self.write(scratch, old)
        return result

    def seed_party(self):
        for i, species in enumerate((25, 185)):
            self.call(0x829fc, species, 17, 0, 1)
            self.write(self.p['party'] + 100 * i, self.read(self.p['enemy'], 100))

    def save_inputs(self):
        return (8, 16, 16, 16, 1, 1, 1)

    def press(self, *keys):
        for key in keys:
            self.c.frames(2, key)
            self.c.frames(240, 0)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--probe', type=Path, required=True)
    parser.add_argument('--only', nargs='*')
    parser.add_argument('--walk-state', type=Path)
    parser.add_argument('--pc-state', type=Path)
    parser.add_argument('--pc-save', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--compact-vectors', type=Path)
    args = parser.parse_args()
    path = Path(os.environ['GEN3_ROM_MERCURY133'])
    with tempfile.TemporaryDirectory(prefix='gen3-mercury-cheats-') as temp:
        native = MercuryStorageProbe(args.probe, path, Path(temp))
        report = {}
        try:
            checks = [('decoder', native.decoder), ('hatch', native.hatch),
                      ('breeding', native.breed), ('generated', native.generated),
                      ('warps', native.warps), ('capture', native.capture),
                      ('walk', native.walk)]
            if args.walk_state:
                checks.append(('walk_map', lambda: native.walk_in_state(args.walk_state)))
            if args.compact_vectors:
                checks.append(('compact', lambda: native.compact_vectors(args.compact_vectors)))
            if args.pc_state and args.pc_save:
                output = args.output or Path(temp)
                output.mkdir(parents=True, exist_ok=True)
                before = hashlib.sha256(args.pc_save.read_bytes()).hexdigest()
                def pc():
                    result = native.gameplay(args.pc_state, args.pc_save, output)
                    assert hashlib.sha256(args.pc_save.read_bytes()).hexdigest() == before
                    return result
                checks.append(('pc', pc))
            for key, check in checks:
                if args.only and key not in args.only:
                    continue
                report[key] = check()
                print(key + ' passed', file=__import__('sys').stderr, flush=True)
        finally:
            native.c.finish()
        assert path.read_bytes() == native.data
    report['source_rom_unchanged'] = True
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
