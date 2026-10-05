"""Read-only native probes for query prerequisites and encounter periods.

Set GEN3_ROM_BW/DP/MERCURY12 to exact private ROMs. Requires Unicorn. Synthetic
RAM only; the clock probes explicitly model the forced-night flag as unset.
They do not establish the virtual clock's save location or current effective time.
"""
import hashlib
import json
import os
import struct
from pathlib import Path
from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_R0, UC_ARM_REG_PC, UC_ARM_REG_LR
from verify_rocket_battle_forms import Native

FINGERPRINTS = {
    'BW': '0d9b129f7dd76895f79bb47ad7dec2fe',
    'DP': 'cb2940215f4dafb1bef133c3af379f44',
    'MERCURY12': 'f323df1792ac68462a34b42fe8571533',
}


def verify_flags(rom):
    cpu = Native(rom)
    pointer = struct.unpack_from('<I', rom, 0x9d714)[0]
    cpu.word(pointer, 0x02030000)
    count = 0
    for fill in [0, 0x55, 0xaa, 0xff]:
        block = bytes([fill]) * 0x3000
        cpu.write(0x02030000, block)
        for flag in [1, 7, 8, 15, 16, 0x1f4, 0x3ff, 0x1000, 0x1800, 0x3fff]:
            expected = (block[0x1270 + (flag >> 3)] >> (flag & 7)) & 1
            assert cpu.call(0x9d790, flag) == expected
            count += 1
        for var in [0x4000, 0x4001, 0x403e, 0x40ff]:
            expected = struct.unpack_from('<H', block, 0x139c + 2 * (var - 0x4000))[0]
            assert cpu.call(0x9d694, var) == expected
            count += 1
    assert cpu.read(0x02030000, len(block)) == block
    return {'flag_and_variable_comparisons': count, 'save_block_unchanged': True}


def verify_periods(rom):
    cpu = Native(rom)
    def forced_night_unset(uc, address, size, user):
        if address == 0x0806e6d0:
            assert uc.reg_read(UC_ARM_REG_R0) == 0x1041
            uc.reg_write(UC_ARM_REG_R0, 0)
            uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))
    cpu.cpu.hook_add(UC_HOOK_CODE, forced_night_unset)
    for hour in range(24):
        cpu.write(0x03005ea0, bytes([0, 0, 0, 1, 1, 0, hour, 0, 0]))
        morning = cpu.call(0x1d20de0)
        dusk = cpu.call(0x1d20df8)
        night = cpu.call(0x1d20814)
        assert bool(morning) == (4 <= hour <= 7), hour
        assert bool(dusk) == (17 <= hour <= 19), hour
        assert bool(night) == (hour < 4 or hour >= 20), hour
    return {'native_period_comparisons': 72, 'forced_night_flag': 'simulated unset', 'current_clock': 'not verified'}


def main():
    result = {}
    for name, expected in FINGERPRINTS.items():
        data = Path(os.environ['GEN3_ROM_' + name]).read_bytes()
        assert hashlib.md5(data).hexdigest() == expected
        result[name] = verify_periods(data) if name == 'MERCURY12' else verify_flags(data)
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
