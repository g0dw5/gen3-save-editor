"""Verify editor tab retention, draft isolation and fixed controls in the real App.

Start Vite, then run with Playwright/Chrome. Uses generated API fixtures only;
no user ROM or save is loaded or modified.
"""
import copy
import json
import os

from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, WORLD, species


def pokemon(identifier, location):
    return {'location': location, 'pokemon': {
        'species': identifier, 'pid': identifier, 'ot_id': 1, 'nickname': f'Mon {identifier}',
        'ot_name': 'TEST', 'language': 2, 'markings': 0, 'held_item': 0,
        'experience': 1000, 'friendship': 70, 'pp_ups': [0]*4, 'moves': [0]*4,
        'pps': [0]*4, 'evs': [0]*6, 'condition': [0]*6, 'ivs': [identifier]*6,
        'ability_slot': 0, 'ability_id': 0, 'egg': False, 'pokerus': 0,
        'met_location': 1, 'met_level': 5, 'origin_game': 3, 'ball': 4,
        'ot_gender': 0, 'ribbons': 0, 'nature': 0, 'gender': 'male', 'shiny': False,
        'level': 10, 'stats': [30]*6, 'current_hp': 30 if location['kind']=='party' else None,
        'status': 0 if location['kind']=='party' else None, 'checksum_ok': True,
    }}


def main():
    catalog = copy.deepcopy(CATALOG)
    catalog['moves'] = [{'id': 0, 'name': '', 'power': 0, 'category': 0, 'move_type': 0, 'pp': 0}]
    catalog['items'] = [{'id': 0, 'name': '', 'tm_move': None}]
    save = {
        'trainer': {'name': 'TEST', 'gender': 0, 'tid': 1, 'sid': 0, 'hours': 0,
                    'minutes': 0, 'seconds': 0, 'money': 0, 'coins': 0, 'registered_item': 0},
        'pokemon': [pokemon(1, {'kind':'party','slot':0}),
                    pokemon(2, {'kind':'box','box_index':0,'slot':0})],
        'boxes': [{'index':i,'name':f'Box {i+1}','wallpaper':0,'count':int(i==0)} for i in range(14)],
        'bag': [], 'dex': [], 'active_slot': 0, 'counter': 1, 'backup_valid': True,
        'dirty': False, 'can_undo': False, 'can_redo': False, 'changes': [],
    }
    actions, errors = [], []
    def respond(route):
        req = route.request.post_data_json
        command, payload = req['command'], req['payload']
        if command == 'state': data = {'catalog':catalog, 'save':save}
        elif command == 'world': data = WORLD
        elif command == 'sprite': data = {'url':''}
        elif command == 'species':
            data = {'species':species(payload['id']), 'evolutions':[], 'learnset':[], 'encounters':[]}
        elif command == 'action':
            action = payload['action']; actions.append(action)
            assert action['type']=='pokemon', action
            row = next(r for r in save['pokemon'] if r['location']==action['location'])
            row['pokemon'].update(action['patch'])
            data = {'save':save}
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json', body=json.dumps({'ok':True,'data':data}))
    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport={'width':1440,'height':940})
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror', lambda e: errors.append(str(e)))
        page.route('**/api',respond)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        tabs = page.locator('.editor-tabs')
        party = page.locator('[data-location="p:0"]')
        boxed = page.locator('[data-location="0:0"]')
        apply = page.locator('.editor-submit button[type=submit]')
        for tab in ['Stats','Moves','Origin','Advanced']:
            tabs.get_by_role('button',name=tab,exact=True).click()
            for slot in [boxed,party]:
                slot.click()
                expect(tabs.get_by_role('button',name=tab,exact=True)).to_have_attribute('aria-pressed','true')
                expect(apply).to_be_disabled()
        # Retain the selected tab through an empty slot and inspector remount.
        page.locator('[data-location="0:1"]').click()
        expect(page.locator('.empty-inspector')).to_be_visible()
        boxed.click()
        expect(tabs.get_by_role('button',name='Advanced')).to_have_attribute('aria-pressed','true')
        page.get_by_role('button',name='Inspector',exact=True).click()
        expect(page.locator('.pokemon-editor')).to_have_count(0)
        page.get_by_role('button',name='Inspector',exact=True).click()
        expect(tabs.get_by_role('button',name='Advanced')).to_have_attribute('aria-pressed','true')
        # A discarded draft must not leak into another Pokemon, despite tab retention.
        tabs.get_by_role('button',name='Stats',exact=True).click()
        iv = page.locator('.stat-table tbody tr').first.locator('input').first
        iv.fill('17')
        expect(apply).to_be_enabled()
        page.once('dialog',lambda dialog:dialog.dismiss())
        party.click()
        expect(page.locator('.editor-hero h2')).to_have_text('Test species 2')
        expect(iv).to_have_value('17')
        page.once('dialog',lambda dialog:dialog.accept())
        party.click()
        expect(iv).to_have_value('1')
        expect(apply).to_be_disabled()
        iv.fill('19')
        apply.click()
        expect(apply).to_be_disabled()
        assert actions==[{'type':'pokemon','location':{'kind':'party','slot':0},'patch':{'ivs':[19,1,1,1,1,1]}}],actions
        expect(tabs.get_by_role('button',name='Stats')).to_have_attribute('aria-pressed','true')
        # Exercise the real scroll container at desktop, minimum and mobile widths.
        tabs.get_by_role('button',name='Advanced',exact=True).click()
        for width,height in [(1440,940),(900,640),(390,844)]:
            page.set_viewport_size({'width':width,'height':height})
            page.locator('.pokemon-editor').scroll_into_view_if_needed()
            body = page.locator('.editor-body')
            body.evaluate('(e)=>e.scrollTop=0')
            before = [page.locator(s).bounding_box() for s in ['.editor-tabs','.editor-footer']]
            assert body.evaluate('(e)=>e.scrollHeight>e.clientHeight')
            body.hover();page.mouse.wheel(0,10000)
            page.wait_for_function("document.querySelector('.editor-body').scrollTop>0")
            after = [page.locator(s).bounding_box() for s in ['.editor-tabs','.editor-footer']]
            assert all(abs(a['y']-b['y'])<1 and abs(a['height']-b['height'])<1 for a,b in zip(before,after)), (width,before,after)
            assert page.locator('.inspector').evaluate('(e)=>e.scrollTop')==0
            rect = page.locator('.pokemon-editor').bounding_box()
            assert after[0]['y']>=rect['y'] and after[1]['y']+after[1]['height']<=rect['y']+rect['height']+1
        preview = os.environ.get('GEN3_EDITOR_PREVIEW')
        if preview:
            page.set_viewport_size({'width':1440,'height':940})
            page.locator('.pokemon-editor').screenshot(path=preview)
        assert not errors,errors
        browser.close()
    print('Passed: retained tabs, empty-slot/hide recovery, draft guard/isolation, correct apply target, fixed header/footer at three sizes.')


if __name__=='__main__':main()
