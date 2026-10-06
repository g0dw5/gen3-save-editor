"""Generated trainer values, automatic reproducible sample, and stale-response isolation.

Synthetic API fixtures only. Start Vite; requires Playwright and Chrome.
"""
import copy
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, WORLD
from test_editor_navigation import pokemon


def main():
    expect.set_options(timeout=30000)
    catalog = copy.deepcopy(CATALOG)
    catalog['profile']['native_trainers'] = {'constructor': 1}
    catalog['abilities'] = [dict(id=0,name='None',description=''), dict(id=1,name='Native Ability',description='')]
    catalog['moves'] = [dict(id=0,name='',pp=0),dict(id=1,name='Native Move',pp=10)]
    catalog['items'] = [dict(id=0,name='',tm_move=None),dict(id=1,name='Native Item',tm_move=None)]
    trainer = dict(id=1,name='Test Trainer',class_name='Test class',portrait=0,female=False,double_battle=False,items=[0]*4,ai=0,offset=1,diagnostics=[],party=[dict(species=1,level=2,iv_quality=0,level_rule='fixed',generation=None,held_item=0,moves=[],moves_explicit=False,offset=2)])
    world = copy.deepcopy(WORLD); world['trainers']=[dict(trainer,id=i,name=f'Test Trainer {i}') for i in [1,2,3]]
    errors, requests, pending = [], [], []
    def respond(route):
        req=route.request.post_data_json;command,payload=req['command'],req['payload'];requests.append(req)
        if command=='state': data=dict(catalog=catalog,save=None)
        elif command=='world': data=world
        elif command=='trainer_references': data=dict(rom_md5=catalog['profile']['md5'],trainer_id=req['payload']['trainer_id'],references=[],total_matches=0,next_offset=None,partial=True,coverage={})
        elif command in ('sprite','trainer_sprite','object_sprite'): data=dict(url='')
        elif command=='acquisition': data=dict(target=payload,sources=[],partial=True,clock=None)
        elif command=='species': data=dict(species=catalog['species'][0],evolutions=[],learnset=[],encounters=[])
        elif command=='trainer_native_preview':
            mon=pokemon(1,dict(kind='party',slot=0))['pokemon']
            mon.update(species=2 if payload['trainer_id']==3 else 1, ivs=[4+payload['trainer_id']]*6,evs=[0]*6,ability_id=1,moves=[1,0,0,0],held_item=1,nature=3,level=17)
            data=dict(rom_md5='test',trainer_id=payload['trainer_id'],seed=payload['seed'],partial=True,scenario='ordinary_zero_context',mons=[mon])
            if payload['trainer_id']==2:
                pending.append((route,data)); return
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=data)))
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome',headless=True)
        page=browser.new_page(viewport=dict(width=1280,height=900))
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on('pageerror',lambda e:errors.append(str(e)))
        page.route('**/api',respond)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        page.get_by_role('button',name='ROM reference',exact=True).click()
        page.locator('.reference-tabs').get_by_role('button',name='Trainers',exact=True).click()
        card=page.locator('.trainer-mon-card')
        expect(card).to_contain_text('Native Ability')
        expect(card).to_contain_text('Native Item')
        expect(card).to_contain_text('Native Move')
        expect(card).to_contain_text('Nature 3')
        expect(card).to_contain_text('Lv. 17')
        expect(card.locator('.trainer-stat-table tbody tr').first.locator('td').first).to_have_text('5')
        expect(page.locator('.trainer-party input')).to_have_count(0)
        page.locator('.reference-rows button').filter(has_text='Test Trainer 2').click()
        page.wait_for_timeout(250)
        assert len(pending)==1
        page.locator('.reference-rows button').filter(has_text='Test Trainer 3').click()
        expect(card.locator('.trainer-stat-table tbody tr').first.locator('td').first).to_have_text('7')
        expect(card.locator('.trainer-mon-title .link-button')).to_have_text('Test species 2')
        route,data=pending.pop();route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=data)))
        expect(card.locator('.trainer-stat-table tbody tr').first.locator('td').first).to_have_text('7')
        assert all(r['payload']['seed']==0 for r in requests if r['command']=='trainer_native_preview')
        assert not errors,errors
        assert not any(r['command']=='action' for r in requests)
        browser.close()
    print('ROM-only native values, baseline limits, automatic sample, stale response isolation and read-only requests passed')


if __name__=='__main__':main()
