#!/usr/bin/env python3
"""Validate the compact official reference independently of any private ROM."""
import json
from pathlib import Path


def main():
    data = json.loads((Path(__file__).resolve().parent.parent / 'ui/data/official-stats.json').read_text())
    assert data['schema'] == 1
    rows = data['rows']
    assert {row[0] for row in rows} == set(range(1, 1026))
    assert len({(row[0], row[2]) for row in rows}) == len(rows)
    assert len(rows) > 1200
    for dex, name, form, generation, *stats in rows:
        assert name and isinstance(form, str)
        assert generation in (7, 8, 9)
        assert len(stats) == 6 and all(isinstance(v, int) and 0 < v <= 255 for v in stats)
    ordinary = next(row for row in rows if row[:3] == [9, '水箭龟', ''])
    mega = next(row for row in rows if row[:3] == [9, '水箭龟', '超级水箭龟'])
    assert ordinary[4:] == [79, 83, 100, 78, 85, 105] and sum(ordinary[4:]) == 530
    assert sum(mega[4:]) == 630
    # Catch stale names producing duplicate ordinary reference keys.
    for dex in (57, 233, 474, 563, 594, 675, 721, 778):
        assert sum(row[0] == dex and not row[2] for row in rows) == 1
    for source in data['sources']:
        assert source['url'].startswith('https://wiki.52poke.com/')
        assert source['revision'] > 0 and len(source['sha256']) == 64
    print(f'Passed: {len(rows)} official species/form rows, 1,025 dex IDs, unique keys, numeric bounds, form distinction and source provenance.')


if __name__ == '__main__': main()
