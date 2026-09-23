#!/usr/bin/env python3
"""Compare Ultimate Emerald adapter output against exact native ARM routines.

Inputs are user-supplied; synthetic individual fixtures and results stay local.
Requires Unicorn. Generate probes with local_ultimate_adapter_regression.
"""
import argparse, hashlib, json, struct
from pathlib import Path
from unicorn import Uc, UC_ARCH_ARM, UC_MODE_THUMB
from unicorn.arm_const import UC_ARM_REG_PC, UC_ARM_REG_SP, UC_ARM_REG_LR, UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2

MD5 = '17ce9785b33319b3dbda9a5d37c57ec1'
PARTY = 0x02020000
STOP = 0x03007000

class Native:
    def __init__(self, rom):
        assert hashlib.md5(rom).hexdigest() == MD5
        self.u = Uc(UC_ARCH_ARM, UC_MODE_THUMB)
        for address, size in [(0x08000000, 0x2000000), (0x02000000, 0x40000), (0x03000000, 0x8000)]:
            self.u.mem_map(address, size)
        self.u.mem_write(0x08000000, rom)

    def call(self, offset, r0=0, r1=0, r2=0):
        for reg, value in [(UC_ARM_REG_SP, STOP-0x200), (UC_ARM_REG_LR, STOP|1), (UC_ARM_REG_R0,r0), (UC_ARM_REG_R1,r1), (UC_ARM_REG_R2,r2)]:
            self.u.reg_write(reg,value)
        self.u.emu_start(0x08000001+offset, STOP, count=200000)
        assert self.u.reg_read(UC_ARM_REG_PC) == STOP, hex(self.u.reg_read(UC_ARM_REG_PC))
        return self.u.reg_read(UC_ARM_REG_R0)

    def read(self, address, size): return bytes(self.u.mem_read(address,size))
    def write(self, address, data): self.u.mem_write(address,bytes(data))

def check(rom, probes):
    n=Native(rom); getters=stats=0
    fields={'species':11,'held_item':12,'experience':25,'friendship':32,'ball':38,'ability_slot':46,'met_level':36,'origin_game':37,'ot_gender':49}
    for case in probes:
        raw=bytes(case['after']); p=case['pokemon']
        n.write(PARTY,raw.ljust(100,b'\0'))
        for name,field in fields.items():
            actual=n.call(0x6a518,PARTY,field,0)
            assert actual==p[name], (name,field,actual,p[name],p['species'])
            getters+=1
        for i in range(6):
            assert n.call(0x6a518,PARTY,39+i,0)==p['ivs'][i]
            getters+=1
        assert n.read(PARTY,len(raw))==raw
        n.call(0x68d0c,PARTY)
        actual=list(struct.unpack('<6H',n.read(PARTY+88,12)))
        assert actual==p['stats'],('stats',p['species'],p['effective_nature'],p['hyper_trained'],actual,p['stats'])
        stats+=1
    return {'md5':MD5,'native_getters':getters,'native_stat_comparisons':stats,'getters_preserve_every_byte':True}

if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--rom',type=Path,required=True);ap.add_argument('--probes',type=Path,required=True);args=ap.parse_args()
    print(json.dumps(check(args.rom.read_bytes(),json.loads(args.probes.read_text())),indent=2))
