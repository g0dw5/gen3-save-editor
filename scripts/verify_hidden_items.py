#!/usr/bin/env python3
"""Exact-ROM hidden-item decoder/consumer checks; synthetic RAM only, no files written."""
import os,sys,struct,hashlib
from pathlib import Path
from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_R0,UC_ARM_REG_PC,UC_ARM_REG_LR
from verify_rocket_battle_forms import Native
from verify_event_state import PROFILES


def emerald(key,rom):
 cpu=Native(rom);entry=0x1359c4 if key=='ROCKET' else 0xfd6d4;flag_get=PROFILES[key][1]
 calls=[];collected=False
 def flag(uc,address,_size,_data):
  if address==0x08000000+flag_get:
   calls.append(uc.reg_read(UC_ARM_REG_R0));uc.reg_write(UC_ARM_REG_R0,int(collected));uc.reg_write(UC_ARM_REG_PC,uc.reg_read(UC_ARM_REG_LR))
 cpu.cpu.hook_add(UC_HOOK_CODE,flag)
 events=0x02010000;bg=0x02010100;cpu.write(events+3,b'\1');cpu.word(events+16,bg)
 count=0
 for index in [0,1,95,255,256,4095]:
  cpu.write(bg,struct.pack('<HHBBHHH',5,9,0,7,0,13,index))
  for collected in [False,True]:
   before=len(calls);assert cpu.call(entry,events,5,9)==int(not collected)
   assert calls[before:]==[index+0x1f4]
   assert cpu.call(entry,events,6,9)==0
   count+=1
 return count


def mercury(rom):
 cpu=Native(rom);regions=[]
 for value in rom[0x1de3a7e:0x1de3a7e+256]:
  if value==255:break
  regions.append(value)
 else:raise AssertionError('unterminated region list')
 count=0
 for region in range(256):
  cpu.write(0x02036dfc+20,bytes([region]))
  for index,quantity,underfoot in [(0,0,False),(1,1,False),(95,5,False),(255,127,True)]:
   raw=13|(index<<16)|(quantity<<24)|(int(underfoot)<<31)
   expected=[13,index+(0xd00 if region in regions else 0x3e8),quantity,int(underfoot)]
   for kind,value in enumerate(expected):assert cpu.call(0xcc44c,raw,kind)==value;count+=1
 # The actual underfoot consumer ignores the packed quantity and writes one.
 def skip_text(uc,address,_size,_data):
  if address==0x080cbed4:uc.reg_write(UC_ARM_REG_PC,uc.reg_read(UC_ARM_REG_LR))
 cpu.cpu.hook_add(UC_HOOK_CODE,skip_text)
 quantity_var=struct.unpack_from('<I',rom,0x13ef98)[0]
 for quantity in [0,1,5,127]:
  raw=13|(95<<16)|(quantity<<24)|0x80000000
  cpu.call(0x13ef40,0,raw)
  assert cpu.read(quantity_var,2)==b'\1\0',(hex(quantity_var),cpu.read(quantity_var,2))
 return count,4


if __name__=='__main__':
 for key,profile in PROFILES.items():
  path=Path(os.environ['GEN3_ROM_'+key]);rom=path.read_bytes();digest=hashlib.sha256(rom).digest()
  assert hashlib.md5(rom).hexdigest()==profile[0]
  print(key,mercury(rom) if key=='MERCURY133' else emerald(key,rom),'native hidden-item comparisons',flush=True)
  assert hashlib.sha256(path.read_bytes()).digest()==digest
