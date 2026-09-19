#!/usr/bin/env python3
"""Build a compact, attributed reference of numeric stats from 52Poké Wiki.

No ROM inputs or game artwork. Python standard library only. HTML is cached
outside the distributable tree; older generations fill absent latest-gen rows.
"""
import argparse
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import urllib.parse
import urllib.request


class Rows(HTMLParser):
    def __init__(self):
        super().__init__()
        self.row = None
        self.cell = None
        self.rows = []

    def handle_starttag(self, tag, attrs):
        if tag == 'tr': self.row = []
        elif tag == 'td' and self.row is not None: self.cell = ''
        elif tag == 'br' and self.cell is not None: self.cell += '\n'

    def handle_data(self, text):
        if self.cell is not None: self.cell += text

    def handle_endtag(self, tag):
        if tag == 'td' and self.cell is not None:
            self.row.append(self.cell.strip())
            self.cell = None
        elif tag == 'tr' and self.row is not None:
            self.rows.append(self.row)
            self.row = None


def parse(html, generation):
    parser = Rows()
    parser.feed(html)
    rows = []
    for cells in parser.rows:
        if len(cells) != 11 or not re.fullmatch(r'\d{3,4}', cells[0]): continue
        try: values = [int(value) for value in cells[3:10]]
        except ValueError: continue
        hp, attack, defense, special_attack, special_defense, speed, total = values
        assert sum(values[:6]) == total, cells
        assert all(0 < value <= 255 for value in values[:6]), cells
        labels = [line.strip() for line in cells[2].splitlines() if line.strip()]
        name, form = labels[0], ' '.join(labels[1:])
        # Same ordering as Gen III ROM data, not the wiki's display order.
        rows.append([int(cells[0]), name, form, generation,
                     hp, attack, defense, speed, special_attack, special_defense])
    assert len(rows) > 500, ('unexpected wiki format', generation, len(rows))
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, default=Path('.local/reference-sources'))
    parser.add_argument('--output', type=Path, default=Path('ui/data/official-stats.json'))
    args = parser.parse_args()
    args.cache.mkdir(parents=True, exist_ok=True)
    records, sources = {}, []
    for generation, numeral in [(9, '九'), (8, '八'), (7, '七')]:
        url = 'https://wiki.52poke.com/wiki/' + urllib.parse.quote(f'种族值列表（第{numeral}世代）')
        path = args.cache / f'official-gen{generation}.html'
        if not path.exists():
            request = urllib.request.Request(url, headers={'User-Agent': 'Gen3RomHackEditor-reference-builder/0.2'})
            path.write_bytes(urllib.request.urlopen(request, timeout=45).read())
        raw = path.read_bytes()
        html = raw.decode('utf-8')
        revision = re.search(r'"wgRevisionId"\s*:\s*(\d+)', html)
        revision = int(revision[1]) if revision else None
        sources.append({'generation': generation, 'url': url, 'revision': revision,
                        'sha256': hashlib.sha256(raw).hexdigest()})
        rows = parse(html, generation)
        # Official names can change across generations; identity is dex + form.
        for row in rows: records.setdefault((row[0], row[2]), row)
        print(generation, len(rows), revision)
    data = {'schema': 1, 'sources': sources, 'rows': sorted(records.values(), key=lambda row: (row[0], row[2]))}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(data, ensure_ascii=False, separators=(',', ':')) + '\n')
    print(args.output, len(data['rows']), args.output.stat().st_size)


if __name__ == '__main__': main()
