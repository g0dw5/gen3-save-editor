"""Public read-only breeding scenario -> offspring -> daycare map/back workflow."""
import copy,json,os
from playwright.sync_api import sync_playwright,expect
from test_reference_navigation import CATALOG
from test_editor_navigation import pokemon


def main():
    expect.set_options(timeout=30000)
    catalog=copy.deepcopy(CATALOG)
    catalog['profile']['breeding']={'pending_width':2}
    catalog['profile']['capabilities']=dict(world=True,save_edit=True,dex=True)
    catalog['items']=[dict(id=0,name='',tm_move=None),dict(id=1,name='ROM incense',tm_move=None)]
    maps=[dict(id='0-0',name='Daycare meadow',region=1,width=4,height=4,map_type=4)]
    condition=dict(kind='flag',id=12,value=1,comparison=1,taken=True)
    offer=dict(offset=100,conditions=[condition])
    marker=dict(id='npc-1',kind='npc',x=2,y=3,elevation=0,local_id=1,graphics_id=None,movement_type=0,underfoot=None,flag=None,receipt_flag=None,offset=99,script=100,rewards=[],pokemon=[],teaching=[],daycare=[offer],stopped_at=[100])
    world=dict(maps=maps,map_events=[dict(map_id='0-0',markers=[marker],unplaced_rewards=[],unplaced_pokemon=[],unplaced_teaching=[],unplaced_daycare=[],stopped_at=[])],encounters=[],trainers=[],trainer_locations=dict(locations=[]),map_groups=[])
    source=dict(kind='daycare',map_id='0-0',x=2,y=3,offset=100,conditions=[dict(condition=condition,actual=0,satisfied=False,unresolved=None)],status='unknown',partial=True)
    child=pokemon(2,dict(kind='party',slot=0))['pokemon'];child['egg']=True
    save=dict(trainer={'name':'TEST'},pokemon=[pokemon(1,dict(kind='party',slot=0))],boxes=[dict(index=i,name=f'Box {i}',count=0,wallpaper=0) for i in range(14)],bag=[],dex=[],active_slot=0,counter=1,backup_valid=True,dirty=False,can_undo=False,can_redo=False,changes=[])
    requests=[];errors=[]
    def respond(route):
        req=route.request.post_data_json;requests.append(req);cmd,p=req['command'],req['payload']
        if cmd=='state':data=dict(catalog=catalog,save=save)
        elif cmd=='world':data=world
        elif cmd=='species':data=dict(species=catalog['species'][p['id']-1],evolutions=[],learnset=[],encounters=[],origins={})
        elif cmd=='acquisition':data=dict(target=p,sources=[],partial=True,clock=None)
        elif cmd=='daycare_sources':data=[source]
        elif cmd=='breeding_preview':
            assert p['offspring_pid']>0 and p['offspring_pid']<=65535
            data=dict(rom_md5=catalog['profile']['md5'],parents=[child,child],compatibility=50,child=child,seed=p['seed'],offspring_pid=p['offspring_pid'],rng_after=123,partial=True)
        elif cmd=='map_navigation':data=dict(map_id=p['id'],outgoing=[],incoming=[],approaches=[],truncated=False,diagnostics=[])
        elif cmd in ('map_image','sprite','object_sprite','trainer_sprite'):data=dict(url='')
        else:raise AssertionError(req)
        route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=data)))
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome',headless=True)
        page=browser.new_page(viewport=dict(width=1100,height=800));page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror',lambda e:errors.append(str(e)));page.route('**/api',respond)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'));page.get_by_role('button',name='ROM reference',exact=True).click()
        pane=page.locator('.breeding-panel');pane.locator('summary').first.click()
        expect(pane).to_contain_text('Daycare meadow')
        pane.get_by_role('button',name='Preview ordinary egg receipt',exact=True).click()
        expect(pane).to_contain_text('native compatibility check accepts')
        expect(pane).to_contain_text('not the chance of producing an egg')
        expect(pane).to_contain_text('another Pokémon')
        pane.get_by_role('button',name='Test species 2 ↗',exact=True).click()
        expect(page.locator('.reference-detail h2')).to_contain_text('Test species 2')
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        pane.locator('summary').first.click()
        pane.get_by_role('combobox',name='Parent source',exact=True).first.click()
        page.get_by_role('option',name='Party · 1',exact=False).click()
        pane.get_by_role('button',name='Preview ordinary egg receipt',exact=True).click()
        expect(pane.locator('.breeding-result')).to_be_visible()
        assert requests[-1]['payload']['parents'][0]==dict(kind='stored',location=dict(kind='party',slot=0))
        pane.get_by_role('button',name='Daycare meadow',exact=False).click()
        expect(page.locator('.reference-detail h2')).to_contain_text('Daycare meadow')
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        page.get_by_role('button',name='简体中文',exact=True).click()
        pane.locator('summary').first.click();expect(pane).to_contain_text('不会添加蛋或改动 SAV')
        expect(pane).to_contain_text('不能证明当前可达')
        page.set_viewport_size(dict(width=720,height=800));expect(pane.get_by_role('button',name='预览普通领蛋结果',exact=True)).to_be_visible()
        pane.get_by_role('button',name='预览普通领蛋结果',exact=True).click();expect(pane.get_by_role('columnheader',name='个体值',exact=True)).to_be_visible()
        if os.environ.get('GEN3_UI_ARTIFACT_DIR'):
            from pathlib import Path
            out=Path(os.environ['GEN3_UI_ARTIFACT_DIR']);out.mkdir(parents=True,exist_ok=True)
            pane.scroll_into_view_if_needed();page.screenshot(path=str(out/'breeding-compact.png'),full_page=True)
        assert not errors,errors
        assert not any(r['command'] in ('action','export_save','save_bytes') for r in requests)
        print('Native scenario, stored parent, offspring/map/back, bilingual and compact read-only flow passed')
        browser.close()

if __name__=='__main__':main()
