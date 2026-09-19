"""Human-readable ROM references with synthetic fixtures, in both UI languages.

Requires Vite, Playwright and Chrome. Never loads a ROM or user save.
"""
import copy
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, WORLD, species


def main():
    catalog = copy.deepcopy(CATALOG)
    catalog['profile'].update(
        save={'pockets': [{'id': 'medicine', 'category': 7}]},
        hidden_power={'move_id': 237, 'formula': 'gen6_fixed60'})
    catalog['moves'] = [dict(id=237, name='Test move', power=1, move_type=18, category=1,
                            pp=15, accuracy=100, priority=0, chance=30,
                            description='Readable move effect.', effect=987, target=4, flags=32, offset=123)]
    catalog['items'] = [dict(id=1, name='Test medicine', description='Heals.', price=100, pocket=7, tm_move=None)]
    catalog['abilities'] = [dict(id=i, name=f'Ability {i}', description='Ability description') for i in range(3)]
    world = copy.deepcopy(WORLD)
    mon = dict(species=1, level=50, level_rule='fixed', held_item=0, moves=[237], moves_explicit=True, offset=0, iv_quality=255,
               generation=dict(gender='random', nature=0, ability_id=1, ability_options=[1, 1], ivs=[31]*6, evs=[0]*6, personality_parameter=0))
    second = copy.deepcopy(mon)
    second['generation']['ability_options'] = [1, 2]
    world['trainers'] = [dict(id=1, name='Test trainer', diagnostics=[], class_name='Class',
                             portrait=0, female=False, double_battle=False, items=[], ai=0,
                             offset=0, party=[mon, second])]
    world['trainers'][0]['class'] = 1
    errors = []

    def respond(route):
        req = route.request.post_data_json
        command = req['command']
        if command == 'state': data = {'catalog': catalog, 'save': None}
        elif command == 'world': data = world
        elif command in ('sprite', 'trainer_sprite'): data = {'url': ''}
        elif command == 'species':
            data = {'species': species(req['payload']['id']), 'learnset': [], 'encounters': [],
                    'evolutions': [{'method': 4, 'parameter': 90, 'condition': 'level', 'target': 2},
                                   {'method': 7, 'parameter': 1, 'condition': 'item', 'target': 2},
                                   {'method': 24, 'parameter': 18, 'condition': 'move_type', 'target': 2}]}
        else: raise AssertionError(req)
        route.fulfill(content_type='application/json', body=json.dumps({'ok': True, 'data': data}))

    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        for locale in ('en', 'zh'):
            page = browser.new_page(viewport={'width': 1400, 'height': 1000})
            page.add_init_script(f"localStorage.setItem('gen3.locale', '{locale}')")
            page.on('pageerror', lambda e: errors.append(str(e)))
            page.route('**/api', respond)
            page.goto(os.environ.get('GEN3_UI_URL', 'http://127.0.0.1:5173'))
            page.get_by_role('button', name='ROM reference' if locale == 'en' else 'ROM 资料', exact=True).click()
            dialog = page.get_by_role('dialog')
            detail = dialog.locator('.reference-detail')
            expect(detail).to_contain_text('Lv. 90' if locale == 'en' else '达到 90 级')
            expect(detail).to_contain_text('Test medicine')
            expect(detail).to_contain_text('Type 18')
            assert '"method"' not in detail.inner_text()
            tabs = dialog.locator('.reference-tabs')
            tabs.get_by_role('button', name='Moves' if locale == 'en' else '招式', exact=True).click()
            expect(detail).to_contain_text('Readable move effect.')
            expect(detail).to_contain_text('100%')
            expect(detail).to_contain_text('30%')
            expect(detail).to_contain_text('60')
            assert '987' not in detail.inner_text()
            detail.locator('summary').click()
            expect(detail.locator('pre')).to_contain_text('987')
            tabs.get_by_role('button', name='Items' if locale == 'en' else '道具', exact=True).click()
            expect(detail.locator('.detail-pairs')).to_contain_text('Medicine' if locale == 'en' else '药品')
            tabs.get_by_role('button', name='Trainers' if locale == 'en' else '对手训练家', exact=True).click()
            cards = dialog.locator('.trainer-mon-card')
            expect(cards).to_have_count(2)
            expect(cards.nth(0).get_by_role('button', name='Ability 1 ↗', exact=True)).to_have_count(1)
            expect(cards.nth(0)).to_contain_text('Fixed ability' if locale == 'en' else '固定特性')
            expect(cards.nth(1)).to_contain_text('Randomly selected' if locale == 'en' else '随机选择')
            expect(cards.nth(1).get_by_role('button', name='Ability 2 ↗', exact=True)).to_be_visible()
            page.close()
        browser.close()
    assert not errors, errors
    print('Passed: bilingual evolution conditions, Fairy, move description/evidence, percentages, fixed power, pocket names and deduplicated trainer ability choices.')


if __name__ == '__main__':
    main()
