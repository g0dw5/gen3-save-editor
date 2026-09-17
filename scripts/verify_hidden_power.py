"""Compare the UI calculator with both supported ROMs' native battle routine.

Requires Unicorn, Node.js with TypeScript stripping, GEN3_ROM_BW and GEN3_ROM_DP.
Runs read-only, without a save or emulator state. No game data is distributed.
"""
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess

from unicorn import Uc, UC_ARCH_ARM, UC_MODE_THUMB
from unicorn.arm_const import UC_ARM_REG_LR, UC_ARM_REG_PC, UC_ARM_REG_SP


def main():
    root = Path(__file__).resolve().parents[1]
    vectors = [[((selector >> (2 * i)) & 3) + high for i in range(6)]
               for high in (0, 28) for selector in range(4096)]
    javascript = """
        import { hiddenPower } from './ui/hiddenPower.ts';
        let input = ''; for await (const chunk of process.stdin) input += chunk;
        const rules = {move_id:237, formula:'gen3_to5'};
        process.stdout.write(JSON.stringify(JSON.parse(input).map(ivs => hiddenPower(rules, ivs))));
    """
    result = subprocess.run([os.environ.get('NODE', 'node'), '--experimental-strip-types',
                             '--input-type=module', '-e', javascript], cwd=root,
                            input=json.dumps(vectors), text=True, capture_output=True, check=True)
    calculated = json.loads(result.stdout)
    for version, digest in [('BW', '0d9b129f7dd76895f79bb47ad7dec2fe'),
                            ('DP', 'cb2940215f4dafb1bef133c3af379f44')]:
        rom = Path(os.environ['GEN3_ROM_' + version]).read_bytes()
        assert hashlib.md5(rom).hexdigest() == digest, 'unsupported ROM'
        # Move effect 135 dispatches a script starting with hiddenpowercalc (0xc1).
        assert rom[0x1900000 + 237 * 12] == 135
        script = struct.unpack_from('<I', rom, 0x1910000 + 135 * 4)[0] - 0x8000000
        assert rom[script] == 0xc1
        entry = struct.unpack_from('<I', rom, 0x310500 + 0xc1 * 4)[0]
        assert entry == 0x8054401
        cpu = Uc(UC_ARCH_ARM, UC_MODE_THUMB)
        for address, size in [(0x8000000, 0x2000000), (0x2000000, 0x40000),
                              (0x3000000, 0x8000)]:
            cpu.mem_map(address, size)
        cpu.mem_write(0x8000000, rom)
        cpu.mem_write(0x202449c, struct.pack('<I', 0x2030000))
        for index, (ivs, expected) in enumerate(zip(vectors, calculated)):
            attacker = index % 4
            cpu.mem_write(0x202420b, bytes([attacker]))
            cpu.mem_write(0x2024084 + attacker * 0x58 + 0x14,
                          struct.pack('<I', sum(iv << (5 * i) for i, iv in enumerate(ivs))))
            cpu.mem_write(0x2024400, b'\xff\xff')
            cpu.mem_write(0x2030013, b'\xff')
            cpu.reg_write(UC_ARM_REG_SP, 0x3007000)
            cpu.reg_write(UC_ARM_REG_LR, 0x8000001)
            cpu.emu_start(entry, 0x8000000, count=5000)
            assert cpu.reg_read(UC_ARM_REG_PC) == 0x8000000, 'routine did not return'
            actual = {'type': cpu.mem_read(0x2030013, 1)[0] & 63,
                      'power': int.from_bytes(cpu.mem_read(0x2024400, 2), 'little')}
            assert actual == expected, (version, ivs, actual, expected)
        print(f'{version}: {len(vectors)} native Hidden Power comparisons passed')


if __name__ == '__main__':
    main()
