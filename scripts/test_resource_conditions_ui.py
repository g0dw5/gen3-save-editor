"""Read-only resource prerequisite -> item -> map/back and collection HTML fixture."""
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
    catalog['profile']['capabilities'] = dict(world=True, save_edit=True, dex=True)
    catalog['items'] = [dict(id=0, name='', tm_move=None),
                        dict(id=1, name='Required stone <script>', tm_move=None, price=100, description='', pocket=1),
                        dict(id=2, name='Reward', tm_move=None, price=100, description='', pocket=1)]
    condition = lambda kind, id, value, taken=True: dict(kind=kind,id=id,value=value,comparison=4,taken=taken)
    checks = [dict(condition=condition('bag_item',1,3),satisfied=False,actual=2,unresolved=None),
              dict(condition=condition('money',0,70000),satisfied=False,actual=500,unresolved=None),
              dict(condition=condition('bag_item_runtime',1,1),satisfied=None,actual=None,unresolved='script_changes_resource'),
              dict(condition=condition('bag_item',1,0,False),satisfied=None,actual=0,unresolved='alternate_bag_unresolved')]
    reward = dict(item=2,quantity=1,offset=100,via='gift',receipt=None,conditions=[c['condition'] for c in checks])
    marker = dict(id='reward',kind='gift',x=1,y=1,elevation=0,local_id=1,graphics_id=None,movement_type=0,flag=None,
                  receipt_flag=None,offset=100,script=100,rewards=[reward],pokemon=[],teaching=[],stopped_at=[])
    maps = [dict(id='0-0',name='Reward room',region=1,width=4,height=4,map_type=4)]
    world = dict(maps=maps,map_events=[dict(map_id='0-0',markers=[marker],unplaced_rewards=[],unplaced_pokemon=[],unplaced_teaching=[],stopped_at=[])],encounters=[],trainers=[],trainer_locations=dict(locations=[]),map_groups=[])
    source = dict(underfoot=None,kind='gift',map_id='0-0',region=1,x=1,y=1,related=[dict(kind='item',id=1)],quantity=1,min_level=None,max_level=None,encounter_percent=None,held_percent=None,periods=[],conditions=checks,requirements=[],evolution=None,status='blocked',receipt_flag=None,receipt=None,repeatable=None,offset=100,partial=True,in_scenario=None)
    save = dict(trainer={'name':'TEST'},pokemon=[pokemon(2,dict(kind='party',slot=0))],boxes=[dict(index=i,name=f'Box {i}',count=0,wallpaper=0) for i in range(14)],bag=[],dex=[],active_slot=0,counter=1,backup_valid=True,dirty=False,can_undo=False,can_redo=False,changes=[])
    task = dict(target=dict(kind='item',id=2),family=[],existing_family_members=[],source=source,alternatives=1)
    plan = dict(rom_md5='test',basis='individuals',families=True,owned_count=1,missing_count=1,regions=[dict(region=1,tasks=[task])],entrances=[],partial=True)
    errors, requests = [], []
    def respond(route):
        req=route.request.post_data_json;command,payload=req['command'],req['payload'];requests.append(req)
        if command=='state': data=dict(catalog=catalog,save=save)
        elif command=='world': data=world
        elif command=='species': data=dict(species=catalog['species'][payload['id']-1],evolutions=[],learnset=[],encounters=[],origins={})
        elif command=='acquisition': data=dict(target=payload,sources=[source],partial=True,clock=None)
        elif command=='map_navigation': data=dict(map_id=payload['id'],outgoing=[],incoming=[],approaches=[],truncated=False,diagnostics=[])
        elif command=='collection': data=plan
        elif command in ('map_image','sprite','object_sprite','trainer_sprite'): data=dict(url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="lightblue"/></svg>')
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=data)))
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome',headless=True)
        page=browser.new_page(viewport=dict(width=1000,height=760),accept_downloads=True)
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror',lambda e:errors.append(str(e)));page.route('**/api',respond)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        page.get_by_role('button',name='ROM reference',exact=True).click()
        panel=page.locator('.acquisition-panel')
        expect(panel).to_contain_text('Money held ≥ 70000 · SAV value 500')
        expect(panel).to_contain_text('Earlier script operations may change this resource')
        expect(panel).to_contain_text('No item record in the bag: Required stone <script>')
        panel.locator('.condition-details').get_by_role('button',name='Required stone <script> ≥ 3',exact=False).click()
        expect(page.locator('.reference-detail h2')).to_contain_text('Required stone <script>')
        panel.get_by_role('button',name='Reward room',exact=False).click()
        page.locator('.map-focus').click()
        details=page.locator('.map-marker-details')
        expect(details).to_contain_text('Money held ≥ 70000')
        details.locator('.condition-details').get_by_role('button',name='Required stone <script> ≥ 3',exact=False).click()
        expect(page.locator('.reference-detail h2')).to_contain_text('Required stone <script>')
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        expect(page.locator('.reference-detail h2')).to_contain_text('Reward room')
        page.get_by_role('button',name='Collection planning',exact=True).click()
        expect(page.locator('.collection-panel')).to_contain_text('Money held ≥ 70000 · SAV value 500')
        page.get_by_role('button',name='简体中文',exact=True).click()
        expect(page.locator('.collection-panel')).to_contain_text('持有金钱 ≥ 70000 · 存档当前值 500')
        with page.expect_download() as info: page.get_by_role('button',name='导出独立 HTML',exact=True).click()
        html=Path(info.value.path()).read_text()
        assert 'Required stone &lt;script&gt; ≥ 3' in html and '<script>' not in html
        assert '持有金钱 ≥ 70000' in html and '此前脚本操作可能改变该资源' in html
        assert '普通背包数量 0' in html and '持有量检查不代表费用' in html
        assert 'bag_item 1' not in html and 'default-src' in html
        assert not errors, errors
        assert not any(r['command'] in ('action','save_bytes','export_save','apply','batch','transfer','create') for r in requests)
        browser.close()
        print('Resource prerequisites -> item -> map/back, bilingual planning and escaped human-readable HTML passed')

if __name__=='__main__': main()
