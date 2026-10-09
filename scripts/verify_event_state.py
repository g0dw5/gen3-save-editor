#!/usr/bin/env python3
"""Read-only exact-ROM persistent flag/variable parity; synthetic RAM only.

Requires all five GEN3_ROM_* paths and Unicorn. --output writes private parity
vectors, never a bundled catalog or input file. Temporary IDs are not saved state.
"""
import argparse
import hashlib
import json
import os
import struct
from pathlib import Path
from verify_rocket_battle_forms import Native

PROFILES = {
 'BW': ('0d9b129f7dd76895f79bb47ad7dec2fe',0x9d790,0x9d694,[(0,0x4000,'main',0x1270)],[(0x4000,0x100,'main',0x139c)]),
 'DP': ('cb2940215f4dafb1bef133c3af379f44',0x9d790,0x9d694,[(0,0x4000,'main',0x1270)],[(0x4000,0x100,'main',0x139c)]),
 'ROCKET': ('59c658a1081f542086de1060bb65f0b3',0xd3ae4,0xd397c,[(0,0x4000,'main',0x1ca8)],[(0x4000,0x100,'main',0x1f6c)]),
 'ULTIMATE': ('17ce9785b33319b3dbda9a5d37c57ec1',0x9d790,0x9d694,[(0,0x4000,'main',0x1270),(0x4000,0x1a0,'main',0x988),(0x41a0,0x1a0,'main',0x3b24),(0x4340,0x1a0,'trainer',0x5c),(0x44e0,0x1a0,'trainer',0x28)],[(0x4000,0x100,'main',0x139c)]),
 'MERCURY133': ('5ffb1cbd5c28cda9b987b3b445da68e0',0x6e6d0,0x6e568,[(0,0x900,'main',0xee0),(0x900,0x1000,'extensions',0)],[(0x4000,0x100,'main',0x1000),(0x5000,0x200,'extensions',0x200)]),
}
BASES={'main':0x02030000,'trainer':0x02038000,'extensions':0x0203B174}
SIZES={'main':0x4000,'trainer':0x1000,'extensions':0x800}


def pattern(size, salt):
 return bytes(((i*73+(i>>5)*19)^salt)&255 for i in range(size))


def verify(key, rom):
 md5,flag_get,var_get,flags,variables=PROFILES[key]
 assert hashlib.md5(rom).hexdigest()==md5
 cpu=Native(rom); vectors=[];count=0
 for salt in [0x55,0xAA]:
  cpu.write(0x02000000,bytes(0x40000));cpu.write(0x03000000,bytes(0x8000))
  blocks={block:pattern(size,salt) for block,size in SIZES.items()}
  for block,base in BASES.items():cpu.write(base,blocks[block])
  for pointer,base in [(0x03005D8C,BASES['main']),(0x03005D90,BASES['trainer']),(0x0300524C,BASES['main']),(0x03005008,BASES['main'])]:cpu.word(pointer,base)
  # ID zero is an object-template sentinel; its raw native handling differs in Ultimate.
  if key != 'ULTIMATE': assert cpu.call(flag_get,0)==0
  for kind,ranges,entry in [('flag',flags,flag_get),('variable',variables,var_get)]:
   for first,n,block,offset in ranges:
    for id in range(first,first+n):
     if kind=='flag' and id==0:continue
     index=id-first
     expected=((blocks[block][offset+index//8]>>(index%8))&1) if kind=='flag' else struct.unpack_from('<H',blocks[block],offset+index*2)[0]
     actual=cpu.call(entry,id)
     assert actual==expected,(key,kind,hex(id),actual,expected)
     count+=1
     if index<16 or index>=n-16 or index%127==0:
      vectors.append(dict(salt=salt,kind=kind,id=id,expected=actual))
  for block,base in BASES.items():assert cpu.read(base,len(blocks[block]))==blocks[block]
 return dict(md5=md5,comparisons=count,vectors=vectors,ram_unchanged=True)


def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path);args=parser.parse_args()
 inputs={k:Path(os.environ['GEN3_ROM_'+k]) for k in PROFILES}
 if args.output and any(args.output.resolve()==p.resolve() or (args.output.exists() and args.output.samefile(p)) for p in inputs.values()):parser.error('--output must not replace a ROM input')
 results={}
 for key,path in inputs.items():
  rom=path.read_bytes();before=hashlib.sha256(rom).digest();results[key]=verify(key,rom)
  assert hashlib.sha256(path.read_bytes()).digest()==before
  print(key,results[key]['comparisons'],'native comparisons; input/RAM unchanged',flush=True)
 if args.output:
  args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(results,indent=2)+'\n')


if __name__=='__main__':main()
