"""Evolution resources -> reverse uses -> species/map/back across five UI profiles.

Synthetic responses only. Tests ROM switches in one browser page, not reloads.
"""

import copy
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, species


def main():
    expect.set_options(timeout=30000)
    keys = ["BW", "DP", "Rocket", "Ultimate", "Mercury"]
    active = [0]
    calls, errors = [], []

    def data():
        key = keys[active[0]]
        catalog = copy.deepcopy(CATALOG)
        catalog["profile"].update(
            id=key,
            md5=f"fixture-{key}",
            label=key,
            capabilities=dict(world=True, save_edit=False),
        )
        catalog["species"] = [
            dict(species(i), name=f"{key} species {i}") for i in (1, 2, 3)
        ]
        catalog["items"] = [
            dict(
                id=i,
                name=f"{key} " + label,
                description="",
                price=0,
                pocket=1,
                tm_move=None,
            )
            for i, label in [(0, ""), (1, "stone"), (2, "held A"), (3, "held B")]
        ]
        catalog["moves"] = [
            dict(
                id=i,
                name=f"{key} move {i}",
                pp=10,
                power=40,
                accuracy=100,
                move_type=0,
                category=0,
            )
            for i in (0, 1)
        ]
        catalog["met_locations"] = [dict(id=1, name=f"{key} territory")]
        rules = [
            dict(
                source=1,
                target=2,
                method=7,
                condition="item",
                parameter=1,
                auxiliary=0,
                offset=100,
                requirements=[],
                related=[dict(kind="item", id=1)],
            )
        ]
        if key == "Mercury":
            rules[0].update(
                method=36,
                condition="item_hold_item",
                auxiliary=2,
                related=[dict(kind="item", id=1), dict(kind="item", id=2)],
            )
            rules[0]["requirements"] = [dict(kind="gender", value=254)]
            rules.append(
                dict(
                    rules[0],
                    offset=101,
                    auxiliary=3,
                    related=[dict(kind="item", id=1), dict(kind="item", id=3)],
                    requirements=[dict(kind="gender", value=0)],
                )
            )
        if key in ["Rocket", "Ultimate", "Mercury"]:
            rules.append(
                dict(
                    source=1,
                    target=3,
                    method={"Rocket": 23, "Ultimate": 16, "Mercury": 26}[key],
                    condition="move",
                    parameter=1,
                    auxiliary=0,
                    offset=110,
                    requirements=[],
                    related=[dict(kind="move", id=1)],
                )
            )
        if key == "Ultimate":
            rules.append(
                dict(
                    source=1,
                    target=2,
                    method=33,
                    condition="level",
                    parameter=20,
                    auxiliary=0,
                    offset=120,
                    requirements=[dict(kind="map", value=256)],
                    related=[],
                )
            )
        maps = [
            dict(
                id=id,
                name=f"{key} {name}",
                region=region,
                width=4,
                height=4,
                map_type=kind,
            )
            for id, name, region, kind in [
                ("0-0", "exterior", 1, 1),
                ("1-0", "cave floor", 1, 4),
                ("1-1", "other region", 2, 1),
            ]
        ]
        return key, catalog, rules, maps

    def source(kind, offset, **extra):
        return dict(
            kind=kind,
            offset=offset,
            map_id=None,
            region=None,
            x=None,
            y=None,
            underfoot=None,
            related=[],
            quantity=None,
            min_level=None,
            max_level=None,
            encounter_percent=None,
            held_percent=None,
            periods=[],
            conditions=[],
            requirements=[],
            evolution=None,
            status="unknown",
            receipt_flag=None,
            repeatable=None,
            partial=True,
            in_scenario=None,
            **extra,
        )

    def respond(route):
        req = route.request.post_data_json
        calls.append(req)
        command, payload = req["command"], req["payload"]
        if command == "open_rom":
            active[0] = int(payload["name"].split(".")[0])
        key, catalog, rules, maps = data()
        if command in ("state", "open_rom"):
            result = dict(catalog=catalog, save=None)
        elif command == "world":
            result = dict(
                maps=maps,
                encounters=[],
                trainers=[],
                map_groups=[],
                trainer_locations=dict(locations=[]),
                map_events=[
                    dict(map_id=m["id"], markers=[], unplaced_rewards=[], stopped_at=[])
                    for m in maps
                ],
            )
        elif command == "species":
            result = dict(
                species=catalog["species"][payload["id"] - 1],
                evolutions=[r for r in rules if r["source"] == payload["id"]],
                learnset=[],
                encounters=[],
                origins={},
                relations=dict(
                    species=[1, 2, 3],
                    evolutions=rules,
                    battle_forms=[],
                    form_families=[],
                    name_relations=[],
                ),
            )
        elif command == "acquisition":
            target = dict(kind=payload["kind"], id=payload["id"])
            uses = [
                dict(
                    source=r["source"],
                    evolution=r,
                    related=[t for t in r["related"] if t != target],
                )
                for r in rules
                if target in r["related"]
            ]
            sources = []
            if target == dict(kind="item", id=1):
                row = source("pickup", 200)
                row.update(map_id="0-0", region=1, x=1, y=1, quantity=1)
                sources.append(row)
            if target["kind"] == "species":
                for r in rules:
                    if r["target"] == target["id"]:
                        row = source("evolution", r["offset"])
                        row.update(
                            evolution=r,
                            requirements=r["requirements"],
                            related=[dict(kind="species", id=r["source"])]
                            + r["related"],
                        )
                        sources.append(row)
            result = dict(
                target=target,
                sources=sources,
                evolution_uses=uses,
                partial=True,
                clock=None,
            )
        elif command == "map_navigation":
            edge = dict(
                kind="warp",
                to="1-0",
                x=2,
                y=1,
                target_x=1,
                target_y=1,
                warp_index=0,
                target_warp=0,
                direction=None,
                displacement=None,
                offset=300,
                unresolved=None,
            )
            edge["from"] = "0-0"
            result = dict(
                map_id=payload["id"],
                outgoing=[edge] if payload["id"] == "0-0" else [],
                incoming=[edge] if payload["id"] == "1-0" else [],
                approaches=[[edge]] if payload["id"] == "1-0" else [],
                truncated=False,
                diagnostics=[],
            )
        elif command in ("sprite", "map_image", "object_sprite"):
            result = dict(
                url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="lightblue"/></svg>'
            )
        else:
            raise AssertionError(req)
        route.fulfill(
            content_type="application/json", body=json.dumps(dict(ok=True, data=result))
        )

    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        page = browser.new_page(viewport=dict(width=1100, height=780))
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.route("**/api", respond)
        page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
        for index, key in enumerate(keys):
            if index:
                with page.expect_file_chooser() as chooser:
                    page.get_by_role(
                        "button", name="Open ROM", exact=True
                    ).first.click()
                chooser.value.set_files(
                    dict(
                        name=f"{index}.gba",
                        mimeType="application/octet-stream",
                        buffer=b"synthetic-only",
                    )
                )
                expect(page.locator(".floating")).to_have_count(0)
            page.get_by_role("button", name="ROM reference", exact=True).first.click()
            pane = page.locator(".floating")
            tree = pane.locator(".evolution-tree")
            expect(tree.locator(".evolution-node.current")).to_contain_text(
                f"{key} species 1"
            )
            if index:
                expect(pane).not_to_contain_text(f"{keys[index-1]} stone")
            expect(
                tree.locator('.evolution-card[data-species="2"] .evolution-condition')
            ).to_have_count(2 if key in ["Ultimate", "Mercury"] else 1)
            if key == "Mercury":
                expect(tree).to_contain_text("Required gender: Female")
                expect(tree).to_contain_text("Required gender: Male")
            tree.get_by_role("button", name=f"{key} stone ↗", exact=True).first.click()
            expect(pane.locator(".reference-detail h2")).to_contain_text(f"{key} stone")
            uses = pane.locator(".evolution-uses")
            uses.locator("summary").first.click()
            expect(uses).to_contain_text("These are uses, not ways to obtain it")
            expect(uses.locator(".encounter-card")).to_have_count(
                2 if key == "Mercury" else 1
            )
            expect(pane.locator(".acquisition-panel > .encounter-card")).to_have_count(
                1
            )
            uses.get_by_role(
                "button", name=f"{key} species 2 ↗", exact=True
            ).first.click()
            expect(pane.locator(".reference-detail h2")).to_contain_text(
                f"{key} species 2"
            )
            page.get_by_role(
                "button", name="Back to previous reference", exact=False
            ).click()
            expect(pane.locator(".reference-detail h2")).to_contain_text(f"{key} stone")
            uses.locator("summary").first.click()
            page.get_by_role("button", name="简体中文", exact=True).click()
            expect(uses).to_contain_text("这是用途，不是获取方式")
            if key == "Mercury":
                expect(uses).to_contain_text("要求性别：雌性")
                expect(uses).to_contain_text("要求性别：雄性")
            page.get_by_role("button", name="English", exact=True).click()
            if key == "Mercury":
                uses.get_by_role("button", name=f"{key} held A ↗", exact=True).click()
                expect(pane.locator(".reference-detail h2")).to_contain_text(
                    f"{key} held A"
                )
                expect(
                    pane.locator(".acquisition-panel > .encounter-card")
                ).to_have_count(0)
                expect(pane.locator(".evolution-uses")).to_contain_text(
                    "Needed for evolution · 1"
                )
                pane.locator(".evolution-uses > summary").click()
                pane.locator(".evolution-uses").get_by_role(
                    "button", name=f"{key} species 2 ↗", exact=True
                ).first.click()
            else:
                uses.get_by_role(
                    "button", name=f"{key} species 2 ↗", exact=True
                ).first.click()
            if key in ["Rocket", "Ultimate", "Mercury"]:
                tree.get_by_role(
                    "button", name=f"{key} move 1 ↗", exact=True
                ).first.click()
                expect(pane.locator(".reference-detail h2")).to_contain_text(
                    f"{key} move 1"
                )
                pane.locator(".evolution-uses > summary").click()
                pane.locator(".evolution-uses").get_by_role(
                    "button", name=f"{key} species 3 ↗", exact=True
                ).click()
                expect(pane.locator(".reference-detail h2")).to_contain_text(
                    f"{key} species 3"
                )
            if key == "Ultimate":
                locations = tree.locator(".evolution-locations")
                locations.locator("summary").click()
                expect(
                    locations.locator(".evolution-location-options button")
                ).to_have_count(1)
                expect(locations).to_contain_text(
                    "specific tile, dynamic layout and current access are unverified"
                )
                locations.get_by_role(
                    "button", name="Ultimate cave floor · 1-0 ↗", exact=True
                ).click()
                expect(pane.locator(".map-navigation")).to_contain_text(
                    "Ultimate exterior"
                )
                pane.locator(".nav-path .link-button").first.click()
                expect(pane.locator(".reference-detail h2")).to_contain_text(
                    "Ultimate exterior"
                )
                page.get_by_role(
                    "button", name="Back to previous reference", exact=False
                ).click()
                page.get_by_role(
                    "button", name="Back to previous reference", exact=False
                ).click()
                expect(tree).to_be_visible()
                tree.locator(".evolution-locations > summary").click()
                page.get_by_role("button", name="简体中文", exact=True).click()
                expect(tree.locator(".evolution-locations")).to_contain_text(
                    "符合地点条件的地图记录"
                )
                expect(tree.locator(".evolution-locations")).to_contain_text(
                    "当前可达性未确认"
                )
                page.get_by_role("button", name="English", exact=True).click()
            for width in (900, 1100):
                page.set_viewport_size(dict(width=width, height=780))
                assert tree.evaluate("e=>e.scrollWidth<=e.clientWidth+1")
            if key in ["Ultimate", "Mercury"] and os.environ.get("GEN3_UI_ARTIFACTS"):
                out = Path(os.environ["GEN3_UI_ARTIFACTS"])
                out.mkdir(parents=True, exist_ok=True)
                tree.screenshot(path=str(out / f"{key.lower()}-evolution-links.png"))
            print(
                f"{key}: resource uses, tree/back, bilingual compact display and ROM isolation passed"
            )
        assert not any(
            r["command"] in ["action", "save_bytes", "export_save"] for r in calls
        )
        assert not errors, errors
        browser.close()


if __name__ == "__main__":
    main()
