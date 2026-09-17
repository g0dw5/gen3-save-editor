"""Synthetic browser coverage for live IV-derived values; no ROM/save required.

Start Vite first. Requires Playwright and Chrome; GEN3_UI_URL overrides its URL.
GEN3_HIDDEN_POWER_PREVIEW optionally saves a screenshot of the editor.
"""
import copy
import json
import os

from playwright.sync_api import sync_playwright, expect
from test_editor_navigation import pokemon
from test_reference_navigation import CATALOG, WORLD, species


def main():
    catalog = copy.deepcopy(CATALOG)
    catalog['profile']['hidden_power'] = {'move_id': 237, 'formula': 'gen3_to5'}
    catalog['moves'] = [dict(id=i, name=f'Move {i}', power=50, category=0,
                             move_type=0, pp=15, accuracy=100, priority=0,
                             effect=0, chance=0, description='') for i in range(238)]
    catalog['moves'][237].update(name='觉醒力量', power=1, category=1, effect=135)
    catalog['items'] = [{'id': 0, 'name': '', 'tm_move': None}]
    rows = [pokemon(1, {'kind': 'party', 'slot': 0}),
            pokemon(2, {'kind': 'box', 'box_index': 0, 'slot': 0})]
    for row, iv in zip(rows, [31, 30]):
        row['pokemon'].update(ivs=[iv]*6, moves=[237, 1, 0, 0], pps=[15, 15, 0, 0])
    save = {
        'trainer': {'name': 'TEST', 'gender': 0, 'tid': 1, 'sid': 0, 'hours': 0,
                    'minutes': 0, 'seconds': 0, 'money': 0, 'coins': 0, 'registered_item': 0},
        'pokemon': rows,
        'boxes': [{'index': i, 'name': f'Box {i+1}', 'wallpaper': 0, 'count': int(i == 0)}
                  for i in range(14)],
        'bag': [], 'dex': [], 'active_slot': 0, 'counter': 1, 'backup_valid': True,
        'dirty': False, 'can_undo': False, 'can_redo': False, 'changes': [],
    }
    world = copy.deepcopy(WORLD)
    world['trainers'] = [dict(id=1, name='Test trainer', diagnostics=[],
                             class_name='Test class', portrait=0, female=False,
                             double_battle=False, items=[], ai=0, offset=0, party=[])]
    world['trainers'][0]['class'] = 1
    for ivs in ([31]*6, None):
        world['trainers'][0]['party'].append(dict(
            species=1, level=50, level_rule='fixed', iv_quality=255 if ivs else 0,
            held_item=0, moves=[237, 0, 0, 0], moves_explicit=True, offset=0,
            generation=dict(gender='male', nature=0, ability_id=0, ivs=ivs,
                            evs=[0]*6, personality_parameter=0)))
    actions, errors = [], []

    def respond(route):
        request = route.request.post_data_json
        command, payload = request['command'], request['payload']
        if command == 'state':
            data = {'catalog': catalog, 'save': save}
        elif command == 'world':
            data = world
        elif command in ('sprite', 'trainer_sprite'):
            data = {'url': ''}
        elif command == 'species':
            data = {'species': species(payload['id']), 'evolutions': [], 'encounters': [],
                    'learnset': [dict(move_id=i, source='level', species=1,
                                      level=1, index=None, offset=0) for i in (1, 237)]}
        elif command == 'action':
            action = payload['action']
            actions.append(action)
            assert action['type'] == 'pokemon'
            row = next(r for r in rows if r['location'] == action['location'])
            row['pokemon'].update(action['patch'])
            data = {'save': save}
        else:
            raise AssertionError(request)
        route.fulfill(content_type='application/json', body=json.dumps({'ok': True, 'data': data}))

    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport={'width': 1440, 'height': 940})
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror', lambda e: errors.append(str(e)))
        page.route('**/api', respond)
        page.goto(os.environ.get('GEN3_UI_URL', 'http://127.0.0.1:5173'))
        tabs = page.locator('.editor-tabs')
        summary = page.locator('.pokemon-editor .hidden-power-summary')
        apply = page.locator('.editor-submit button[type=submit]')
        tabs.get_by_role('button', name='Stats', exact=True).click()
        expect(summary).to_contain_text('Dark')
        expect(summary).to_contain_text('70')
        expect(apply).to_be_disabled()
        # Each row contains one IV and one EV input in separate table cells.
        ivs = page.locator('.stat-table tbody tr td:nth-child(2) input')
        ivs.nth(1).fill('30')
        ivs.nth(2).fill('30')
        expect(summary).to_contain_text('Ice')
        tabs.get_by_role('button', name='Moves', exact=True).click()
        expect(summary).to_contain_text('Ice')
        expect(page.get_by_role('combobox', name='Move 1', exact=True)).to_have_value('【Spec.】【Ice】【70】觉醒力量 #237')
        expect(page.get_by_role('combobox', name='Move 2', exact=True)).to_have_value('【Phys.】【Normal】【50】Move 1 #1')
        apply.click()
        expect(apply).to_be_disabled()
        assert actions == [{'type': 'pokemon', 'location': {'kind': 'party', 'slot': 0},
                            'patch': {'ivs': [31, 30, 30, 31, 31, 31]}}], actions
        page.locator('[data-location="0:0"]').click()
        expect(summary).to_contain_text('Fighting')
        expect(page.get_by_role('combobox', name='Move 1', exact=True)).to_have_value('【Spec.】【Fighting】【70】觉醒力量 #237')
        tabs.get_by_role('button', name='Stats', exact=True).click()
        ivs.nth(0).fill('32')
        expect(summary).to_contain_text('unknown')
        ivs.nth(0).fill('30')
        page.once('dialog', lambda d: d.accept())
        page.locator('[data-location="p:0"]').click()
        expect(summary).to_contain_text('Ice')
        assert len(actions) == 1
        page.get_by_role('button', name='简体中文', exact=True).click()
        expect(summary).to_contain_text('威力 · 70')
        expect(summary.locator('.type-tag')).to_have_text('冰')
        if os.environ.get('GEN3_HIDDEN_POWER_PREVIEW'):
            summary.scroll_into_view_if_needed()
            page.locator('.pokemon-editor').screenshot(path=os.environ['GEN3_HIDDEN_POWER_PREVIEW'])
        page.get_by_role('button', name='English', exact=True).click()
        page.get_by_role('button', name='ROM reference', exact=True).click()
        dialog = page.get_by_role('dialog')
        ref_tabs = dialog.locator('.reference-tabs')
        ref_tabs.get_by_role('button', name='Moves', exact=True).click()
        dialog.locator('.reference-rows button').filter(has_text='觉醒力量').click()
        expect(dialog.locator('.reference-detail')).to_contain_text('30–70')
        expect(dialog.locator('.reference-detail')).to_contain_text('depend on')
        expect(dialog.locator('.reference-detail .type-tag')).to_have_count(0)
        ref_tabs.get_by_role('button', name='Trainers', exact=True).click()
        moves = dialog.locator('.trainer-moves')
        expect(moves.nth(0)).to_contain_text('觉醒力量 · Dark · Power 70')
        expect(moves.nth(1)).to_contain_text('Type / power unknown')
        assert not errors, errors
        assert len(actions) == 1
        browser.close()
    print('Passed: live IVs, dropdowns, party/box selection, IV-only patch, invalid IVs, '
          'Chinese/English, generic reference and known/random trainer IVs.')


if __name__ == '__main__':
    main()
