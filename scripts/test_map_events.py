"""Map layer/coordinate regression with synthetic events; no ROM or save needed.

Run Vite, then invoke with Python + Playwright/Chrome installed.
"""
import json
import os
import re
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, WORLD


def reward(item):
    return {"item": item, "quantity": 1, "offset": 256, "via": "gift", "conditions": []}


def marker(identifier, kind, x, y, rewards):
    return {"id": identifier, "kind": kind, "x": x, "y": y, "elevation": 3,
            "local_id": 1, "graphics_id": 999 if identifier == "unknown" else 1, "movement_type": 0,
            "flag": 100, "offset": 256, "script": 512,
            "rewards": rewards, "stopped_at": []}


catalog = {**CATALOG, "items": [{"id": 1, "name": "Potion"}, {"id": 2, "name": "Moon Stone"}]}
world = {**WORLD, "maps": [{"id": "26-13", "name": "Test map", "width": 8, "height": 6}],
         "map_events": [{"map_id": "26-13", "markers": [
             marker("ball", "pickup", 1, 2, [reward(1)]),
             marker("hidden", "hidden", 3, 4, [reward(2)]),
             marker("gift", "gift", 3, 4, [reward(1)]),
             marker("npc", "npc", 5, 1, []),
             marker("unknown", "npc", 6, 1, []),
         ], "unplaced_rewards": [reward(2)], "stopped_at": []}]}


def respond(route):
    request = route.request.post_data_json
    command = request["command"]
    if command == "state":
        data = {"catalog": catalog, "save": None}
    elif command == "world":
        data = world
    elif command == "map_image":
        data = {"url": "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='128' height='96'><rect width='128' height='96' fill='%23273632'/></svg>"}
    elif command == "object_sprite":
        if request["payload"]["id"] == 999:
            route.fulfill(content_type="application/json", body=json.dumps({"ok": False, "error": {"code": "object_graphics_dynamic", "detail": "999"}}))
            return
        data = {"url": "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='16' height='32'><rect width='16' height='32' fill='%23e9a76e'/></svg>"}
    elif command == "species":
        data = {"species": CATALOG["species"][0], "evolutions": [], "learnset": [], "encounters": []}
    elif command == "sprite":
        data = {"url": ""}
    else:
        raise AssertionError(command)
    route.fulfill(content_type="application/json", body=json.dumps({"ok": True, "data": data}))


if __name__ == "__main__":
    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        page = browser.new_page(viewport={"width": 1280, "height": 960})
        page.add_init_script("localStorage.setItem('gen3.locale', 'en')")
        errors = []
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.route("**/api", respond)
        page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
        page.get_by_role("button", name="ROM reference", exact=True).click()
        page.get_by_role("dialog").get_by_role("button", name="Maps", exact=True).click()
        explorer = page.locator(".map-explorer")
        expect(explorer).to_be_visible()
        expect(explorer.locator(".map-marker")).to_have_count(4)
        expect(explorer.locator(".map-actor > img")).to_have_count(2)
        npc = explorer.locator(".map-actor.layer-npc")
        assert abs(float(npc.evaluate("e => parseFloat(e.style.top)")) - 100 * 2 / 6) < 1e-3
        assert npc.evaluate("e => e.style.width") == "12.5%"  # One map tile wide.
        assert abs(float(npc.evaluate("e => parseFloat(e.style.height)")) - 100 * 2 / 6) < 1e-3
        # Dynamic graphics fail gracefully without replacing another actor's image.
        expect(explorer.locator(".map-marker.layer-npc:not(.map-actor)")).to_have_attribute("title", re.compile("Image not resolved"))
        explorer.get_by_role("checkbox", name="NPCs / objects").uncheck()
        expect(explorer.locator(".map-marker")).to_have_count(2)  # Same-tile rewards share one pin.
        ball = explorer.locator(".map-marker.layer-pickup")
        assert ball.evaluate("e => e.style.left") == "18.75%"
        assert abs(float(ball.evaluate("e => parseFloat(e.style.top)")) - 100 * 2.5 / 6) < 1e-3
        explorer.get_by_role("checkbox", name="Hidden items").uncheck()
        expect(explorer.locator(".map-marker.layer-hidden")).to_have_count(0)
        explorer.locator(".map-marker.layer-gift").click()
        expect(explorer.locator(".map-marker-details")).to_contain_text("Potion × 1")
        explorer.get_by_role("checkbox", name="Dialogue rewards").uncheck()
        expect(explorer.locator(".map-marker")).to_have_count(1)
        explorer.get_by_role("checkbox", name="NPCs / objects").check()
        expect(explorer.locator(".map-marker")).to_have_count(3)
        explorer.get_by_role("checkbox", name="Hidden items").check()
        search = explorer.get_by_role("textbox")
        search.fill("Moon")
        expect(explorer.locator(".map-marker")).to_have_count(1)
        explorer.locator(".map-marker").click()
        expect(explorer.locator(".map-marker-details")).to_contain_text("Moon Stone × 1")
        explorer.get_by_label("Grid", exact=True).check()
        expect(explorer.locator(".map-grid")).to_be_visible()
        search.fill("")
        npc = explorer.locator(".map-actor.layer-npc")
        before = npc.bounding_box()
        explorer.get_by_label("Zoom", exact=True).select_option("2")
        assert explorer.locator(".map-canvas").evaluate("e => e.style.width") == "200%"
        after = npc.bounding_box()
        assert abs(after["width"] - 2 * before["width"]) < 1
        assert abs(after["height"] - 2 * before["height"]) < 1
        assert not errors, errors
        browser.close()
        print("Map layers, NPC sprites, fallback, tile alignment, search and zoom passed.")
