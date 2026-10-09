#!/usr/bin/env python3
"""Independent exact-ROM held-item assignment probes with synthetic RAM only.

No SAV is opened or written; no ROM is patched. All native random, assignment,
ability/getter/setter and exceptional-layout code executes without hooks. Output
contains synthetic identifiers/counts, not names/artwork or a bundled catalog.
Scope: ordinary single wild battle; facilities, scripted overrides and full
battle entry/current map-layout replacement are separate. Keep probes private.
"""
import hashlib,json,os,struct
from pathlib import Path
from verify_rocket_battle_forms import Native,pack

CONFIG = {
 'BW': (0x6ea68,0x6f5cc,0x03005d80,0x02024744,0x020244ec,0x6a518,0x6b6d8,0x3203cc,412,28,0x02037318,0x03005d8c,0x03005d90,0x6ea54),
 'DP': (0x6ea68,0x6f5cc,0x03005d80,0x02024744,0x020244ec,0x6a518,0x6b6d8,0x3203cc,412,28,0x02037318,0x03005d8c,0x03005d90,0x6ea54),
 'ROCKET': (0x9c610,0x9d40c,0x03005240,0x020253c8,0x02025170,0x976d0,0x988f0,0x5b4764,1395,36,0x02036de0,0x0300524c,0x03005250,0x9c5fc),
 'ULTIMATE': (0x6ea68,0x6f5cc,0x03005d80,0x02024744,0x020244ec,0x6a518,0x6b6d8,0xf186e0,1200,28,0x02037318,0x03005d8c,0x03005d90,0x6ea54),
 'MERCURY133': (0x443f4,0x44ec8,0x03005000,0x0202402c,0x02024284,0x3fbe8,0x40d38,0x176dfbc,1554,28,0,0x03005008,0x0300500c,0),
}
MD5 = dict(BW='0d9b129f7dd76895f79bb47ad7dec2fe',DP='cb2940215f4dafb1bef133c3af379f44',ROCKET='59c658a1081f542086de1060bb65f0b3',ULTIMATE='17ce9785b33319b3dbda9a5d37c57ec1',MERCURY133='5ffb1cbd5c28cda9b987b3b445da68e0')

def fixture(name,species,egg=False,hp=100,slot=0):
 raw=bytearray(100);struct.pack_into('<II',raw,0,24,0x12345678)
 if name=='ROCKET': raw[18]=0x12 | (0x20 if egg else 0)
 else: raw[18]=2;raw[19]=2 | (4 if egg else 0)
 c=bytearray(48);struct.pack_into('<H',c,0,species)
 if name=='ROCKET': c[47]=slot;struct.pack_into('<I',c,8,26<<13)
 else: struct.pack_into('<I',c,40,((1 if slot else 0)<<31)|(int(egg)<<30))
 if name=='ULTIMATE' and slot==2:raw[30]|=1
 if name=='MERCURY133' and slot==1:struct.pack_into('<I',raw,0,25)
 if name=='ROCKET' and egg:struct.pack_into('<I',c,40,1<<30)
 raw[84]=50;struct.pack_into('<HH',raw,86,hp,100)
 if name in ['MERCURY133','ULTIMATE']:
  raw[32:80]=c;struct.pack_into('<H',raw,28,sum(struct.unpack('<24H',c))&65535)
  return bytes(raw)
 return pack(raw,c)

def verify(name):
 rom=Path(os.environ['GEN3_ROM_'+name]).read_bytes();assert hashlib.md5(rom).hexdigest()==MD5[name]
 code,rng,rngaddr,enemy,player,get,ability,base,total,stride,header,sb1,sb2,specialptr=CONFIG[name]
 cpu=Native(rom);cpu.word(sb1,0x02030000);cpu.word(sb2,0x02034000)
 a=0x41c64e6d;c=0x6073;inverse=pow(a,-1,1<<32)
 # Verify the original RNG's entire 16-bit output domain rather than assuming
 # that the %100 branches have equal weights. No function is replaced.
 for value in range(65536):
  cpu.word(rngaddr,((value<<16)-c)*inverse & 0xffffffff)
  assert cpu.call(rng)==value
 samples={}
 for species in range(1,total):
  common,rare=struct.unpack_from('<HH',rom,base+species*stride+(14 if stride==36 else 12))
  pattern='same' if common==rare and common else 'both' if common and rare else 'common' if common else 'rare' if rare else 'none'
  if pattern not in samples and any(rom[base+species*stride:base+species*stride+6]):samples[pattern]=species
 special=[]
 if specialptr:
  table=struct.unpack_from('<I',rom,specialptr)[0]-0x08000000
  special=[struct.unpack_from('<HH',rom,table+i*4) for i in range(1,9)]
 leads=[dict(species=0,ability=None,egg=False,hp=100,raw=None)]
 # Discover actual first-slot identities from current ROM data and verify them
 # through the native getter. These are engine selector operands, not name aliases.
 found={}
 for species in range(1,total):
  if name=='ULTIMATE': options=struct.unpack_from('<3H',rom,0x17a0000+species*6)
  elif stride==36: options=struct.unpack_from('<3H',rom,base+species*stride+24)
  else: options=(rom[base+species*stride+22],rom[base+species*stride+23])
  for slot,expected in enumerate(options):
   if expected in (14,28,105) and expected not in found:
    raw=fixture(name,species,slot=slot);cpu.write(player,raw)
    actual=cpu.call(ability,player)
    if actual==expected:found[actual]=(species,slot)
 for effect,(species,slot) in sorted(found.items()):
  for egg,hp in [(False,100),(False,0),(True,100)]:
   leads.append(dict(species=species,ability=effect,slot=slot,egg=egg,hp=hp,raw=list(fixture(name,species,egg,hp,slot))))
 rows=[]
 for species in sorted(set(samples.values())|{s for s,i in special if s<total}):
  for layout in ([0,420] if header else [0]):
   if header:cpu.half(header+18,layout)
   for lead in leads:
    raw_lead=bytes(lead['raw']) if lead['raw'] is not None else fixture(name,0)
    cpu.write(player,raw_lead); outcomes=[]
    for residue in range(100):
     values=[residue,100+residue,65400+residue]
     results=[]
     for value in values:
      raw=fixture(name,species);cpu.write(enemy,raw)
      cpu.word(rngaddr,((value<<16)-c)*inverse & 0xffffffff)
      cpu.call(code)
      item=cpu.call(get,enemy,12,0)
      assert struct.unpack('<I',cpu.read(rngaddr,4))[0]==value<<16
      assert cpu.read(player,100)==raw_lead, 'native selection changed the leading individual'
      results.append(item)
     assert len(set(results))==1,(name,species,layout,lead['ability'],residue,results)
     outcomes.append(results[0])
    counts={}
    for residue,item in enumerate(outcomes):counts[item]=counts.get(item,0)+(656 if residue<36 else 655)
    rows.append(dict(species=species,layout=layout,lead=lead,outcomes=outcomes,counts=counts))
 return dict(md5=MD5[name],rng_outputs_verified=65536,native_assignment_cases=len(rows)*300,rows=rows,special=special,scope='ordinary single wild battle; static layout; synthetic neutral or real species lead')

if __name__=='__main__':
 import argparse
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
 data={}
 for name in CONFIG:
  data[name]=verify(name)
  args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(data,indent=2),encoding='utf-8')
  print(name,len(data[name]['rows']),data[name]['native_assignment_cases'],'native assignment cases',flush=True)
