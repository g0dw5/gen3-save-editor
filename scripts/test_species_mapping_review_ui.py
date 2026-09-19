"""Exercise the standalone reviewer against a temporary store, never real decisions."""
import copy
import json
from pathlib import Path
import tempfile
import threading
from http.server import ThreadingHTTPServer
from playwright.sync_api import sync_playwright, expect
from review_species_mappings import ROOT, ReviewStore, handler_for


def main():
    official = json.loads((ROOT / 'ui/data/official-stats.json').read_text())
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        catalogs = {}
        for md5 in ['game-a', 'game-b']:
            config = dict(schema=1, profile_id=md5, rom_md5=md5, revision=0, entries={})
            species = []
            for id, target in [(990, '150:超级超梦Ｘ'), (991, '150:超级超梦Ｙ')]:
                config['entries'][str(id)] = dict(status='pending', target=target, candidates=['150:超级超梦Ｘ', '150:超级超梦Ｙ'], confidence='high', reason='rom-mega-stone-xy', suggested_mode='direct', previously_matched=False, note='')
                species.append(dict(id=id, name='超梦', stats=[106,190,100,130,154,100]))
            (root / f'{md5}.json').write_text(json.dumps(config))
            catalogs[md5] = dict(profile=dict(label=md5,md5=md5), species=species,items=[dict(id=307,name='超梦石X'),dict(id=308,name='超梦石Y')],battle_forms=[dict(source=150,target=990,kind='mega',trigger=dict(kind='held_item',id=307)),dict(source=150,target=991,kind='mega',trigger=dict(kind='held_item',id=308))])
        store = ReviewStore(root, official, catalogs, root / 'backups', root / 'output')
        server = ThreadingHTTPServer(('127.0.0.1', 0), lambda *args: None)
        port = server.server_port
        server.RequestHandlerClass = handler_for(store, 'test-token', port)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        try:
            with sync_playwright() as p:
                browser = p.chromium.launch(channel='chrome', headless=True)
                page = browser.new_page(viewport=dict(width=1440,height=1050))
                errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
                page.goto(f'http://127.0.0.1:{port}')
                expect(page.locator('.candidate.active')).to_contain_text('超级超梦Ｘ')
                expect(page.locator('.rom-form-label')).to_have_text('Mega 进化 X')
                page.get_by_role('button',name='确认直接映射',exact=True).click()
                expect(page.locator('#progressText')).to_have_text('已审核 1 / 2')
                expect(page.locator('.candidate.active')).to_contain_text('超级超梦Ｙ')
                page.reload()
                expect(page.locator('#progressText')).to_have_text('已审核 1 / 2')
                page.get_by_role('button',name='game-b',exact=True).click()
                expect(page.locator('#progressText')).to_have_text('已审核 0 / 2')
                page.get_by_role('button',name='game-a',exact=True).click()
                page.get_by_role('button',name='无官方对应',exact=True).click()
                expect(page.locator('#count')).to_have_text('0 条')
                page.locator('#filter').select_option('all')
                page.locator('[data-species="991"]').click()
                expect(page.locator('.heading .badge')).to_have_text('无官方对应')
                page.get_by_role('button',name='恢复待审核',exact=True).click()
                page.get_by_role('searchbox',name='搜索官方参照').fill('超级超梦X')
                page.locator('#officialOptions .candidate').first.click()
                expect(page.locator('.candidate.active')).to_contain_text('超级超梦Ｘ')
                expect(page.locator('.rom-form-label')).to_have_text('Mega 进化 Y')
                page.locator('#note').fill('手动选择 X，对比用途')
                page.get_by_role('button',name='仅作数值参照',exact=True).click()
                expect(page.locator('#progressText')).to_have_text('已审核 2 / 2')
                page.get_by_role('button',name='撤销上次',exact=True).click()
                expect(page.locator('#progressText')).to_have_text('已审核 1 / 2')
                with page.expect_download() as download:
                    page.get_by_role('button',name='生成修改器配置',exact=True).click()
                out=json.loads((root/'output/game-a.json').read_text())
                assert out['entries']=={'990':dict(status='direct',target='150:超级超梦Ｘ')},out
                assert 'candidates' not in json.dumps(out)
                page.set_viewport_size(dict(width=900,height=900))
                assert page.evaluate('document.documentElement.scrollWidth <= innerWidth')
                page.set_viewport_size(dict(width=390,height=850))
                assert page.evaluate('document.documentElement.scrollWidth <= innerWidth')
                assert not errors,errors
                # API must reject writes without the session token; no CORS bypass.
                rejected=page.request.post(f'http://127.0.0.1:{port}/decision',data={})
                assert rejected.status==403
                browser.close()
        finally:
            server.shutdown();server.server_close()
    print('Passed: X/Y candidates, persistent review, game isolation, none/reopen/search, comparison, undo, minimal publishing, responsive widths and write authorization.')


if __name__ == '__main__': main()
