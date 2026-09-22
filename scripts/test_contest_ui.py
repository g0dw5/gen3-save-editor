"""Contest fields, scoped bounds, stale responses and cross-ROM display.

Uses generated UI fixtures only; no user saves or emulator state.
"""

import copy
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_editor_navigation import pokemon
from test_reference_navigation import CATALOG, WORLD, species


def main():
    catalog = copy.deepcopy(CATALOG)
    catalog["profile"]["contest"] = {"flavor_preferences": 1, "npc_blender": {}}
    catalog["profile"]["md5"] = "contest-fixture"
    save = dict(
        trainer={"name": "TEST"},
        pokemon=[pokemon(1, {"kind": "party", "slot": 0})],
        boxes=[],
        bag=[],
        dex=[],
        dirty=False,
        can_undo=False,
        can_redo=False,
        changes=[],
        backup_valid=True,
        active_slot=0,
        counter=1,
    )
    checks, actions, errors = [], [], []

    def respond(route):
        req = route.request.post_data_json
        command, payload = req["command"], req["payload"]
        if command == "state":
            data = {"catalog": catalog, "save": save}
        elif command == "world":
            data = WORLD
        elif command == "sprite":
            data = {"url": ""}
        elif command == "species":
            data = {
                "species": species(payload["id"]),
                "evolutions": [],
                "learnset": [],
                "encounters": [],
            }
        elif command == "contest_check":
            checks.append(payload)
            assert payload["expected_rom_md5"] == "contest-fixture"
            data = {
                "status": (
                    "outside_npc_bound"
                    if payload["condition"][1] and payload["condition"][5] == 0
                    else "not_disproved"
                ),
                "minimum_sheen_lower_bound": None,
            }
        elif command == "action":
            actions.append(payload)
            patch = payload["action"]["patch"]
            save["pokemon"][0]["pokemon"].update(
                {k: v for k, v in patch.items() if k != "contest_scope"}
            )
            data = {"save": save}
        else:
            raise AssertionError(req)
        route.fulfill(
            content_type="application/json", body=json.dumps({"ok": True, "data": data})
        )

    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        page = browser.new_page(viewport={"width": 1440, "height": 960})
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.route("**/api", respond)
        for locale in ("zh", "en"):
            page.add_init_script(f"localStorage.setItem('gen3.locale','{locale}')")
            page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
            page.locator(".editor-tabs").get_by_role(
                "button", name="能力" if locale == "zh" else "Stats", exact=True
            ).click()
            section = page.locator(".contest-condition")
            beauty = section.get_by_role(
                "spinbutton", name="美丽" if locale == "zh" else "Beauty", exact=True
            )
            sheen = section.get_by_role(
                "spinbutton",
                name="饱腹度（光泽）" if locale == "zh" else "Fullness (Sheen)",
                exact=True,
            )
            expect(section.get_by_role("spinbutton")).to_have_count(6)
            expect(sheen).to_have_attribute("min", "0")
            expect(sheen).to_have_attribute("max", "255")
            beauty.fill("255")
            sheen.fill("0")
            expect(section.get_by_role("status")).to_contain_text(
                "超出" if locale == "zh" else "exceed"
            )
            sheen.fill("227")
            expect(section.get_by_role("status")).to_contain_text(
                "未被必要条件排除" if locale == "zh" else "Not ruled out"
            )
            page.locator(".editor-submit button[type=submit]").click()
            # Wait for the asynchronous apply/reset before beginning the next edit.
            expect(page.locator(".editor-submit button[type=submit]")).to_be_disabled()
            assert actions[-1]["action"]["patch"] == {
                "condition": [0, 255, 0, 0, 0, 227],
                "contest_scope": "npc",
            }
            sheen.fill("255")
            expect(section).to_contain_text("已吃满" if locale == "zh" else "Full:")
            sheen.fill("256")
            page.locator(".editor-submit button[type=submit]").click()
            assert len(actions) == (1 if locale == "zh" else 2), actions
            sheen.fill("227")
        # Rocket uses the same six fields, with no fallback to the NPC model.
        catalog["profile"]["md5"] = "rocket-fixture"
        catalog["profile"]["contest"]["npc_blender"] = None
        page.reload()
        page.locator(".editor-tabs").get_by_role(
            "button", name="Stats", exact=True
        ).click()
        expect(
            page.locator(".contest-condition").get_by_role("spinbutton")
        ).to_have_count(6)
        expect(page.locator(".contest-condition")).to_contain_text("Only field ranges")
        assert all(c["expected_rom_md5"] == "contest-fixture" for c in checks)
        browser.close()
    assert not errors, errors
    print(
        "Contest UI: both locales, numeric limits, scoped submit and cross-ROM isolation passed"
    )


if __name__ == "__main__":
    main()
