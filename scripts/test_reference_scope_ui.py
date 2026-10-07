"""Six-tab references, automatic trainer scenarios and cross-ROM state isolation.

Public synthetic fixtures only. Vite, Playwright and Chrome required.
Real native numeric parity is exercised separately by gen3-core native tests.
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
    keys = ['BW', 'DP', 'Rocket', 'Ultimate', 'Mercury']
    active = [0]
    saved = [True]
    requests, errors = [], []
    held = []

    def fixtures():
        key = keys[active[0]]
        cat = copy.deepcopy(CATALOG)
        cat['profile'].update(id='ultimate-emerald-55' if key == 'Ultimate' else key,
                              md5=key, label=key, max_level=100,
                              capabilities=dict(world=True, save_edit=True, dex=True))
        cat['species'] = [dict(species(i), name=f'{key} Pokémon {i}') for i in [1, 2]]
        cat['form_families'] = [dict(species=[1, 2], offset=1)]
        cat['moves'] = [dict(id=1, name=f'{key} Move', pp=10, power=50, accuracy=90, priority=0,
                             move_type=0, category=0, description='Description', effect=0, chance=0, target=0, flags=0)]
        cat['items'] = [dict(id=0, name='', tm_move=None), dict(id=1, name=f'{key} Item', tm_move=None, description='Item description', price=100, pocket=1)]
        cat['abilities'] = [dict(id=0, name='', description=''), dict(id=1, name=f'{key} Ability', description='Ability description')]
        generation = dict(context='ultimate_template' if key == 'Ultimate' else 'ordinary', gender='female', nature=0,
                          ability_id=1, ability_options=[1], ivs=[24]*6, evs=None if key == 'Ultimate' else [0]*6,
                          personality_parameter=1, ev_increment=4)
        trainer = dict(id=1, name=f'{key} Trainer', class_name='Class', portrait=0, female=False, double_battle=False,
                       items=[], ai=0, diagnostics=[], offset=1, party=[dict(species=1, level=50, level_rule='difficulty' if key=='Ultimate' else 'fixed',
                       held_item=1, moves=[1,0,0,0], moves_explicit=True, offset=2, iv_quality=255, generation=generation)])
        game_map = dict(id='0-0', name=f'{key} Map', region=1, width=4, height=4, map_type=1)
        world = dict(maps=[game_map], map_events=[dict(map_id='0-0',markers=[],unplaced_rewards=[],stopped_at=[])],
                     trainers=[trainer], encounters=[], map_groups=[], trainer_locations=dict(locations=[]))
        mon = pokemon(1, dict(kind='party', slot=0))
        mon['pokemon'].update(level=70, stats=[100,100,100,95,100,100], current_hp=60, egg=False)
        snap = dict(trainer={'name':'TEST'},pokemon=[mon],boxes=[dict(index=i,name=f'Box {i}',count=0) for i in range(14)],
                    bag=[],dex=[],active_slot=0,counter=1,backup_valid=True,dirty=False,can_undo=False,can_redo=False,changes=[])
        return cat, world, snap if saved[0] else None

    def respond(route):
        req = route.request.post_data_json
        command, payload = req['command'], req['payload']
        if command == 'open_rom':
            active[0] = int(payload['name'].split('.')[0]); saved[0] = True
        requests.append((keys[active[0]], req))
        cat, world, snap = fixtures()
        if command in ['state','open_rom']: result = dict(catalog=cat,save=snap)
        elif command == 'open_save': result = snap
        elif command == 'world': result = world
        elif command == 'species':
            result = dict(species=cat['species'][payload['id']-1],evolutions=[],encounters=[],learnset=[dict(move_id=1,level=i,source='level',species=1,offset=i) for i in range(1,45)])
        elif command == 'acquisition': result = dict(target=payload,sources=[],partial=True,clock=None)
        elif command == 'trainer_references':
            assert payload['expected_rom_md5'] == cat['profile']['md5']
            result = dict(rom_md5=cat['profile']['md5'],trainer_id=1,references=[],total_matches=0,next_offset=None,partial=True,coverage={})
        elif command == 'trainer_battle_preview':
            assert keys[active[0]] == 'Ultimate'
            level = max(50,payload['player_max_level']) if payload['difficulty']==4 and payload['player_max_level'] else 50
            result = dict(trainer_id=1,difficulty=payload['difficulty'],player_max_level=payload['player_max_level'],
                          mons=[dict(species=1,base_level=50,level=level,level_source='rom',moves=[1,0,0,0],ivs=None,evs=None)])
        elif command == 'trainer_ev_preview':
            assert payload['player_party'][0]['speed'] == 95 and len(payload['player_party']) == 1
            level = 70 if payload['difficulty']==4 else 50
            assert payload['opponent_levels'] == [level]
            result = dict(trainer_id=1,difficulty=payload['difficulty'],mons=[dict(species=1,level=level,ivs=[31]*6,evs=[payload['difficulty']]*6,alternate_ivs=[31,30,31,31,31,31],alternate_evs=[payload['difficulty'],0,0,0,0,0])])
        elif command == 'map_navigation': result = dict(map_id=payload['id'],incoming=[],outgoing=[],approaches=[],truncated=False,diagnostics=[])
        elif command in ['map_image','sprite','trainer_sprite','object_sprite']:
            result = dict(url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="%23cce5d5"/></svg>',warnings=[])
        else: raise AssertionError(f'Unexpected product request: {req}')
        route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=result)))

    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome',headless=True)
        for locale in ['en','zh']:
            active[0] = 0; saved[0] = True
            page = browser.new_page(viewport=dict(width=1050,height=820))
            page.add_init_script(f"localStorage.setItem('gen3.locale','{locale}')")
            page.on('pageerror',lambda e: errors.append(str(e)))
            page.route('**/api',respond)
            page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
            for index,key in enumerate(keys):
                if index:
                    with page.expect_file_chooser() as fc: page.get_by_role('button',name='Open ROM' if locale=='en' else '打开 ROM',exact=True).click()
                    fc.value.set_files(dict(name=f'{index}.gba',mimeType='application/octet-stream',buffer=b'fixture'))
                    with page.expect_file_chooser() as fc: page.get_by_role('button',name='Open save' if locale=='en' else '打开存档',exact=True).click()
                    fc.value.set_files(dict(name='fixture.sav',mimeType='application/octet-stream',buffer=b'fixture'))
                page.get_by_role('button',name='ROM reference' if locale=='en' else 'ROM 资料',exact=True).click()
                dialog = page.get_by_role('dialog')
                expect(dialog.locator('.reference-tabs button')).to_have_count(6)
                expect(dialog.locator('.reference-detail h2')).to_have_text(f'{key} Pokémon 1 #1')
                learnset = dialog.locator('.reference-section')
                expect(learnset).not_to_have_attribute('open','')
                expect(learnset.get_by_role('button',name=f'{key} Move',exact=True).first).not_to_be_visible()
                learnset.locator('summary').click()
                expect(learnset.get_by_role('button',name=f'{key} Move',exact=True).first).to_be_visible()
                # Cross-link, return, and retained overview expansion.
                learnset.get_by_role('button',name=f'{key} Move',exact=True).first.click()
                expect(dialog.locator('.reference-detail h2')).to_contain_text(f'{key} Move')
                dialog.get_by_role('button',name='Back to previous reference' if locale=='en' else '返回上一条资料',exact=False).click()
                expect(dialog.locator('.reference-detail h2')).to_contain_text(f'{key} Pokémon')
                dialog.locator('.reference-tabs button').nth(5).click()
                expect(dialog.locator('.trainer-party-entry')).to_contain_text(f'{key} Ability')
                expect(dialog.locator('.trainer-party input')).to_have_count(0)
                mode = dialog.get_by_label('Difficulty · whole trainer reference' if locale=='en' else '难度 · 全部训练家资料')
                if key == 'Ultimate':
                    for difficulty in [1,2,3,4]:
                        mode.select_option(str(difficulty))
                        expect(dialog.locator('.trainer-stat-table tbody tr').nth(1).locator('td').first).to_have_text(str(difficulty))
                        expect(dialog.locator('.trainer-stat-table tbody tr').first.locator('td').nth(1)).to_have_text('31 / 30')
                        expect(dialog.locator('.trainer-stat-table tbody tr').nth(1).locator('td').nth(1)).to_have_text(f'{difficulty} / 0')
                        expect(dialog.locator('.trainer-values-row')).to_contain_text(f'{difficulty*6} / {difficulty}')
                    expect(dialog.locator('.trainer-party-entry')).to_contain_text('Lv. 70')
                    expect(dialog.locator('.trainer-mode-note')).to_contain_text('Species base stats' if locale=='en' else '种族值仍取 ROM')
                else: expect(mode).to_have_count(0)
                if os.environ.get('GEN3_UI_ARTIFACTS') and key=='Ultimate':
                    out=Path(os.environ['GEN3_UI_ARTIFACTS']);out.mkdir(parents=True,exist_ok=True)
                    page.screenshot(path=str(out/f'trainer-{locale}.png'))
                dialog.get_by_role('button',name='Close' if locale=='en' else '关闭',exact=True).click()
                assert all(r['command'] not in ['collection','collection_export','collection_prerequisites','event_search','clock_query','clock_rtc_preview','training_catalog','training_preview','training_services','training_service_preview','action','save_bytes','export_save'] for _,r in requests)
            # Reload Ultimate with no SAV: no template IVs disguised as generated values.
            active[0]=3;saved[0]=False
            page.reload()
            page.get_by_role('button',name='ROM reference' if locale=='en' else 'ROM 资料',exact=True).click()
            dialog=page.get_by_role('dialog')
            dialog.locator('.reference-tabs button').nth(5).click()
            expect(dialog.locator('.trainer-stat-table tbody tr').first.locator('td').first).to_have_text('?')
            expect(dialog.locator('.trainer-stat-table tbody tr').nth(1).locator('td').first).to_have_text('?')
            page.set_viewport_size(dict(width=720,height=740))
            expect(dialog.locator('.reference-tabs button').nth(5)).to_be_visible()
            expect(dialog.get_by_label('Difficulty · whole trainer reference' if locale=='en' else '难度 · 全部训练家资料')).to_be_visible()
            if os.environ.get('GEN3_UI_ARTIFACTS'):page.screenshot(path=str(Path(os.environ['GEN3_UI_ARTIFACTS'])/f'compact-{locale}.png'))
            page.close()
        assert not errors,errors
        browser.close()
    print('Passed: five-adapter switches, six tabs, bilingual/compact UI, source/map/back links, automatic IV/EV inputs and unknown no-SAV template values; no dedicated or write requests.')


if __name__=='__main__':main()
