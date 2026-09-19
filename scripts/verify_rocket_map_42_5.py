#!/usr/bin/env python3
"""Verify Rocket map 42-5 against native layout, grid, layer and palette routines.

Requires the exact private ROM, a PNG returned by the editor's map_image API,
Unicorn and Pillow. Only synthetic RAM is changed; no ROM/save is written.
Optional extracted PNG output stays outside release inputs. BIOS CpuSet is
implemented at its call boundary; weather palette backup is skipped. This checks
static rendering, not full game entry, event scripts, reachability or animations.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
from verify_rocket_battle_forms import Native, ROM_MD5
from unicorn import UC_HOOK_CODE
from unicorn.arm_const import (UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2,
                              UC_ARM_REG_PC, UC_ARM_REG_LR)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rom', type=Path, required=True)
    parser.add_argument('--rendered', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    r=args.rom.read_bytes()
    assert hashlib.md5(r).hexdigest() == ROM_MD5
    n=Native(r)
    def cpuset(cpu,*_):
     src,dst,ctl=[cpu.reg_read(v) for v in [UC_ARM_REG_R0,UC_ARM_REG_R1,UC_ARM_REG_R2]];width=4 if ctl&(1<<26) else 2;count=ctl&0x1fffff
     data=bytes(cpu.mem_read(src,width if ctl&(1<<24) else count*width))
     cpu.mem_write(dst,data*count if ctl&(1<<24) else data);cpu.reg_write(UC_ARM_REG_PC,cpu.reg_read(UC_ARM_REG_LR))
    n.cpu.hook_add(UC_HOOK_CODE,cpuset,begin=0x822b178,end=0x822b178)
    n.word(0x0300524c,0x02030000);n.half(0x02030032,62)
    assert n.call(0xb9fd0)==0x880808c
    n.word(0x03005280,29);n.word(0x03005284,24);n.word(0x03005288,0x02010000)
    n.call(0xbd894,0x8807f74,14,10)
    headerptr=struct.unpack_from('<I',r,0xbdcec)[0];n.write(headerptr,r[0x9f07a4:0x9f07a4+28])
    for p,v in [(0x03005264,0x02018000),(0x0300525c,0x02019000),(0x03005260,0x0201a000)]:n.word(p,v)
    count=0;types={}
    for y in range(10):
     for x in range(14):
      raw=struct.unpack_from('<H',r,0x807f74+(y*14+x)*2)[0];id=raw&1023
      assert n.call(0xbdc6c,x+7,y+7)==id
      typ=n.call(0xbdd58,x+7,y+7);types[typ]=types.get(typ,0)+1
      n.call(0xbfb90,0x880808c,0,x+7,y+7)
      meta=(0x7680d2+id*24) if id<640 else (0x7253ce+(id-640)*24)
      got=[]
      for base in [0x02018000,0x02019000,0x0201a000]:
       got.extend(struct.unpack('<H',n.read(base+o,2))[0] for o in [0,2,64,66])
      exp=list(struct.unpack_from('<12H',r,meta))
      assert got==exp,(x,y,typ,got,exp)
      count+=1
    print(json.dumps(dict(layout_from_native=hex(n.call(0xb9fd0)),metatiles_matched=count,layer_types=types),indent=2))
    from PIL import Image
    from verify_pokemon_sprites import decompress
    palette=bytearray(512)
    def palette_load(cpu,*_):
     src,dst,size=[cpu.reg_read(v) for v in [UC_ARM_REG_R0,UC_ARM_REG_R1,UC_ARM_REG_R2]]
     palette[dst*2:dst*2+size]=cpu.mem_read(src,size)
     cpu.reg_write(UC_ARM_REG_PC,cpu.reg_read(UC_ARM_REG_LR))
    def skip(cpu,*_):cpu.reg_write(UC_ARM_REG_PC,cpu.reg_read(UC_ARM_REG_LR))
    h1=n.cpu.hook_add(UC_HOOK_CODE,palette_load,begin=0x80d8090,end=0x80d8090)
    h2=n.cpu.hook_add(UC_HOOK_CODE,skip,begin=0x80be7ac,end=0x80be7ac)
    n.call(0xbe918,0x880808c)
    expected=bytes(2)+r[0x6c6aa4+2:0x6c6aa4+224]+r[0x6826bc+224:0x6826bc+416]+bytes(96)
    assert palette==expected
    n.cpu.hook_del(h1);n.cpu.hook_del(h2)
    # Independent pixel composition using the native palette and recorded metatiles.
    colors=[tuple(((v>>sh&31)<<3)|((v>>sh&31)>>2) for sh in [0,5,10])+(255,) for v in struct.unpack('<256H',palette)]
    tiles=decompress(r,0x86c6c44)+decompress(r,0x8681638)
    im=Image.new('RGBA',(224,160))
    for y in range(10):
     for x in range(14):
      n.call(0xbfb90,0x880808c,0,x+7,y+7)
      for layer,base in enumerate([0x02018000,0x02019000,0x0201a000]):
       for part,o in enumerate([0,2,64,66]):
        e=struct.unpack('<H',n.read(base+o,2))[0];tile=e&1023
        for py in range(8):
         for px in range(8):
          tx=7-px if e&0x400 else px;ty=7-py if e&0x800 else py
          i=tiles[tile*32+ty*4+tx//2]>>(tx%2*4)&15
          if layer and i==0:continue
          im.putpixel((x*16+part%2*8+px,y*16+part//2*8+py),colors[(e>>12)*16+i])
    if args.output: im.save(args.output)
    actual=Image.open(args.rendered).convert('RGBA')
    assert actual.size == im.size and actual.tobytes() == im.tobytes()
    print('Native layout, 140 grid IDs, 1680 tile entries, palette loading and all 35840 static pixels match the editor.')
    # WarpEvent stores destination map number before destination map group.
    assert struct.unpack_from('<hhBBBB', r, 0xB861C0) == (21, 16, 0, 0, 4, 42)
    assert struct.unpack_from('<hhBBBB', r, 0xBBBAD0) == (1, 6, 4, 0, 5, 42)
    source_header = 0x9F0788  # 42-4, the incoming room.
    layout = struct.unpack_from('<I', r, source_header)[0] - 0x08000000
    width = struct.unpack_from('<I', r, layout)[0]
    blocks = struct.unpack_from('<I', r, layout + 12)[0] - 0x08000000
    entry_cell = struct.unpack_from('<H', r, blocks + (6 * width + 1) * 2)[0]
    n.write(headerptr, r[source_header:source_header + 28])
    n.word(0x03005280, 1)
    n.word(0x03005284, 1)
    n.half(0x02010000, entry_cell)
    behavior = n.call(0xBDD40, 0, 0)
    assert behavior == 0 and n.call(0xD307C, behavior) == 0
    assert n.call(0xD307C, 0x69) == 1  # Positive control from the outdoor door.
    print('Incoming warp records exist, but the static 42-4 entrance cell fails the native warp-behavior predicate.')


if __name__ == "__main__":
    main()
