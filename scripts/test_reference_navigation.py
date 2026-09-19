"""Browser regression for reference-tab IDs, using synthetic API fixtures.

Start Vite, then run this script with Playwright and Chrome installed. No ROM,
save, or development bridge is required. GEN3_UI_URL can override the Vite URL.
"""

import json
import os

from playwright.sync_api import sync_playwright, expect


def species(identifier):
    return {
        "id": identifier, "name": f"Test species {identifier}",
        "types": [0, 0], "stats": [50] * 6, "abilities": [0, 0],
    }


CATALOG = {
    "profile": {"id": "synthetic", "label": "Synthetic", "md5": "test", "size": 0},
    "species": [species(1), species(2)], "moves": [], "items": [],
    "abilities": [], "met_locations": [],
}
WORLD = {
    "maps": [{"id": "26-13", "name": "Test map", "width": 2, "height": 2}],
    "trainers": [], "encounters": [], "map_groups": [],
    "trainer_locations": {"locations": []},
}


if __name__ == "__main__":
    requests = []
    errors = []
    delayed = []
    hold_species = [False]

    def respond(route):
        request = route.request.post_data_json
        requests.append(request)
        command, payload = request["command"], request["payload"]
        result = {"ok": True}
        if command == "state":
            result["data"] = {"catalog": CATALOG, "save": None}
        elif command == "world":
            result["data"] = WORLD
        elif command == "species":
            if hold_species[0]:
                hold_species[0] = False
                delayed.append(route)
                return
            identifier = payload.get("id")
            if identifier not in (1, 2):
                result = {"ok": False, "error": {
                    "code": "json", "detail": "invalid type: null, expected u16",
                }}
            else:
                result["data"] = {
                    "species": species(identifier), "evolutions": [],
                    "learnset": [], "encounters": [],
                }
        elif command in ("sprite", "map_image"):
            result["data"] = {"url": ""}
        else:
            raise AssertionError(f"unexpected command: {command}")
        route.fulfill(content_type="application/json", body=json.dumps(result))

    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        page = browser.new_page(viewport={"width": 1280, "height": 900})
        page.add_init_script("localStorage.setItem('gen3.locale', 'en')")
        page.on("pageerror", lambda error: errors.append(str(error)))
        page.route("**/api", respond)
        page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
        page.get_by_role("button", name="ROM reference", exact=True).click()
        dialog = page.get_by_role("dialog")
        expect(dialog.locator(".dex-hero")).to_be_visible()
        tabs = dialog.locator(".reference-tabs")
        for _ in range(3):
            tabs.get_by_role("button", name="Maps", exact=True).click()
            expect(dialog.locator(".reference-detail h2")).to_contain_text("Test map")
            tabs.get_by_role("button", name="Pokémon", exact=True).click()
            expect(dialog.locator(".dex-hero")).to_be_visible()
            expect(dialog.locator(".reference-detail h2")).to_contain_text("Test species 1")
        dialog.locator(".reference-rows button").filter(has_text="Test species 2").click()
        expect(dialog.locator(".reference-detail h2")).to_contain_text("Test species 2")
        expect(dialog.locator(".dex-hero")).to_be_visible()
        # A request from a tab that has been left must not surface a late error.
        tabs.get_by_role("button", name="Maps", exact=True).click()
        expect(dialog.locator(".reference-detail h2")).to_contain_text("Test map")
        hold_species[0] = True
        with page.expect_request(lambda r: r.url.endswith("/api") and r.post_data_json["command"] == "species"):
            tabs.get_by_role("button", name="Pokémon", exact=True).click()
        expect(dialog.locator(".reference-detail h2")).to_contain_text("Test species 1")
        expect(dialog.locator(".dex-hero")).to_have_count(0)
        tabs.get_by_role("button", name="Maps", exact=True).click()
        expect(dialog.locator(".reference-detail h2")).to_contain_text("Test map")
        assert len(delayed) == 1
        delayed.pop().fulfill(content_type="application/json", body=json.dumps({
            "ok": False, "error": {"code": "json", "detail": "obsolete request"},
        }))
        tabs.get_by_role("button", name="Pokémon", exact=True).click()
        expect(dialog.locator(".dex-hero")).to_be_visible()
        invalid = [r for r in requests if r["command"] == "species" and r["payload"]["id"] not in (1, 2)]
        assert not invalid, f"invalid species requests during tab switch: {invalid}"
        assert not errors, errors
        expect(page.locator(".error-banner")).to_have_count(0)
        browser.close()
        print("Passed: map → species navigation sends valid IDs, renders matching details, and ignores obsolete request errors.")
