"""Exact-ROM Pikachu family: unique cards, folded forms and cross-profile navigation.

Uses private ROMs, generated API responses and no save. Requires Vite + Playwright.
"""
import base64,json,os,re,subprocess
from pathlib import Path
from playwright.sync_api import sync_playwright,expect
from test_reference_navigation import WORLD


def main():
    errors=[]
    cli=os.environ['GEN3_CLI']
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome',headless=True)
        for key in ('GEN3_ROM_BW','GEN3_ROM_ROCKET','GEN3_ROM_DP'):
            path=os.environ[key]
            catalog=json.loads(subprocess.check_output([cli,'catalog',path]))
            details={};images={}
            def detail(id):
                if id not in details:details[id]=json.loads(subprocess.check_output([cli,'species',path,str(id)]))
                return details[id]
            def respond(route):
                req=route.request.post_data_json;cmd=req['command']
                if cmd=='state':data={'catalog':catalog,'save':None}
                elif cmd=='species':data=detail(req['payload']['id'])
                elif cmd=='world':data=WORLD
                elif cmd=='fishing_spots':data=None
                elif cmd=='sprite':
                    id=req['payload']['id']
                    if id not in images:images[id]='data:image/png;base64,'+base64.b64encode(subprocess.check_output([cli,'sprite',path,str(id)])).decode()
                    data={'url':images[id]}
                else:raise AssertionError(req)
                route.fulfill(content_type='application/json',body=json.dumps({'ok':True,'data':data}))
            page=browser.new_page(viewport={'width':1440,'height':1000})
            page.add_init_script("localStorage.setItem('gen3.locale','zh')")
            page.on('pageerror',lambda e:errors.append(str(e)))
            page.route('**/api',respond)
            page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5174'))
            page.get_by_role('button',name='ROM 资料',exact=True).click()
            dialog=page.get_by_role('dialog')
            dialog.locator('.reference-rows button').filter(has_text=re.compile(r'^25皮卡丘(?: ·|$)')).click()
            tree=dialog.locator('.evolution-tree')
            expect(tree.locator('.evolution-node.current')).to_contain_text('#25')
            expected=set(detail(25)['relations']['species'])
            actual=tree.locator('.evolution-card').evaluate_all('(rows)=>rows.map(r=>Number(r.dataset.species))')
            assert len(actual)==len(set(actual)) and set(actual)==expected,(key,actual,expected)
            expect(tree.locator('.evolution-card[data-species="25"]')).to_have_count(1)
            if key.endswith('ROCKET'):
                for f in tree.locator('.evolution-forms').all():expect(f).not_to_have_attribute('open','')
                assert tree.locator('.evolution-card:visible').count()==5
                tree.scroll_into_view_if_needed()
                output=os.environ.get('GEN3_EVOLUTION_SCREENSHOT')
                if output:page.screenshot(path=output)
                folded=tree.locator('.evolution-forms').first
                folded.locator('summary').click()
                folded.locator('.evolution-node').first.click()
                expect(tree.locator('.evolution-forms[open]')).to_have_count(1)
                actual=tree.locator('.evolution-card').evaluate_all('(rows)=>rows.map(r=>Number(r.dataset.species))')
                assert len(actual)==len(set(actual))
            for width in (900,1440):
                page.set_viewport_size({'width':width,'height':1000})
                assert tree.evaluate('e=>e.scrollWidth<=e.clientWidth+1')
            page.close()
            print(f'{key}: unique Pikachu cards, retained relations and responsive layout passed')
        browser.close()
    assert not errors,errors


if __name__=='__main__':main()
