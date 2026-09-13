"""Refresh localized aliases while preserving reviewed, profile-specific IDs.

No ROM is needed. This never infers a canonical identity from a ROM index.
"""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
from urllib.request import urlopen

COMMIT = '88f332f7a68a77162c64a48c230b19420e1ed3be'
SOURCES = {
    'moves': ('move_names.csv', 'move_id', '99e23ee38ea53d1473474d463b87651deac3cd4928750f8186feae66da45c147'),
    'items': ('item_names.csv', 'item_id', '7b1b4fe6edf7946110050a5dddabf62c3a1dc3e1616099c4ae97abcb5ebd0f97'),
    'species': ('pokemon_species_names.csv', 'pokemon_species_id', '820cde17074cdb1c2b0595c997fb8f998e773bd5da3bb525dec85703c86c5fd9'),
}
LANGUAGES = {'12': 'zh', '9': 'en', '4': 'zhHant'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Verify without writing')
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / 'ui/data/dark-phantom-aliases.json'
    existing = json.loads(path.read_text())
    refreshed = {}
    for kind, (filename, key, expected_hash) in SOURCES.items():
        url = f'https://raw.githubusercontent.com/PokeAPI/pokeapi/{COMMIT}/data/v2/csv/{filename}'
        with urlopen(url, timeout=60) as response:
            raw = response.read()
        if hashlib.sha256(raw).hexdigest() != expected_hash:
            raise ValueError(f'Unexpected source content: {filename}')
        names = {}
        for row in csv.DictReader(io.StringIO(raw.decode('utf-8'))):
            lang = LANGUAGES.get(row['local_language_id'])
            if lang:
                names.setdefault(int(row[key]), {})[lang] = row['name']
        refreshed[kind] = {
            rom_id: {'canonical_id': row['canonical_id'], **names[row['canonical_id']]}
            for rom_id, row in existing[kind].items()
        }
    if args.check:
        if existing != refreshed:
            raise ValueError('Alias text differs from pinned sources')
        print('Verified aliases against pinned sources; ROM mappings unchanged.')
    else:
        path.write_text(json.dumps(refreshed, ensure_ascii=False, indent=2) + '\n')
        print('Updated localized names. Run npm run format before committing.')


if __name__ == '__main__':
    main()
