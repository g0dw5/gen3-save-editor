"""Optional local UI regression using the opt-in bridge and user-supplied ROM.

Start the bridge and Vite with the same GEN3_DEV_TOKEN, then set GEN3_ROM_BW and
GEN3_TEST_SAVE. The save must be a disposable generated fixture with occupied
party slot 0 and box 0 slot 0. Requires Playwright and installed Chrome.
"""
import json
import os
import urllib.request
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

TOKEN = os.environ['GEN3_DEV_TOKEN']
OUTPUT = Path(os.environ.get('GEN3_UI_OUTPUT', '/tmp/gen3-ui-tests'))
OUTPUT.mkdir(parents=True, exist_ok=True)


def command(name, payload=None):
    request = urllib.request.Request('http://127.0.0.1:8766/api',
        data=json.dumps({'command': name, 'payload': payload or {}}).encode(),
        headers={'Content-Type': 'application/json', 'X-Gen3-Token': TOKEN})
    result = json.load(urllib.request.urlopen(request))
    assert result['ok'], result
    return result['data']


command('open_rom', {'path': os.environ['GEN3_ROM_BW']})
command('open_save', {'path': os.environ['GEN3_TEST_SAVE']})
with sync_playwright() as p:
    browser = p.chromium.launch(channel='chrome', headless=True)
    page = browser.new_page(viewport={'width': 1440, 'height': 940})
    expect.set_options(timeout=30000)
    errors = []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.on('dialog', lambda dialog: dialog.accept())
    page.goto('http://127.0.0.1:5173')
    expect(page.get_by_role('heading', name='全部盒子')).to_be_visible()
    expect(page.locator('.storage-slot')).to_have_count(426)
    expect(page.locator('.box-panel')).to_have_count(14)
    page.screenshot(path=str(OUTPUT / 'workspace-zh.png'))
    page.get_by_role('button', name='English', exact=True).click()
    expect(page.get_by_role('heading', name='All boxes')).to_be_visible()
    page.get_by_label('Level', exact=True).fill('77')
    page.get_by_role('button', name='Apply changes', exact=True).click()
    expect(page.get_by_label('Level', exact=True)).to_have_value('77')
    assert command('state')['save']['pokemon'][0]['pokemon']['level'] == 77
    page.get_by_role('button', name='Undo', exact=True).click()
    expect(page.get_by_label('Level', exact=True)).to_have_value('50')
    page.get_by_role('button', name='Redo', exact=True).click()
    expect(page.get_by_label('Level', exact=True)).to_have_value('77')
    page.locator('[data-location="0:0"]').drag_to(page.locator('[data-location="p:1"]'))
    expect(page.locator('[data-location="p:1"]')).to_have_class(__import__('re').compile('occupied'))
    assert len([v for v in command('state')['save']['pokemon'] if v['location']['kind']=='party']) == 2
    page.get_by_role('button', name='ROM reference', exact=True).click()
    dialog = page.get_by_role('dialog')
    expect(dialog).to_be_visible()
    expect(dialog.locator('.dex-hero')).to_be_visible()
    # Move the nonmodal window to expose empty cells and prove actual template DnD.
    handle = dialog.locator('.floating-header')
    if handle.count():
        box = handle.bounding_box()
        page.mouse.move(box['x']+80, box['y']+15); page.mouse.down()
        page.mouse.move(780, 100, steps=8); page.mouse.up()
    template = dialog.locator('.dex-hero')
    target = page.locator('[data-location="2:0"]')
    template.drag_to(target)
    expect(page.locator('.draft-editor')).to_be_visible()
    # Close reference to access the inspector without dismissing the draft.
    dialog.get_by_role('button', name='Close', exact=True).click()
    page.get_by_role('button', name='Create Pokémon', exact=True).click()
    expect(page.locator('[data-location="2:0"]')).to_have_class(__import__('re').compile('occupied'))
    page.get_by_role('button', name='Player', exact=True).click()
    page.get_by_label('Money', exact=True).fill('54321')
    page.get_by_role('button', name='Apply changes', exact=True).click()
    expect(page.get_by_label('Money', exact=True)).to_have_value('54321')
    page.get_by_role('button', name=__import__('re').compile('^Changes')).click()
    expect(page.locator('.change-card')).not_to_have_count(0)
    page.screenshot(path=str(OUTPUT / 'changes-en.png'))
    with page.expect_download() as event:
        page.get_by_role('button', name='Export save', exact=True).click()
    event.value.save_as(str(OUTPUT / 'exported.sav'))
    assert (OUTPUT / 'exported.sav').stat().st_size == 131072
    page.set_viewport_size({'width': 1000, 'height': 720})
    page.get_by_role('button', name='Pokémon', exact=True).click()
    page.screenshot(path=str(OUTPUT / 'workspace-small.png'))
    assert not errors, errors
    print('UI passed: 426 slots, bilingual, edit, undo/redo, move, template draft, player, export, resize')
    browser.close()
