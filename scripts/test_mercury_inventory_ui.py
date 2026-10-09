"""Read-only Mercury UI regression; no actions or save exports are sent.

Requires GEN3_ROM_MERCURY133/GEN3_SAVE_MERCURY133 and Vite/dev bridge.
The zero-quantity case changes only a mocked HTTP response, never the SAV.
"""
import json
import os
import re
from playwright.sync_api import sync_playwright, expect


def main():
    expect.set_options(timeout=60000)
    url = os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173")
    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        page = browser.new_page(viewport={"width": 1360, "height": 920})
        page.set_default_navigation_timeout(120000)
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        errors, writes = [], []
        page.on("pageerror", lambda error: errors.append(str(error)))
        def track(request):
            if request.url.endswith('/api') and request.method == 'POST':
                command = request.post_data_json['command']
                if command in ['action', 'export_save', 'save_bytes']:
                    writes.append(command)
        page.on('request', track)
        for command, key in [("open_rom", "GEN3_ROM_MERCURY133"), ("open_save", "GEN3_SAVE_MERCURY133")]:
            response = page.request.post(url + '/api', data={
                'command': command, 'payload': {'path': os.path.abspath(os.environ[key])}
            }, timeout=60000).json()
            assert response['ok'], response
        snapshot = response['data']
        item = next(e for e in snapshot['bag'] if e['pocket'] == 'items' and e['slot'] == 0)
        assert item['item'] != 0, 'fixture needs one general item at slot 0'
        page.goto(url)
        if any(e['location'] == {'kind': 'party', 'slot': 0}
               and e['pokemon']['met_location'] == 143 for e in snapshot['pokemon']):
            page.locator('.party-slots .storage-slot.occupied').first.click()
            page.locator('.editor-tabs').get_by_role('button', name='Origin', exact=True).click()
            expect(page.get_by_role('combobox', name='Met location', exact=True)).to_have_value(re.compile(r'^若叶镇 · #143'))
        page.get_by_role('button', name='Items', exact=True).click()
        quantity = page.get_by_label('Quantity', exact=True)
        expect(quantity).to_have_value(str(item['quantity']))
        row = page.locator('.bag-layout tbody tr').first
        expect(row.locator('td').nth(2)).to_have_text(str(item['quantity']))
        row.get_by_role('button', name='Details', exact=True).click()
        expect(quantity).to_have_value(str(item['quantity']))
        expect(page.get_by_role('button', name='Apply changes', exact=True)).to_be_disabled()
        page.get_by_role('button', name='ROM reference', exact=True).click()
        page.locator('.reference-tabs').get_by_role('button', name='Maps', exact=True).click()
        page.locator('.reference-rows button').filter(has=page.locator('span.id', has_text=re.compile(r'^1-0$'))).click()
        expect(page.locator('.map-explorer')).to_contain_text('also renders with mismatched tiles')
        expect(page.locator('.reference-detail')).to_contain_text('常磐森林')
        # A corrupt/nonempty zero is shown consistently rather than silently becoming 1.
        def zero_response(route):
            if route.request.post_data_json['command'] != 'state':
                route.continue_()
                return
            result = route.fetch().json()
            entry = next(e for e in result['data']['save']['bag'] if e['pocket']=='items' and e['slot']==0)
            entry['quantity'] = 0
            route.fulfill(json=result)
        page.route('**/api', zero_response)
        page.reload()
        page.get_by_role('button', name='Items', exact=True).click()
        expect(page.get_by_label('Quantity', exact=True)).to_have_value('0')
        expect(page.locator('.bag-layout tbody tr').first.locator('td').nth(2)).to_have_text('0')
        expect(page.get_by_role('button', name='Apply changes', exact=True)).to_be_disabled()
        assert not writes, writes
        assert not errors, errors
        browser.close()
    print('Mercury real inventory, location-map warning and zero-quantity UI passed; no writes')


if __name__ == '__main__':
    main()
