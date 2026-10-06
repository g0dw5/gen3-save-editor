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
        for index, key in enumerate(
            os.environ.get(
                "GEN3_UI_COLLECTION_PROFILES", "BW,DP,ROCKET,ULTIMATE,MERCURY12"
            ).split(",")
        ):
            tag = [key]
            scripted_passages = os.environ.get("GEN3_UI_SCRIPT_WARPS") == "1"
            passage_satisfied = [False]
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

            def scripted_edge(met=None, identifier=17, kind="flag"):
                result = copy.deepcopy(edge)
                c = dict(kind=kind, id=identifier, value=1, comparison=1, taken=True)
                result.update(
                    kind="script_warp",
                    offset=201,
                    x=3,
                    y=2,
                    target_x=2,
                    target_y=2,
                    script=dict(
                        root=300,
                        source_kind="npc",
                        local_id=1,
                        opcode=59,
                        conditions=[c],
                        checks=[
                            dict(
                                condition=c,
                                satisfied=met,
                                actual=None if met is None else int(met),
                                unresolved=None,
                            )
                        ],
                        stopped_at=[300],
                        entry_unresolved=True,
                    ),
                )
                return result

            def collection_entrances():
                if not scripted_passages:
                    return [dict(map_id="0-1", chains=[[edge]], truncated=False)]
                missing = scripted_edge(passage_satisfied[0])
                dynamic = scripted_edge(None, 0x8001, "variable")
                dynamic.update(
                    offset=202,
                    target_x=None,
                    target_y=None,
                    unresolved="dynamic_coordinates",
                )
                return [
                    dict(
                        map_id="0-1",
                        chains=[
                            [edge],
                            [missing],
                            [scripted_edge(None, 0x8001, "variable")],
                            [scripted_edge(True, 19)],
                            [missing],
                        ],
                        unresolved_incoming=[dynamic],
                        truncated=True,
                    )
                ]

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
                entrances=collection_entrances(),
                partial=True,
                clock=None,
            )

            if scripted_passages:
                tasks[0]["preparation"] = dict(
                    origin=2,
                    current_count=0,
                    source=source,
                    steps=[],
                    breeding=None,
                    needs_hatching=False,
                    truncated=False,
                    partial=True,
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
            hold_trace, pending_traces = [False], []
            prerequisites_enabled = os.environ.get("GEN3_UI_PREREQUISITES") == "1"

            def prerequisites():
                def guard(identifier, met=False):
                    return dict(
                        condition=dict(
                            kind="flag",
                            id=identifier,
                            value=1,
                            comparison=1,
                            taken=True,
                        ),
                        actual=int(met),
                        satisfied=met,
                        unresolved=None,
                    )

                coverage = dict(
                    checked_scripts=3,
                    total_scripts=3,
                    failed_scripts=0,
                    truncated=False,
                )

                def writer(identifier, guards, number):
                    return dict(
                        effect=dict(
                            kind="flag",
                            id=identifier,
                            operation="set",
                            operand=None,
                            value=1,
                            offset=300 + number,
                            conditions=[g["condition"] for g in guards],
                        ),
                        reference=dict(
                            map_id="0-1",
                            map_name=tag[0] + " Floor",
                            region=1,
                            kind="npc",
                            x=1,
                            y=1,
                            local_id=1,
                            offset=400 + number,
                            root=300 + number,
                            entry_unresolved=True,
                        ),
                        conditions=guards,
                        text=[
                            dict(
                                offset=500 + number,
                                text=tag[0]
                                + " event <script>clue</script> "
                                + str(number),
                            )
                        ],
                        stopped_at=[],
                        path_complete=True,
                    )

                writers = [writer(11, [guard(12)], 0)] + [
                    writer(11, [], i) for i in range(1, 9)
                ]
                reports = [
                    dict(
                        rom_md5=catalog["profile"]["md5"],
                        condition=guard(11),
                        writers=writers,
                        coverage=coverage,
                        total_matches=9,
                        next_offset=None,
                        partial=True,
                    ),
                    dict(
                        rom_md5=catalog["profile"]["md5"],
                        condition=guard(12),
                        writers=[writer(12, [guard(11)], 9)],
                        coverage=coverage,
                        total_matches=1,
                        next_offset=None,
                        partial=True,
                    ),
                    dict(
                        rom_md5=catalog["profile"]["md5"],
                        condition=guard(13, True),
                        writers=[
                            dict(
                                writer(13, [guard(11)], 10),
                                text=[
                                    dict(
                                        offset=510,
                                        text="Do not replay completed-only event",
                                    )
                                ],
                            )
                        ],
                        coverage=coverage,
                        total_matches=0,
                        next_offset=None,
                        partial=True,
                    ),
                ]
                if scripted_passages:
                    reports.append(
                        dict(
                            rom_md5=catalog["profile"]["md5"],
                            condition=guard(17, passage_satisfied[0]),
                            writers=[],
                            coverage=coverage,
                            total_matches=0,
                            next_offset=None,
                            partial=True,
                        )
                    )
                return dict(
                    reports=reports,
                    routes=[
                        dict(
                            report_index=0,
                            goals=[[0, 0]],
                            candidates=[
                                dict(
                                    writer_index=i,
                                    requires=[1] if i == 0 else [],
                                    untraced_conditions=[],
                                    recursive=i == 0,
                                )
                                for i in range(9)
                            ],
                        ),
                        dict(
                            report_index=1,
                            goals=[[0, 0]],
                            candidates=[
                                dict(
                                    writer_index=0,
                                    requires=[0],
                                    untraced_conditions=[],
                                    recursive=True,
                                )
                            ],
                        ),
                        dict(report_index=2, goals=[[0, 0]], candidates=[]),
                    ]
                    + (
                        [dict(report_index=3, goals=[[0, 0]], candidates=[])]
                        if scripted_passages
                        else []
                    ),
                    entrances=collection_entrances(),
                    skipped_conditions=0,
                    truncated=False,
                    partial=True,
                )

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
                elif cmd in [
                    "collection",
                    "collection_export",
                    "collection_prerequisites",
                ]:
                    if cmd != "collection":
                        assert payload["expected_rom_md5"] == catalog["profile"]["md5"]
                    data = dict(
                        plan,
                        entrances=collection_entrances(),
                        prerequisites=(
                            prerequisites()
                            if prerequisites_enabled and cmd != "collection"
                            else None
                        ),
                    )
                    if cmd == "collection_prerequisites" and hold_trace[0]:
                        hold_trace[0] = False
                        pending_traces.append(
                            (route, json.dumps(dict(ok=True, data=data)))
                        )
                        return
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
                    scripted = copy.deepcopy(edge)
                    scripted.update(
                        kind="script_warp",
                        offset=201,
                        x=3,
                        y=2,
                        target_x=2,
                        target_y=2,
                        script=dict(
                            root=300,
                            source_kind="npc",
                            local_id=1,
                            opcode=59,
                            conditions=[
                                dict(
                                    kind="flag",
                                    id=17,
                                    value=1,
                                    comparison=1,
                                    taken=True,
                                )
                            ],
                            checks=[
                                dict(
                                    condition=dict(
                                        kind="flag",
                                        id=17,
                                        value=1,
                                        comparison=1,
                                        taken=True,
                                    ),
                                    satisfied=passage_satisfied[0],
                                    actual=int(passage_satisfied[0]),
                                    unresolved=None,
                                )
                            ],
                            stopped_at=[300],
                            entry_unresolved=True,
                        ),
                    )
                    dynamic = copy.deepcopy(scripted)
                    dynamic.update(
                        to=None,
                        target_x=None,
                        target_y=None,
                        offset=202,
                        unresolved="dynamic_coordinates",
                    )
                    data = dict(
                        map_id=payload["id"],
                        outgoing=(
                            ([dynamic] if payload["id"] == "0-1" else [edge, scripted])
                            if scripted_passages
                            else []
                        ),
                        incoming=(
                            ([edge, scripted] if payload["id"] == "0-1" else [])
                            if scripted_passages
                            else [edge]
                        ),
                        approaches=(
                            [[scripted]]
                            if scripted_passages and payload["id"] == "0-1"
                            else [[edge]]
                        ),
                        truncated=False,
                        diagnostics=[],
                    )
                elif cmd in ["map_image", "sprite", "object_sprite"]:
                    data = dict(
                        url=(
                            'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="lightblue"/></svg>'
                            if cmd == "map_image"
                            else ""
                        )
                    )
                elif cmd == "open_rom":
                    configure(key + "-SWITCH")
                    data = dict(catalog=catalog)
                elif cmd == "open_save":
                    passage_satisfied[0] = True
                    data = copy.deepcopy(save)
                else:
                    raise AssertionError(req)
                route.fulfill(
                    content_type="application/json",
                    body=json.dumps(dict(ok=True, data=data)),
                )

            page = browser.new_page(
                viewport=(
                    dict(width=900, height=640)
                    if os.environ.get("GEN3_UI_COMPACT") == "1"
                    else dict(width=1050, height=780)
                ),
                accept_downloads=True,
            )
            page.add_init_script("localStorage.setItem('gen3.locale','en')")
            page.on("pageerror", lambda error: errors.append(str(error)))
            page.route("**/api", respond)
            page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
            page.get_by_role("button", name="ROM reference", exact=True).click()
            page.get_by_role("button", name="Collection planning", exact=True).click()
            panel = page.locator(".collection-panel")
            card = panel.locator("article.encounter-card").first
            expect(card.locator(".acquisition-source-facts").first).to_contain_text(
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
            if scripted_passages:
                entries = card.locator(".collection-entrances").first
                expect(entries).to_contain_text(
                    "Each approach is a separate alternative"
                )
                expect(entries.locator(".entrance-path")).to_have_count(3)
                expect(entries.locator(".entrance-path").nth(1)).to_contain_text(
                    "Missing parsed conditions"
                )
                expect(entries.locator(".entrance-path").nth(2)).to_contain_text(
                    "Conditions unresolved"
                )
                entries.get_by_role(
                    "button", name="Show more approaches", exact=True
                ).click()
                expect(entries.locator(".entrance-path")).to_have_count(5)
                expect(entries.locator(".entrance-path").nth(3)).to_contain_text(
                    "Parsed conditions met; access unverified"
                )
                entries.locator(".entrance-unresolved > summary").click()
                expect(entries.locator(".entrance-unresolved")).to_contain_text(
                    "do not prove this passage is currently usable"
                )
                expect(
                    card.locator(".collection-preparation .collection-entrances")
                ).to_contain_text("Missing parsed conditions")
                if os.environ.get("GEN3_UI_ARTIFACTS"):
                    pictures = Path(os.environ["GEN3_UI_ARTIFACTS"])
                    pictures.mkdir(parents=True, exist_ok=True)
                    entries.scroll_into_view_if_needed()
                    page.locator(".floating.wide").screenshot(
                        path=str(pictures / (key + "-collection-entrances.png"))
                    )
                entries.locator(".entrance-path").nth(1).get_by_role(
                    "button", name=key + " Outside (3, 2) ↗", exact=True
                ).click()
                expect(page.locator(".map-focus")).to_be_visible()
                assert page.locator(".map-focus").evaluate(
                    "e => [e.style.left, e.style.top]"
                ) == ["87.5%", "62.5%"]
                page.get_by_role(
                    "button", name="Back to previous reference", exact=False
                ).click()
                expect(card).to_contain_text("Each approach is a separate alternative")
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
            card.get_by_role("button", name=key + " Floor ↗", exact=True).first.click()
            expect(page.locator(".map-navigation")).to_contain_text(key + " Outside")
            if scripted_passages:
                navigation = page.locator(".map-navigation")
                expect(navigation).to_contain_text("Scripted passage")
                expect(navigation).to_contain_text("access unverified")
                source_details = navigation.locator(".nav-script-details").first
                source_details.locator("summary").first.click()
                expect(source_details).to_contain_text(
                    "stepping on it does not establish a warp"
                )
                expect(source_details).to_contain_text("Missing prerequisite")
                with page.expect_file_chooser() as chooser:
                    page.get_by_role(
                        "button", name="Open save", exact=True
                    ).first.click()
                chooser.value.set_files(
                    dict(
                        name="synthetic-script-guards.sav",
                        mimeType="application/octet-stream",
                        buffer=b"fixture-only",
                    )
                )
                source_details.locator("summary").first.click()
                expect(source_details).to_contain_text("Satisfied")
                expect(source_details).to_contain_text(
                    "do not prove this passage is currently usable"
                )
                assert source_details.evaluate(
                    "el => el.scrollWidth <= el.clientWidth + 1"
                )
                if os.environ.get("GEN3_UI_ARTIFACTS"):
                    pictures = Path(os.environ["GEN3_UI_ARTIFACTS"])
                    pictures.mkdir(parents=True, exist_ok=True)
                    source_details.scroll_into_view_if_needed()
                    page.locator(".floating.wide").first.screenshot(
                        path=str(pictures / (key + "-script-passages.png"))
                    )
                navigation.get_by_role(
                    "button", name=key + " Outside ↗", exact=True
                ).first.click()
                expect(page.locator(".map-focus")).to_be_visible()
                marker = page.get_by_role(
                    "button", name="Scripted passage (3, 2)", exact=True
                )
                expect(marker).to_be_visible()
                marker.click()
                expect(page.locator(".map-navigation")).to_contain_text(
                    key + " Outside"
                )
                expect(page.locator(".map-focus")).to_be_visible()
                page.get_by_role("button", name="简体中文", exact=True).click()
                expect(page.locator(".map-navigation")).to_contain_text("脚本通道")
                expect(page.locator(".map-navigation")).to_contain_text("可达性待验证")
                page.get_by_role("button", name="English", exact=True).click()
                # Undo the two map hops to restore the original collection back target.
                for _ in range(2):
                    page.get_by_role(
                        "button", name="Back to previous reference", exact=False
                    ).click()
            page.get_by_role(
                "button", name="Back to previous reference", exact=False
            ).click()
            if scripted_passages:
                expect(
                    card.locator(".collection-entrances")
                    .first.locator(".entrance-path")
                    .nth(1)
                ).to_contain_text("Parsed conditions met; access unverified")
            if prerequisites_enabled:
                panel.get_by_role(
                    "button", name="Trace prerequisites", exact=True
                ).click()
                prerequisites_panel = panel.locator(".collection-prerequisites")
                expect(prerequisites_panel).to_be_visible()
                expect(
                    prerequisites_panel.locator(".prerequisite-route")
                ).to_have_count(4 if scripted_passages else 3)
                first = prerequisites_panel.locator(".prerequisite-route").nth(0)
                first.locator("summary").first.click()
                expect(first.locator(".prerequisite-candidate")).to_have_count(8)
                expect(first).to_contain_text("contains a recursive clue")
                expect(first).to_contain_text("Parsed conditions met")
                first.get_by_role(
                    "button", name="Show more matching clues", exact=True
                ).click()
                expect(first.locator(".prerequisite-candidate")).to_have_count(9)
                met = prerequisites_panel.locator(".prerequisite-route").nth(2)
                met.locator("summary").first.click()
                expect(met.locator(".prerequisite-candidate")).to_have_count(0)
                first.locator(".prerequisite-candidate").first.get_by_role(
                    "button", name="Prerequisite clue 2", exact=False
                ).click()
                second = prerequisites_panel.locator(".prerequisite-route").nth(1)
                expect(second).to_have_attribute("open", "")
                second.locator("summary").first.click()
                first.locator(".prerequisite-candidate").first.get_by_role(
                    "button", name="Prerequisite clue 2", exact=False
                ).click()
                expect(second).to_have_attribute("open", "")
                first.locator(".prerequisite-candidate").first.get_by_role(
                    "button", name=key + " Floor (1, 1) ↗", exact=True
                ).first.click()
                expect(page.locator(".map-focus")).to_be_visible()
                page.get_by_role(
                    "button", name="Back to previous reference", exact=False
                ).click()
                expect(prerequisites_panel).to_be_visible()
                expect(first.locator(".prerequisite-candidate")).to_have_count(9)
                first.locator(".prerequisite-candidate").first.get_by_text(
                    "Text referenced by this script", exact=True
                ).click()
                expect(first.locator("blockquote").first).to_have_text(
                    key + " event <script>clue</script> 0"
                )
            page.get_by_role("button", name="简体中文", exact=True).click()
            expect(card).to_contain_text("相遇槽位概率 20%")
            expect(card).to_contain_text("时间条件:")
            expect(card).to_contain_text("关联资料")
            if prerequisites_enabled:
                expect(prerequisites_panel).to_contain_text("按区域查看前置线索")
                expect(first).to_contain_text("循环不代表目标无法完成")
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
            if scripted_passages:
                assert "每条入口是独立备选路线" in html and "条件无法确定" in html
                assert "已解析条件满足；可达性未确认" in html
                assert 'href="#prerequisite-flag%3A17%3A1%3A1%3Atrue"' in html
                assert 'id="prerequisite-flag%3A17%3A1%3A1%3Atrue"' in html
                assert "入口路线 5" in html and 'class="entrance-unresolved"' in html
                assert "只表示 NPC 所在格位" in html or "踩上" in html
            if prerequisites_enabled:
                assert "关联收集目标" in html and 'href="#task-0-0"' in html
                assert "循环不代表目标无法完成" in html
                assert "&lt;script&gt;clue&lt;/script&gt;" in html
                assert "Do not replay completed-only event" not in html
            if os.environ.get("GEN3_UI_ARTIFACTS"):
                (out / (key + "-collection.html")).write_text(html)
            # A movable window can overlap the compact wrapped app toolbar.
            # Move it through its native pointer interaction before using that toolbar.
            titlebar = page.locator(".floating-header").first.bounding_box()
            page.mouse.move(titlebar["x"] + 60, titlebar["y"] + 12)
            page.mouse.down()
            page.mouse.move(titlebar["x"] + 60, 300, steps=8)
            page.mouse.up()
            page.get_by_role("button", name="English", exact=True).click()
            pending_test = (
                prerequisites_enabled and os.environ.get("GEN3_UI_PENDING") == "1"
            )
            if pending_test:
                hold_trace[0] = True
                with page.expect_request(
                    lambda request: request.url.endswith("/api")
                    and request.post_data_json["command"] == "collection_prerequisites"
                ):
                    panel.get_by_role(
                        "button", name="Trace prerequisites", exact=True
                    ).click()
                with page.expect_file_chooser() as chooser:
                    page.get_by_role(
                        "button", name="Open save", exact=True
                    ).first.click()
                chooser.value.set_files(
                    dict(
                        name="synthetic-reload.sav",
                        mimeType="application/octet-stream",
                        buffer=b"fixture-only",
                    )
                )
                expect(page.locator(".collection-prerequisites")).to_have_count(0)
                assert len(pending_traces) == 1
                route, body = pending_traces.pop()
                route.fulfill(content_type="application/json", body=body)
                expect(page.locator(".collection-prerequisites")).to_have_count(0)
                hold_trace[0] = True
                with page.expect_request(
                    lambda request: request.url.endswith("/api")
                    and request.post_data_json["command"] == "collection_prerequisites"
                ):
                    panel.get_by_role(
                        "button", name="Trace prerequisites", exact=True
                    ).click()
            with page.expect_file_chooser() as chooser:
                page.get_by_role("button", name="Open ROM", exact=True).first.click()
            chooser.value.set_files(
                dict(
                    name="synthetic.gba",
                    mimeType="application/octet-stream",
                    buffer=b"fixture-only",
                )
            )
            if pending_test:
                assert len(pending_traces) == 1
                route, body = pending_traces.pop()
                route.fulfill(content_type="application/json", body=body)
            # No previous SAV collection remains after a ROM switch.
            page.get_by_role("button", name="ROM reference", exact=True).click()
            page.get_by_role("button", name="Collection planning", exact=True).click()
            expect(page.locator(".collection-panel")).to_contain_text("Open a SAV")
            expect(page.locator(".collection-prerequisites")).to_have_count(0)
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
            if prerequisites_enabled:
                expect(page.locator(".collection-prerequisites")).to_have_count(0)
            assert not errors, errors
            assert not any(
                r["command"] in ["edit", "edit_bag", "batch", "export_save", "drag"]
                for r in requests
            )
            print(
                key
                + " collection source facts / target-map-back / HTML / switch passed"
                + (
                    "; scripted passages / fresh SAV guards / bilingual markers passed"
                    if scripted_passages
                    else ""
                )
                + (
                    "; prerequisite alternatives/cycles/pagination/return passed"
                    if prerequisites_enabled
                    else ""
                )
                + ("; pending SAV/ROM responses discarded" if pending_test else ""),
                flush=True,
            )
            page.close()
        browser.close()


if __name__ == "__main__":
    main()
