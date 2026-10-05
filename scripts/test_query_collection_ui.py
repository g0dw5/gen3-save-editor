"""Public browser fixtures: query -> tile -> entrance -> back, planning and safe HTML.

Run Vite first; requires Playwright and Chrome. Never opens or modifies real files.
"""
import copy
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, species
from test_editor_navigation import pokemon


def main():
    expect.set_options(timeout=30000)
    catalog = copy.deepcopy(CATALOG)
    catalog['profile']['capabilities'] = {'world': True, 'save_edit': True, 'dex': True}
    catalog['species'][0]['name'] = 'Test species <script>alert(1)</script>'
    catalog['items'] = [dict(id=0,name='',tm_move=None),dict(id=1,name='Test stone',tm_move=None,description='',price=100,pocket=1)]
    catalog['moves'] = [dict(id=0,name='',pp=0)]
    maps = [dict(id='0-0',name='Test region entrance',region=1,width=4,height=4,map_type=1),dict(id='0-1',name='Test cave floor',region=1,width=4,height=4,map_type=4)]
    reward = dict(item=1,quantity=1,offset=100,via='pickup',conditions=[])
    marker = dict(id='pickup-1',kind='pickup',x=1,y=1,elevation=0,local_id=1,graphics_id=None,movement_type=0,flag=10,offset=100,script=100,rewards=[reward],stopped_at=[])
    world = dict(maps=maps,map_events=[dict(map_id='0-1',markers=[marker],unplaced_rewards=[],stopped_at=[])],encounters=[],trainers=[],trainer_locations=dict(locations=[]),map_groups=[])
    edge = dict(from_='0-0',to='0-1',kind='warp',x=2,y=1,target_x=1,target_y=1,warp_index=0,target_warp=0,direction=None,displacement=None,offset=200,unresolved=None)
    edge['from'] = edge.pop('from_')
    source = dict(underfoot=True,kind='pickup',map_id='0-1',region=1,x=1,y=1,related=[],quantity=1,min_level=None,max_level=None,encounter_percent=None,held_percent=None,periods=[],conditions=[],requirements=[],evolution=None,status='unknown',receipt_flag=10,repeatable=False,offset=100,partial=True,in_scenario=None)
    save = dict(trainer={'name':'TEST'},pokemon=[pokemon(2,dict(kind='party',slot=0))],boxes=[dict(index=i,name=f'Box {i}',count=0,wallpaper=0) for i in range(14)],bag=[],dex=[],active_slot=0,counter=1,backup_valid=True,dirty=False,can_undo=False,can_redo=False,changes=[])
    task = dict(target=dict(kind='species',id=1),family=[1],existing_family_members=[],source=source,alternatives=1)
    plan = dict(rom_md5='test',basis='individuals',families=True,owned_count=1,missing_count=1,regions=[dict(region=1,tasks=[task])],entrances=[dict(map_id='0-1',chains=[[edge]],truncated=False)],partial=True)
    errors, requests = [], []
    def respond(route):
        req = route.request.post_data_json
        command,payload = req['command'],req['payload']; requests.append(req)
        if command=='state': data=dict(catalog=catalog,save=save)
        elif command=='world': data=world
        elif command=='species': data=dict(species=catalog['species'][payload['id']-1],evolutions=[],learnset=[],encounters=[],origins={})
        elif command=='acquisition': data=dict(target=payload,sources=[source],partial=True,clock=None)
        elif command=='map_navigation': data=dict(map_id=payload['id'],outgoing=[edge] if payload['id']=='0-0' else [],incoming=[edge] if payload['id']=='0-1' else [],approaches=[[edge]] if payload['id']=='0-1' else [],truncated=False,diagnostics=[])
        elif command=='collection': data=plan
        elif command in ('map_image','sprite','object_sprite','trainer_sprite'): data=dict(url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="lightblue"/></svg>')
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=data)))
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome',headless=True)
        page=browser.new_page(viewport=dict(width=1100,height=780),accept_downloads=True)
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror',lambda e: errors.append(str(e)))
        page.route('**/api',respond)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        page.get_by_role('button',name='ROM reference',exact=True).click()
        pane=page.locator('.floating')
        expect(page.locator('.acquisition-panel')).to_be_visible()
        page.locator('.acquisition-panel .link-button').filter(has_text='Test cave floor').click()
        expect(page.locator('.map-focus')).to_be_visible()
        expect(page.locator('.map-navigation')).to_contain_text('Test region entrance')
        page.locator('.nav-path .link-button').first.click()
        expect(page.locator('.map-focus')).to_be_visible()
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        expect(page.locator('.map-navigation')).to_contain_text('Test cave floor')
        page.locator('.map-focus').click()
        page.locator('.map-marker-details .link-button').filter(has_text='Test stone').click()
        expect(page.locator('.acquisition-panel')).to_contain_text('Collected' if False else 'Undetermined')
        page.get_by_role('button',name='Collection planning',exact=True).click()
        expect(page.locator('.collection-panel')).to_contain_text('Missing goals 1')
        with page.expect_download() as info:
            page.get_by_role('button',name='Export standalone HTML',exact=True).click()
        html=Path(info.value.path()).read_text()
        assert '&lt;script&gt;alert(1)&lt;/script&gt;' in html
        assert '<script>' not in html and 'default-src' in html
        assert 'Stand on this tile and use the Itemfinder' in html
        assert 'Test region entrance (2, 1)' in html and 'Test cave floor' in html
        assert not any(r['command'] in ('action','export_save','save_bytes') for r in requests)
        assert not errors,errors
        page.set_viewport_size(dict(width=720,height=740))
        expect(page.get_by_role('button',name='Export standalone HTML',exact=True)).to_be_visible()
        print('query tile/entrance/back, reward link, read-only planner, escaped standalone HTML and compact window passed')
        browser.close()


if __name__=='__main__': main()
