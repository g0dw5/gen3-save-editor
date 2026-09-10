"""Inventory regression against a disposable fixture loaded by test_ui.py."""
from playwright.sync_api import expect


def check_inventory(page, command, output):
    page.get_by_role('button', name='Items', exact=True).click()
    pc = page.get_by_role('button', name='PC items', exact=True)
    pc.click()
    editor = page.locator('.bag-layout aside')
    expect(page.locator('.bag-layout tbody tr')).to_have_count(50)
    expect(editor.get_by_role('heading', name='PC items · 1', exact=True)).to_be_visible()

    def apply():
        editor.get_by_role('button', name='Apply changes', exact=True).click()
        expect(editor.get_by_role('button', name='Apply changes', exact=True)).to_be_disabled()
        expect(pc).to_have_attribute('aria-pressed', 'true')

    def slot():
        return next(e for e in command('state')['save']['bag'] if e['pocket'] == 'pc' and e['slot'] == 0)

    before_bag = [e for e in command('state')['save']['bag'] if e['pocket'] != 'pc']
    editor.get_by_label('Items', exact=True).select_option('13')
    editor.get_by_label('Quantity', exact=True).fill('999')
    apply()
    assert slot()['item'] == 13 and slot()['quantity'] == 999
    expect(editor.get_by_label('Quantity', exact=True)).to_have_value('999')
    editor.get_by_label('Quantity', exact=True).fill('42')
    apply()
    page.get_by_role('button', name='Undo', exact=True).click()
    expect(editor.get_by_label('Quantity', exact=True)).to_have_value('999')
    page.get_by_role('button', name='Redo', exact=True).click()
    expect(editor.get_by_label('Quantity', exact=True)).to_have_value('42')
    editor.get_by_role('button', name='Clear slot', exact=True).click()
    apply()
    assert slot()['item'] == 0 and slot()['quantity'] == 0
    page.get_by_role('button', name='Undo', exact=True).click()
    expect(editor.get_by_label('Quantity', exact=True)).to_have_value('42')
    assert [e for e in command('state')['save']['bag'] if e['pocket'] != 'pc'] == before_bag
    page.get_by_role('button', name='简体中文', exact=True).click()
    expect(editor.get_by_role('heading', name='电脑道具 · 1', exact=True)).to_be_visible()
    page.screenshot(path=str(output / 'pc-items-zh.png'))
    page.set_viewport_size({'width': 780, 'height': 940})
    editor.scroll_into_view_if_needed()
    assert editor.bounding_box()['y'] < page.locator('.bag-layout table').bounding_box()['y']
    page.screenshot(path=str(output / 'pc-items-small-zh.png'))
    page.set_viewport_size({'width': 1440, 'height': 940})
    page.get_by_role('button', name='English', exact=True).click()
    page.get_by_role('button', name='Pokémon', exact=True).click()
    print('PC items UI passed: add, quantity, clear, undo/redo, pocket retention, bilingual, small window')
