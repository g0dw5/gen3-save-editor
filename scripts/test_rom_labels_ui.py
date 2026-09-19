"""Exact-ROM catalog display/search across locales; synthetic saves and writes only.

Requires GEN3_ROM_BW/DP/ROCKET, GEN3_CLI, Vite and Playwright/Chrome.
"""
import copy
import json
import os
import subprocess
from playwright.sync_api import sync_playwright, expect
from test_editor_navigation import pokemon
from test_reference_navigation import WORLD


def main():
    errors = []
    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport={'width':1440,'height':1000})
        page.on('pageerror', lambda e: errors.append(str(e)))
        # Reuse the page to catch cross-ROM/locale caching of labels.
        for key in ('GEN3_ROM_BW','GEN3_ROM_ROCKET','GEN3_ROM_DP'):
            catalog = json.loads(subprocess.check_output([os.environ['GEN3_CLI'],'catalog',os.environ[key]]))
            names = catalog['natures']; expected = names[15]['name']
            row = pokemon(1, {'kind':'party','slot':0})
            row['pokemon'].update(nature=15, effective_nature=15, nature_override=15, pid=40)
            save = dict(trainer={'name':'TEST'}, pokemon=[row], boxes=[], bag=[], dex=[], dirty=False,
                        can_undo=False, can_redo=False, changes=[], backup_valid=True, active_slot=0, counter=1)
            world = copy.deepcopy(WORLD)
            world['trainers'] = [dict(id=1,name='Label trainer',diagnostics=[],class_name='Test',portrait=0,
                female=False,double_battle=False,items=[],ai=0,offset=0,party=[dict(species=1,level=50,
                held_item=0,moves=[],moves_explicit=True,offset=0,iv_quality=255,
                generation=dict(gender='random',nature=15,ability_id=0,ability_options=[0],ivs=[31]*6,evs=[0]*6,personality_parameter=0))])]
            world['trainers'][0]['class']=1
            def respond(route):
                req=route.request.post_data_json; cmd=req['command']
                if cmd=='state': data={'catalog':catalog,'save':save}
                elif cmd=='world': data=world
                elif cmd=='fishing_spots': data=None
                elif cmd in ('sprite','trainer_sprite'): data={'url':''}
                elif cmd=='species': data={'species':catalog['species'][0],'evolutions':[],'learnset':[],'encounters':[]}
                else: raise AssertionError(req)  # No user save edits or export.
                route.fulfill(content_type='application/json',body=json.dumps({'ok':True,'data':data}))
            page.unroute('**/api');page.route('**/api',respond)
            for locale in ('zh','en'):
                page.add_init_script(f"localStorage.setItem('gen3.locale','{locale}')")
                page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5174'))
                nature=page.get_by_role('combobox',name='性格' if locale=='zh' else 'Nature',exact=True)
                expect(nature).to_have_value(expected)
                nature.fill(expected.split('(')[0])
                expect(page.get_by_role('option',name=expected,exact=True)).to_be_visible()
                nature.press('Escape')
                page.locator('.editor-tabs').get_by_role('button',name='能力' if locale=='zh' else 'Stats',exact=True).click()
                expect(page.locator('.nature-summary')).to_contain_text(expected)
                expect(page.locator('.nature-modifier.increase')).to_have_count(1)
                expect(page.locator('.nature-modifier.decrease')).to_have_count(1)
                page.get_by_role('button',name='ROM 资料' if locale=='zh' else 'ROM reference',exact=True).first.click()
                dialog=page.get_by_role('dialog')
                for t in set(catalog['species'][0]['types']):
                    expect(dialog.locator('.type-tag').filter(has_text=catalog['type_names'][t]).first).to_be_visible()
                dialog.locator('.reference-tabs').get_by_role('button',name='对手训练家' if locale=='zh' else 'Trainers',exact=True).click()
                expect(dialog.locator('.trainer-mon-facts')).to_contain_text(expected)
            print(f'{key}: nature search, stat markers, ROM types and trainer nature in both locales passed')
        browser.close()
    assert not errors, errors


if __name__=='__main__': main()
