#!/usr/bin/env python3
"""Fixed and held-item sources -> companion -> map -> back in the bilingual UI.

Synthetic API responses only. No actual ROM/SAV opened or edited. Run Vite first.
"""

import copy, json, os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG


def main():
    expect.set_options(timeout=30000)
    cat = copy.deepcopy(CATALOG)
    cat["profile"]["capabilities"] = dict(world=True, save_edit=True)
    cat["items"] = [
        dict(id=0, name="", tm_move=None),
        dict(id=1, name="Test item", tm_move=None),
    ]
    cat["moves"] = [dict(id=0, name="", pp=0)]
    map = dict(
        id="0-0",
        name="Test fixed encounter room",
        region=1,
        width=4,
        height=4,
        map_type=1,
    )
    members = [
        dict(member=0, species=1, level=20, held_item=0),
        dict(member=1, species=2, level=21, held_item=1),
    ]
    mons = [
        dict(
            species=m["species"],
            level=m["level"],
            held_item=m["held_item"],
            method="static",
            offset=600,
            member=m["member"],
            conditions=[],
            trade=None,
            battle_members=members,
        )
        for m in members
    ]
    marker = dict(
        id="gift-1",
        kind="gift",
        x=1,
        y=1,
        elevation=0,
        local_id=1,
        graphics_id=None,
        movement_type=0,
        flag=0,
        receipt_flag=None,
        offset=100,
        script=600,
        rewards=[],
        pokemon=mons,
        teaching=[],
        daycare=[],
        stopped_at=[],
    )
    events = dict(
        map_id="0-0",
        markers=[marker],
        unplaced_rewards=[],
        unplaced_pokemon=[],
        unplaced_teaching=[],
        unplaced_daycare=[],
        stopped_at=[],
    )
    single_mon = dict(mons[1], battle_members=[], offset=700)
    events["markers"].append(
        dict(
            marker,
            id="gift-2",
            x=2,
            local_id=2,
            offset=700,
            script=700,
            pokemon=[single_mon],
        )
    )
    world = dict(
        maps=[map],
        map_events=[events],
        encounters=[],
        trainers=[],
        trainer_locations=dict(locations=[]),
        map_groups=[],
    )
    requests, errors = [], []

    def respond(route):
        req = route.request.post_data_json
        requests.append(req)
        command, payload = req["command"], req["payload"]
        if command == "state":
            data = dict(catalog=cat, save=None)
        elif command == "world":
            data = world
        elif command == "species":
            data = dict(
                species=cat["species"][payload["id"] - 1],
                evolutions=[],
                learnset=[],
                encounters=[],
                origins={},
            )
        elif command == "acquisition":
            mon = single_mon if payload["kind"] == "item" else mons[payload["id"] - 1]
            source = dict(
                kind="static_held" if payload["kind"] == "item" else "static",
                map_id="0-0",
                region=1,
                x=2 if payload["kind"] == "item" else 1,
                y=1,
                underfoot=None,
                related=(
                    [dict(kind="species", id=2)] if payload["kind"] == "item" else []
                ),
                quantity=1 if payload["kind"] == "item" else None,
                min_level=mon["level"],
                max_level=mon["level"],
                encounter_percent=None,
                held_percent=None,
                periods=[],
                conditions=[],
                requirements=[],
                evolution=None,
                status="unknown",
                receipt_flag=None,
                repeatable=None,
                offset=600,
                partial=True,
                script_source=mon,
                trade_context=None,
            )
            data = dict(target=payload, sources=[source], clock=None, partial=True)
        elif command == "map_navigation":
            data = dict(
                map_id="0-0",
                outgoing=[],
                incoming=[],
                approaches=[],
                truncated=False,
                diagnostics=[],
            )
        elif command in ("map_image", "sprite", "object_sprite"):
            data = dict(
                url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="lightblue"/></svg>'
            )
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
        page.get_by_role("button", name="ROM reference", exact=True).click()
        panel = page.locator(".acquisition-panel")
        expect(panel).to_contain_text("Two opponents in this encounter")
        expect(panel).to_contain_text("No held item")
        expect(panel).to_contain_text(
            "Capture permissions and battle-start conditions remain unverified"
        )
        panel.get_by_role("button", name="Test species 2 ↗", exact=True).click()
        expect(page.locator(".reference-detail h2")).to_contain_text("Test species 2")
        expect(panel).to_contain_text("Two opponents in this encounter")
        panel.get_by_role(
            "button", name="Test fixed encounter room", exact=False
        ).click()
        expect(page.locator(".map-focus")).to_be_visible()
        page.locator(".map-focus").click()
        details = page.locator(".map-marker-details")
        expect(details).to_contain_text("Two opponents in this encounter")
        expect(
            details.get_by_text("Two opponents in this encounter", exact=True)
        ).to_have_count(1)
        page.get_by_role("button", name="简体中文", exact=True).click()
        expect(details).to_contain_text("本次相遇的两只对手")
        expect(details).to_contain_text("能否捕捉及开始战斗的条件仍需确认")
        if os.environ.get("GEN3_UI_ARTIFACTS"):
            folder = Path(os.environ["GEN3_UI_ARTIFACTS"])
            folder.mkdir(parents=True, exist_ok=True)
            details.screenshot(path=str(folder / "paired-fixed-encounter.png"))
        page.get_by_role("button", name="返回上一条资料", exact=False).click()
        expect(page.locator(".reference-detail h2")).to_contain_text("Test species 2")
        expect(panel).to_contain_text("本次相遇的两只对手")
        panel.get_by_role("button", name="Test item ↗", exact=True).click()
        expect(page.locator(".reference-detail h2")).to_contain_text("Test item")
        expect(panel).to_contain_text("定点宝可梦指定携带的道具")
        expect(panel).to_contain_text("生成时指定携带: Test item")
        expect(panel).to_contain_text("能否通过捕捉或夺取道具的招式获得仍待确认")
        page.get_by_role("button", name="English", exact=True).click()
        expect(panel).to_contain_text("Item assigned to a fixed encounter")
        expect(panel).to_contain_text("Explicit setup item: Test item")
        expect(panel).not_to_contain_text("Encounter slot probability")
        expect(panel).not_to_contain_text("100%")
        panel.get_by_role(
            "button", name="Test fixed encounter room", exact=False
        ).click()
        page.locator(".map-focus").click()
        expect(details).to_contain_text("Explicit setup item: Test item")
        expect(details).to_contain_text("capture or a taking move remain unverified")
        if os.environ.get("GEN3_UI_ARTIFACTS"):
            details.screenshot(path=str(folder / "fixed-held-item-map.png"))
        details.get_by_role("button", name="Test item ↗", exact=True).click()
        expect(panel).to_contain_text("Explicit setup item: Test item")
        panel.get_by_role("button", name="Test species 2 ↗", exact=True).click()
        expect(page.locator(".reference-detail h2")).to_contain_text("Test species 2")
        assert not any(
            r["command"] in ("action", "export_save", "save_bytes") for r in requests
        )
        assert not errors, errors
        browser.close()
    print(
        "Fixed/pair item -> species -> map -> item -> back, EN/ZH and read-only queries passed"
    )


if __name__ == "__main__":
    main()
