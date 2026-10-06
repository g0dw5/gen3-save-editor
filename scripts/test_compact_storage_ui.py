"""Native compact-box field boundaries and ROM switching; synthetic UI data only."""
import copy
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_editor_navigation import pokemon
from test_reference_navigation import CATALOG, WORLD, species


def main():
    errors = []
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(channel="chrome", headless=True)
        for locale in ("en", "zh"):
            for compact in (True, False):
                catalog = copy.deepcopy(CATALOG)
                catalog["profile"]["save"] = {"pockets": [], "compressed_boxes": {} if compact else None}
                # An engine format is the switch, not the displayed ROM label.
                catalog["profile"]["label"] = "Test storage engine"
                catalog["moves"] = [dict(id=0, name="", pp=0), dict(id=1, name="Test move", pp=10, category=0, move_type=0)]
                party = pokemon(1, dict(kind="party", slot=0))
                box = pokemon(2, dict(kind="box", box_index=0, slot=0))
                for row in (party, box):
                    row["pokemon"].update(moves=[1, 0, 0, 0], pps=[10, 0, 0, 0])
                save = dict(trainer=dict(name="TEST"), pokemon=[party, box],
                            boxes=[dict(index=i, name=f"Box {i+1}", wallpaper=0, count=int(i == 0)) for i in range(25 if compact else 14)],
                            bag=[], dex=[], active_slot=0, counter=1, backup_valid=True,
                            dirty=False, can_undo=False, can_redo=False, changes=[])
                def respond(route):
                    request = route.request.post_data_json
                    command = request["command"]
                    if command == "state":
                        data = dict(catalog=catalog, save=save)
                    elif command == "world":
                        data = WORLD
                    elif command == "species":
                        data = dict(species=species(request["payload"]["id"]), evolutions=[], learnset=[], encounters=[])
                    elif command in ("sprite", "map_image"):
                        data = dict(url="")
                    elif command == "acquisition":
                        data = dict(target=request["payload"], sources=[], partial=True, clock=None)
                    else:
                        raise AssertionError(request)
                    route.fulfill(json=dict(ok=True, data=data))
                page = browser.new_page(viewport=dict(width=1100, height=840))
                page.add_init_script(f"localStorage.setItem('gen3.locale','{locale}')")
                page.on("pageerror", lambda error: errors.append(str(error)))
                page.route("**/api", respond)
                page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
                tabs = page.locator(".editor-tabs")
                tabs.get_by_role("button", name="招式" if locale == "zh" else "Moves", exact=True).click()
                pp = page.get_by_role("spinbutton", name="当前 PP" if locale == "zh" else "Current PP", exact=True).first
                expect(pp).to_be_enabled()
                page.locator('[data-location="0:0"]').click()
                if compact:
                    expect(pp).to_be_disabled()
                else:
                    expect(pp).to_be_enabled()
                tabs.get_by_role("button", name="来源" if locale == "zh" else "Origin", exact=True).click()
                flag = page.get_by_role("checkbox", name="命运的相遇标志" if locale == "zh" else "Fateful encounter flag", exact=True)
                expect(flag).to_have_count(0 if compact else 1)
                page.locator('[data-location="p:0"]').click()
                expect(flag).to_have_count(1)
                page.close()
        browser.close()
    assert not errors, errors
    print("Compact storage UI: bilingual box PP/read-only metadata, party controls and ordinary-format isolation passed")


if __name__ == "__main__":
    main()
