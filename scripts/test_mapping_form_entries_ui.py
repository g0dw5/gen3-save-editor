"""Every same-name form is a peer review record, with no parent decision."""
import copy,json,tempfile,threading
from pathlib import Path
from http.server import ThreadingHTTPServer
from playwright.sync_api import sync_playwright,expect
from review_species_mappings import ROOT,ReviewStore,handler_for


def main():
    md5='59c658a1081f542086de1060bb65f0b3'
    ids=[386,920,921,1128,1129,1130]
    forms=['普通形态','速度形态','速度形态','攻击形态','防御形态','速度形态']
    entries={str(id):dict(status='pending',target='386:'+form,candidates=['386:'+form],confidence='medium',reason='rom-form-family',suggested_mode='direct',previously_matched=True,note='')for id,form in zip(ids,forms)}
    catalog=dict(profile=dict(md5=md5,label='Independent forms'),species=[dict(id=id,name='代欧奇希斯',stats=[50]*6)for id in ids],items=[],form_families=[dict(species=[386,1128,1129,1130],offset=100)],battle_forms=[])
    with tempfile.TemporaryDirectory() as tmp:
        root=Path(tmp);path=root/'rocket.json';path.write_text(json.dumps(dict(schema=1,profile_id='test',rom_md5=md5,revision=0,entries=entries)))
        store=ReviewStore(root,json.loads((ROOT/'ui/data/official-stats.json').read_text()),{md5:catalog},root/'backups')
        server=ThreadingHTTPServer(('127.0.0.1',0),lambda *args:None);port=server.server_port;server.RequestHandlerClass=handler_for(store,'test',port);threading.Thread(target=server.serve_forever,daemon=True).start()
        try:
            with sync_playwright() as p:
                browser=p.chromium.launch(channel='chrome',headless=True);page=browser.new_page(viewport=dict(width=1440,height=1000));page.goto(f'http://127.0.0.1:{port}')
                expect(page.locator('.row')).to_have_count(6)
                expect(page.locator('.rom-relations,.rom-form-card')).to_have_count(0)
                for id in ids:expect(page.locator(f'[data-species="{id}"]')).to_have_count(1)
                expect(page.locator('[data-species="1128"] .entry-form')).to_have_text('攻击形态')
                page.locator('[data-species="920"]').click();page.locator('#relatedFilter').click()
                expect(page.locator('.row')).to_have_count(6)
                page.locator('[data-decision="none"]').click();expect(page.locator('#globalMessage')).to_contain_text('已保存 ROM #920')
                data=json.loads(path.read_text())['entries'];assert data['920']['status']=='none'
                for id in ids:
                    if id!=920:assert data[str(id)]==entries[str(id)]
                page.locator('[data-species="1128"]').click();page.locator('[data-decision="direct"]').click();expect(page.locator('#globalMessage')).to_contain_text('已保存 ROM #1128')
                data=json.loads(path.read_text())['entries'];assert data['1128']['status']=='direct' and data['1128']['target']=='386:攻击形态'
                for id in [386,921,1129,1130]:assert data[str(id)]==entries[str(id)]
                page.reload();page.locator('#filter').select_option('all');expect(page.locator('.row')).to_have_count(6)
                expect(page.locator('[data-species="920"]')).to_contain_text('无官方对应')
                expect(page.locator('[data-species="386"]')).to_contain_text('待审核')
                browser.close()
        finally:server.shutdown();server.server_close()
    print('Passed: six peer rows, no nested form cards, every form visible including previous auto matches, independent decisions and reload.')


if __name__=='__main__':main()
