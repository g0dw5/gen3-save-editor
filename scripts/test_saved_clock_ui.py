"""Read-only exact Mercury clock -> encounter query -> planner/HTML UI checks.

Run a token-authorized development bridge and Vite. Requires Playwright/Chrome,
GEN3_ROM_MERCURY12, GEN3_SAVE_MERCURY12 and the other four GEN3_ROM_* paths.
No SAVE action/export or ROM write is sent. HTML exports are private test output.
"""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect


def main():
    expect.set_options(timeout=60000)
    url = os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173')
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome',headless=True)
        page=browser.new_page(viewport=dict(width=1280,height=900),accept_downloads=True)
        errors,writes=[],[]
        page.on('pageerror',lambda e:errors.append(str(e)))
        def track(req):
            if req.url.endswith('/api') and req.method=='POST' and req.post_data_json['command'] in ('action','export_save','save_bytes'):
                writes.append(req.post_data_json['command'])
        page.on('request',track)
        def call(command,payload):
            r=page.request.post(url+'/api',data=dict(command=command,payload=payload),timeout=60000).json()
            assert r['ok'],r
            return r['data']
        call('open_rom',dict(path=os.path.abspath(os.environ['GEN3_ROM_MERCURY12'])))
        call('open_save',dict(path=os.path.abspath(os.environ['GEN3_SAVE_MERCURY12'])))
        clock=call('clock_query',dict(hour=None,weekday=None))
        assert clock['source']=='save_virtual' and clock['current_clock_verified']
        saved=clock['saved'];text=f"{saved['year']}-{saved['month']:02d}-{saved['day']:02d}"
        for locale in ['en','zh']:
            page.add_init_script(f"localStorage.setItem('gen3.locale','{locale}')")
            page.goto(url)
            page.get_by_role('button',name='ROM reference' if locale=='en' else 'ROM 资料',exact=True).click()
            panel=page.locator('.acquisition-panel')
            mode=panel.get_by_role('combobox')
            expect(mode).to_have_value('save')
            expect(panel.locator('.clock-report')).to_contain_text(text)
            expect(panel.locator('.clock-report')).to_contain_text('Saved virtual clock' if locale=='en' else '存档虚拟时钟')
            mode.select_option('17')
            expect(panel.locator('.clock-report')).to_contain_text('Simulated condition' if locale=='en' else '模拟条件')
            expect(panel.locator('.clock-report')).to_contain_text('17:00')
            expect(panel.locator('.clock-report')).not_to_contain_text(text)
            mode.select_option('all');expect(panel.locator('.clock-report')).to_have_count(0)
            mode.select_option('save');expect(panel.locator('.clock-report')).to_contain_text(text)
            page.get_by_role('button',name='Collection planning' if locale=='en' else '收集规划',exact=True).click()
            planning=page.locator('.collection-panel')
            expect(planning.locator('.clock-report')).to_contain_text(text)
            with page.expect_download() as info:
                page.get_by_role('button',name='Export standalone HTML' if locale=='en' else '导出独立 HTML',exact=True).click()
            html=Path(info.value.path()).read_text()
            assert text in html and ('Saved virtual clock' if locale=='en' else '存档虚拟时钟') in html
            assert '<script>' not in html
            page.set_viewport_size(dict(width=720,height=780))
            expect(planning.locator('.clock-report')).to_be_visible()
            print(locale,'saved time, simulated/all periods, planner and standalone HTML passed',flush=True)
            page.set_viewport_size(dict(width=1280,height=900))
        # Each other exact fingerprint must lose the previous clock and SAVE on ROM load.
        for key in ['BW','DP','ROCKET','ULTIMATE']:
            call('open_rom',dict(path=os.path.abspath(os.environ['GEN3_ROM_'+key])))
            report=call('clock_query',dict(hour=None,weekday=None))
            assert report['saved'] is None and report['effective_hour'] is None
            page.goto(url)
            page.get_by_role('button',name='ROM 资料',exact=True).click()
            expect(page.locator('.clock-report')).to_have_count(0)
            expect(page.locator('.clock-scenario')).to_have_count(0)
            print(key,'no Mercury SAVE clock or query rules retained',flush=True)
        assert not errors,errors
        assert not writes,writes
        browser.close()
    print('No save mutations or exports; only standalone HTML downloaded')


if __name__=='__main__':main()
