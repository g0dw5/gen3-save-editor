"""Shared Pokémon reference: bidirectional tree, official stats, and ROM isolation.

Synthetic fixtures only. Requires Vite, Playwright and Chrome.
"""
import copy
import json
import os
import re
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, species


def main():
    catalog = copy.deepcopy(CATALOG)
    names = {7: '杰尼龟', 8: '卡咪龟', 9: '水箭龟', 899: '水箭龟Z', 980: '水箭龟'}
    catalog['species'] = [dict(species(i), name=name, dex_number=899 if i == 899 else 9 if i == 980 else i)
                          for i, name in names.items()]
    for row in catalog['species']:
        row['stats'] = [99, 83, 120, 58, 85, 105] if row['id'] == 9 else [99, 103, 120, 78, 135, 115] if row['id'] == 980 else [50]*6
    catalog['items'] = [dict(id=297, name='ROM Mega item')]
    battle = dict(source=9, target=980, kind='mega', trigger={'kind': 'held_item', 'id': 297}, offset=100)
    relations = dict(species=list(names), evolutions=[dict(source=a, target=b, method=4, condition='level', parameter=level, offset=a)
                     for a, b, level in [(7, 8, 16), (8, 9, 36)]], battle_forms=[battle],
                     form_families=[dict(species=[9, 980], offset=200)], name_relations=[dict(source=9, target=899)])
    calls, errors = [], []

    def respond(route):
        req = route.request.post_data_json
        calls.append(req['command'])
        if req['command'] == 'state': data = {'catalog': catalog, 'save': None}
        elif req['command'] == 'sprite': data = {'url': ''}
        elif req['command'] == 'species':
            i = req['payload']['id']
            data = dict(species=next(s for s in catalog['species'] if s['id'] == i),
                        evolutions=[e for e in relations['evolutions'] if e['source'] == i],
                        battle_forms=[battle] if i in (9, 980) else [], relations=relations,
                        learnset=[], encounters=[])
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json', body=json.dumps({'ok': True, 'data': data}))

    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page(viewport={'width': 1440, 'height': 1000})
        page.add_init_script("localStorage.setItem('gen3.locale','zh')")
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.route('**/api', respond)
        for profile in ['BW fixture', 'Rocket fixture', 'DP fixture']:
            catalog['profile'].update(id=profile, md5=profile, label=profile)
            page.goto(os.environ.get('GEN3_UI_URL', 'http://127.0.0.1:5173'))
            page.get_by_role('button', name='ROM 资料', exact=True).click()
            dialog = page.get_by_role('dialog')
            expect(dialog.locator('.reference-tabs').get_by_role('button', name='宝可梦', exact=True)).to_be_visible()
            dialog.locator('.reference-rows button').filter(has_text=re.compile(r'^9水箭龟(?: ·|$)')).click()
            table = dialog.locator('.base-stats-comparison')
            expect(table.locator('tbody tr')).to_have_count(6)
            expect(table.locator('tbody tr').first).to_contain_text('79')
            expect(table.locator('tbody tr').first).to_contain_text('99')
            expect(table.locator('tfoot')).to_contain_text('530')
            expect(table.locator('tfoot')).to_contain_text('550')
            expect(table.locator('thead th').nth(1)).to_have_text('官方参照')
            expect(table.locator('thead th').nth(2)).to_have_text('当前 ROM')
            tree = dialog.locator('.evolution-tree')
            cards = tree.locator('.evolution-card')
            expect(cards).to_have_count(len(names))
            for identifier in names:
                expect(tree.locator(f'.evolution-card[data-species="{identifier}"]')).to_have_count(1)
            if profile == 'Rocket fixture':
                for width in (900, 1440):
                    page.set_viewport_size({'width': width, 'height': 1000})
                    assert table.evaluate('(el) => el.scrollWidth <= el.clientWidth + 1')
                    assert tree.evaluate('(el) => el.scrollWidth <= el.clientWidth + 1')
                output = os.environ.get('GEN3_REFERENCE_SCREENSHOTS')
                if output:
                    os.makedirs(output, exist_ok=True)
                    dialog.screenshot(path=os.path.join(output, 'stats-and-evolution.png'))
                    tree.scroll_into_view_if_needed()
                    dialog.screenshot(path=os.path.join(output, 'evolution-tree.png'))
            for target in (8, 7, 9, 980, 899, 9, 899):
                tree.locator('.evolution-node').filter(has_text=re.compile(rf'#{target}(?: ·|$)')).first.click()
                expect(dialog.locator('.reference-detail h2')).to_contain_text(f'#{target}')
                expect(tree).to_contain_text('转换方法未确认')
                if target == 980:
                    expect(table.locator('tfoot')).to_contain_text('630')
                    expect(table.locator('tfoot')).to_contain_text('650')
                if target == 899:
                    expect(table.locator('tbody tr').first.locator('td').first).to_have_text('—')
            # Custom species using official dex 899 must not pick Wyrdeer.
            expect(dialog.locator('.species-stats')).to_contain_text('尚无唯一的官方对应条目')
            choice = dialog.get_by_role('combobox', name='参照宝可梦／形态', exact=True)
            choice.fill('水箭龟')
            page.get_by_role('option', name='#9 水箭龟', exact=True).click()
            expect(table.locator('tfoot')).to_contain_text('530')
            tree.locator('.evolution-node').filter(has_text=re.compile(r'#9(?: ·|$)')).first.click()
            expect(dialog.locator('.reference-detail h2')).to_contain_text('#9')
            tree.locator('.evolution-node').filter(has_text=re.compile(r'#899(?: ·|$)')).first.click()
            expect(dialog.locator('.reference-detail h2')).to_contain_text('#899')
            expect(dialog.locator('.species-stats')).to_contain_text('手动选择的对照条目')
            expect(table.locator('tfoot')).to_contain_text('530')
        assert 'action' not in calls and not errors, errors
        browser.close()
    print('Passed: all-profile reference UI, six vertical stats + totals, Mega/ordinary distinction, custom dex collision, bidirectional family traversal, local reference persistence and read-only navigation.')


if __name__ == '__main__': main()
