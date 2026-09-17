"""Storage drag/drop requests, including same-turn duplicate drops.

Run Vite first. Uses only generated API fixtures; never reads a ROM or user save.
The response is deliberately held to exercise the in-flight mutation guard.
"""
import copy
import json
import os

from playwright.sync_api import sync_playwright, expect
from test_editor_navigation import pokemon
from test_reference_navigation import CATALOG, species


def main():
    catalog = copy.deepcopy(CATALOG)
    catalog['moves'] = [{'id': 0, 'name': '', 'power': 0, 'category': 0, 'move_type': 0, 'pp': 0}]
    catalog['items'] = [{'id': 0, 'name': '', 'tm_move': None}]
    origin = {'kind': 'box', 'box_index': 0, 'slot': 0}
    destination = {'kind': 'box', 'box_index': 1, 'slot': 0}
    save = {
        'trainer': {'name': 'TEST', 'gender': 0, 'tid': 1, 'sid': 0, 'hours': 0,
                    'minutes': 0, 'seconds': 0, 'money': 0, 'coins': 0, 'registered_item': 0},
        'pokemon': [pokemon(1, {'kind': 'party', 'slot': 0}), pokemon(2, origin), pokemon(1, destination)],
        'boxes': [{'index': i, 'name': f'Box {i+1}', 'wallpaper': 0, 'count': int(i < 2)} for i in range(14)],
        'bag': [], 'dex': [], 'active_slot': 0, 'counter': 1, 'backup_valid': True,
        'dirty': False, 'can_undo': False, 'can_redo': False, 'changes': [],
    }
    save['pokemon'][1]['pokemon']['pid'] = 0xbadc5647
    pending, actions, errors = [], [], []

    def reply(route, data):
        route.fulfill(content_type='application/json', body=json.dumps({'ok': True, 'data': data}))

    def respond(route):
        req = route.request.post_data_json
        command, payload = req['command'], req['payload']
        if command == 'state': data = {'catalog': catalog, 'save': save}
        elif command == 'sprite': data = {'url': ''}
        elif command == 'species': data = {'species': species(payload['id']), 'evolutions': [], 'learnset': [], 'encounters': []}
        elif command == 'action':
            actions.append(payload['action']); pending.append(route); return
        else: raise AssertionError(req)
        reply(route, data)

    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport={'width': 1440, 'height': 940})
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror', lambda e: errors.append(str(e)))
        page.route('**/api', respond)
        page.goto(os.environ.get('GEN3_UI_URL', 'http://127.0.0.1:5173'))
        source = page.locator('[data-location="0:0"]')
        target = page.locator('[data-location="1:0"]')
        expect(source).to_be_enabled()
        # Two drops in one JS turn race before React can render disabled slots.
        page.evaluate('''() => {
            const source = document.querySelector('[data-location="0:0"]');
            const target = document.querySelector('[data-location="1:0"]');
            const data = new DataTransfer();
            source.dispatchEvent(new DragEvent('dragstart', {bubbles:true,dataTransfer:data}));
            for (let i=0; i<2; i++) target.dispatchEvent(new DragEvent('drop', {bubbles:true,dataTransfer:data}));
        }''')
        expect(source).to_be_disabled()
        page.wait_for_timeout(250)  # Keep the server response pending throughout the race window.
        assert actions == [{'type': 'transfer', 'from': origin, 'to': destination, 'copy': False}], actions
        assert len(pending) == 1
        source_row, target_row = save['pokemon'][1:]
        source_row['location'], target_row['location'] = destination, origin
        reply(pending.pop(), {'save': save})
        expect(target).to_be_enabled()
        expect(target).to_have_attribute('aria-pressed', 'true')
        expect(page.locator('.editor-hero h2')).to_have_text('Test species 2')
        assert save['pokemon'][1]['pokemon']['pid'] == 0xbadc5647

        # A normal subsequent drag is allowed after the first request settles.
        target.drag_to(source)
        expect(source).to_be_disabled()
        assert len(actions) == 2 and actions[-1] == {'type': 'transfer', 'from': destination, 'to': origin, 'copy': False}
        source_row['location'], target_row['location'] = origin, destination
        reply(pending.pop(), {'save': save})
        expect(source).to_be_enabled()
        expect(source).to_have_attribute('aria-pressed', 'true')

        # Unsaved field edits block dragstart and must never become a transfer patch.
        page.get_by_role('button', name='Stats', exact=True).click()
        iv = page.locator('.stat-table tbody tr').first.locator('input').first
        iv.fill('17')
        iv.press('Tab')
        expect(page.locator('.editor-submit button[type=submit]')).to_be_enabled()
        dirty_drag = page.evaluate('''() => {
            const event = new DragEvent('dragstart', {bubbles:true,cancelable:true,dataTransfer:new DataTransfer()});
            document.querySelector('[data-location="0:0"]').dispatchEvent(event);
            return {prevented:event.defaultPrevented,payload:event.dataTransfer.getData('application/x-gen3')};
        }''')
        assert dirty_drag['prevented'] and not dirty_drag['payload'], dirty_drag
        assert len(actions) == 2 and not errors, (actions, errors)
        browser.close()
    print('Passed: single mutation for duplicate drops, subsequent drag, location-only payload and dirty-form drag guard.')


if __name__ == '__main__':
    main()
