"""Cross-profile UI controls and asynchronous world isolation. Synthetic data only.

Run Vite first; no ROM, user save or native bridge is required.
"""
import copy
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_editor_navigation import pokemon
from test_reference_navigation import CATALOG, WORLD, species


def main():
    catalog = copy.deepcopy(CATALOG)
    catalog['items'] = [{'id': 0, 'name': '', 'tm_move': None}, {'id':1, 'name':'Test item', 'tm_move':None}]
    catalog['moves'] = [{'id': 0, 'name': '', 'pp': 0}]
    legacy = dict(save_edit=True, rom_edit=True, world=True, dex=True,
                  complete_learnsets=True, individual_sprites=True, battle_forms=False)
    catalog['profile']['capabilities'] = legacy
    rocket = copy.deepcopy(catalog)
    rocket['profile'].update(id='rocket', md5='rocket', label='Rocket test', capabilities={**legacy, 'battle_forms': True}, max_level=150)
    rocket['editor_rules'] = dict(balls=[1,2,3,4], nature_override=True, contest_ranks=[1,1,4,4,4])
    dp = copy.deepcopy(catalog)
    dp['profile'].update(id='dp', md5='dp', label='DP test')
    save = dict(trainer={'name': 'TEST'}, pokemon=[pokemon(1, {'kind': 'party', 'slot': 0})],
                boxes=[{'index': i, 'name': f'Box {i+1}', 'count': 0} for i in range(14)],
                bag=[{'pocket': 'pc', 'slot': 0, 'item': 1, 'quantity': 1}], dex=[],
                dirty=False, can_undo=False, can_redo=False, changes=[], backup_valid=True, active_slot=0, counter=1)
    requests, errors, worlds = [], [], []
    current = [catalog]
    targets = [rocket, dp]

    def reply(route, data):
        route.fulfill(content_type='application/json', body=json.dumps({'ok': True, 'data': data}))

    def respond(route):
        req = route.request.post_data_json
        command = req['command']; requests.append(command)
        if command == 'state': data = {'catalog': current[0], 'save': save}
        elif command in ('sprite', 'map_image', 'object_sprite', 'trainer_sprite'): data = {'url': ''}
        elif command == 'world':
            worlds.append(route)
            return
        elif command == 'species':
            data = {'species': species(req['payload']['id']), 'evolutions': [], 'learnset': [], 'encounters': [],
                    'encounters_verified': current[0]['profile']['capabilities']['world'],
                    'battle_forms': [{'source': 1, 'target': 2, 'kind': 'mega', 'trigger': {'kind': 'held_item', 'id': 0}, 'offset': 42}] if current[0] == rocket else []}
        elif command == 'open_rom':
            current[0] = targets.pop(0)
            data = {'catalog': current[0]}
        elif command == 'open_save': data = save
        elif command == 'action':
            action = req['payload']['action']
            assert action['type'] == 'pokemon', action
            save['pokemon'][0]['pokemon'].update(action['patch'])
            data = {'save': save}
        else: raise AssertionError(command)
        reply(route, data)

    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport={'width': 1440, 'height': 940})
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror', lambda e: errors.append(str(e)))
        page.route('**/api', respond)
        page.goto(os.environ.get('GEN3_UI_URL', 'http://127.0.0.1:5173'))
        expect(page.get_by_role('button', name='Export save', exact=True)).to_be_enabled()
        page.get_by_role('button', name='ROM reference', exact=True).first.click()
        page.locator('.reference-tabs').get_by_role('button', name='Maps', exact=True).click()
        page.wait_for_function("document.querySelector('.reference-tabs button.active')?.textContent === 'Maps'")
        assert len(worlds) == 1

        def open_file(name, filename):
            with page.expect_file_chooser() as chooser:
                page.get_by_role('button', name=name, exact=True).first.click()
            chooser.value.set_files({'name': filename, 'mimeType': 'application/octet-stream', 'buffer': b'test'})

        open_file('Open ROM', 'rocket.gba')
        expect(page.locator('.capability-banner')).to_have_count(0)
        reply(worlds.pop(), WORLD)  # Old BW response must not populate Rocket.
        open_file('Open save', 'test.sav')
        expect(page.locator('.pokemon-editor')).to_be_visible()
        level = page.get_by_role('spinbutton', name='Level', exact=True)
        expect(level).to_have_attribute('max', '150')
        level.fill('150')
        page.locator('.editor-submit button[type=submit]').click()
        expect(page.locator('.editor-submit button[type=submit]')).to_be_disabled()
        expect(page.locator('.readonly-pokemon')).to_have_count(0)
        expect(page.get_by_role('button', name='Export save', exact=True)).to_be_enabled()
        expect(page.locator('[data-location="p:0"]')).to_have_attribute('draggable', 'true')
        expect(page.locator('.workspace-toolbar').get_by_role('button', name='Pokédex', exact=True)).to_be_visible()
        page.locator('.editor-tabs').get_by_role('button', name='Stats', exact=True).click()
        iv = page.locator('.stat-table tbody tr').first.locator('input').first
        iv.fill('19')
        page.locator('.editor-submit button[type=submit]').click()
        expect(page.locator('.editor-submit button[type=submit]')).to_be_disabled()
        assert requests.count('action') == 2
        if os.environ.get('GEN3_ADAPTER_SCREENSHOT'):
            page.screenshot(path=os.environ['GEN3_ADAPTER_SCREENSHOT'])
        page.locator('.workspace-toolbar').get_by_role('button', name='Items', exact=True).click()
        page.get_by_role('button', name='PC items', exact=True).click()
        expect(page.get_by_role('spinbutton', name='Quantity', exact=True)).to_be_enabled()
        page.get_by_role('button', name='ROM reference', exact=True).first.click()
        page.locator('.reference-list button').first.click()
        tree = page.get_by_role('region', name='Evolution tree', exact=True)
        expect(tree.get_by_role('heading', name='Battle transformations', exact=True)).to_be_visible()
        form = tree.locator('.evolution-card[data-species="2"]')
        expect(form).to_have_count(1)
        expect(form).to_be_visible()
        expect(form.locator('.evolution-condition')).to_contain_text('Mega')
        page.locator('.reference-tabs').get_by_role('button', name='Maps', exact=True).click()
        page.wait_for_timeout(100)
        assert len(worlds) == 1 and requests.count('world') == 2
        reply(worlds.pop(), WORLD)

        # Switching back restores legacy controls and must request a fresh world.
        page.get_by_role('button', name='Close', exact=True).last.click()
        open_file('Open ROM', 'dp.gba')
        expect(page.locator('.capability-banner')).to_have_count(0)
        open_file('Open save', 'test.sav')
        expect(page.get_by_role('button', name='Export save', exact=True)).to_be_enabled()
        expect(page.locator('[data-location="p:0"]')).to_have_attribute('draggable', 'true')
        page.get_by_role('button', name='ROM reference', exact=True).first.click()
        page.locator('.reference-tabs').get_by_role('button', name='Maps', exact=True).click()
        page.wait_for_timeout(100)
        assert len(worlds) == 1 and requests.count('world') == 3
        reply(worlds.pop(), WORLD)
        assert not errors, errors
        browser.close()
    print('Passed: BW → Rocket → DP, shared editing, item controls, dex, form references and stale world isolation.')


if __name__ == '__main__':
    main()
