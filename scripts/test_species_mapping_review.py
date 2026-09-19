"""Persistence and rejection tests; only synthetic temporary configurations."""
import json
from pathlib import Path
import tempfile
import unittest
from review_species_mappings import ReviewStore


class ReviewTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.catalogs = {'a': {}, 'b': {}}
        self.official = {'rows': [[150, 'Mewtwo', 'X'], [150, 'Mewtwo', 'Y']]}
        for key in ['a', 'b']:
            (self.root / f'{key}.json').write_text(json.dumps(dict(schema=1, profile_id=key, rom_md5=key, revision=0, entries={'1': dict(status='pending', target='150:X', note='')})))
        self.store = ReviewStore(self.root, self.official, self.catalogs, self.root / 'backups', self.root / 'output')

    def change(self, status='direct', target='150:X'):
        return {'1': dict(status=status, target=target, note='review')}

    def test_persistence_and_isolation(self):
        self.store.update('a', 0, self.change())
        restored = ReviewStore(self.root, self.official, self.catalogs, self.root / 'backups')
        state = {g['config']['rom_md5']: g['config'] for g in restored.state()['games']}
        self.assertEqual(state['a']['entries']['1']['status'], 'direct')
        self.assertEqual(state['b']['entries']['1']['status'], 'pending')
        self.assertEqual(len(list((self.root / 'backups').glob('*.json'))), 1)
        self.store.update('a', 1, self.change('none', None))
        self.assertIsNone(json.loads((self.root / 'a.json').read_text())['entries']['1']['target'])

    def test_publish_excludes_pending_and_review_metadata(self):
        self.assertEqual(self.store.publish('a', 0)['entries'], {})
        self.store.update('a', 0, self.change())
        result = self.store.publish('a', 1)
        self.assertEqual(result['entries'], {'1': {'status': 'direct', 'target': '150:X'}})
        self.assertEqual(json.loads((self.root / 'output/a.json').read_text()), result)
        with self.assertRaises(FileExistsError): self.store.publish('a', 0)

    def test_rejects_stale_revision(self):
        self.store.update('a', 0, self.change())
        with self.assertRaises(FileExistsError): self.store.update('a', 0, self.change(target='150:Y'))

    def test_invalid_batch_is_atomic(self):
        before = (self.root / 'a.json').read_bytes()
        invalids = [self.change('direct', None), self.change('none', '150:X'), self.change(target='wrong'), self.change('invented'), {'2': self.change()['1']}, {'1': {'status': 'none'}}, {**self.change(), 'invalid': self.change()['1']}]
        for change in invalids:
            with self.assertRaises(ValueError): self.store.update('a', 0, change)
            self.assertEqual((self.root / 'a.json').read_bytes(), before)
        with self.assertRaises(ValueError): self.store.update('../a', 0, self.change())


if __name__ == '__main__': unittest.main()
