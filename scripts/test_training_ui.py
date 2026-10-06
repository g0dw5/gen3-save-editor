"""Bilingual read-only training -> item -> map -> back fixtures, no real files."""

import copy
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, WORLD
from test_editor_navigation import pokemon


def main():
    expect.set_options(timeout=30000)
    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        context = browser.new_context(viewport=dict(width=1100, height=800))
        for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"]:
            catalog = copy.deepcopy(CATALOG)
            catalog["profile"].update(
                md5="fixture-" + key, max_level=100, training={"classify": 1}
            )
            catalog["items"] = [
                dict(id=0, name="", tm_move=None, price=0),
                dict(
                    id=1,
                    name=key + " EV item",
                    description="Runtime fixture",
                    tm_move=None,
                    price=500,
                ),
            ]
            if key == "ROCKET":
                catalog["items"].append(
                    dict(
                        id=2,
                        name="ROCKET runtime mint",
                        description="Fixture mint",
                        tm_move=None,
                        price=100,
                    )
                )
            requests = []
            errors = []
            count = [0]
            delayed = []
            hold = [False]
            save = dict(
                trainer={"name": "TEST"},
                pokemon=[
                    pokemon(2, dict(kind="party", slot=0)),
                    pokemon(2, dict(kind="box", box_index=0, slot=0)),
                ],
                boxes=[
                    dict(index=i, name=f"Box {i+1}", count=0, wallpaper=0)
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

            def respond(route):
                req = route.request.post_data_json
                requests.append(req)
                cmd, payload = req["command"], req["payload"]
                if cmd == "state":
                    data = dict(catalog=catalog, save=None)
                elif cmd == "world":
                    data = WORLD
                elif cmd == "open_rom":
                    catalog["profile"]["md5"] = "fixture-SWITCH"
                    catalog["items"] = catalog["items"][:2]
                    catalog["items"][1]["name"] = "SWITCH EV item"
                    count[0] = 0
                    data = dict(catalog=catalog)
                elif cmd == "open_save":
                    count[0] += 1
                    data = dict(save, counter=count[0])
                elif cmd in ["sprite", "map_image"]:
                    data = dict(url="")
                elif cmd == "species":
                    data = dict(
                        species=catalog["species"][payload["id"] - 1],
                        evolutions=[],
                        learnset=[],
                        encounters=[],
                        origins={},
                    )
                elif cmd == "map_navigation":
                    data = dict(
                        map_id=payload["id"],
                        outgoing=[],
                        incoming=[],
                        approaches=[],
                        truncated=False,
                        diagnostics=[],
                    )
                elif cmd == "acquisition":
                    data = dict(
                        target=payload,
                        sources=[
                            dict(
                                kind="pickup",
                                map_id="26-13",
                                region=1,
                                x=1,
                                y=1,
                                related=[],
                                quantity=1,
                                min_level=None,
                                max_level=None,
                                encounter_percent=None,
                                held_percent=None,
                                encounter_method=None,
                                periods=[],
                                conditions=[],
                                requirements=[],
                                status="unknown",
                                receipt_flag=None,
                                repeatable=None,
                                offset=100,
                                partial=True,
                            )
                        ],
                        partial=True,
                        clock=None,
                    )
                elif cmd == "training_catalog":
                    assert payload["expected_rom_md5"] == catalog["profile"]["md5"]
                    data = dict(
                        rom_md5=catalog["profile"]["md5"],
                        offers=[
                            dict(
                                item=1,
                                stat=0,
                                direction="increase",
                                handler=1,
                                native_category=13,
                                partial=True,
                            )
                        ],
                        nature_items=(
                            [dict(item=2, nature=1, handler=2, partial=True)]
                            if key == "ROCKET"
                            and catalog["profile"]["md5"] == "fixture-ROCKET"
                            else []
                        ),
                        partial=True,
                    )
                elif cmd == "training_preview":
                    assert payload["expected_rom_md5"] == catalog["profile"]["md5"]
                    before = copy.deepcopy(save["pokemon"][0]["pokemon"])
                    after = copy.deepcopy(before)
                    if payload["individual"]["kind"] == "simulated":
                        before["evs"] = payload["individual"]["evs"]
                        after["evs"] = copy.deepcopy(before["evs"])
                    mint = payload["item"] == 2
                    if mint:
                        initial = payload["individual"].get("nature_override", 26)
                        before["effective_nature"] = (
                            initial if initial < 25 else before["nature"]
                        )
                        after["effective_nature"] = 1
                        after["stats"][1] = (
                            30 if before["effective_nature"] == 1 else 33
                        )
                    elif key != "MERCURY12":
                        after["evs"][0] += 10
                    data = dict(
                        rom_md5=catalog["profile"]["md5"],
                        item=payload["item"],
                        before=before,
                        after=after,
                        party_stats_before=[30] * 6,
                        party_stats_after=(
                            [30, 55, 30, 30, 30, 30]
                            if mint and before["effective_nature"] != 1
                            else [30] * 6
                        ),
                        native_no_effect=mint and before["effective_nature"] == 1,
                        effect_scope=(
                            "nature_persistent_stage" if mint else "field_effect"
                        ),
                        changed=(
                            (before["effective_nature"] != 1)
                            if mint
                            else key != "MERCURY12"
                        ),
                        scenario="simulated_individual",
                        context="save_blocks" if count[0] else "zero_save_blocks",
                        party_state=(
                            "boxed_full_hp_scenario"
                            if payload["individual"].get("location", {}).get("kind")
                            == "box"
                            else "simulated_full_hp"
                        ),
                        partial=True,
                    )
                    if hold[0]:
                        hold[0] = False
                        delayed.append((route, data))
                        return
                else:
                    raise AssertionError(req)
                route.fulfill(
                    content_type="application/json",
                    body=json.dumps(dict(ok=True, data=data)),
                )

            page = context.new_page()
            page.add_init_script("localStorage.setItem('gen3.locale','en')")
            page.on("pageerror", lambda e: errors.append(str(e)))
            page.route("**/api", respond)
            page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
            page.get_by_role("button", name="ROM reference", exact=True).click()
            page.get_by_role("button", name="Training reference", exact=True).click()
            panel = page.locator(".training-panel")
            expect(
                panel.get_by_role("combobox", name="Training item", exact=True)
            ).to_have_value(key + " EV item · HP · Increase EVs")
            panel.get_by_label("HP EVs", exact=True).fill("90")
            panel.get_by_role(
                "button", name="Preview native effect", exact=True
            ).click()
            expect(panel.locator(".training-result")).to_contain_text(
                "Native field-use result"
            )
            expect(panel.locator("tbody tr").first).to_contain_text("90")
            if key == "MERCURY12":
                expect(panel.locator(".training-result")).to_contain_text(
                    "No EV value changed"
                )
            hold[0] = True
            panel.get_by_role(
                "button", name="Preview native effect", exact=True
            ).click()
            page.wait_for_timeout(100)
            assert delayed
            panel.get_by_label("HP EVs", exact=True).fill("99")
            route, data = delayed.pop()
            route.fulfill(
                content_type="application/json",
                body=json.dumps(dict(ok=True, data=data)),
            )
            page.wait_for_timeout(100)
            expect(panel.locator(".training-result")).to_have_count(0)
            panel.get_by_role(
                "button", name="Find acquisition sources", exact=True
            ).click()
            expect(page.locator(".acquisition-panel")).to_contain_text("Test map")
            page.locator(".acquisition-panel .link-button").filter(
                has_text="Test map"
            ).click()
            expect(page.locator(".map-navigation")).to_be_visible()
            page.get_by_role(
                "button", name="← Back to previous reference", exact=True
            ).click()
            expect(page.locator(".acquisition-panel")).to_be_visible()
            page.get_by_role(
                "button", name="← Back to previous reference", exact=True
            ).click()
            expect(panel.get_by_label("HP EVs", exact=True)).to_have_value("99")
            with page.expect_file_chooser() as choice:
                page.get_by_role("button", name="Open save", exact=True).first.click()
            choice.value.set_files(
                dict(
                    name="synthetic.sav",
                    mimeType="application/octet-stream",
                    buffer=b"fixture-only",
                )
            )
            panel.get_by_label("Individual for preview", exact=True).select_option(
                "0:0"
            )
            panel.get_by_role(
                "button", name="Preview native effect", exact=True
            ).click()
            expect(panel.locator(".training-result")).to_contain_text(
                "boxed individual"
            )
            with page.expect_file_chooser() as choice:
                page.get_by_role("button", name="Open save", exact=True).first.click()
            choice.value.set_files(
                dict(
                    name="synthetic.sav",
                    mimeType="application/octet-stream",
                    buffer=b"fixture-only",
                )
            )
            expect(panel.locator(".training-result")).to_have_count(0)
            if key == "ROCKET":
                panel.get_by_label("Individual for preview", exact=True).select_option(
                    "simulated"
                )
                selector = panel.get_by_role(
                    "combobox", name="Training item", exact=True
                )
                selector.fill("runtime mint")
                page.get_by_role(
                    "option",
                    name="ROCKET runtime mint · Effective nature · Nature 1",
                    exact=True,
                ).click()
                panel.get_by_label(
                    "Initial effective nature", exact=True
                ).select_option("3")
                panel.get_by_role(
                    "button", name="Preview native effect", exact=True
                ).click()
                expect(panel.locator(".training-result")).to_contain_text(
                    "persistent nature-and-stat stage"
                )
                expect(panel.locator("tbody tr").first).to_contain_text("Nature 3")
                expect(panel.locator("tbody tr").first).to_contain_text("Nature 1")
                expect(panel.locator("tbody tr").nth(2)).to_contain_text("55")
                expect(panel.locator(".training-result")).not_to_contain_text(
                    "No EV value changed"
                )
                panel.get_by_role(
                    "button", name="Find acquisition sources", exact=True
                ).click()
                page.locator(".acquisition-panel .link-button").filter(
                    has_text="Test map"
                ).click()
                page.get_by_role(
                    "button", name="← Back to previous reference", exact=True
                ).click()
                page.get_by_role(
                    "button", name="← Back to previous reference", exact=True
                ).click()
                expect(selector).to_have_value(
                    "ROCKET runtime mint · Effective nature · Nature 1"
                )
                expect(
                    panel.get_by_label("Initial effective nature", exact=True)
                ).to_have_value("3")
                panel.get_by_label(
                    "Initial effective nature", exact=True
                ).select_option("1")
                panel.get_by_role(
                    "button", name="Preview native effect", exact=True
                ).click()
                expect(panel.locator(".training-result")).to_contain_text(
                    "reports no effect"
                )
            else:
                expect(
                    panel.get_by_label("Initial effective nature", exact=True)
                ).to_have_count(0)
            page.get_by_role("button", name="简体中文", exact=True).click()
            expect(panel).to_contain_text("培育道具与原生效果预览")
            if key == "ROCKET":
                expect(panel).to_contain_text("持久化性格与能力处理阶段")
            page.set_viewport_size(dict(width=720, height=740))
            expect(panel).to_be_visible()
            assert panel.evaluate("(e)=>e.scrollWidth <= e.clientWidth + 1")
            if key in ["BW", "MERCURY12"] and os.environ.get("GEN3_UI_ARTIFACTS"):
                d = Path(os.environ["GEN3_UI_ARTIFACTS"])
                d.mkdir(parents=True, exist_ok=True)
                panel.screenshot(path=str(d / f"training-{key}.png"))
            if key in ["ROCKET", "MERCURY12"]:
                page.get_by_role("button", name="English", exact=True).click()
                hold[0] = True
                panel.get_by_role(
                    "button", name="Preview native effect", exact=True
                ).click()
                page.wait_for_timeout(100)
                assert delayed
                with page.expect_file_chooser() as choice:
                    page.get_by_role(
                        "button", name="Open ROM", exact=True
                    ).first.click()
                choice.value.set_files(
                    dict(
                        name="synthetic-switch.gba",
                        mimeType="application/octet-stream",
                        buffer=b"fixture-only",
                    )
                )
                expect(panel).to_have_count(0)
                route, data = delayed.pop()
                route.fulfill(
                    content_type="application/json",
                    body=json.dumps(dict(ok=True, data=data)),
                )
                page.get_by_role("button", name="ROM reference", exact=True).click()
                page.get_by_role(
                    "button", name="Training reference", exact=True
                ).click()
                expect(
                    panel.get_by_role("combobox", name="Training item", exact=True)
                ).to_have_value("SWITCH EV item · HP · Increase EVs")
                expect(panel.get_by_label("HP EVs", exact=True)).to_have_value("0")
                expect(panel.locator(".training-result")).to_have_count(0)
                expect(
                    panel.get_by_label("Initial effective nature", exact=True)
                ).to_have_count(0)
                expect(
                    panel.get_by_label("Individual for preview", exact=True).locator(
                        "option"
                    )
                ).to_have_count(1)
                assert requests[-1]["payload"]["expected_rom_md5"] == "fixture-SWITCH"
            assert not errors, errors
            assert not any(
                r["command"] in ["action", "export_save", "save_bytes"]
                for r in requests
            )
            page.close()
            print(
                key,
                "training/item/map/back/state/read-only bilingual fixture passed",
                flush=True,
            )
        browser.close()


if __name__ == "__main__":
    main()
