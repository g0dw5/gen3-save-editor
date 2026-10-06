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
    marker = dict(id='pickup-1',kind='pickup',x=1,y=1,elevation=0,local_id=1,graphics_id=None,movement_type=0,flag=10,receipt_flag=10,offset=100,script=100,rewards=[reward],stopped_at=[])
    world = dict(maps=maps,map_events=[dict(map_id='0-1',markers=[marker],unplaced_rewards=[],stopped_at=[])],encounters=[],trainers=[],trainer_locations=dict(locations=[]),map_groups=[])
    world['trainers']=[dict(id=1,name='ROM trainer',class_name='Test class',portrait=1,female=False,double_battle=False,items=[],ai=0,diagnostics=[],offset=250,party=[dict(species=2,level=10,iv_quality=0,level_rule='fixed',generation=None,held_item=0,moves=[0,0,0,0],moves_explicit=True,offset=250)],**{'class':0})]
    edge = dict(from_='0-0',to='0-1',kind='warp',x=2,y=1,target_x=1,target_y=1,warp_index=0,target_warp=0,direction=None,displacement=None,offset=200,unresolved=None)
    edge['from'] = edge.pop('from_')
    source = dict(underfoot=True,kind='pickup',map_id='0-1',region=1,x=1,y=1,related=[],quantity=1,min_level=None,max_level=None,encounter_percent=None,held_percent=None,periods=[],conditions=[],requirements=[],evolution=None,status='unknown',receipt_flag=None,repeatable=None,offset=100,partial=True,in_scenario=None)
    g1=dict(condition=dict(kind='flag',id=10,value=1,comparison=1,taken=True),satisfied=False,actual=0,unresolved=None)
    g2=dict(condition=dict(kind='flag',id=11,value=1,comparison=1,taken=True),satisfied=None,actual=None,unresolved=None)
    source['conditions']=[g1]
    reward['conditions']=[g1['condition']]
    writer=dict(effect=dict(kind='flag',id=10,operation='set',operand=None,value=1,offset=300,conditions=[g2['condition']]),reference=dict(map_id='0-1',map_name='Test cave floor',region=1,kind='trigger',x=1,y=1,local_id=None,offset=200,root=250,conditions=[],entry_unresolved=True),conditions=[g2],text=[dict(offset=800,text='<script>context</script>')],stopped_at=[400],path_complete=False)
    report=dict(rom_md5='test',condition=g1,writers=[writer],coverage=dict(checked_scripts=10,total_scripts=12,failed_scripts=1,truncated=True),total_matches=1,next_offset=None,partial=True)
    save = dict(trainer={'name':'TEST'},pokemon=[pokemon(2,dict(kind='party',slot=0))],boxes=[dict(index=i,name=f'Box {i}',count=0,wallpaper=0) for i in range(14)],bag=[],dex=[],active_slot=0,counter=1,backup_valid=True,dirty=False,can_undo=False,can_redo=False,changes=[])
    task = dict(target=dict(kind='species',id=1),family=[1],existing_family_members=[],source=source,alternatives=1)
    catalog['species'][1]['name'] = 'ROM parent'
    capture = dict(source, kind='grass',underfoot=None,min_level=5,max_level=8,encounter_percent=20,periods=['night'],in_scenario=False)
    evolution = dict(method=7,condition='item',parameter=1,auxiliary=0,target=1,offset=300,requirements=[])
    task['preparation'] = dict(origin=2,current_count=0,source=capture,steps=[{'from':2,'evolution':evolution,'related':[dict(kind='item',id=1)]}],needs_hatching=False,truncated=False,partial=True)
    plan = dict(rom_md5='test',basis='individuals',families=True,owned_count=1,missing_count=1,regions=[dict(region=1,tasks=[task])],entrances=[dict(map_id='0-1',chains=[[edge]],truncated=False)],partial=True)
    plan['prerequisites']=dict(reports=[report],entrances=[dict(map_id='0-1',chains=[[edge]],truncated=False)],skipped_conditions=1,truncated=True,partial=True)
    errors, requests = [], []
    def respond(route):
        req = route.request.post_data_json
        command,payload = req['command'],req['payload']; requests.append(req)
        if command=='state': data=dict(catalog=catalog,save=save)
        elif command=='open_save':
            writer['conditions']=[dict(g2,actual=1,satisfied=True)]
            data=copy.deepcopy(save);data['counter']=2
        elif command=='world': data=world
        elif command=='species': data=dict(species=catalog['species'][payload['id']-1],evolutions=[],learnset=[],encounters=[],origins={})
        elif command=='acquisition': data=dict(target=payload,sources=[source],partial=True,clock=None)
        elif command=='map_navigation': data=dict(map_id=payload['id'],outgoing=[edge] if payload['id']=='0-0' else [],incoming=[edge] if payload['id']=='0-1' else [],approaches=[[edge]] if payload['id']=='0-1' else [],truncated=False,diagnostics=[])
        elif command in ('collection','collection_export'):
            if command=='collection_export': assert payload['expected_rom_md5']=='test' and payload['query']['basis']=='individuals'
            data=plan
        elif command=='trainer_references':
            assert payload['expected_rom_md5']=='test' and payload['trainer_id']==1
            battle=dict(trainer_id=1,battle_type=3,offset=250,conditions=[g2['condition']],role='primary')
            data=dict(rom_md5='test',trainer_id=1,references=[dict(clue_id='0-1:trigger:0',battle=battle,reference=writer['reference'],conditions=writer['conditions'],visibility=[g1],text=writer['text'],stopped_at=[400])],total_matches=1,next_offset=None,coverage=report['coverage'],partial=True)
        elif command=='event_search':
            assert payload['expected_rom_md5']=='test'
            refs=[dict(id=f'0-1:trigger:{i}',reference=copy.deepcopy(writer['reference']),text=[dict(offset=800,text=f'Reward <script>context</script> line {i}')],visibility=[],effects=[dict(effect=writer['effect'],conditions=writer['conditions'],observed=True if sum(r['command']=='open_save' for r in requests)>1 else None)],battles=[dict(trainer_id=1,battle_type=3,offset=250,conditions=[g2['condition']],role='primary')],effects_truncated=False,stopped_at=[700],path_complete=False) for i in range(35)]
            refs=[entry for entry in refs if (not payload['map_id'] or entry['reference']['map_id']==payload['map_id']) and (not payload['search'] or payload['search'].lower() in entry['text'][0]['text'].lower())]
            start=payload['offset'];selected=next((entry for entry in refs if entry['id']==payload['selected_id']),None)
            data=dict(rom_md5='test',entries=refs[start:start+32],selected=selected,total_matches=len(refs),next_offset=start+32 if start+32<len(refs) else None,coverage=report['coverage'],partial=True)
        elif command=='event_dependencies':
            assert payload['expected_rom_md5']=='test'
            data=copy.deepcopy(report)
            if payload['id']==11:
                data['condition']=g2;data['writers'][0]['conditions']=[g1]
            elif sum(r['command']=='event_dependencies' and r['payload']['id']==10 for r in requests)>1:
                data['condition']=dict(g1,actual=1,satisfied=True)
        elif command=='breeding_preview':
            assert payload['parents']==[dict(kind='stored',location=dict(kind='party',slot=0)),dict(kind='stored',location=dict(kind='box',box_index=0,slot=1))]
            data=dict(rom_md5='test',parents=[],compatibility=50,child=pokemon(2,dict(kind='party',slot=0))['pokemon'],seed=42,offspring_pid=24,rng_after=1,partial=True,production=None)
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
        expect(page.locator('.acquisition-panel')).to_contain_text('The receipt protocol is not verified for this event')
        page.get_by_role('button', name='简体中文', exact=True).click()
        expect(page.locator('.acquisition-panel')).to_contain_text('尚未确认此事件的领取规则')
        page.get_by_role('button', name='English', exact=True).click()
        assert not any(r['command']=='event_dependencies' for r in requests)
        trace=page.locator('.acquisition-panel .event-dependencies').first
        trace.locator('summary').first.click()
        expect(trace).to_contain_text('May set this event')
        expect(trace).to_contain_text('Tile-triggered event')
        trace.get_by_text('Text referenced by this script',exact=True).click()
        expect(trace.locator('blockquote')).to_have_text('<script>context</script>')
        nested=trace.locator('.event-dependencies').first
        nested.locator('summary').first.click()
        expect(nested.locator('article')).to_have_count(1)
        cycle=nested.locator('.event-dependencies').first
        cycle.locator('summary').first.click()
        expect(cycle).to_contain_text('This dependency repeats')
        assert sum(r['command']=='event_dependencies' for r in requests)==2
        trace.get_by_role('button',name='Refresh saved-state checks',exact=True).first.click()
        expect(trace).to_contain_text('Current query snapshot · Satisfied · Event condition must be set · SAV value set')
        if os.environ.get('GEN3_UI_ARTIFACTS'):
            trace.get_by_text('Text referenced by this script',exact=True).first.click()
            Path(os.environ['GEN3_UI_ARTIFACTS']).mkdir(parents=True,exist_ok=True)
            trace.screenshot(path=str(Path(os.environ['GEN3_UI_ARTIFACTS'])/'prerequisite-clues.png'))
        trace.get_by_role('button',name='Test cave floor (1, 1) ↗',exact=True).first.click()
        expect(page.locator('.map-focus')).to_be_visible()
        expect(page.locator('.map-navigation')).to_contain_text('Test region entrance')
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        page.locator('.acquisition-panel .link-button').filter(has_text='Test cave floor').click()
        expect(page.locator('.map-focus')).to_be_visible()
        expect(page.locator('.map-navigation')).to_contain_text('Test region entrance')
        page.locator('.nav-path .link-button').first.click()
        expect(page.locator('.map-focus')).to_be_visible()
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        expect(page.locator('.map-navigation')).to_contain_text('Test cave floor')
        page.locator('.map-focus').click()
        # Map conditions are ROM-only data; the open trace must recheck a new SAV
        # even when its outer condition and map report did not change.
        maptrace=page.locator('.map-marker-details .event-dependencies').first
        maptrace.locator('summary').first.click()
        expect(maptrace).to_contain_text('May set this event')
        before_refresh=sum(r['command']=='event_dependencies' for r in requests)
        with page.expect_file_chooser() as choice:
            page.get_by_role('button',name='Open save',exact=True).first.click()
        choice.value.set_files(dict(name='synthetic.sav',mimeType='application/octet-stream',buffer=b'fixture-only'))
        expect(maptrace.locator('.condition-details').first).to_contain_text('SAV value set')
        assert sum(r['command']=='event_dependencies' for r in requests)>before_refresh
        page.locator('.map-marker-details .link-button').filter(has_text='Test stone').click()
        expect(page.locator('.acquisition-panel')).to_contain_text('Collected' if False else 'Undetermined')
        page.get_by_role('button',name='Collection planning',exact=True).click()
        expect(page.locator('.collection-panel')).to_contain_text('Missing goals 1')
        prep = page.locator('.collection-preparation')
        expect(prep).to_contain_text('Evolution preparation suggestion')
        expect(prep).to_contain_text('Use Test stone')
        expect(prep).to_contain_text('Encounter slot probability 20%')
        expect(prep).to_contain_text('Outside the query period')
        if os.environ.get('GEN3_UI_ARTIFACTS'):
            out = Path(os.environ['GEN3_UI_ARTIFACTS']); out.mkdir(parents=True, exist_ok=True)
            prep.screenshot(path=str(out/'evolution-preparation.png'))
        prep.get_by_role('button',name='Test stone ↗',exact=True).click()
        expect(page.locator('.acquisition-panel')).to_be_visible()
        assert requests[-1]['command'] == 'acquisition' and requests[-1]['payload']['kind'] == 'item'
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        page.locator('.collection-preparation').get_by_role('button',name='Test cave floor (1, 1) ↗',exact=True).click()
        expect(page.locator('.map-focus')).to_be_visible()
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        page.get_by_role('button',name='简体中文',exact=True).click()
        expect(page.locator('.collection-preparation')).to_contain_text('进化准备建议')
        expect(page.locator('.collection-preparation')).to_contain_text('使用「Test stone」')
        page.get_by_role('button',name='English',exact=True).click()
        with page.expect_download() as info:
            page.get_by_role('button',name='Export standalone HTML',exact=True).click()
        html=Path(info.value.path()).read_text()
        assert '&lt;script&gt;alert(1)&lt;/script&gt;' in html
        assert '<script>' not in html and 'default-src' in html
        assert 'Stand on this tile and use the Itemfinder' in html
        assert 'Test region entrance (2, 1)' in html and 'Test cave floor' in html
        assert 'Find prerequisite clues' in html and 'prerequisite-flag' in html and '&lt;script&gt;context&lt;/script&gt;' in html
        assert 'May set this event' in html and 'Test region entrance (2, 1)' in html
        assert 'Evolution preparation suggestion' in html and 'ROM parent → Test species' in html and 'Use Test stone' in html
        assert 'Encounter slot probability 20%' in html and 'Outside the query period' in html
        task['preparation'] = dict(task['preparation'],current_count=2,source=None)
        page.get_by_role('checkbox',name='Permanent evolution family',exact=True).uncheck()
        expect(page.locator('.collection-preparation')).to_contain_text('Existing non-egg individuals: ROM parent ↗ × 2')
        # Native existing-parent suggestions keep exact locations, item links and HTML.
        task['preparation'] = dict(task['preparation'],current_count=0,source=capture,needs_hatching=True,breeding=dict(parents=[dict(location=dict(kind='party',slot=0),species=2,nickname='<script>parent</script>',gender='female',held_item=1),dict(location=dict(kind='box',box_index=0,slot=1),species=2,nickname='Father',gender='male',held_item=0)],compatibility=50,seed=42,offspring_pid=24,partial=True))
        plan['breeding_coverage']=dict(parent_count=101,checked_pairs=2048,total_pairs=5050,failed_pairs=1,truncated=True,sampled=True,issue='test-only unresolved path')
        page.get_by_role('checkbox',name='Permanent evolution family',exact=True).check()
        expect(page.locator('.collection-breeding')).to_contain_text('Pair existing non-egg individuals')
        expect(page.locator('.collection-panel')).to_contain_text('2048 / 5050')
        expect(page.locator('.collection-panel')).to_contain_text('missing suggestions do not prove breeding is impossible')
        page.get_by_role('button',name='Preview this pairing with the native receipt routine',exact=True).click()
        expect(page.locator('.collection-breeding')).to_contain_text('The full receipt scenario confirms')
        page.locator('.collection-breeding').get_by_role('button',name='Test stone ↗',exact=True).click()
        expect(page.locator('.acquisition-panel')).to_be_visible()
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        page.get_by_role('button',name='简体中文',exact=True).click()
        expect(page.locator('.collection-breeding')).to_contain_text('使用现有非蛋个体配对')
        expect(page.locator('.collection-breeding')).to_contain_text('携带道具')
        with page.expect_download() as info:
            page.get_by_role('button',name='导出独立 HTML',exact=True).click()
        html=Path(info.value.path()).read_text()
        assert '追查前置线索' in html and '可能设置此事件' in html
        assert '孵蛋与进化准备建议' in html and '2048 / 5050' in html
        assert '&lt;script&gt;parent&lt;/script&gt;' in html and '<script>' not in html
        assert '盒子 1 / 2' in html and '不向 SAV 添加个体' in html and '携带道具' in html
        if os.environ.get('GEN3_UI_ARTIFACTS'):
            page.locator('.collection-breeding').screenshot(path=str(Path(os.environ['GEN3_UI_ARTIFACTS'])/'breeding-preparation.png'))
        page.get_by_role('button',name='English',exact=True).click()
        # Event text search -> paged reference -> guard trace -> precise map -> back.
        page.get_by_role('button',name='Event clues',exact=True).click()
        events=page.locator('.event-clues-panel')
        expect(events).to_contain_text('Matching references 35')
        events.get_by_role('textbox',name='Search ROM dialogue or map name').fill('Reward')
        events.get_by_role('combobox',name='Filter event clues by map').select_option('0-1')
        expect(events).to_contain_text('Matching references 35')
        events.get_by_role('button',name='Next references',exact=True).click()
        expect(events.locator('.reference-rows button')).to_have_count(3)
        events.locator('.reference-rows button').last.click()
        expect(events.locator('.event-clue-text blockquote')).to_have_text('Reward <script>context</script> line 34')
        expect(events).to_contain_text('Task completion undetermined')
        assert events.locator('script').count()==0
        events.get_by_role('button',name='Test cave floor (1, 1) ↗',exact=True).click()
        expect(page.locator('.map-focus')).to_be_visible()
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        events=page.locator('.event-clues-panel')
        expect(events.get_by_role('textbox',name='Search ROM dialogue or map name')).to_have_value('Reward')
        expect(events.get_by_role('combobox',name='Filter event clues by map')).to_have_value('0-1')
        expect(events.locator('.reference-rows button')).to_have_count(3)
        expect(events.locator('.event-clue-text blockquote')).to_have_text('Reward <script>context</script> line 34')
        events.locator('.event-dependencies').first.locator('summary').first.click()
        expect(events.locator('.event-dependencies').first).to_contain_text('May set this event')
        before=sum(r['command']=='event_search' for r in requests)
        with page.expect_file_chooser() as choice:
            page.get_by_role('button',name='Open save',exact=True).first.click()
        choice.value.set_files(dict(name='synthetic.sav',mimeType='application/octet-stream',buffer=b'fixture-only'))
        expect(events).to_contain_text('Known result matches the saved snapshot')
        expect(events.locator('.event-clue-effect .condition-details').first).to_contain_text('SAV value set')
        assert sum(r['command']=='event_search' for r in requests)>before
        page.get_by_role('button',name='简体中文',exact=True).click()
        expect(events).to_contain_text('任务完成状态无法确定')
        expect(events).to_contain_text('可能设置此事件')
        if os.environ.get('GEN3_UI_ARTIFACTS'):
            events.screenshot(path=str(Path(os.environ['GEN3_UI_ARTIFACTS'])/'event-clues.png'))
        page.set_viewport_size(dict(width=720,height=740))
        page.wait_for_function("(()=>{const r=document.querySelector('.floating').getBoundingClientRect();return r.right<=innerWidth-7 && r.bottom<=innerHeight-7;})()")
        expect(events.get_by_role('textbox',name='搜索 ROM 对话或地图名称')).to_be_visible()
        expect(events.get_by_role('button',name='Test cave floor (1, 1) ↗',exact=True)).to_be_visible()
        if os.environ.get('GEN3_UI_ARTIFACTS'):
            events.screenshot(path=str(Path(os.environ['GEN3_UI_ARTIFACTS'])/'event-clues-compact.png'))
        page.set_viewport_size(dict(width=1100,height=780))
        events.get_by_role('textbox',name='搜索 ROM 对话或地图名称').fill('not found')
        expect(events).to_contain_text('未找到匹配的可读引用')
        events.get_by_role('textbox',name='搜索 ROM 对话或地图名称').fill('Reward')
        events.get_by_role('button',name='Test cave floor (1, 1) ↗',exact=True).click()
        page.get_by_role('button',name='查询此地图的事件线索 ↗',exact=True).click()
        expect(events.get_by_role('combobox',name='按地图筛选事件线索')).to_have_value('0-1')
        page.get_by_role('button',name='English',exact=True).click()
        page.get_by_role('button',name='Collection planning',exact=True).click()
        page.get_by_role('button',name='Trainers',exact=True).click()
        trainer=page.locator('.trainer-reference-panel')
        expect(trainer).to_contain_text('Opponent record reference')
        expect(trainer).to_contain_text('NPC visibility checks (not task completion)')
        trainer.get_by_role('button',name='Test cave floor (1, 1) ↗',exact=True).click()
        expect(page.locator('.map-focus')).to_be_visible()
        expect(page.locator('.map-navigation')).to_contain_text('Test region entrance')
        page.get_by_role('button',name='Back to previous reference',exact=False).click()
        expect(trainer).to_be_visible()
        trainer.get_by_role('button',name='Read this event context ↗',exact=True).click()
        expect(events).to_contain_text('Task completion undetermined')
        events.get_by_role('button',name='ROM trainer ↗',exact=True).click()
        expect(trainer).to_be_visible()
        before=sum(r['command']=='trainer_references' for r in requests)
        with page.expect_response(lambda response: response.request.method=='POST' and response.request.post_data_json.get('command')=='trainer_references'):
            with page.expect_file_chooser() as choice:
                page.get_by_role('button',name='Open save',exact=True).first.click()
            choice.value.set_files(dict(name='synthetic.sav',mimeType='application/octet-stream',buffer=b'fixture-only'))
        expect(trainer).to_contain_text('SAV value set')
        assert sum(r['command']=='trainer_references' for r in requests)>before
        page.get_by_role('button',name='简体中文',exact=True).click()
        expect(trainer).to_contain_text('战斗引用与格位')
        expect(trainer).to_contain_text('对手记录引用')
        page.get_by_role('button',name='English',exact=True).click()
        page.get_by_role('button',name='Collection planning',exact=True).click()
        assert not any(r['command'] in ('action','export_save','save_bytes') for r in requests)
        assert not errors,errors
        page.set_viewport_size(dict(width=720,height=740))
        expect(page.get_by_role('button',name='Export standalone HTML',exact=True)).to_be_visible()
        print('query tile/entrance/back, reward link, read-only planner, escaped standalone HTML and compact window passed')
        browser.close()


if __name__=='__main__': main()
