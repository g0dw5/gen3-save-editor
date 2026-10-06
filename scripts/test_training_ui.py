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
        for key in os.environ.get(
            "GEN3_UI_TRAINING_PROFILES", "BW,DP,ROCKET,ULTIMATE,MERCURY12"
        ).split(","):
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
            if key in ["ROCKET", "MERCURY12", "ULTIMATE"]:
                catalog["abilities"] = [
                    dict(id=i, name=n, description="")
                    for i, n in [
                        (1, "First ability"),
                        (2, "Second ability"),
                        (3, "Hidden ability"),
                    ]
                ]
                catalog["species"][0]["abilities"] = [1, 2, 3]
                catalog["species"][1]["abilities"] = [1, 1, 0]
                catalog["items"].append(
                    dict(
                        id=3,
                        name=key + " runtime capsule",
                        description="Fixture",
                        tm_move=None,
                        price=100,
                    )
                )
                if key in ["ROCKET", "ULTIMATE"]:
                    catalog["items"].append(
                        dict(
                            id=4,
                            name=key + " runtime patch",
                            description="Fixture",
                            tm_move=None,
                            price=100,
                        )
                    )
            if key in ["ULTIMATE", "MERCURY12"]:
                catalog["items"].extend(
                    [
                        dict(id=5, name=key + " runtime gold", tm_move=None, price=0),
                        dict(id=6, name=key + " runtime silver", tm_move=None, price=0),
                    ]
                )
            requests = []
            errors = []
            count = [0]
            delayed = []
            hold = [False]
            hold_service = [False]
            delayed_service = []
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
                elif cmd == "training_services":
                    assert payload["expected_rom_md5"] == catalog["profile"]["md5"]
                    credit = dict(
                        condition=dict(
                            kind="variable",
                            id=0x40FB,
                            value=0,
                            comparison=5,
                            taken=True,
                        ),
                        actual=(count[0] + 4 if count[0] else None),
                        satisfied=(True if count[0] else None),
                        unresolved=None,
                    )
                    requirement = dict(
                        condition=dict(
                            kind="bag_item", id=5, value=1, comparison=4, taken=True
                        ),
                        actual=(0 if count[0] else None),
                        satisfied=(False if count[0] else None),
                        unresolved=None,
                    )
                    services = (
                        [
                            dict(
                                kind=(
                                    "base_iv_training"
                                    if key == "MERCURY12"
                                    else "hyper_training_flags"
                                ),
                                minimum_level=(50 if key == "MERCURY12" else 100),
                                conditions=(
                                    []
                                    if key == "ULTIMATE"
                                    else [
                                        dict(
                                            condition=dict(
                                                kind="flag",
                                                id=0xB15,
                                                value=1,
                                                comparison=1,
                                                taken=True,
                                            ),
                                            actual=(
                                                int(bool(count[0]))
                                                if count[0]
                                                else None
                                            ),
                                            satisfied=(
                                                bool(count[0]) if count[0] else None
                                            ),
                                            unresolved=None,
                                        )
                                    ]
                                ),
                                choices=[
                                    dict(
                                        menu_index=i,
                                        name=(
                                            "All stats (gold)"
                                            if i == 0
                                            else f"Stat {i} (silver)"
                                        ),
                                        item=(5 if i == 0 else 6),
                                        quantity=1,
                                        stat=(None if i == 0 else i - 1),
                                        mask=(126 if i == 0 else 1 << i),
                                        credit=(credit if key == "ULTIMATE" else None),
                                        item_requirement=requirement,
                                        payment=dict(
                                            item=(
                                                5 if key == "MERCURY12" or i == 0 else 6
                                            ),
                                            quantity=1,
                                            before_stat_selection=(
                                                key == "MERCURY12" and i != 0
                                            ),
                                            result_checked=False,
                                        ),
                                    )
                                    for i in range(7)
                                ],
                                locations=[
                                    dict(
                                        map_id="26-13",
                                        map_name="Test map",
                                        x=3,
                                        y=4,
                                        visibility=[],
                                    )
                                ],
                                text=[key + " runtime service dialogue"],
                                menus=[
                                    dict(
                                        stage="training_choice",
                                        single_stat_only=False,
                                        cancel_with_b=True,
                                        payment_precedes_menu=False,
                                        command=124,
                                    )
                                ]
                                + (
                                    [
                                        dict(
                                            stage="stat_choice",
                                            single_stat_only=True,
                                            cancel_with_b=False,
                                            payment_precedes_menu=True,
                                            command=125,
                                        )
                                    ]
                                    if key == "MERCURY12"
                                    else []
                                ),
                                selection=dict(
                                    scope="party",
                                    cancel_with_b=True,
                                    rejects_fainted_during_selection=False,
                                    rejects_egg_during_selection=False,
                                ),
                                evidence=dict(root=123),
                                partial=True,
                            )
                        ]
                        if key in ["ULTIMATE", "MERCURY12"]
                        and catalog["profile"]["md5"] == "fixture-" + key
                        else []
                    )
                    data = dict(
                        rom_md5=catalog["profile"]["md5"],
                        services=services,
                        partial=True,
                    )
                elif cmd == "training_service_preview":
                    assert payload["expected_rom_md5"] == catalog["profile"]["md5"]
                    assert payload["service_root"] == 123
                    source = payload["individual"]
                    before = copy.deepcopy(save["pokemon"][0]["pokemon"])
                    level = source.get("level", 100)
                    before["ivs"] = source.get("ivs", [13] * 6)
                    before["hyper_trained"] = [False] * 6
                    before["current_hp"] = 70
                    after = copy.deepcopy(before)
                    threshold = 50 if key == "MERCURY12" else 100
                    enough = level >= threshold
                    index = payload["choice_index"]
                    if enough:
                        for stat in range(6):
                            if index == 0 or stat == index - 1:
                                if key == "MERCURY12":
                                    after["ivs"][stat] = 31
                                else:
                                    after["hyper_trained"][stat] = True
                    data = dict(
                        rom_md5=catalog["profile"]["md5"],
                        service_root=123,
                        choice_index=index,
                        before=before,
                        after=after,
                        party_stats_before=[70, 50, 40, 30, 20, 10],
                        party_stats_after=[70, 50, 40, 30, 20, 10],
                        native_level=level,
                        minimum_level=threshold,
                        level_satisfied=enough,
                        known_requirements_met=False,
                        changed=enough,
                        stat_refresh="immediate" if key == "MERCURY12" else "deferred",
                        scenario=(
                            "simulated_individual"
                            if source["kind"] == "simulated"
                            else "stored_individual"
                        ),
                        party_state=(
                            "simulated_full_hp"
                            if source["kind"] == "simulated"
                            else (
                                "stored_party"
                                if source["location"]["kind"] == "party"
                                else "boxed_full_hp_scenario"
                            )
                        ),
                        withdrawal_required=(
                            source["kind"] == "stored"
                            and source["location"]["kind"] == "box"
                        ),
                        partial=True,
                    )
                    if hold_service[0]:
                        hold_service[0] = False
                        delayed_service.append((route, data))
                        return
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
                        ability_items=(
                            [
                                dict(
                                    item=3,
                                    handler=3,
                                    mechanism="normal_swap",
                                    random_pid=key == "MERCURY12",
                                    requires_rng_seed=key == "MERCURY12",
                                    partial=True,
                                )
                            ]
                            + (
                                [
                                    dict(
                                        item=4,
                                        handler=4,
                                        mechanism="hidden_toggle",
                                        random_pid=False,
                                        requires_rng_seed=key == "ULTIMATE",
                                        partial=True,
                                    )
                                ]
                                if key in ["ROCKET", "ULTIMATE"]
                                else []
                            )
                            if key in ["ROCKET", "MERCURY12", "ULTIMATE"]
                            and catalog["profile"]["md5"] == "fixture-" + key
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
                    ability_item = payload["item"] in [3, 4]
                    accepted = True
                    if ability_item:
                        initial = payload["individual"].get("ability_slot", 0)
                        before.update(
                            pid=42, ability_slot=initial, ability_id=[1, 2, 3][initial]
                        )
                        after = copy.deepcopy(before)
                        accepted = (
                            key == "ULTIMATE" or payload["item"] == 4 or initial < 2
                        )
                        if key == "MERCURY12" or (
                            key == "ULTIMATE" and payload["item"] == 4
                        ):
                            assert (
                                isinstance(payload["rng_seed"], int)
                                and 0 <= payload["rng_seed"] <= 4294967295
                            )
                        unchanged_ability = (
                            key == "ULTIMATE" and payload["item"] == 3 and initial >= 2
                        )
                        if accepted and not unchanged_ability:
                            target = (
                                (0 if initial == 2 else 2)
                                if payload["item"] == 4
                                else initial ^ 1
                            )
                            after.update(
                                ability_slot=target,
                                ability_id=[1, 2, 3][target],
                                pid=4242 if key == "MERCURY12" else 42,
                            )
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
                    elif not ability_item and key != "MERCURY12":
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
                        native_no_effect=(
                            not accepted
                            if ability_item
                            else mint and before["effective_nature"] == 1
                        ),
                        ability_target=(
                            after["ability_id"] if ability_item and accepted else None
                        ),
                        rng_seed=payload.get("rng_seed"),
                        rng_after=123 if payload.get("rng_seed") is not None else None,
                        effect_scope=(
                            "ability_persistent_stage"
                            if ability_item
                            else "nature_persistent_stage" if mint else "field_effect"
                        ),
                        changed=(
                            accepted and not unchanged_ability
                            if ability_item
                            else (
                                (before["effective_nature"] != 1)
                                if mint
                                else key != "MERCURY12"
                            )
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
            if key in ["ULTIMATE", "MERCURY12"]:
                service = panel.locator(".training-service")
                expect(service).to_contain_text(
                    "Required level: at least "
                    + ("50" if key == "MERCURY12" else "100")
                )
                expect(service.locator(".training-service-choice")).to_have_count(1)
                expect(service).to_contain_text("B cancels this menu")
                expect(service).not_to_contain_text(
                    "B is ignored; select an option to continue"
                )
                expect(
                    service.get_by_role(
                        "combobox", name="Choose a training option", exact=True
                    )
                ).to_have_value("All stats (gold)")
                expect(service).to_contain_text(
                    "The referenced NPC script has an unlock flag"
                    if key == "MERCURY12"
                    else "A crown and a corresponding earned certification credit"
                )
                service.get_by_role(
                    "button", name="Test map · (3, 4) ↗", exact=True
                ).click()
                expect(page.locator(".map-navigation")).to_be_visible()
                page.get_by_role(
                    "button", name="← Back to previous reference", exact=True
                ).click()
                expect(service).to_be_visible()
                service.get_by_role(
                    "button", name=key + " runtime gold × 1 ↗", exact=True
                ).first.click()
                expect(page.locator(".acquisition-panel")).to_be_visible()
                assert any(
                    r["command"] == "acquisition" and r["payload"]["id"] == 5
                    for r in requests
                )
                page.get_by_role(
                    "button", name="← Back to previous reference", exact=True
                ).click()
                expect(service).to_be_visible()
                menu = service.get_by_role(
                    "combobox", name="Choose a training option", exact=True
                )
                menu.fill("silver")
                expect(page.get_by_role("listbox").get_by_role("option")).to_have_count(
                    6
                )
                page.get_by_role("option", name="Stat 3 (silver)", exact=True).click()
                expect(
                    service.get_by_role(
                        "button", name=key + " runtime silver × 1 ↗", exact=True
                    )
                ).to_be_visible()
                if key == "MERCURY12":
                    expect(service).to_contain_text(
                        "The ROM checks one item but attempts to remove a different one"
                    )
                    expect(service).to_contain_text("Before the single-stat menu")
                    expect(service).to_contain_text(
                        "B is ignored; select an option to continue"
                    )
                    expect(service).to_contain_text(
                        "Payment has already been attempted at this stage"
                    )
                    expect(
                        service.get_by_role(
                            "button", name="MERCURY12 runtime gold × 1 ↗", exact=True
                        )
                    ).to_be_visible()
                    expect(service).not_to_contain_text("Saved certification credits")
            else:
                expect(panel.locator(".training-service")).to_have_count(0)
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
            if key == "ULTIMATE":
                expect(panel.locator(".training-service")).to_contain_text(
                    "Saved certification credits · 5"
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
            if key == "ULTIMATE":
                expect(panel.locator(".training-service")).to_contain_text(
                    "Saved certification credits · 6"
                )
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
            if key in ["ROCKET", "MERCURY12", "ULTIMATE"]:
                panel.get_by_label("Individual for preview", exact=True).select_option(
                    "simulated"
                )
                selector = panel.get_by_role(
                    "combobox", name="Training item", exact=True
                )
                selector.fill("runtime capsule")
                page.get_by_role(
                    "option",
                    name=key + " runtime capsule · Normal ability change",
                    exact=True,
                ).click()
                initial = panel.get_by_label("Initial ability", exact=True)
                initial.select_option("0")
                if key == "MERCURY12":
                    seed_input = panel.get_by_label(
                        "Random seed for this preview", exact=True
                    )
                    for invalid in ["", "-1", "4294967296", "1.5"]:
                        seed_input.fill(invalid)
                        expect(
                            panel.get_by_role(
                                "button", name="Preview native effect", exact=True
                            )
                        ).to_be_disabled()
                    seed_input.fill("123456")
                else:
                    expect(
                        panel.get_by_label("Random seed for this preview", exact=True)
                    ).to_have_count(0)
                panel.get_by_role(
                    "button", name="Preview native effect", exact=True
                ).click()
                expect(panel.locator(".training-result")).to_contain_text(
                    "native ability guard accepts"
                )
                expect(panel.locator("tbody tr").nth(1)).to_contain_text(
                    "First ability"
                )
                expect(panel.locator("tbody tr").nth(1)).to_contain_text(
                    "Second ability"
                )
                expect(panel.locator("tbody tr").nth(2)).to_contain_text(
                    "4242" if key == "MERCURY12" else "42"
                )
                expect(panel.locator(".training-result")).not_to_contain_text(
                    "No EV value changed"
                )
                if key == "MERCURY12":
                    assert requests[-1]["payload"]["rng_seed"] == 123456
                    expect(panel.locator(".training-result")).to_contain_text(
                        "can reroll PID"
                    )
                else:
                    expect(panel.locator(".training-result")).to_contain_text(
                        "retains PID"
                    )
                panel.get_by_role(
                    "button", name="Find acquisition sources", exact=True
                ).click()
                page.locator(".acquisition-panel .link-button").filter(
                    has_text="Test map"
                ).click()
                for _ in range(2):
                    page.get_by_role(
                        "button", name="← Back to previous reference", exact=True
                    ).click()
                expect(selector).to_have_value(
                    key + " runtime capsule · Normal ability change"
                )
                if key == "MERCURY12":
                    expect(seed_input).to_have_value("123456")
                initial.select_option("2")
                panel.get_by_role(
                    "button", name="Preview native effect", exact=True
                ).click()
                expect(panel.locator(".training-result")).to_contain_text(
                    "no individual data changes"
                    if key == "ULTIMATE"
                    else "native ability guard rejects"
                )
                if key in ["ROCKET", "ULTIMATE"]:
                    selector.fill("runtime patch")
                    page.get_by_role(
                        "option",
                        name=key + " runtime patch · Hidden ability toggle",
                        exact=True,
                    ).click()
                    if key == "ULTIMATE":
                        seed_input = panel.get_by_label(
                            "Random seed for this preview", exact=True
                        )
                        expect(seed_input).to_have_value("42")
                        seed_input.fill("123456")
                    panel.get_by_role(
                        "button", name="Preview native effect", exact=True
                    ).click()
                    expect(panel.locator(".training-result")).to_contain_text(
                        "native ability guard accepts"
                    )
                    if key == "ULTIMATE":
                        assert requests[-1]["payload"]["rng_seed"] == 123456
                        expect(panel.locator(".training-result")).to_contain_text(
                            "randomly choose a normal slot"
                        )
                        expect(panel.locator("tbody tr").nth(2)).to_contain_text("42")
                    expect(panel.locator("tbody tr").nth(1)).to_contain_text(
                        "Hidden ability"
                    )
                    expect(panel.locator("tbody tr").nth(1)).to_contain_text(
                        "First ability"
                    )
                hold[0] = True
                panel.get_by_role(
                    "button", name="Preview native effect", exact=True
                ).click()
                page.wait_for_timeout(100)
                assert delayed
                if key == "MERCURY12":
                    seed_input.fill("123457")
                else:
                    initial.select_option("0")
                route, data = delayed.pop()
                route.fulfill(
                    content_type="application/json",
                    body=json.dumps(dict(ok=True, data=data)),
                )
                page.wait_for_timeout(100)
                expect(panel.locator(".training-result")).to_have_count(0)
                panel.get_by_role(
                    "button", name="Preview native effect", exact=True
                ).click()
                expect(panel.locator(".training-result")).to_be_visible()
            else:
                expect(panel.get_by_label("Initial ability", exact=True)).to_have_count(
                    0
                )
                expect(
                    panel.get_by_label("Random seed for this preview", exact=True)
                ).to_have_count(0)
            page.get_by_role("button", name="简体中文", exact=True).click()
            expect(panel).to_contain_text("培育道具与原生效果预览")
            if key == "ROCKET":
                expect(panel).to_contain_text("独立特性槽位并保留 PID")
            if key == "ULTIMATE":
                expect(panel.locator(".training-service")).to_contain_text(
                    "NPC 极限特训"
                )
                expect(panel.locator(".training-service")).to_contain_text(
                    "存档中的认证次数 · 6"
                )
            if key == "MERCURY12":
                # Reloading the SAV remounts this service card, so select the
                # silver option again before asserting its Chinese-only warning.
                menu = panel.locator(".training-service").get_by_role(
                    "combobox", name="选择训练项目", exact=True
                )
                menu.fill("silver")
                page.get_by_role("listbox").get_by_role(
                    "option", name="Stat 3 (silver)", exact=True
                ).click()
                expect(panel.locator(".training-service")).to_contain_text(
                    "NPC 基础个体值训练"
                )
                expect(panel.locator(".training-service")).to_contain_text(
                    "ROM 检查的道具与尝试扣除的不同"
                )
                expect(panel.locator(".training-service")).not_to_contain_text(
                    "存档中的认证次数"
                )
            page.set_viewport_size(dict(width=720, height=740))
            expect(panel).to_be_visible()
            assert panel.evaluate("(e)=>e.scrollWidth <= e.clientWidth + 1")
            if key in ["BW", "MERCURY12", "ULTIMATE"] and os.environ.get(
                "GEN3_UI_ARTIFACTS"
            ):
                d = Path(os.environ["GEN3_UI_ARTIFACTS"])
                d.mkdir(parents=True, exist_ok=True)
                panel.screenshot(path=str(d / f"training-{key}.png"))
                if key in ["ULTIMATE", "MERCURY12"]:
                    panel.evaluate("e => e.scrollTop = 0")
                    page.screenshot(path=str(d / f"training-{key}-service.png"))
            if key in ["MERCURY12", "ULTIMATE"]:
                page.get_by_role("button", name="English", exact=True).click()
                card = panel.locator(".training-service")
                card.get_by_role(
                    "combobox", name="Choose a training option", exact=True
                ).fill("gold")
                page.get_by_role("listbox").get_by_role(
                    "option", name="All stats (gold)", exact=True
                ).click()
                card.get_by_text(
                    "Preview this service on an individual", exact=True
                ).click()
                effect = card.locator(".training-service-preview")
                expect(
                    effect.get_by_label("Service preview individual", exact=True)
                ).to_be_visible()
                field = effect.get_by_label("Service preview level", exact=True)
                minimum = 50 if key == "MERCURY12" else 100
                field.fill(str(minimum - 1))
                effect.get_by_role(
                    "button", name="Preview service field effects", exact=True
                ).click()
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "Below the required level"
                )
                field.fill(str(minimum))
                effect.get_by_label("Simulated service IV · HP", exact=True).fill("13")
                effect.get_by_role(
                    "button", name="Preview service field effects", exact=True
                ).click()
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "Native individual-field result"
                )
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "recalculates party HP/stats"
                    if key == "MERCURY12"
                    else "leaves stored party stats unchanged"
                )
                expect(
                    effect.locator(".training-service-result tbody tr").first
                ).to_contain_text("13 → 31" if key == "MERCURY12" else "13 → 13")
                hold_service[0] = True
                effect.get_by_role(
                    "button", name="Preview service field effects", exact=True
                ).click()
                page.wait_for_timeout(100)
                assert delayed_service
                field.fill(str(minimum - 1))
                route, data = delayed_service.pop()
                route.fulfill(
                    content_type="application/json",
                    body=json.dumps(dict(ok=True, data=data)),
                )
                expect(effect.locator(".training-service-result")).to_have_count(0)
                field.fill(str(minimum))
                effect.get_by_label(
                    "Service preview individual", exact=True
                ).select_option("p:0")
                effect.get_by_role(
                    "button", name="Preview service field effects", exact=True
                ).click()
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "Uses the saved party record"
                )
                expect(card).to_contain_text(
                    "training selects a Pokémon from your party"
                )
                expect(card).to_contain_text("Press B to cancel party selection")
                expect(card).to_contain_text("Fainted Pokémon are not excluded")
                effect.get_by_label(
                    "Service preview individual", exact=True
                ).select_option("0:0")
                expect(effect).to_contain_text(
                    "First withdraw this Pokémon into your party"
                )
                effect.get_by_role(
                    "button", name="Preview service field effects", exact=True
                ).click()
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "simulated full-HP party projection"
                )
                assert requests[-1]["payload"]["individual"] == dict(
                    kind="stored", location=dict(kind="box", box_index=0, slot=0)
                )
                page.get_by_role("button", name="简体中文", exact=True).click()
                expect(card).to_contain_text("服务只能选择同行宝可梦")
                expect(card).to_contain_text("选择同行时可按 B 取消")
                expect(effect).to_contain_text("先在游戏内把这只宝可梦取回同行")
                page.get_by_role("button", name="English", exact=True).click()
                effect.get_by_label(
                    "Service preview individual", exact=True
                ).select_option("p:0")
                expect(
                    effect.get_by_text(
                        "First withdraw this Pokémon into your party", exact=False
                    )
                ).to_have_count(0)
                effect.get_by_role(
                    "button", name="Preview service field effects", exact=True
                ).click()
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "Uses the saved party record"
                )
                card.get_by_role(
                    "button", name="Test map · (3, 4) ↗", exact=True
                ).click()
                expect(page.locator(".map-navigation")).to_be_visible()
                page.get_by_role(
                    "button", name="← Back to previous reference", exact=True
                ).click()
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "Uses the saved party record"
                )
                card.get_by_role(
                    "button", name=key + " runtime gold × 1 ↗", exact=True
                ).first.click()
                expect(page.locator(".acquisition-panel")).to_be_visible()
                page.get_by_role(
                    "button", name="← Back to previous reference", exact=True
                ).click()
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "Uses the saved party record"
                )
                # A new SAV snapshot invalidates a request already in flight.
                hold_service[0] = True
                effect.get_by_role(
                    "button", name="Preview service field effects", exact=True
                ).click()
                page.wait_for_timeout(100)
                assert delayed_service
                with page.expect_file_chooser() as chooser:
                    page.get_by_role(
                        "button", name="Open save", exact=True
                    ).first.click()
                chooser.value.set_files(
                    dict(
                        name="synthetic-refresh.sav",
                        mimeType="application/octet-stream",
                        buffer=b"fixture-only",
                    )
                )
                route, data = delayed_service.pop()
                route.fulfill(
                    content_type="application/json",
                    body=json.dumps(dict(ok=True, data=data)),
                )
                expect(panel.locator(".training-service-result")).to_have_count(0)
                card.get_by_text(
                    "Preview this service on an individual", exact=True
                ).click()
                effect.get_by_label(
                    "Service preview individual", exact=True
                ).select_option("p:0")
                effect.get_by_role(
                    "button", name="Preview service field effects", exact=True
                ).click()
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "Uses the saved party record"
                )
                page.get_by_role("button", name="简体中文", exact=True).click()
                expect(effect.locator(".training-service-result")).to_contain_text(
                    "原生个体字段结果"
                )
                expect(panel.locator(".training-service")).to_contain_text(
                    "可按 B 取消此菜单"
                )
                assert panel.evaluate("e => e.scrollWidth <= e.clientWidth + 1")
                if os.environ.get("GEN3_UI_ARTIFACTS"):
                    effect.locator(
                        ".training-service-result"
                    ).scroll_into_view_if_needed()
                    page.screenshot(
                        path=str(
                            Path(os.environ["GEN3_UI_ARTIFACTS"])
                            / f"service-preview-{key}.png"
                        )
                    )
            if key in ["ROCKET", "MERCURY12", "ULTIMATE"]:
                page.get_by_role("button", name="English", exact=True).click()
                if key in ["MERCURY12", "ULTIMATE"]:
                    hold_service[0] = True
                    effect.get_by_role(
                        "button", name="Preview service field effects", exact=True
                    ).click()
                    page.wait_for_timeout(100)
                    assert delayed_service
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
                if key in ["MERCURY12", "ULTIMATE"]:
                    route, data = delayed_service.pop()
                    route.fulfill(
                        content_type="application/json",
                        body=json.dumps(dict(ok=True, data=data)),
                    )
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
                expect(panel.get_by_label("Initial ability", exact=True)).to_have_count(
                    0
                )
                expect(
                    panel.get_by_label("Random seed for this preview", exact=True)
                ).to_have_count(0)
                expect(
                    panel.get_by_label("Initial effective nature", exact=True)
                ).to_have_count(0)
                expect(
                    panel.get_by_label("Individual for preview", exact=True).locator(
                        "option"
                    )
                ).to_have_count(1)
                assert requests[-1]["payload"]["expected_rom_md5"] == "fixture-SWITCH"
            expect(panel.locator(".training-service")).to_have_count(0)
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
