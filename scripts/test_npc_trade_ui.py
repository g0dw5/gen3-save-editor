"""Public read-only fixture: trade offer -> donor -> NPC tile."""
import copy
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG
from test_editor_navigation import pokemon


def main():
    expect.set_options(timeout=30000)
    catalog = copy.deepcopy(CATALOG)
    catalog['profile']['capabilities'] = {'world': True, 'save_edit': True, 'dex': True}
    catalog['species'][0]['name'] = 'Received Pokémon'
    catalog['species'][1]['name'] = 'Donor <script>alert(1)</script>'
    catalog['items'] = [dict(id=0, name='', tm_move=None), dict(id=1, name='Trade held item', tm_move=None, description='', price=100, pocket=1)]
    catalog['moves'] = [dict(id=0, name='', pp=0)]
    mon = dict(species=1, level=None, held_item=1, method='npc_trade', offset=100, member=0, conditions=[], trade=dict(index=0, record_offset=200, requested_species=2, level_rule='offered_pokemon'))
    marker = dict(id='trade', kind='gift', x=1, y=1, elevation=0, local_id=1, graphics_id=1, movement_type=0, flag=None, receipt_flag=None, underfoot=None, offset=80, script=90, rewards=[], pokemon=[mon], stopped_at=[100])
    map = dict(id='0-0', name='Trade room', region=1, width=4, height=4, map_type=1)
    world = dict(maps=[map], map_events=[dict(map_id='0-0', markers=[marker], unplaced_rewards=[], unplaced_pokemon=[], stopped_at=[100])], encounters=[], trainers=[], trainer_locations=dict(locations=[]), map_groups=[])
    source = dict(kind='npc_trade', map_id='0-0', region=1, x=1, y=1, underfoot=None, related=[dict(kind='species', id=2), dict(kind='item', id=1)], quantity=None, min_level=None, max_level=None, encounter_percent=None, held_percent=None, periods=[], conditions=[], requirements=[], evolution=None, status='unknown', receipt_flag=None, receipt=None, repeatable=None, offset=100, partial=True, in_scenario=None, script_source=mon, trade_context=dict(party_levels=[44], box_levels=[]))
    save = dict(trainer={'name':'TEST'}, pokemon=[pokemon(2, dict(kind='party',slot=0))], boxes=[dict(index=i,name=f'Box {i}',count=0,wallpaper=0) for i in range(14)], bag=[], dex=[], active_slot=0, counter=1, backup_valid=True, dirty=False, can_undo=False, can_redo=False, changes=[])
    errors, requests = [], []

    def respond(route):
        req = route.request.post_data_json; requests.append(req)
        command, payload = req['command'], req['payload']
        if command == 'state': data = dict(catalog=catalog, save=save)
        elif command == 'world': data = world
        elif command == 'species': data = dict(species=catalog['species'][payload['id']-1], evolutions=[], learnset=[], encounters=[], origins={})
        elif command == 'acquisition':
            item_source = source | dict(kind='npc_trade_item', quantity=1, related=[dict(kind='species',id=1),dict(kind='species',id=2)])
            data = dict(target=payload, sources=[item_source if payload['kind']=='item' else source] if payload['id']==1 else [], partial=True, clock=None)
        elif command == 'map_navigation': data = dict(map_id='0-0', outgoing=[], incoming=[], approaches=[], truncated=False, diagnostics=[])
        elif command in ('map_image','sprite','object_sprite','trainer_sprite'): data = dict(url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="lightblue"/></svg>')
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json', body=json.dumps(dict(ok=True, data=data)))

    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport=dict(width=1050, height=760), accept_downloads=True)
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror', lambda e: errors.append(str(e))); page.route('**/api', respond)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        page.get_by_role('button', name='ROM reference', exact=True).click()
        panel = page.locator('.acquisition-panel')
        expect(panel).to_contain_text('NPC Pokémon trade')
        expect(panel).to_contain_text('received level equals')
        expect(panel).to_contain_text('Lv. 44')
        panel.locator('.trade-details button').first.click()
        expect(panel).to_contain_text('No source was found within the parsed coverage.')
        page.get_by_role('button', name='Back to previous reference', exact=False).click()
        panel.get_by_role('button', name='Trade room', exact=False).click()
        page.locator('.map-focus').click()
        details = page.locator('.map-marker-details')
        expect(details).to_contain_text('NPC Pokémon trade')
        expect(details).to_contain_text('Donor <script>')
        details.get_by_role('button', name='Trade held item', exact=False).click()
        expect(panel).to_contain_text('Held item from an NPC trade')
        page.get_by_role('button', name='Back to previous reference', exact=False).click()
        details.locator('.trade-details button').first.click()
        expect(panel).to_contain_text('No source was found within the parsed coverage.')
        page.get_by_role('button', name='Back to previous reference', exact=False).click()
        expect(details).to_contain_text('NPC Pokémon trade')
        page.get_by_role('button',name='简体中文',exact=True).click();expect(details).to_contain_text('等级与交出的宝可梦相同')
        assert not errors, errors
        assert all(r['command'] not in ['action','export_save','save_bytes'] for r in requests)
        browser.close()
        print('Trade donor/held-item/level, source -> donor -> map -> back passed; no writes')


if __name__ == '__main__': main()
