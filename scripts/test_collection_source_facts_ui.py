"""Five-profile collection -> related target/map/back -> standalone HTML fixtures.

Synthetic API content only. No real files, SAV edits or private ROM assets. Native
source semantics are verified separately; these tests check presentation/closure.
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
    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        for index, key in enumerate(["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"]):
            tag = [key]
            catalog = copy.deepcopy(CATALOG)
            save = dict(
                trainer=dict(name="TEST"),
                pokemon=[pokemon(2, dict(kind="party", slot=0))],
                boxes=[
                    dict(index=i, name=f"Box {i}", count=0, wallpaper=0)
                    for i in range(14)
                ],
                bag=[],
                dex=[],
                active_slot=0,
                counter=1,
                backup_valid=True,
                dirty=False,
                can_undo=False,
                can_redo=False,
                changes=[],
            )
            source = dict(
                kind="good_rod",
                map_id="0-1",
                region=1,
                x=None,
                y=None,
                underfoot=None,
                related=[
                    dict(kind="item", id=1),
                    dict(kind="move", id=2),
                    dict(kind="species", id=2),
                ],
                quantity=None,
                min_level=5,
                max_level=10,
                encounter_percent=20,
                held_percent=None,
                periods=["night"],
                conditions=[],
                requirements=[],
                evolution=None,
                status="unknown",
                receipt_flag=None,
                repeatable=True,
                offset=100,
                partial=True,
                in_scenario=False,
            )
            hidden = dict(
                source,
                kind="hidden",
                x=1,
                y=1,
                underfoot=True,
                related=[],
                quantity=3,
                min_level=None,
                max_level=None,
                encounter_percent=None,
                periods=[],
                status="available",
                repeatable=None,
                in_scenario=None,
                receipt_flag=10,
            )
            edge = {
                "from": "0-0",
                "to": "0-1",
                "kind": "warp",
                "x": 2,
                "y": 1,
                "target_x": 1,
                "target_y": 1,
                "warp_index": 0,
                "target_warp": 0,
                "direction": None,
                "displacement": None,
                "offset": 200,
                "unresolved": None,
            }
            maps = [
                dict(id="0-0", name="Outside", region=1, width=4, height=4, map_type=1),
                dict(id="0-1", name="Floor", region=1, width=4, height=4, map_type=4),
            ]
            tasks = [
                dict(
                    target=dict(kind="species", id=1),
                    family=[1],
                    existing_family_members=[],
                    source=source,
                    alternatives=1,
                ),
                dict(
                    target=dict(kind="item", id=1),
                    family=[],
                    existing_family_members=[],
                    source=hidden,
                    alternatives=1,
                ),
                dict(
                    target=dict(kind="item", id=1),
                    family=[],
                    existing_family_members=[],
                    source=dict(hidden, kind="gift", quantity=1, repeatable=False),
                    alternatives=1,
                ),
            ]
            plan = dict(
                rom_md5="",
                basis="individuals",
                families=True,
                owned_count=1,
                missing_count=1,
                regions=[dict(region=1, tasks=tasks)],
                entrances=[dict(map_id="0-1", chains=[[edge]], truncated=False)],
                partial=True,
                clock=None,
            )

            def configure(label):
                tag[0] = label
                rods = [30 + index * 3 + i for i in range(3)]
                catalog["profile"].update(
                    id="synthetic",
                    label=label,
                    md5="fixture-" + label,
                    capabilities=dict(world=True, save_edit=True, dex=True),
                    fishing_rods=rods,
                )
                catalog["species"][0]["name"] = label + " goal <script>unsafe</script>"
                catalog["species"][1]["name"] = label + " parent"
                catalog["items"] = [
                    dict(id=0, name="", tm_move=None),
                    dict(
                        id=1,
                        name=label + " stone",
                        tm_move=None,
                        description="",
                        pocket=1,
                        price=10,
                    ),
                ] + [
                    dict(
                        id=id,
                        name=label + " ROM rod " + str(i),
                        tm_move=None,
                        description="",
                        pocket=1,
                        price=0,
                    )
                    for i, id in enumerate(rods)
                ]
                catalog["moves"] = [
                    dict(id=0, name="", pp=0),
                    dict(
                        id=2,
                        name=label + " required move",
                        pp=10,
                        move_type=0,
                        power=40,
                        accuracy=100,
                        priority=0,
                        chance=0,
                        category=0,
                        description="",
                        effect=0,
                        target=0,
                        flags=0,
                        offset=0,
                    ),
                ]
                maps[0]["name"] = label + " Outside"
                maps[1]["name"] = label + " Floor"
                plan["rom_md5"] = catalog["profile"]["md5"]

            configure(key)
            requests, errors = [], []

            def respond(route):
                req = route.request.post_data_json
                requests.append(req)
                cmd, payload = req["command"], req["payload"]
                if cmd == "state":
                    data = dict(catalog=catalog, save=save)
                elif cmd == "world":
                    data = dict(
                        maps=maps,
                        map_events=[],
                        encounters=[],
                        trainers=[],
                        map_groups=[],
                        trainer_locations=dict(locations=[]),
                    )
                elif cmd in ["collection", "collection_export"]:
                    if cmd == "collection_export":
                        assert payload["expected_rom_md5"] == catalog["profile"]["md5"]
                    data = plan
                elif cmd == "species":
                    data = dict(
                        species=catalog["species"][payload["id"] - 1],
                        evolutions=[],
                        learnset=[],
                        encounters=[],
                        origins={},
                    )
                elif cmd == "acquisition":
                    data = dict(
                        target=payload,
                        sources=[source if payload["kind"] == "species" else hidden],
                        partial=True,
                        clock=None,
                    )
                elif cmd == "map_navigation":
                    data = dict(
                        map_id=payload["id"],
                        outgoing=[],
                        incoming=[edge],
                        approaches=[[edge]],
                        truncated=False,
                        diagnostics=[],
                    )
                elif cmd in ["map_image", "sprite", "object_sprite"]:
                    data = dict(url="")
                elif cmd == "open_rom":
                    configure(key + "-SWITCH")
                    data = dict(catalog=catalog)
                elif cmd == "open_save":
                    data = save
                else:
                    raise AssertionError(req)
                route.fulfill(
                    content_type="application/json",
                    body=json.dumps(dict(ok=True, data=data)),
                )

            page = browser.new_page(
                viewport=dict(width=1050, height=780), accept_downloads=True
            )
            page.add_init_script("localStorage.setItem('gen3.locale','en')")
            page.on("pageerror", lambda error: errors.append(str(error)))
            page.route("**/api", respond)
            page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
            page.get_by_role("button", name="ROM reference", exact=True).click()
            page.get_by_role("button", name="Collection planning", exact=True).click()
            panel = page.locator(".collection-panel")
            card = panel.locator("article.encounter-card").first
            expect(card.locator(".acquisition-source-facts")).to_contain_text(
                key + " ROM rod 1"
            )
            expect(card).to_contain_text("Lv. 5–10")
            expect(card).to_contain_text("Encounter slot probability 20%")
            expect(card).to_contain_text("Time conditions: Night")
            expect(card).to_contain_text("Repeatable source")
            expect(panel.locator("article.encounter-card").nth(1)).to_contain_text(
                "Quantity × 3"
            )
            expect(
                panel.locator("article.encounter-card")
                .nth(1)
                .locator(".acquisition-source-facts")
            ).not_to_contain_text("probability")
            expect(panel.locator("article.encounter-card").nth(2)).to_contain_text(
                "Single receipt"
            )
            # Each target type resolves its own current-ROM name and returns here.
            for kind, label in [
                ("item", "stone"),
                ("move", "required move"),
                ("species", "parent"),
            ]:
                with page.expect_response(
                    lambda response: response.url.endswith("/api")
                    and response.request.post_data_json["command"] == "acquisition"
                    and response.request.post_data_json["payload"]["kind"] == kind
                ):
                    card.locator(".collection-related").get_by_role(
                        "button", name=key + " " + label + " ↗", exact=True
                    ).click()
                expect(page.locator(".acquisition-panel")).to_be_visible()
                page.get_by_role(
                    "button", name="Back to previous reference", exact=False
                ).click()
                expect(card).to_contain_text(key + " ROM rod 1")
            panel.get_by_role(
                "textbox", name="Search goals / regions / maps", exact=True
            ).fill(key + " required move")
            expect(panel.locator("article.encounter-card")).to_have_count(1)
            panel.get_by_role(
                "textbox", name="Search goals / regions / maps", exact=True
            ).fill("")
            card.get_by_role("button", name=key + " Floor ↗", exact=True).click()
            expect(page.locator(".map-navigation")).to_contain_text(key + " Outside")
            page.get_by_role(
                "button", name="Back to previous reference", exact=False
            ).click()
            page.get_by_role("button", name="简体中文", exact=True).click()
            expect(card).to_contain_text("相遇槽位概率 20%")
            expect(card).to_contain_text("时间条件:")
            expect(card).to_contain_text("关联资料")
            page.set_viewport_size(dict(width=720, height=780))
            assert panel.evaluate("e => e.scrollWidth <= e.clientWidth + 1")
            if os.environ.get("GEN3_UI_ARTIFACTS"):
                out = Path(os.environ["GEN3_UI_ARTIFACTS"])
                out.mkdir(parents=True, exist_ok=True)
                page.screenshot(path=str(out / (key + "-planning.png")))
            with page.expect_download() as download:
                panel.get_by_role("button", name="导出独立 HTML", exact=True).click()
            html = Path(download.value.path()).read_text()
            assert "相遇槽位概率 20%" in html and key + " ROM rod 1" in html
            assert 'href="#task-0-1"' in html and 'id="task-0-2"' in html
            assert (
                "&lt;script&gt;unsafe&lt;/script&gt;" in html and "<script>" not in html
            )
            assert key + " required move" in html and "default-src 'none'" in html
            if os.environ.get("GEN3_UI_ARTIFACTS"):
                (out / (key + "-collection.html")).write_text(html)
            page.get_by_role("button", name="English", exact=True).click()
            with page.expect_file_chooser() as chooser:
                page.get_by_role("button", name="Open ROM", exact=True).first.click()
            chooser.value.set_files(
                dict(
                    name="synthetic.gba",
                    mimeType="application/octet-stream",
                    buffer=b"fixture-only",
                )
            )
            # No previous SAV collection remains after a ROM switch.
            page.get_by_role("button", name="ROM reference", exact=True).click()
            page.get_by_role("button", name="Collection planning", exact=True).click()
            expect(page.locator(".collection-panel")).to_contain_text("Open a SAV")
            with page.expect_file_chooser() as chooser:
                page.get_by_role("button", name="Open save", exact=True).first.click()
            chooser.value.set_files(
                dict(
                    name="synthetic.sav",
                    mimeType="application/octet-stream",
                    buffer=b"fixture-only",
                )
            )
            expect(
                page.locator(".collection-panel .acquisition-source-facts").first
            ).to_contain_text(key + "-SWITCH ROM rod 1")
            assert not errors, errors
            assert not any(
                r["command"] in ["edit", "edit_bag", "batch", "export_save", "drag"]
                for r in requests
            )
            print(
                key
                + " collection source facts / target-map-back / HTML / switch passed",
                flush=True,
            )
            page.close()
        browser.close()


if __name__ == "__main__":
    main()
