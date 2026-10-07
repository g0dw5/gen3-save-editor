"""ROM names and compact complete trainer teams across five UI contexts.

Synthetic API fixtures only. Verifies same-name IDs still navigate separately,
player data survives the table layout, and long bilingual labels fit narrow panes.
Requires Vite, Playwright and Chrome. No ROM or SAV mutations.
"""
import copy
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, WORLD, species
from test_editor_navigation import pokemon


def main():
    expect.set_options(timeout=15000)
    errors, requests = [], []
    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        for locale in ('zh', 'en'):
            for profile in ('BW', 'DP', 'Rocket', 'Ultimate', 'Mercury12'):
                print(profile, locale, flush=True)
                name = f'{profile} ROM 宝可梦'
                catalog = copy.deepcopy(CATALOG)
                catalog['profile'].update(id=profile, md5=profile, label=profile)
                catalog['species'] = [dict(species(i), name=name) for i in (1, 2)]
                catalog['form_families'] = [dict(species=[1, 2], offset=1)]
                catalog['battle_forms'] = [dict(source=1, target=2, kind='mega', trigger=dict(kind='held_item', id=1))]
                catalog['items'] = [dict(id=0, name='', tm_move=None), dict(id=1, name=f'{profile} ROM Item', tm_move=None)]
                catalog['abilities'] = [dict(id=i, name=f'{profile} ROM Ability {i}', description='') for i in range(3)]
                catalog['moves'] = [dict(id=i, name=f'{profile} ROM Move {i}', pp=10, category=0, move_type=0, power=50) for i in range(5)]
                world = copy.deepcopy(WORLD)
                world['maps'][0].update(region=1, map_type=1)
                encounter = dict(species=2, map_id='26-13', map_name='Test map', region=1, method='grass', min_level=3, max_level=5, weight=100, periods=[], selector=None, slot=0, offset=1)
                world['encounters'] = [encounter]
                world['map_events'] = []
                base = dict(species=1, level=50, level_rule='fixed', held_item=1, moves=[1, 2, 3, 4], moves_explicit=True, offset=1, iv_quality=255,
                            generation=dict(context='ordinary', gender='female', nature=15, ability_id=1, ability_options=[1, 1], ivs=[31, 30, 29, 28, 27, 26], evs=[252, 0, 0, 252, 4, 0], personality_parameter=0))
                team = [copy.deepcopy(base) for _ in range(6)]
                team[1]['generation']['ability_options'] = [1, 2]
                team[5].update(species=2, generation=None)
                world['trainers'] = [dict(id=1, name='Table trainer', diagnostics=[], class_name='Class', portrait=0, female=False, double_battle=False, items=[], ai=0, offset=1, party=team)]
                row = pokemon(1, dict(kind='party', slot=0))
                snap = dict(trainer=dict(name='TEST'), pokemon=[row], boxes=[], bag=[], dex=[], active_slot=0, counter=1, backup_valid=True, dirty=False, can_undo=False, can_redo=False, changes=[])

                def respond(route):
                    req = route.request.post_data_json
                    requests.append(req)
                    cmd, payload = req['command'], req['payload']
                    if cmd == 'state': data = dict(catalog=catalog, save=snap)
                    elif cmd == 'world': data = world
                    elif cmd == 'species':
                        data = dict(species=catalog['species'][payload['id']-1], evolutions=[], learnset=[], encounters=[], relations=dict(species=[1, 2], evolutions=[], battle_forms=catalog['battle_forms'], form_families=catalog['form_families']))
                    elif cmd == 'acquisition': data = dict(target=payload, sources=[], partial=True, clock=None)
                    elif cmd == 'trainer_references': data = dict(rom_md5=profile, trainer_id=1, references=[], total_matches=0, next_offset=None, partial=True, coverage={})
                    elif cmd == 'map_navigation': data = dict(map_id=payload['id'], incoming=[], outgoing=[], approaches=[], truncated=False, diagnostics=[])
                    elif cmd in ('sprite', 'trainer_sprite', 'map_image', 'object_sprite'): data = dict(url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="%23cce5d5"/></svg>')
                    else: raise AssertionError(req)
                    route.fulfill(content_type='application/json', body=json.dumps(dict(ok=True, data=data)))

                page = browser.new_page(viewport=dict(width=1100, height=840))
                page.add_init_script(f"localStorage.setItem('gen3.locale','{locale}')")
                page.on('pageerror', lambda e: errors.append(str(e)))
                page.route('**/api', respond)
                page.goto(os.environ.get('GEN3_UI_URL', 'http://127.0.0.1:5173'))
                expect(page.locator('.pokemon-editor h2')).to_have_text(name)
                page.get_by_role('button', name='ROM 资料' if locale == 'zh' else 'ROM reference', exact=True).click()
                dialog = page.get_by_role('dialog')
                expect(dialog.locator('.reference-rows > button > span:last-child')).to_have_text([name, name])
                # Display names are unadorned; numeric IDs keep duplicate entries distinct.
                nodes = dialog.locator('.evolution-node')
                expect(nodes).to_have_count(2)
                for node in nodes.all():
                    assert node.locator(':scope > span:last-child').evaluate('(el)=>el.firstChild.textContent') == name
                dialog.locator('.evolution-card[data-species="2"] .evolution-node').click()
                expect(dialog.locator('.reference-detail h2')).to_have_text(f'{name} #2')
                dialog.locator('.reference-tabs button').nth(4).click()
                expect(dialog.locator('.reference-detail h2')).to_have_text('Test map #26-13')
                expect(dialog.locator('.encounter-pokemon')).to_have_text(name)
                dialog.locator('.encounter-pokemon').click()
                expect(dialog.locator('.reference-detail h2')).to_have_text(f'{name} #2')
                dialog.locator('.reference-tabs button').nth(5).click()
                entries = dialog.locator('.trainer-party-table > tbody')
                expect(entries).to_have_count(6)
                entry = entries.first
                expect(entry.locator('.trainer-mon-title button')).to_have_text(name)
                expect(entry.locator('.trainer-mon-facts')).to_contain_text('Nature 15')
                expect(entry.locator('.trainer-mon-facts')).to_contain_text(f'{profile} ROM Item')
                expect(entry.get_by_role('button', name=f'{profile} ROM Ability 1 ↗', exact=True)).to_have_count(1)
                expect(entries.nth(1).get_by_role('button', name=f'{profile} ROM Ability 2 ↗', exact=True)).to_be_visible()
                expect(entry.locator('.gender-badge')).to_contain_text('雌性' if locale == 'zh' else 'Female')
                expect(entry).to_contain_text('Lv. 50')
                expect(entry.locator('.trainer-moves > div > span')).to_have_text([f'{profile} ROM Move {i}' for i in range(1, 5)])
                expect(entry.locator('.trainer-stat-table tbody tr').first.locator('td')).to_have_text(['31', '30', '29', '28', '27', '26'])
                expect(entry.locator('.trainer-stat-table tbody tr').nth(1).locator('td')).to_have_text(['252', '0', '0', '252', '4', '0'])
                expect(entries.last.locator('.trainer-stat-table tbody tr').first.locator('td')).to_have_text(['?']*6)
                expect(entries.last.locator('.trainer-mon-facts')).to_contain_text('待解析' if locale == 'zh' else 'Unresolved')
                table = dialog.locator('.trainer-party-table')
                expect(table.locator(':scope > thead th')).to_have_count(3)
                for width in (1100, 900, 720):
                    page.set_viewport_size(dict(width=width, height=840))
                    entry.scroll_into_view_if_needed()
                    assert dialog.locator('.reference-detail').evaluate('(el)=>el.scrollWidth <= el.clientWidth+1'), (profile, locale, width)
                    assert table.evaluate('(el)=>el.scrollWidth <= el.clientWidth+1'), (profile, locale, width)
                    assert entry.locator('tr:first-child > td').evaluate_all('(cells)=>cells.every(el=>el.scrollWidth <= el.clientWidth+1)'), (profile, locale, width)
                if os.environ.get('GEN3_UI_ARTIFACTS') and profile == 'Rocket':
                    out = Path(os.environ['GEN3_UI_ARTIFACTS']); out.mkdir(parents=True, exist_ok=True)
                    page.set_viewport_size(dict(width=1100, height=840))
                    entry.scroll_into_view_if_needed()
                    dialog.screenshot(path=str(out/f'trainer-table-{locale}.png'))
                page.close()
        browser.close()
    assert not errors, errors
    assert not any(r['command'] in ('action', 'export_save', 'save_bytes') for r in requests)
    print('Passed: five ROM-scoped bilingual names, distinct same-name links, six-member compact teams, complete fields, random/unresolved values and no horizontal overflow at three widths.')


if __name__ == '__main__':
    main()
