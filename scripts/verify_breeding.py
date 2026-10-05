#!/usr/bin/env python3
"""Ordinary daycare compatibility/egg receipt in five exact private ROMs.

Runs complete native routines without hooks or replacements, using synthetic
parents and disposable RAM. Opens no SAV and writes no ROM. Private probe output
must not be bundled. This is not a next-egg, production-rate or access verifier.
"""
import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
from verify_common_cheats_mgba import pack, unpack
MD5 = dict(BW='0d9b129f7dd76895f79bb47ad7dec2fe',DP='cb2940215f4dafb1bef133c3af379f44',ROCKET='59c658a1081f542086de1060bb65f0b3',ULTIMATE='17ce9785b33319b3dbda9a5d37c57ec1',MERCURY12='f323df1792ac68462a34b42fe8571533')

# Addresses/layouts only; no extracted species, items or game text catalog.
CONFIG = {
    'BW': (0x708c8, 0x70d4c, 0x3030, 0x03005d8c, 0x03005d90, 0x03005d80, 0x020244ec, 0x020244e9, 4),
    'DP': (0x708c8, 0x70d4c, 0x3030, 0x03005d8c, 0x03005d90, 0x03005d80, 0x020244ec, 0x020244e9, 4),
    'ROCKET': (0x9e8b0, 0x9ed3c, 0x297c, 0x0300524c, 0x03005250, 0x03005240, 0x02025170, 0x0202516d, 4),
    'ULTIMATE': (0x708c8, 0x70d4c, 0x3030, 0x03005d8c, 0x03005d90, 0x03005d80, 0x020244ec, 0x020244e9, 4),
    'MERCURY12': (0x460d4, 0x4654c, 0x2f80, 0x03005008, 0x0300500c, 0x03005000, 0x02024284, 0x02024029, 2),
}

class MGBANative:
    """Disposable RAM, complete GBA ARM7 routines; no native function hooks."""
    c = None
    current = None
    def __init__(self, rom):
        import ctypes
        self.ctypes = ctypes
        if self.__class__.c is None:
            self.__class__.c = ctypes.CDLL(str(MGBA_PROBE))
        self.c = self.__class__.c
        path = ROM_PATH
        if self.__class__.current != path:
            self.c.finish()
            assert self.c.start(str(path).encode())
            self.__class__.current = path
        self.write(0x02000000, bytes(0x40000))
        self.write(0x03000000, bytes(0x8000))
    def write(self, address, data):
        data = bytes(data)
        assert self.c.writebytes(address, data, len(data)), 'only RAM writes are allowed'
    def read(self, address, size):
        data = self.ctypes.create_string_buffer(size)
        self.c.readbytes(address, data, size)
        return data.raw
    def half(self, address, value): self.write(address, struct.pack('<H', value))
    def word(self, address, value): self.write(address, struct.pack('<I', value))
    def call(self, offset, *args):
        output = self.ctypes.c_uint()
        assert len(args) <= 4
        assert self.c.callfunc(0x08000000 + offset, *(list(args)+[0]*(4-len(args))), self.ctypes.byref(output)), hex(offset)
        return output.value

def parent(key, species, pid, ot, item=0):
    raw = bytearray(80)
    raw[18] = 0x12 if key == "ROCKET" else 2
    if key != "ROCKET": raw[19] = 2
    raw[8:18] = bytes([255])*10
    raw[20:27] = bytes([255])*7
    struct.pack_into('<II', raw, 0, pid, ot)
    c = bytearray(48)
    struct.pack_into('<HH', c, 0, species, item)
    struct.pack_into('<4H', c, 12, 1, 2, 3, 4)
    c[20:24] = bytes([10]*4)
    struct.pack_into('<I', c, 40, 0x12345678)
    if key == 'ROCKET': struct.pack_into('<I', c, 8, 26 << 13)
    if key in ('ULTIMATE','MERCURY12'):
        raw[32:80] = c
        struct.pack_into('<H', raw, 28, sum(struct.unpack('<24H',c)) & 65535)
    else: raw = pack(raw, c)
    return bytes(raw[:80])

def verify(key):
    global ROM_PATH
    ROM_PATH = Path(os.environ['GEN3_ROM_'+key])
    rom = ROM_PATH.read_bytes()
    assert hashlib.md5(rom).hexdigest() == MD5[key]
    entry, compat, offset, sb1, sb2, rng, party, count, width = CONFIG[key]
    # Explicit scenarios, not a bundled claim about identities or available items.
    pairs = [((25,0,1,0),(25,255,1,0)), ((25,255,1,0),(25,0,2,0)),
             ((25,0,1,0),(25,0,2,0)), ((132,24,1,0),(132,255,2,0)),
             ((132,24,1,0),(137,24,2,0)), ((137,24,2,0),(132,24,1,0)),
             ((173,0,1,0),(174,255,2,0)),
             ((202,0,1,0),(202,255,2,0)), ((202,0,1,221),(202,255,2,0)),
             ((183,0,1,0),(183,255,2,0)), ((183,0,1,220),(183,255,2,0)),
             ((35,0,1,0),(35,255,2,0)), ((133,0,1,0),(133,255,2,0))]
    rows=[]
    for pair in pairs:
        parents=[parent(key,*p) for p in pair]
        for seed,pid in [(42,24),(0x12345678,0x8018),(1,65535)]:
            cpu=Native(rom)
            cpu.word(sb1,0x02030000);cpu.word(sb2,0x02034000)
            daycare=0x02030000+offset
            for i,raw in enumerate(parents):cpu.write(daycare+i*140,raw)
            cpu.word(rng,seed)
            score=cpu.call(compat,daycare)
            child=None
            if score:
                cpu.write(daycare+280, pid.to_bytes(width,'little'))
                cpu.call(entry,daycare)
                assert cpu.read(count,1)==b'\x01', (key,pair,'party count')
                child=list(cpu.read(party,100))
            assert [cpu.read(daycare+i*140,80) for i in range(2)]==parents
            rows.append(dict(parents=[list(p) for p in parents],seed=seed,offspring_pid=pid,
                             compatibility=score,child_raw=child,rng_after=int.from_bytes(cpu.read(rng,4),'little')))
    # Independent UI-default simulated parents: current-ROM growth thresholds,
    # zero IVs/no moves, level 5, different OTs. Verify the frontend request path
    # uses exactly these native input bytes rather than another game's template.
    exp_base,exp_stride,stats,stat_stride = {
        'BW':(0x31f72c,404,0x3203cc,28),'DP':(0x31f72c,404,0x3203cc,28),
        'ROCKET':(0x5b3484,604,0x5b4764,36),'ULTIMATE':(0x31f72c,404,0xf186e0,28),
        'MERCURY12':(0x1e052c8,1024,0x176dfbc,28)}[key]
    growth=rom[stats+25*stat_stride+(21 if stat_stride==36 else 19)]
    xp=struct.unpack_from('<I',rom,exp_base+growth*exp_stride+5*4)[0]
    simulated=[]
    for pid,ot in [(0,1),(255,2)]:
        raw=bytearray(parent(key,25,pid,ot))
        c=bytearray(48);struct.pack_into('<HI',c,0,25,0)
        struct.pack_into('<I',c,4,xp)
        if key=='ROCKET':struct.pack_into('<I',c,8,26<<13)
        if key in ('ULTIMATE','MERCURY12'):
            raw[32:80]=c;struct.pack_into('<H',raw,28,0)
        else:raw=bytearray(pack(raw,c))
        simulated.append(bytes(raw))
    for seed,pid in [(42,24),(0x12345678,0x8018),(1,65535)]:
        cpu=Native(rom);cpu.word(sb1,0x02030000);cpu.word(sb2,0x02034000)
        daycare=0x02030000+offset
        for i,raw in enumerate(simulated):cpu.write(daycare+i*140,raw)
        cpu.word(rng,seed);score=cpu.call(compat,daycare)
        assert score
        cpu.write(daycare+280,pid.to_bytes(width,'little'));cpu.call(entry,daycare)
        assert cpu.read(count,1)==b'\x01'
        assert [cpu.read(daycare+i*140,80) for i in range(2)]==simulated
        rows.append(dict(simulated=True,parents=[list(p) for p in simulated],seed=seed,offspring_pid=pid,
                         compatibility=score,child_raw=list(cpu.read(party,100)),rng_after=int.from_bytes(cpu.read(rng,4),'little')))
    alignment=[]
    if MGBA_PROBE:
        cpu=Native(rom)
        for instruction in [0x8808,0x5a88,0x5e88]:
            for low,high in [(0x80,0xff),(0xff,0x80),(0x34,0x12),(0xff,0xff)]:
                for odd in [0,1]:
                    cpu.write(0x02020000,struct.pack('<HH',instruction,0x4770))
                    cpu.write(0x02021000,bytes([low,high,0x55,0xaa]))
                    value=cpu.call(0x02020000-0x08000000,0,0x02021000+odd,0)
                    alignment.append(dict(instruction=instruction,bytes=[low,high,0x55,0xaa],odd=odd,output=value))
        assert cpu.read(0x08000000,len(rom))==rom,'native ROM memory changed'
        assert hashlib.md5(ROM_PATH.read_bytes()).hexdigest()==MD5[key],'source ROM changed'
    return dict(alignment=alignment,engine='mGBA ARM7' if MGBA_PROBE else 'Unicorn',md5=MD5[key],rows=rows,scope='ordinary daycare receipt; synthetic parents; no production, next egg or access prediction')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--mgba-probe',type=Path,help='Compiled read-only scripts/native/breeding_probe.c fixture library; required for exact GBA unaligned access evidence')
    args=parser.parse_args();results={}
    MGBA_PROBE = args.mgba_probe.resolve() if args.mgba_probe else None
    if MGBA_PROBE: Native = MGBANative
    else: from verify_rocket_battle_forms import Native
    for key in CONFIG:
        results[key]=verify(key)
        args.output.parent.mkdir(parents=True,exist_ok=True)
        args.output.write_text(json.dumps(results,indent=2),encoding='utf-8')
        print(key,len(results[key]['rows']),'native daycare scenarios',flush=True)
