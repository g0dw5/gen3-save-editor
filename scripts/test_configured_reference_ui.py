"""Production-config integration: intercept JSON modules, not application logic."""
import copy
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, species


def main():
    catalog = copy.deepcopy(CATALOG)
    catalog['profile'].update(id='mapping-fixture', md5='mapping-fixture')
    catalog['species'] = [dict(species(i),name=name) for i,name in [(9,'水箭龟'),(990,'超梦'),(991,'超梦')]]
    approved = {'9': dict(status='none',target=None), '990':dict(status='direct',target='150:超级超梦Ｘ'), '991':dict(status='comparison',target='150:超级超梦Ｙ')}
    def api(route):
        req=route.request.post_data_json
        if req['command']=='state': data=dict(catalog=catalog,save=None)
        elif req['command']=='sprite': data=dict(url='')
        elif req['command']=='species':
            s=next(s for s in catalog['species'] if s['id']==req['payload']['id'])
            data=dict(species=s,evolutions=[],battle_forms=[],relations=dict(species=[s['id']],evolutions=[],battle_forms=[],form_families=[],name_relations=[]),learnset=[],encounters=[])
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=data)))
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome',headless=True)
        page=browser.new_page();page.add_init_script("localStorage.setItem('gen3.locale','zh')")
        page.route('**/api',api)
        def config(route):
            route.fulfill(content_type='text/javascript',body='export default '+json.dumps(dict(schema=1,profile_id='mapping-fixture',rom_md5='mapping-fixture',entries=approved if '/bw.json' in route.request.url else {})))
        page.route('**/ui/data/species-mappings/*.json*',config)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        page.get_by_role('button',name='ROM 资料',exact=True).click()
        dialog=page.get_by_role('dialog')
        for id,label in [(990,'游戏映射表 · 已确认对应'),(991,'游戏映射表 · 仅作数值参照'),(9,'游戏映射表已标注：无官方对应条目。')]:
            dialog.locator('.reference-rows button').filter(has_text=str(id)).first.click()
            expect(dialog.locator('.species-stats')).to_contain_text(label)
            if id==9:expect(dialog.locator('.base-stats-comparison tbody tr').first.locator('td').first).to_have_text('—')
        catalog['profile']['md5']='another-game'
        page.reload();page.get_by_role('button',name='ROM 资料',exact=True).click()
        dialog.locator('.reference-rows button').filter(has_text='9水箭龟').first.click()
        expect(dialog.locator('.base-stats-comparison tbody tr').first.locator('td').first).to_have_text('79')
        browser.close()
    print('Passed: generated JSON loader, direct/comparison labels, explicit-none fallback suppression and other-ROM isolation.')


if __name__=='__main__': main()
