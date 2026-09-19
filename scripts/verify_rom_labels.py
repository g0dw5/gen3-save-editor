"""Validate runtime labels and all nature modifiers against the three native ROMs.

Requires private GEN3_ROM_BW/DP/ROCKET, GEN3_CLI and Unicorn. No save is loaded.
"""
import json
import os
import struct
import subprocess
from pathlib import Path
from verify_rocket_battle_forms import Native


def main():
    for key, function, references in [
        ('GEN3_ROM_BW', 0x6d8d4, (0x73188, 0x166f4, 0x6d914)),
        ('GEN3_ROM_DP', 0x6d8d4, (0x73188, 0x166f4, 0x6d914)),
        ('GEN3_ROM_ROCKET', 0x9afb4, (0xa1ea4, 0x16780, 0x9aff4)),
    ]:
        path = os.environ[key]
        r = Path(path).read_bytes()
        catalog = json.loads(subprocess.check_output([os.environ['GEN3_CLI'], 'catalog', path]))
        p = catalog['profile']
        for literal, table in zip(references, (p['nature_names'], p['type_names']['offset'], p['nature_effects'])):
            assert struct.unpack_from('<I', r, literal)[0] == table + 0x8000000
        names = [n['name'] for n in catalog['natures']]
        assert len(names) == len(set(names)) == 25
        assert all(names) and all(catalog['type_names'])
        assert ('内敛' == names[15]) if key.endswith('ROCKET') else names[15].startswith('保守(')
        native = Native(r)
        probes = 0
        for n in catalog['natures']:
            expected = list(struct.unpack_from('<5b', r, p['nature_effects'] + n['id'] * 5))
            assert n['stat_changes'] == expected
            for stat in range(6):
                for value in (1, 9, 10, 99, 100, 333, 999):
                    actual = native.call(function, n['id'], value, stat)
                    change = expected[stat-1] if stat else 0
                    product = value * (100 + change * 10)
                    if p['nature_product_u16']: product &= 0xffff
                    wanted = product // 100 if change else value
                    assert actual == wanted, (key, n['id'], stat, value, actual, wanted)
                    probes += 1
        print(f'{key}: 25 runtime nature names, {len(catalog["type_names"])} types, {probes} native stat probes passed')


if __name__ == '__main__':
    main()
