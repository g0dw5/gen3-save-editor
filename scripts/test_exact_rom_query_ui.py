"""Real-ROM read-only UI smoke across every registered game profile.

Requires GEN3_ROM_BW/DP/ROCKET/ULTIMATE/MERCURY133, an authorized development
bridge and Vite. ROMs stay private. This replaces the bridge's current ROM but
never opens a SAV or sends mutation/export requests. Run fixtures separately.
"""
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect


def main():
    expect.set_options(timeout=60000)
    url = os.environ.get('GEN3_UI_URL', 'http://127.0.0.1:5173')
    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport={'width': 1360, 'height': 920})
        page.set_default_navigation_timeout(120000)
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        errors = []
        page.on('pageerror', lambda e: errors.append(str(e)))
        for key in ['BW', 'DP', 'ROCKET', 'ULTIMATE', 'MERCURY133']:
            path = Path(os.environ[f'GEN3_ROM_{key}']).resolve()
            response = page.request.post(f'{url}/api', data={
                'command': 'open_rom', 'payload': {'path': str(path)}
            }, timeout=60000).json()
            assert response['ok'], response
            catalog = response['data']['catalog']
            page.goto(url)
            page.get_by_role('button', name='ROM reference', exact=True).click()
            panel = page.locator('.acquisition-panel')
            expect(panel).to_be_visible()
            expect(panel.get_by_text('Reading…', exact=True)).to_have_count(0)
            expect(page.locator('.reference-detail')).to_contain_text(catalog['species'][0]['name'])
            page.locator('.reference-tabs').get_by_role('button', name='Maps', exact=True).click()
            expect(page.locator('.map-navigation h4').first).to_be_visible()
            if catalog['profile'].get('native_trainers'):
                page.locator('.reference-tabs').get_by_role('button', name='Trainers', exact=True).click()
                expect(page.get_by_text('Generated team preview', exact=True)).to_be_visible()
                cell = page.locator('.trainer-party-entry .trainer-stat-table tbody tr').first.locator('td').first
                import re
                expect(cell).to_have_text(re.compile(r'^\d+$'))
                expect(page.locator('.trainer-party [role=alert]')).to_have_count(0)
            print(key, 'actual query/map UI', catalog['profile']['md5'], flush=True)
        assert not errors, errors
        browser.close()
    print('Five exact-ROM query/map UI checks passed; no saves or mutations')


if __name__ == '__main__':
    main()
