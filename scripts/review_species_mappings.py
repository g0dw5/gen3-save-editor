#!/usr/bin/env python3
"""Local, persistent species identity review. Reads ROMs; writes mapping JSON only."""
import argparse
import copy
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import secrets
import subprocess
import threading
from urllib.parse import urlparse, parse_qs
from functools import lru_cache

ROOT = Path(__file__).resolve().parent.parent
STATUSES = {'pending', 'direct', 'comparison', 'none'}


def compiled_mapping(config):
    """Only reviewed identities belong in the editor's distributable configuration."""
    return dict(schema=1, profile_id=config['profile_id'], rom_md5=config['rom_md5'],
                entries={species: dict(status=entry['status'], target=entry['target'])
                         for species, entry in config['entries'].items()
                         if entry['status'] in {'direct', 'comparison', 'none'}})


def atomic_json(path, value):
    temporary = path.with_name(path.name + '.tmp')
    temporary.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    temporary.replace(path)


@lru_cache(maxsize=1)
def form_module():
    return subprocess.check_output(['node', str(ROOT / 'scripts/build_review_form_module.mjs')], cwd=ROOT)


class ReviewStore:
    def __init__(self, directory, official, catalogs, backup, output=None, sprite_loader=None):
        self.directory, self.official, self.catalogs, self.backup = directory, official, catalogs, backup
        self.output = output
        self.sprite_loader = sprite_loader
        self.sprite_cache = {}
        self.keys = {f'{r[0]}:{r[2]}' for r in official['rows']}
        self.lock = threading.Lock()
        self.paths = {}
        for path in directory.glob('*.json'):
            config = json.loads(path.read_text())
            if config.get('schema') != 1:
                raise ValueError('Unsupported mapping schema')
            md5 = config['rom_md5']
            if md5 in self.paths:
                raise ValueError('Duplicate ROM fingerprint')
            self.paths[md5] = path

    def state(self):
        with self.lock:
            return {'form_rules': json.loads((ROOT / 'ui/data/species-form-rules.json').read_text()), 'official': self.official, 'games': [dict(
                config=json.loads(self.paths[md5].read_text()), catalog=catalog,
            ) for md5, catalog in self.catalogs.items() if md5 in self.paths]}

    def sprite(self, md5, species):
        if md5 not in self.paths or md5 not in self.catalogs or not self.sprite_loader:
            raise ValueError('Sprite unavailable for this ROM')
        if str(species) not in json.loads(self.paths[md5].read_text())['entries']:
            raise ValueError('Unknown species')
        key = (md5, species)
        if key not in self.sprite_cache:
            self.sprite_cache[key] = self.sprite_loader(md5, species)
        return self.sprite_cache[key]

    def publish(self, md5, revision):
        with self.lock:
            if md5 not in self.catalogs or md5 not in self.paths or self.output is None:
                raise ValueError('Unknown ROM or no output directory')
            config = json.loads(self.paths[md5].read_text())
            if config['revision'] != revision:
                raise FileExistsError('Review changed elsewhere. Reload before publishing.')
            compiled = compiled_mapping(config)
            self.output.mkdir(parents=True, exist_ok=True)
            path = self.output / self.paths[md5].name
            atomic_json(path, compiled)
            return compiled

    def update(self, md5, revision, changes):
        with self.lock:
            if md5 not in self.catalogs or md5 not in self.paths:
                raise ValueError('Unknown ROM')
            path = self.paths[md5]
            before = json.loads(path.read_text())
            if before['revision'] != revision:
                raise FileExistsError('Review changed elsewhere. Reload before saving.')
            if not isinstance(changes, dict) or not 1 <= len(changes) <= 1500:
                raise ValueError('Invalid changes')
            config = copy.deepcopy(before)
            for species, change in changes.items():
                if species not in config['entries'] or not isinstance(change, dict):
                    raise ValueError('Unknown species')
                if set(change) != {'status', 'target', 'note'}:
                    raise ValueError('Invalid fields')
                status, target, note = change['status'], change['target'], change['note']
                if status not in STATUSES or not isinstance(note, str) or len(note) > 2000:
                    raise ValueError('Invalid decision or note')
                if target is not None and (not isinstance(target, str) or target not in self.keys):
                    raise ValueError('Unknown official reference')
                if status in {'direct', 'comparison'} and target is None:
                    raise ValueError('Select an official reference first')
                if status == 'none' and target is not None:
                    raise ValueError('No-counterpart decision must have no target')
                config['entries'][species].update(change)
            config['revision'] += 1
            self.backup.mkdir(parents=True, exist_ok=True)
            # Content hash prevents overwriting an earlier backup after a Git rollback.
            digest = hashlib.sha256(path.read_bytes()).hexdigest()[:12]
            backup = self.backup / f'{md5}-{before["revision"]}-{digest}.json'
            if not backup.exists():
                atomic_json(backup, before)
            atomic_json(path, config)
            return config


def handler_for(store, token, port):
    class Handler(BaseHTTPRequestHandler):
        def send(self, code, value, mime='application/json; charset=utf-8'):
            data = value if isinstance(value, bytes) else json.dumps(value, ensure_ascii=False).encode()
            self.send_response(code)
            self.send_header('Content-Type', mime)
            self.send_header('Content-Length', str(len(data)))
            self.send_header('Cache-Control', 'no-store')
            self.send_header('X-Content-Type-Options', 'nosniff')
            self.end_headers()
            self.wfile.write(data)

        def allowed(self):
            return self.headers.get('Host') == f'127.0.0.1:{port}'

        def do_GET(self):
            if not self.allowed():
                return self.send(403, {'error': 'Invalid host'})
            path = urlparse(self.path).path
            if path == '/':
                page = (ROOT / 'scripts/mapping-review/index.html').read_text().replace('__TOKEN__', token)
                return self.send(200, page.encode(), 'text/html; charset=utf-8')
            if path == '/form-labels.js':
                return self.send(200, form_module(), 'text/javascript; charset=utf-8')
            if path == '/sprite':
                try:
                    query = parse_qs(urlparse(self.path).query)
                    if query.get('token') != [token]:
                        return self.send(403, {'error': 'Invalid review session'})
                    return self.send(200, store.sprite(query['md5'][0], int(query['species'][0])), 'image/png')
                except (ValueError, KeyError, IndexError, subprocess.SubprocessError):
                    return self.send(404, {'error': 'Sprite unavailable'})
            if path == '/state' and self.headers.get('X-Review-Token') == token:
                return self.send(200, store.state())
            self.send(404, {'error': 'Not found'})

        def do_POST(self):
            if (not self.allowed() or self.headers.get('X-Review-Token') != token
                or self.headers.get('Origin') != f'http://127.0.0.1:{port}'):
                return self.send(403, {'error': 'Invalid review session'})
            if self.path not in {'/decision', '/publish'}:
                return self.send(404, {'error': 'Not found'})
            try:
                length = int(self.headers.get('Content-Length', '0'))
                if not 0 < length <= 1_000_000:
                    raise ValueError('Invalid request size')
                request = json.loads(self.rfile.read(length))
                result = (store.publish(request['md5'], request['revision']) if self.path == '/publish'
                          else store.update(request['md5'], request['revision'], request['changes']))
                self.send(200, result)
            except FileExistsError as error:
                self.send(409, {'error': str(error)})
            except (ValueError, KeyError, TypeError) as error:
                self.send(400, {'error': str(error)})
            except OSError:
                self.send(500, {'error': 'Could not persist configuration. Check disk permissions/space.'})

        def log_message(self, *_):
            pass
    return Handler


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rom', type=Path, action='append', required=True)
    parser.add_argument('--cli', type=Path, default=ROOT / 'target/debug/gen3')
    parser.add_argument('--port', type=int, default=8791)
    args = parser.parse_args()
    catalogs, rom_paths = {}, {}
    for rom in args.rom:
        catalog = json.loads(subprocess.check_output([str(args.cli), 'catalog', str(rom)]))
        actual = hashlib.md5(rom.read_bytes()).hexdigest()
        if actual != catalog['profile']['md5']:
            raise ValueError('ROM fingerprint mismatch')
        catalogs[actual] = catalog
        rom_paths[actual] = rom
    def load_sprite(md5, species):
        rom = rom_paths[md5]
        if hashlib.md5(rom.read_bytes()).hexdigest() != md5:
            raise ValueError('ROM changed on disk')
        return subprocess.check_output([str(args.cli), 'sprite', str(rom), str(species)])

    form_module()  # Fail before opening the server if Node dependencies are missing.
    store = ReviewStore(ROOT / 'config/species-mapping-reviews',
                        json.loads((ROOT / 'ui/data/official-stats.json').read_text()),
                        catalogs, ROOT / '.local/mapping-review-backups', ROOT / 'ui/data/species-mappings', load_sprite)
    if any(md5 not in store.paths for md5 in catalogs):
        raise ValueError('No game configuration exists for one of the selected ROMs')
    token = secrets.token_urlsafe(32)
    server = ThreadingHTTPServer(('127.0.0.1', args.port), handler_for(store, token, args.port))
    print(f'Review: http://127.0.0.1:{args.port}', flush=True)
    server.serve_forever()


if __name__ == '__main__':
    main()
