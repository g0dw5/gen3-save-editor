"""ROM editor regression: one export includes multiple fields of one species.

Run against Vite using GEN3_UI_URL. All API data is synthetic; no ROM is opened.
"""

import copy
import os

from playwright.sync_api import expect, sync_playwright

from test_reference_navigation import CATALOG, species


def main():
    catalog = copy.deepcopy(CATALOG)
    gyarados = species(130)
    gyarados.update(name="Gyarados", stats=[95, 125, 79, 81, 60, 100], abilities=[1, 0])
    magikarp = species(129)
    magikarp.update(name="Magikarp", stats=[20, 10, 55, 80, 15, 20], abilities=[1, 0])
    catalog["species"] = [gyarados, magikarp]
    catalog["abilities"] = [
        {"id": 0, "name": "None", "description": ""},
        {"id": 1, "name": "Intimidate", "description": ""},
        {"id": 2, "name": "Moxie", "description": ""},
    ]
    edits_sent = []
    errors = []

    def respond(route):
        request = route.request.post_data_json
        command, payload = request["command"], request["payload"]
        if command == "state":
            data = {"catalog": catalog, "save": None}
        elif command == "species":
            selected = next(s for s in catalog["species"] if s["id"] == payload["id"])
            data = {"species": selected, "evolutions": [], "learnset": [], "encounters": []}
        elif command == "sprite":
            data = {"url": ""}
        elif command == "patch_rom":
            edits_sent.append(payload["edits"])
            data = {"manifest": {"edits": payload["edits"]}}
        else:
            raise AssertionError(request)
        route.fulfill(json={"ok": True, "data": data})

    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(channel="chrome", headless=True)
        page = browser.new_page(viewport={"width": 1280, "height": 900}, accept_downloads=True)
        page.add_init_script("localStorage.setItem('gen3.locale', 'en')")
        page.on("pageerror", lambda error: errors.append(str(error)))
        page.route("**/api", respond)
        page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
        page.get_by_role("button", name="ROM reference", exact=True).click()
        dialog = page.get_by_role("dialog", name="ROM reference")
        dialog.get_by_role("button", name="Edit ROM table").click()
        form = dialog.locator("form.rom-patch")
        export = form.get_by_role("button", name="Export derived ROM")
        expect(export).to_be_disabled()

        form.get_by_label("Field").select_option("speed")
        form.get_by_label("Value").fill("101")
        expect(form.locator(".rom-patch-change")).to_have_count(1)
        form.get_by_label("Field").select_option("ability1")
        expect(form.get_by_role("combobox", name="Value")).to_have_value("Intimidate")
        form.get_by_role("combobox", name="Value").fill("Moxie")
        page.locator(".select-popup").get_by_role("option", name="Moxie").click()
        expect(form.locator(".rom-patch-change")).to_have_count(2)
        form.get_by_label("Field").select_option("speed")
        expect(form.get_by_label("Value")).to_have_value("101")

        with page.expect_download():
            export.click()
        assert edits_sent == [[
            {"table": "species", "id": 130, "field": "speed", "value": 101},
            {"table": "species", "id": 130, "field": "ability1", "value": 2},
        ]], edits_sent

        # Reverting and removing fields must not accidentally export old drafts.
        form.get_by_label("Value").fill("81")
        expect(form.locator(".rom-patch-change")).to_have_count(1)
        form.get_by_role("button", name="Remove Ability 1").click()
        expect(export).to_be_disabled()
        dialog.locator(".reference-rows button").filter(has_text="Magikarp").click()
        expect(form).to_have_count(0)
        dialog.get_by_role("button", name="Edit ROM table").click()
        expect(dialog.locator("form.rom-patch").get_by_role("button", name="Export derived ROM")).to_be_disabled()
        assert not errors, errors
        browser.close()
    print("ROM patch UI: multiple fields exported together; revert, removal and species reset passed")


if __name__ == "__main__":
    main()
