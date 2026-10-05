"""Read-only fixture: move -> tutor tile -> entrance -> back -> move."""
import copy
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG


def main():
    expect.set_options(timeout=30000)
    catalog = copy.deepcopy(CATALOG)
    catalog['profile']['capabilities'] = dict(world=True, save_edit=True, dex=True)
    catalog['moves'] = [dict(id=1, name='Tutor move <script>', description='', effect=0,
                            power=40, move_type=0, category=0, accuracy=100, pp=20,
                            chance=0, target=0, priority=0, flags=0, offset=20)]
    teacher = dict(move_id=1, parameter=3, offset=100, conditions=[])
    marker = dict(id='teacher', kind='npc', x=1, y=1, elevation=0, local_id=1,
                  graphics_id=1, movement_type=0, underfoot=None, flag=1,
                  receipt_flag=None, offset=80, script=90, rewards=[], pokemon=[],
                  teaching=[teacher], stopped_at=[100])
    maps = [dict(id='0-0', name='Outside teacher house', region=1, width=4, height=4, map_type=1),
            dict(id='0-1', name='Teacher house', region=1, width=4, height=4, map_type=4)]
    world = dict(maps=maps, map_events=[dict(map_id='0-1', markers=[marker], unplaced_rewards=[],
                  unplaced_pokemon=[], unplaced_teaching=[], stopped_at=[])], encounters=[],
                 trainers=[], trainer_locations=dict(locations=[]), map_groups=[])
    edge = dict(to='0-1', kind='warp', x=2, y=1, target_x=1, target_y=1,
                warp_index=0, target_warp=0, direction=None, displacement=None,
                offset=200, unresolved=None)
    edge['from'] = '0-0'
    source = dict(kind='move_tutor', map_id='0-1', region=1, x=1, y=1, underfoot=None,
                  related=[], quantity=None, min_level=None, max_level=None,
                  encounter_percent=None, held_percent=None, periods=[], conditions=[],
                  requirements=[], evolution=None, status='unknown', receipt_flag=None,
                  receipt=None, repeatable=None, script_source=None, trade_context=None,
                  teaching_source=teacher, offset=100, partial=True, in_scenario=None)
    errors, requests = [], []

    def respond(route):
        req = route.request.post_data_json
        command, payload = req['command'], req['payload']; requests.append(req)
        if command == 'state': data = dict(catalog=catalog, save=None)
        elif command == 'world': data = world
        elif command == 'species': data = dict(species=catalog['species'][payload['id']-1], evolutions=[], learnset=[dict(move_id=1,source='tutor',species=1,level=None,index=4,offset=20)], encounters=[], origins={})
        elif command == 'acquisition': data = dict(target=payload, sources=[source] if payload['kind']=='move' else [], partial=True, clock=None)
        elif command == 'map_navigation': data = dict(map_id=payload['id'], outgoing=[edge] if payload['id']=='0-0' else [], incoming=[edge] if payload['id']=='0-1' else [], approaches=[[edge]] if payload['id']=='0-1' else [], truncated=False, diagnostics=[])
        elif command in ('map_image','sprite','object_sprite','trainer_sprite'): data = dict(url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="lightblue"/></svg>')
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json', body=json.dumps(dict(ok=True,data=data)))

    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport=dict(width=1050,height=760))
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror', lambda e: errors.append(str(e))); page.route('**/api', respond)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        page.get_by_role('button', name='ROM reference', exact=True).click()
        page.locator('.reference-detail').get_by_role('button', name='Tutor move <script>', exact=True).click()
        panel = page.locator('.acquisition-panel')
        expect(panel).to_contain_text('Move tutor NPC')
        expect(panel).to_contain_text('payment')
        panel.get_by_role('button', name='Teacher house', exact=False).click()
        page.locator('.map-focus').click()
        details = page.locator('.map-marker-details')
        expect(details).to_contain_text('Move tutor NPC')
        expect(details).not_to_contain_text('No parsed item reward')
        details.get_by_role('button', name='Tutor move <script>', exact=False).click()
        expect(panel).to_contain_text('Move tutor NPC')
        page.get_by_role('button', name='Back to previous reference', exact=False).click()
        page.get_by_role('button', name='Outside teacher house', exact=False).first.click()
        expect(page.locator('.reference-detail h2')).to_contain_text('Outside teacher house')
        page.get_by_role('button', name='Back to previous reference', exact=False).click()
        page.locator('.map-focus').click()
        details.get_by_role('button', name='Tutor move <script>', exact=False).click()
        page.get_by_role('button', name='简体中文', exact=True).click()
        expect(panel).to_contain_text('招式教学 NPC')
        expect(panel).to_contain_text('教学报价不等于当前可教或已完成')
        page.get_by_role('button',name='返回上一条资料',exact=False).click()
        page.get_by_role('button',name='返回上一条资料',exact=False).click()
        # Earlier map/entrance visits stay in history; the original learning row is retained.
        for _ in range(6):
            if 'Test species 1' in page.locator('.reference-detail h2').inner_text(): break
            page.get_by_role('button',name='返回上一条资料',exact=False).click()
        expect(page.locator('.reference-detail h2')).to_contain_text('Test species 1')
        expect(page.locator('.reference-detail')).to_contain_text('Tutor move <script>')
        assert not errors, errors
        assert not any(r['command'] in ('apply','batch','save','export','transfer','create','action','export_save','save_bytes') for r in requests)
        browser.close()
        print('Move -> tutor tile -> move -> exterior entrance -> back, bilingual uncertainty and no mutations passed')


if __name__ == '__main__': main()
