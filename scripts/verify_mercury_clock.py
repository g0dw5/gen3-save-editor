#!/usr/bin/env python3
"""Read-only Mercury 1.33 virtual-clock and jump-time native verification.

Requires Unicorn. Exact ROM and optional SAV stay private and unchanged. Native
routines operate only on isolated RAM. --output produces private parity vectors
for the opt-in Rust test (GEN3_CLOCK_PROBES). No device-clock inference.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_PC, UC_ARM_REG_LR
from verify_rocket_battle_forms import Native

MD5 = '5ffb1cbd5c28cda9b987b3b445da68e0'
BASE = 0x0203B174
CLOCK = 0x03005EA0
WORDS = 0x5DE


def set_flag(cpu, flag, value):
    offset = (flag - 0x900) // 8
    current = cpu.read(BASE + offset, 1)[0]
    mask = 1 << (flag % 8)
    cpu.write(BASE + offset, bytes([(current | mask) if value else (current & ~mask)]))


def setup(cpu, values, forced=False, speed=1):
    cpu.write(0x02000000, bytes(0x40000))
    cpu.write(0x03000000, bytes(0x8000))
    set_flag(cpu, 0x1335, True)
    set_flag(cpu, 0x1041, forced)
    cpu.write(BASE + WORDS, struct.pack('<5H', *values, speed))
    # The tick routine requires the native active-play gate.
    cpu.write(0x03000E7C, b'\x01')


def clock(cpu):
    raw = cpu.read(CLOCK, 10)
    return dict(year=int.from_bytes(raw[:2], 'little'), month=raw[3], day=raw[4],
                weekday=raw[5], hour=raw[6], minute=raw[7], second=raw[8])


def packed(cpu):
    return list(struct.unpack('<4H', cpu.read(BASE + WORDS, 8)))


def saved_extensions(data):
    sizes = [0xF24,0xFF0,0xFF0,0xFF0,0xD98,*([0xFF0]*8),0x450]
    banks = []
    for bank in range(2):
        sections = {}
        counter = None
        for i in range(14):
            sec = data[(bank * 14 + i) * 4096:(bank * 14 + i + 1) * 4096]
            ident, checksum, magic, number = struct.unpack_from('<HHII', sec, 0xFF4)
            if ident >= 14 or ident in sections or magic != 0x08012025:
                break
            total = sum(struct.unpack('<' + 'I' * (sizes[ident] // 4), sec[:sizes[ident]])) & 0xFFFFFFFF
            if ((total >> 16) + (total & 65535)) & 65535 != checksum or counter is not None and counter != number:
                break
            sections[ident] = sec
            counter = number
        if len(sections) == 14:
            banks.append((counter, sections))
    assert banks, 'no valid SAVE bank'
    # Supplied files have small counters; reject wrap ambiguity rather than invent selection.
    assert all(n < 0x80000000 for n, _ in banks)
    counter, sections = max(banks, key=lambda b: b[0])
    extra = b''.join(sections[i][sizes[i]:0xFF0] for i in range(14))
    return counter, extra + data[30*4096:30*4096+0xFF0] + data[31*4096:31*4096+0xFF0]


def verify(rom, save=None):
    assert hashlib.md5(rom).hexdigest() == MD5
    cpu = Native(rom)
    vectors = []
    for hour in range(24):
        for weekday in range(7):
            for forced in [False, True]:
                values = [2026, 0xA05, hour << 8 | 24, 17 << 8 | weekday]
                setup(cpu, values, forced)
                before = cpu.read(BASE, 0x700)
                cpu.call(0x1D5C840)
                expected = clock(cpu)
                assert expected == dict(year=2026,month=10,day=5,weekday=weekday,hour=hour,minute=24,second=17)
                night = cpu.call(0x1D21108)
                morning = cpu.call(0x1D216D4)
                dusk = cpu.call(0x1D216EC)
                period = 'night' if night else 'morning' if morning else 'dusk' if dusk else 'day'
                assert period == ('night' if forced or hour < 4 or hour >= 20 else 'morning' if hour < 8 else 'day' if hour < 17 else 'dusk')
                # Clock restore/queries retain these packed words and flags; speed normalization may write only its own word.
                assert cpu.read(BASE, 0x700) == before
                cpu.call(0x1D5C408)
                assert packed(cpu) == values
                vectors.append(dict(words=values, forced=forced, speed=1, expected=expected, period=period))
    # Native month table and leap semantics, including the century exception.
    calendar = 0
    for year in [2000, 2004, 2026, 2100, 2400, 3199]:
        for month in range(1, 13):
            base = struct.unpack_from('<I', rom, 0x1DE4D0C + 4*(month-1))[0] & 255
            days = base + int(month == 2 and year % 4 == 0 and (year % 100 != 0 or year % 400 == 0))
            assert cpu.call(0x1D5C3C4, year, month) == days
            calendar += 1
    jumps = []
    for year,month,day,weekday,start,target,force_next in [
        (2026,10,5,1,10,17,0), (2026,10,5,1,20,4,0),
        (2026,10,5,1,4,4,0), (2026,10,5,1,4,20,1),
        (2024,2,28,3,20,4,0), (2024,2,29,4,20,4,0),
        (2100,2,28,0,20,4,0), (2026,12,31,4,20,4,0),
    ]:
        values = [year, month<<8|day, start<<8|31, 29<<8|weekday]
        setup(cpu, values)
        assert cpu.call(0x1D5C98C, target, force_next) == 1
        result = clock(cpu)
        assert result['hour'] == (start if force_next else target), result
        assert (result['minute'], result['second']) == ((31,29) if force_next else (0,0)), result
        assert packed(cpu) == [result['year'], result['month']<<8|result['day'], result['hour']<<8|result['minute'], result['second']<<8|result['weekday']]
        jumps.append(dict(before=dict(year=year,month=month,day=day,weekday=weekday,hour=start),target=target,force_next_day=bool(force_next),after=result))
    assert jumps[1]['after']['day'] == 6 and jumps[2]['after']['day'] == 6
    assert jumps[3]['after']['day'] == 6 and jumps[4]['after']['day'] == 29
    assert jumps[5]['after']['month'] == jumps[6]['after']['month'] == 3
    assert jumps[7]['after']['year'] == 2027
    speeds = []
    for speed in [0,1,2,3,5,10,30,60,255]:
        values = [2026,0xA05,0xA18,0x1101]
        setup(cpu, values, speed=speed)
        cpu.call(0x1D5C840)
        normalized = speed if speed in [1,2,5,10,30,60] else 1
        assert cpu.call(0x1D5C45C) == normalized
        cpu.call(0x1D5C754)
        assert cpu.read(CLOCK+9,1)[0] == normalized % 60
        assert clock(cpu)['second'] == (18 if normalized == 60 else 17)
        assert packed(cpu) == (values if normalized != 60 else [2026,0xA05,0xA18,0x1201])
        speeds.append(dict(input=speed,effective=normalized))
    invalid = []
    # Identify the native fallback without faking an RTC reading.
    fallbacks = []
    def rtc_fallback(uc, address, _size, _data):
        if address == 0x9D5C6B8:
            fallbacks.append(True)
            uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))
    hook = cpu.cpu.hook_add(UC_HOOK_CODE, rtc_fallback)
    for values in [[1999,0xA05,0,0],[3200,0xA05,0,0],[2026,0xA00,0,0],
                   [2026,0xD01,0,0],[2100,0x21D,0,0],[2026,0xA05,0x1800,0],
                   [2026,0xA05,0x3C,0],[2026,0xA05,0,0x3C00],[2026,0xA05,0,7]]:
        setup(cpu, values)
        before = len(fallbacks)
        cpu.call(0x1D5C840)
        assert len(fallbacks) == before+1
        invalid.append(values)
    cpu.cpu.hook_del(hook)
    actual = None
    if save is not None:
        counter,extra = saved_extensions(save)
        assert len(extra)>0x5E8
        cpu.write(BASE,extra)
        assert cpu.call(0x6E6D0,0x1335) == 1, 'current file does not use virtual time'
        values = list(struct.unpack_from('<4H',extra,WORDS))
        cpu.call(0x1D5C840)
        actual=dict(counter=counter,words=values,clock=clock(cpu),forced_night=bool(cpu.call(0x6E6D0,0x1041)))
        assert packed(cpu)==values
    return dict(rom_md5=MD5,restore_vectors=vectors,month_comparisons=calendar,
                jump_vectors=jumps,speed_vectors=speeds,invalid_vectors=invalid,current_save=actual)


if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--rom',type=Path,required=True);p.add_argument('--save',type=Path);p.add_argument('--output',type=Path)
    args=p.parse_args()
    if args.output:
        inputs = [args.rom] + ([args.save] if args.save else [])
        if any(args.output.resolve() == path.resolve()
               or (args.output.exists() and args.output.samefile(path)) for path in inputs):
            p.error('--output must not replace a ROM or SAVE input')
    rom=args.rom.read_bytes();save=args.save.read_bytes() if args.save else None
    original=(hashlib.sha256(rom).hexdigest(),hashlib.sha256(save).hexdigest() if save else None)
    result=verify(rom,save)
    result["save_sha256"] = original[1]
    result["rom_sha256"] = original[0]
    if args.output:args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(result,indent=2)+'\n')
    assert hashlib.sha256(args.rom.read_bytes()).hexdigest()==original[0]
    if args.save:assert hashlib.sha256(args.save.read_bytes()).hexdigest()==original[1]
    print(json.dumps(dict(restore_vectors=len(result['restore_vectors']),month_comparisons=result['month_comparisons'],
                         jump_vectors=result['jump_vectors'],speed_vectors=result['speed_vectors'],invalid_vectors=len(result['invalid_vectors']),current_save=result['current_save'],inputs_unchanged=True),indent=2))
