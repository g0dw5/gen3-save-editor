#!/usr/bin/env python3
"""Native Dex read notices in the real editor, using synthetic API responses.

No user files, save actions, ROM writes or exports. Run Vite first.
"""

import copy
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG
from test_editor_navigation import pokemon


def main():
    expect.set_options(timeout=30000)
    catalog = copy.deepcopy(CATALOG)
    catalog["profile"]["capabilities"] = dict(dex=True, save_edit=True)
    for i, species in enumerate(catalog["species"], 1):
        species["dex_number"] = i
    catalog["moves"] = [dict(id=0, name="", power=0, category=0, move_type=0, pp=0)]
    catalog["items"] = [dict(id=0, name="", tm_move=None)]
    save = dict(
        trainer=dict(name="TEST"),
        pokemon=[pokemon(1, dict(kind="party", slot=0))],
        boxes=[
            dict(index=i, name=f"Box {i+1}", count=0, wallpaper=0) for i in range(14)
        ],
        bag=[],
        dex=[
            dict(number=1, seen=False, owned=False),
            dict(number=2, seen=True, owned=True),
        ],
        dex_status=dict(
            count=416,
            read_only=False,
            uninitialized_ranges=[],
            inconsistent_numbers=list(range(1, 26)),
        ),
        active_slot=0,
        counter=1,
        backup_valid=True,
        dirty=False,
        can_undo=False,
        can_redo=False,
        changes=[],
    )
    requests, errors = [], []

    def respond(route):
        req = route.request.post_data_json
        requests.append(req)
        if req["command"] == "state":
            data = dict(catalog=catalog, save=save)
        elif req["command"] == "sprite":
            data = dict(url="")
        elif req["command"] == "species":
            data = dict(
                species=catalog["species"][req["payload"]["id"] - 1],
                evolutions=[],
                learnset=[],
                encounters=[],
            )
        elif req["command"] == "acquisition":
            data = dict(target=req["payload"], sources=[], partial=True, clock=None)
        else:
            raise AssertionError(req)
        route.fulfill(
            content_type="application/json", body=json.dumps(dict(ok=True, data=data))
        )

    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        page = browser.new_page(viewport=dict(width=1000, height=740))
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.route("**/api", respond)
        page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
        page.get_by_role("button", name="Pokédex", exact=True).click()
        pane = page.locator(".data-page")
        expect(pane).to_contain_text("Dex records failing the game's checks: 25")
        expect(pane).to_contain_text("20…")
        expect(pane).not_to_contain_text("21, 22")
        expect(pane.get_by_role("checkbox", name="Seen").nth(0)).not_to_be_checked()
        expect(pane.get_by_role("checkbox", name="Owned").nth(0)).not_to_be_checked()
        expect(pane.get_by_role("checkbox", name="Owned").nth(1)).to_be_checked()
        page.get_by_role("button", name="简体中文", exact=True).click()
        expect(pane).to_contain_text("未通过游戏校验的图鉴记录： 25")
        expect(pane).to_contain_text("查询不会自动修复存档")
        if os.environ.get("GEN3_UI_ARTIFACTS"):
            out = Path(os.environ["GEN3_UI_ARTIFACTS"])
            out.mkdir(parents=True, exist_ok=True)
            pane.screenshot(path=str(out / "dex-native-checks.png"))
        save["dex_status"]["inconsistent_numbers"] = []
        page.reload()
        page.get_by_role("button", name="Pokédex", exact=True).click()
        expect(page.locator(".data-page")).not_to_contain_text("Dex records failing")
        assert not errors, errors
        assert all(
            r["command"] in ["state", "sprite", "species", "acquisition"]
            for r in requests
        )
        print(
            "Bilingual native Dex notices, bounded flag-number list, projected toggles and fresh-save notice invalidation passed; no save action sent"
        )
        browser.close()


if __name__ == "__main__":
    main()
