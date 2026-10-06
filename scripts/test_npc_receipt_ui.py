"""Public fixture: NPC receipt evidence -> map.

Vite/Chrome/Playwright required. No private ROM/SAV or mutation requests.
"""
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
    catalog['items'] = [dict(id=0, name='', tm_move=None), dict(id=1, name='Test gift', tm_move=None, description='', price=100, pocket=1)]
    catalog['moves'] = [dict(id=0, name='', pp=0)]
    receipt = dict(flag=1, root=90, award_offset=100, success_set_offsets=[110])
    reward = dict(item=1, quantity=1, offset=100, via='gift', conditions=[], receipt=receipt)
    marker = dict(id='gift', kind='gift', x=1, y=1, elevation=0, local_id=1, graphics_id=1, movement_type=0, flag=2, receipt_flag=None, underfoot=None, offset=80, script=90, rewards=[reward], stopped_at=[])
    map = dict(id='0-0', name='Test gift room', region=1, width=4, height=4, map_type=1)
    world = dict(maps=[map], map_events=[dict(map_id='0-0', markers=[marker], unplaced_rewards=[], stopped_at=[])], encounters=[], trainers=[], trainer_locations=dict(locations=[]), map_groups=[])
    source = dict(kind='gift',map_id='0-0',region=1,x=1,y=1,underfoot=None,related=[],quantity=1,min_level=None,max_level=None,encounter_percent=None,held_percent=None,periods=[],conditions=[],requirements=[],evolution=None,status='available',receipt_flag=1,receipt=receipt,repeatable=None,offset=100,partial=False,in_scenario=None)
    unknown = source | dict(status='unknown',receipt_flag=None,receipt=None,offset=200,partial=True)
    save = dict(trainer={'name':'TEST'},pokemon=[pokemon(2,dict(kind='party',slot=0))],boxes=[dict(index=i,name=f'Box {i}',count=0,wallpaper=0) for i in range(14)],bag=[],dex=[],active_slot=0,counter=1,backup_valid=True,dirty=False,can_undo=False,can_redo=False,changes=[])
    requests, errors = [], []
    def respond(route):
        req=route.request.post_data_json;requests.append(req); command,payload=req['command'],req['payload']
        if command=='state': data=dict(catalog=catalog,save=save)
        elif command=='world': data=world
        elif command=='species': data=dict(species=catalog['species'][payload['id']-1],evolutions=[],learnset=[],encounters=[],origins={})
        elif command=='acquisition': data=dict(target=payload,sources=[source,unknown],partial=True,clock=None)
        elif command=='map_navigation': data=dict(map_id='0-0',outgoing=[],incoming=[],approaches=[],truncated=False,diagnostics=[])
        elif command in ('map_image','sprite','object_sprite','trainer_sprite'): data=dict(url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="lightblue"/></svg>')
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=data)))
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome',headless=True)
        page=browser.new_page(viewport=dict(width=1100,height=780),accept_downloads=True)
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror',lambda e:errors.append(str(e))); page.route('**/api',respond)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        page.get_by_role('button',name='ROM reference',exact=True).click()
        panel=page.locator('.acquisition-panel')
        expect(panel).to_contain_text('Parsed conditions met')
        expect(panel).to_contain_text('The receipt protocol is not verified for this event')
        panel.get_by_role('button',name='Test gift room',exact=False).first.click()
        page.locator('.map-focus').click()
        page.locator('.map-marker-details summary').click()
        expect(page.locator('.map-marker-details')).to_contain_text('Flag 0x1 · Script 0x5a · 0x6e')
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        page.get_by_role('button',name='简体中文',exact=True).click();expect(panel).to_contain_text('尚未确认此事件的领取规则')
        assert not errors,errors
        assert all(r['command'] not in ['action','export_save','save_bytes'] for r in requests)
        browser.close()
        print('NPC receipt and unknown help, target map/evidence/back passed; no mutations')


if __name__=='__main__': main()
