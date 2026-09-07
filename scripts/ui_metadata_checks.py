"""Semantic Pokémon-editor checks run by test_ui.py against a disposable save."""
from playwright.sync_api import expect


def check_metadata(page, command, output):
    tabs = page.locator('.editor-tabs')
    editor = page.locator('.pokemon-editor')

    def apply():
        editor.get_by_role('button', name='Apply changes', exact=True).click()
        expect(editor.get_by_role('button', name='Apply changes', exact=True)).to_be_disabled()

    def pokemon():
        return next(row['pokemon'] for row in command('state')['save']['pokemon']
                    if row['location'] == {'kind': 'party', 'slot': 0})

    page.get_by_label('Nature', exact=True).select_option('15')  # Modest
    tabs.get_by_role('button', name='Stats', exact=True).click()
    expect(editor.locator('.nature-modifier.increase')).to_have_count(1)
    expect(editor.locator('.nature-modifier.decrease')).to_have_count(1)
    expect(editor.locator('.nature-modifier.increase')).to_have_attribute('aria-label', 'Sp. Atk increased by 10%')
    expect(editor.locator('.nature-modifier.decrease')).to_have_attribute('aria-label', 'Attack reduced by 10%')
    apply()
    assert pokemon()['nature'] == 15
    page.screenshot(path=str(output / 'editor-nature-en.png'))

    tabs.get_by_role('button', name='Moves', exact=True).click()
    move = editor.locator('.move-card').first
    move.get_by_label('PP Ups', exact=True).fill('3')
    base = command('state')['catalog']['moves'][pokemon()['moves'][0]]['pp']
    maximum = base * 8 // 5
    expect(move.get_by_label('Maximum PP', exact=True)).to_have_value(str(maximum))
    expect(move.get_by_label('Maximum PP', exact=True)).to_be_disabled()
    move.get_by_label('Current PP', exact=True).fill(str(min(7, maximum)))
    apply()
    assert pokemon()['pp_ups'][0] == 3
    assert pokemon()['pps'][0] == min(7, maximum)
    page.screenshot(path=str(output / 'editor-pp-en.png'))

    tabs.get_by_role('button', name='Origin', exact=True).click()
    editor.get_by_label('Trainer ID (TID)', exact=True).fill('321')
    editor.get_by_label('Secret ID (SID)', exact=True).fill('123')
    editor.get_by_label('OT gender', exact=True).select_option('1')
    catalog = command('state')['catalog']
    location = next(l['id'] for l in catalog['met_locations'] if l['name'] == '绿岭市')
    editor.get_by_label('Met location', exact=True).select_option(str(location))
    editor.get_by_label('Caught in', exact=True).select_option('4')
    expect(editor.get_by_label('Origin game', exact=True)).to_have_value('3')
    assert editor.get_by_label('Pokémon language', exact=True).locator('option[value="6"]').count() == 0
    editor.get_by_label('Fateful encounter flag', exact=True).check()
    editor.locator('.metadata-details summary').click()
    expect(editor.get_by_label('Combined OT ID', exact=True)).to_be_disabled()
    apply()
    current = pokemon()
    assert current['ot_id'] == (123 << 16) | 321
    assert current['met_location'] == location and current['ot_gender'] == 1
    assert current['ribbons'] == 0xF8000000
    page.get_by_role('button', name='简体中文', exact=True).click()
    expect(editor.get_by_label('捕获球', exact=True)).to_have_value('4')
    page.screenshot(path=str(output / 'editor-origin-zh.png'))
    page.get_by_role('button', name='English', exact=True).click()

    tabs.get_by_role('button', name='Advanced', exact=True).click()
    editor.get_by_label('Circle', exact=True).check()
    editor.get_by_label('Heart', exact=True).check()
    expect(editor.get_by_label('Days contagious', exact=True)).to_be_disabled()
    editor.get_by_label('Infection state', exact=True).select_option('active')
    editor.get_by_label('Strain (1–15)', exact=True).select_option('3')
    editor.get_by_label('Days contagious', exact=True).select_option('4')
    editor.get_by_label('Cool ribbon rank', exact=True).select_option('4')
    editor.get_by_label('Champion Ribbon', exact=True).check()
    editor.locator('.metadata-details summary').click()
    expect(editor.get_by_label('Personality ID', exact=True)).to_be_disabled()
    expect(editor.get_by_label('Reserved ribbon bits (preserved)', exact=True)).to_have_value('15')
    expect(editor.get_by_label('Reserved ribbon bits (preserved)', exact=True)).to_be_disabled()
    apply()
    current = pokemon()
    assert current['markings'] & 15 == 9
    assert current['pokerus'] == 0x34
    assert current['ribbons'] == 0xF8008004
    editor.get_by_label('Infection state', exact=True).select_option('cured')
    apply()
    assert pokemon()['pokerus'] == 0x30
    page.get_by_role('button', name='简体中文', exact=True).click()
    expect(editor.get_by_label('感染状态', exact=True)).to_have_value('cured')
    editor.get_by_label('感染状态', exact=True).scroll_into_view_if_needed()
    page.screenshot(path=str(output / 'editor-advanced-zh.png'))
    page.get_by_role('button', name='English', exact=True).click()
    tabs.get_by_role('button', name='Overview', exact=True).click()
    print('Metadata UI passed: nature, PP, origin labels, unsigned ribbons, markings, Pokérus')
