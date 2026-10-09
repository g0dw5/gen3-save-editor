"""Player reference/guide workflows in five ROM-scoped synthetic UI contexts.
Native tables and safe read-only status derivation are exercised by Rust separately.
Run with Vite, Playwright and Chrome; no private ROM or save is shipped here.
"""
import copy
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, species
from test_editor_navigation import pokemon


def main():
    expect.set_options(timeout=15000)
    requests, errors = [], []
    profiles = ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY133"]
    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        for locale in ["en", "zh"]:
            for key in profiles:
                cat = copy.deepcopy(CATALOG)
                cat['profile'].update(id=key, md5=key, label=key, clock=dict(starts=[5,10,17,20]), capabilities=dict(world=True, save_edit=True, dex=True))
                cat['species'] = [dict(species(i), name=f'{key} Pokémon {i}') for i in [1,2]]
                cat['moves'] = [dict(id=i, name=f'{key} Move {i}', pp=10, power=50, accuracy=90, priority=0, move_type=i%2, category=(i-1)%3, description='Effect', effect=0, chance=0, target=0, flags=0) for i in range(1,7)]
                cat['items'] = [dict(id=i, name=f'{key} Item {i}' if i!=3 else '?????', description='Item', price=100, pocket=1, tm_move=1 if i==2 else None) for i in range(1,5)]
                game_map = dict(id='0-0',name=f'{key} Map',region=1,width=4,height=4,map_type=1)
                encounter = dict(selector=None,species=1,map_id='0-0',map_name=f'{key} Map',region=1,method='grass',min_level=3,max_level=5,weight=50,encounter_rate=20,slot=0,offset=1,conditional=True,periods=['day'])
                night = dict(encounter,species=2,offset=2,periods=['night'],weight=100)
                surf = dict(encounter,species=2,method='surf',periods=['base'],offset=3,weight=100)
                marker = dict(id='npc',kind='npc',x=1,y=1,elevation=0,local_id=1,graphics_id=1,movement_type=0,underfoot=None,flag=None,receipt_flag=None,offset=1,script=2,rewards=[],pokemon=[],teaching=[],daycare=[],scripted_movements=[dict(kind='apply',offset=2,local_id=1,map_id=None,movement_script=3,reuses_last_actor=False,conditions=[])],stopped_at=[])
                world = dict(maps=[game_map,dict(game_map,id='0-1',name='Broken unused map'),dict(game_map,id='0-2',name='No known entrance')],trainers=[],encounters=[encounter,night,surf],map_groups=[],trainer_locations=dict(locations=[]),map_events=[dict(map_id='0-0',markers=[marker],unplaced_rewards=[],unplaced_pokemon=[],unplaced_movements=marker['scripted_movements'],stopped_at=[])],reference_visibility=dict(items=[dict(id=3,reason='reserved'),dict(id=4,reason='unreferenced')],maps=[dict(id='0-1',reason='invalid_layout'),dict(id='0-2',reason='unreferenced')]))
                mon = pokemon(1,dict(kind='party',slot=0))
                snap = dict(trainer={'name':'TEST'},pokemon=[mon],boxes=[],bag=[],dex=[],active_slot=0,counter=1,backup_valid=True,dirty=False,can_undo=False,can_redo=False,changes=[])
                def source(kind, offset, **extra):
                    row=dict(kind=kind,offset=offset,map_id=None,x=None,y=None,related=[],quantity=None,min_level=None,max_level=None,encounter_percent=None,held_percent=None,conditions=[],periods=[],status='unknown',repeatable=None,partial=True);row.update(extra);return row
                acquisition = [source('machine',1,related=[dict(kind='item',id=2)]),source('pickup',2,map_id='0-0',x=1,y=1,related=[dict(kind='item',id=2)]),source('shop',3,map_id='0-0',related=[dict(kind='item',id=2)]),source('move_tutor',4,map_id='0-0',x=1,y=1),source('learn_egg',5,related=[dict(kind='species',id=2)])]
                def task(id, **extra):
                    row=dict(id=id,kind='side',map_id='0-0',x=1,y=1,actor=1,goals=[],text=[f'{key} Dialogue clue'],checks=[],status='unknown',stage=None,next_candidate=False,prerequisites=[],partial=True,journal=None)
                    row.update(extra);return row
                tasks=[task('next',kind='main',stage=7,next_candidate=True,status='ready',prerequisites=[['prior']]),task('reward',goals=[dict(kind='item',id=2)],status='completed'),task('prior',kind='prerequisite',text=['Fulfil this earlier scene'],prerequisites=[['next']])]
                if key == 'MERCURY133':
                    tasks.append(task('journal:2', kind='journal', status='in_progress', journal=dict(
                        title='Test journey', objective='Visit the forest', accepted=True,
                        locations=[dict(map_id='0-0',x=1,y=1,actor=1)],
                        phases=[dict(title='Current entry',text='The current journal text',visible=True),
                                dict(title='Future entry',text='A later journal text',visible=False)])))
                def respond(route):
                    req=route.request.post_data_json;command,payload=req['command'],req['payload'];requests.append((key,req))
                    if command=='state': result=dict(catalog=cat,save=snap)
                    elif command=='world': result=world
                    elif command=='species': result=dict(species=cat['species'][payload['id']-1],evolutions=[],encounters=[],learnset=[dict(move_id=i%6+1,source=['level','egg','tm','tutor'][i//12],species=payload['id'],level=i+1 if i<12 else None,index=None,offset=i) for i in range(48)])
                    elif command=='acquisition': result=dict(target=payload,sources=acquisition if payload['kind']=='move' else [],partial=True,clock=None)
                    elif command=='adventure_guide':
                        assert payload['expected_rom_md5']==key
                        result=dict(rom_md5=key,current_stage=6 if key=='ROCKET' else None,story_supported=key=='ROCKET',tasks=tasks,partial=True)
                    elif command=='map_navigation': result=dict(map_id=payload['id'],outgoing=[],incoming=[],approaches=[],truncated=False,diagnostics=[])
                    elif command=='event_dependencies': result=dict(rom_md5=key,condition={},writers=[],coverage={},next_offset=None,partial=True)
                    elif command in ['sprite','map_image','object_sprite']: result=dict(url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="%23cce5d5"/></svg>',warnings=[])
                    else: raise AssertionError(f'Unexpected request {req}')
                    route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=result)))
                page=browser.new_page(viewport=dict(width=1100,height=840))
                page.set_default_navigation_timeout(120000)
                page.on('pageerror',lambda error: errors.append(str(error)))
                page.add_init_script(f"localStorage.setItem('gen3.locale','{locale}')")
                page.route('**/api',respond)
                page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
                page.get_by_role('button',name='ROM reference' if locale=='en' else 'ROM 资料',exact=True).click()
                dialog=page.get_by_role('dialog')
                expect(dialog.locator('.reference-tabs button')).to_have_count(6)
                expect(dialog.locator('pre')).to_have_count(0)
                expect(dialog.locator('.species-stats select')).to_have_count(0)
                learn=dialog.locator('.reference-section').first
                learn.locator('summary').click()
                expect(learn.locator('tbody tr')).to_have_count(24)
                learn.get_by_label('Source' if locale=='en' else '来源',exact=True).select_option('egg')
                expect(learn.locator('tbody tr')).to_have_count(12)
                learn.get_by_label('Move category' if locale=='en' else '招式分类',exact=True).select_option('1')
                expect(learn.locator('tbody tr')).to_have_count(4)
                learn.get_by_label('Type' if locale=='en' else '属性',exact=True).select_option('1')
                expect(learn.locator('tbody tr')).to_have_count(2)
                learn.get_by_placeholder('Search learnable moves' if locale=='en' else '搜索可学招式').fill(f'{key} Move 5')
                expect(learn.locator('tbody tr')).to_have_count(2)
                dialog.locator('.reference-tabs button').nth(1).click()
                expect(dialog.locator('.acquisition-panel')).to_contain_text('How to obtain' if locale=='en' else '招式获取与学习')
                expect(dialog.locator('.acquisition-panel article')).to_have_count(4)
                dialog.locator('.acquisition-panel').get_by_label('Method' if locale=='en' else '方式',exact=True).select_option('shop')
                expect(dialog.locator('.acquisition-panel article')).to_have_count(1)
                expect(dialog.locator('pre')).to_have_count(0)
                dialog.locator('.reference-tabs button').nth(2).click()
                expect(dialog.locator('.reference-rows > button')).to_have_count(2)
                dialog.get_by_label('Show entries with uncertain use' if locale=='en' else '显示用途未确认的条目',exact=False).check()
                expect(dialog.locator('.reference-rows > button')).to_have_count(4)
                dialog.get_by_label('Show entries with uncertain use' if locale=='en' else '显示用途未确认的条目',exact=False).uncheck()
                dialog.locator('.reference-tabs button').nth(4).click()
                expect(dialog.locator('.reference-rows > button')).to_have_count(1)
                expect(dialog.locator('.map-encounters')).to_be_visible()
                expect(dialog.locator('.encounter-method-table')).to_have_count(3)
                dialog.locator('.map-encounters select').select_option('night')
                expect(dialog.locator('.encounter-method-table')).to_have_count(2)
                expect(dialog.locator('.map-encounters')).not_to_contain_text(f'{key} Pokémon 1')
                expect(dialog.locator('.map-encounters')).to_contain_text(f'{key} Pokémon 2')
                expect(dialog.locator('pre')).to_have_count(0)
                expect(dialog).not_to_contain_text('movement_script')
                dialog.get_by_role('button',name='Close' if locale=='en' else '关闭',exact=True).click()
                page.get_by_role('button',name='Adventure guide' if locale=='en' else '冒险攻略',exact=True).click()
                guide=page.get_by_role('dialog')
                expect(guide.locator('.adventure-layout')).to_be_visible()
                expect(guide.locator('.guide-map')).to_be_visible()
                if key=='ROCKET': expect(guide.locator('.guide-next')).to_contain_text('6')
                elif key == 'MERCURY133':
                    expect(guide.locator('.guide-filters select').first).to_have_value('journal')
                    expect(guide.locator('.reference-detail h2')).to_have_text('Test journey')
                    expect(guide.locator('.quest-journal')).to_contain_text('Visit the forest')
                    expect(guide.get_by_text('The current journal text',exact=True)).to_be_visible()
                    expect(guide.get_by_text('A later journal text',exact=True)).not_to_be_visible()
                    guide.get_by_text('Later entries' if locale=='en' else '后续日志',exact=True).click()
                    guide.get_by_text('Future entry',exact=True).click()
                    expect(guide.get_by_text('A later journal text',exact=True)).to_be_visible()
                    guide.locator('.guide-filters select').first.select_option('all')
                else: expect(guide).to_contain_text('not yet verified' if locale=='en' else '全局主线进度尚未核实')
                guide.locator('.guide-filters input:not([type=checkbox])').fill(f'{key} Item 2')
                expect(guide.locator('.reference-rows > button')).to_have_count(1)
                guide.locator('.reference-rows > button').click()
                expect(guide.locator('.reference-detail')).to_contain_text('Completed' if locale=='en' else '已完成')
                expect(guide.locator('.reference-detail')).to_contain_text('receipt marker' if locale=='en' else '领取标记')
                guide.locator('.guide-filters input:not([type=checkbox])').fill('')
                guide.locator('.reference-rows > button').first.click()
                guide.locator('.reference-detail > details').nth(1).locator('summary').first.click()
                expect(guide.locator('.quest-dependency-tree')).to_contain_text('Prerequisite clue' if locale=='en' else '前置任务线索')
                guide.locator('.quest-dependency-tree summary button').first.click()
                expect(guide.locator('.reference-detail')).to_contain_text('Fulfil this earlier scene')
                guide.locator('.reference-detail > button').first.click()
                expect(guide.locator('.reference-detail h2')).to_contain_text('7')
                expect(guide.locator('pre')).to_have_count(0)
                page.set_viewport_size(dict(width=720,height=740))
                expect(guide.locator('.guide-filters input:not([type=checkbox])')).to_be_visible()
                if os.environ.get('GEN3_UI_ARTIFACTS') and key=='ROCKET':
                    out=Path(os.environ['GEN3_UI_ARTIFACTS']);out.mkdir(parents=True,exist_ok=True);page.screenshot(path=str(out/f'guide-{locale}.png'))
                guide.locator('.guide-map').click()
                reference = page.get_by_role('dialog',name='ROM reference' if locale=='en' else 'ROM 资料',exact=True)
                expect(reference.locator('.map-focus')).to_be_visible()
                expect(reference.locator('.reference-detail h2')).to_contain_text(f'{key} Map')
                reference.get_by_role('button',name='Close' if locale=='en' else '关闭',exact=True).click()
                expect(guide.locator('.reference-detail h2')).to_contain_text('7')
                page.close()
        assert not errors,errors
        assert all(req['command'] not in ['action','export_save','save_bytes'] for _,req in requests)
        browser.close()
    print('Passed: five ROM-scoped bilingual learnset/source filters, compact time encounters, hidden-entry reveal, no developer traces, read-only illustrated guide/search/receipt/dependency/back and compact layout.')


if __name__=='__main__': main()
