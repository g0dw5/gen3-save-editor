"""Save-dependent fishing UI, including reload races and untranslated ROM names.

Run Vite first. Uses only synthetic API responses, Playwright and Chrome.
"""
import copy
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, WORLD

catalog = copy.deepcopy(CATALOG)
catalog['profile']['feebas'] = {'map_id': '0-34'}
catalog['species'][0]['name'] = 'ROM Fish'
world = {**WORLD, 'maps': [
    {'id': '0-34', 'name': 'Route 119', 'width': 40, 'height': 140},
    {'id': '0-1', 'name': 'Other map', 'width': 10, 'height': 10},
], 'map_events': []}
base_save = {
    'trainer': {'name': 'TEST', 'gender': 0, 'tid': 1, 'sid': 0, 'hours': 0,
                'minutes': 0, 'seconds': 0, 'money': 0, 'coins': 0, 'registered_item': 0},
    'pokemon': [], 'boxes': [], 'bag': [], 'dex': [], 'active_slot': 0,
    'counter': 1, 'backup_valid': True, 'dirty': False,
    'can_undo': False, 'can_redo': False, 'changes': [],
}


def main():
    loaded = [0]
    delayed = []
    defer = [False]
    errors = []

    def report():
        return {'map_id': '0-34', 'species': 1, 'min_level': 20, 'max_level': 25,
                'percent': 50, 'seed': loaded[0] or None,
                'spots': [] if not loaded[0] else [{'x': loaded[0] + 10, 'y': 107, 'spot_id': 42}]}

    def respond(route):
        request = route.request.post_data_json
        command = request['command']
        if command == 'state': data = {'catalog': catalog, 'save': None}
        elif command == 'world': data = world
        elif command == 'species': data = {'species': catalog['species'][0], 'evolutions': [], 'learnset': [], 'encounters': []}
        elif command == 'sprite': data = {'url': ''}
        elif command == 'map_image': data = {'url': "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='640' height='2240'><rect width='640' height='2240' fill='%232f4544'/></svg>"}
        elif command == 'fishing_spots':
            data = report()
            if defer[0]:
                defer[0] = False; delayed.append((route, data)); return
        elif command == 'open_save':
            loaded[0] += 1; data = {**base_save, 'counter': loaded[0]}
        else: raise AssertionError(command)
        route.fulfill(content_type='application/json', body=json.dumps({'ok': True, 'data': data}))

    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport={'width': 1280, 'height': 960})
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror', lambda e: errors.append(str(e)))
        page.route('**/api', respond)
        page.goto(os.environ.get('GEN3_UI_URL', 'http://127.0.0.1:5173'))
        page.get_by_role('button', name='ROM reference', exact=True).click()
        dialog = page.get_by_role('dialog')
        dialog.get_by_role('button', name='View fishing map · 0-34').click()
        expect(dialog.locator('.fishing-info')).to_contain_text('Open a battery save')
        expect(dialog.get_by_role('checkbox', name='Save-specific fishing spots')).to_be_disabled()
        expect(dialog.locator('.map-marker.layer-fishing')).to_have_count(0)

        def load():
            # Floating reference windows are nonmodal; the top toolbar stays accessible.
            with page.expect_file_chooser() as chooser:
                page.get_by_role('button', name='Open save', exact=True).first.click(force=True)
            chooser.value.set_files({'name': 'synthetic.sav', 'mimeType': 'application/octet-stream', 'buffer': b'test'})

        load()
        pin = dialog.locator('.map-marker.layer-fishing')
        expect(pin).to_have_attribute('title', 'ROM Fish · (11, 107) · 50%')
        dialog.get_by_role('button', name='Locate water tile (11, 107)').click()
        expect(pin).to_have_class(__import__('re').compile('selected'))
        page.wait_for_function("document.querySelector('.map-viewport').scrollTop > 0")
        dialog.get_by_role('checkbox', name='Save-specific fishing spots').uncheck()
        expect(pin).to_have_count(0)
        dialog.get_by_role('button', name='Locate water tile (11, 107)').click()
        expect(pin).to_have_count(1)
        dialog.locator('.map-search input').fill('no match')
        expect(pin).to_have_count(0)
        dialog.locator('.map-search input').fill('ROM Fish')
        expect(pin).to_have_count(1)

        defer[0] = True; load()
        expect(pin).to_have_count(0)  # Clear previous seed while the request is pending.
        load()
        expect(pin).to_have_attribute('title', 'ROM Fish · (13, 107) · 50%')
        old, old_data = delayed.pop()
        old.fulfill(content_type='application/json', body=json.dumps({'ok': True, 'data': old_data}))
        expect(pin).to_have_attribute('title', 'ROM Fish · (13, 107) · 50%')
        dialog.locator('.reference-rows button').filter(has_text='Other map').click()
        expect(dialog.locator('.fishing-info')).to_have_count(0)
        expect(pin).to_have_count(0)
        assert not errors, errors
        browser.close()
        print('Fishing layer, no-save state, species link, search, navigation, save refresh and stale-response rejection passed.')


if __name__ == '__main__':
    main()
