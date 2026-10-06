"""Bilingual clock/scenario/cache fixtures, no real ROM or SAV files.

Run Vite first; uses Playwright and Chrome. Native arithmetic is verified
separately with independent CPU vectors, not by this UI mock.
"""

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
        for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"]:
            mercury = key == "MERCURY12"
            catalog = copy.deepcopy(CATALOG)
            catalog["profile"].update(
                id="synthetic-" + key,
                md5="fixture-" + key,
                hardware_clock=None if mercury else dict(difference=0),
                clock=dict(starts=[4, 8, 17, 20]) if mercury else None,
            )
            saved = [0]
            requests = []
            errors = []
            delayed = []
            hold = [False]
            save = dict(
                trainer={"name": "TEST"},
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

            def report(payload):
                hour, weekday = payload.get("hour"), payload.get("weekday")
                r = dict(
                    rom_md5=catalog["profile"]["md5"],
                    source="unresolved",
                    effective_hour=None,
                    weekday=None,
                    period=None,
                    next_period_hour=None,
                    seconds_until_next_period=None,
                    saved=None,
                    forced_night=None,
                    issue=None,
                    rules=None,
                    hardware=None,
                    current_clock_verified=False,
                    forced_night_state_verified=False,
                )
                if mercury:
                    if hour is not None or weekday is not None:
                        r.update(
                            source="scenario", effective_hour=hour, weekday=weekday
                        )
                        if hour is not None:
                            r["period"] = (
                                "night"
                                if hour < 4 or hour >= 20
                                else (
                                    "morning"
                                    if hour < 8
                                    else "day" if hour < 17 else "dusk"
                                )
                            )
                    elif saved[0]:
                        r.update(
                            source="save_virtual",
                            effective_hour=17,
                            weekday=2,
                            period="dusk",
                            next_period_hour=20,
                            seconds_until_next_period=9000,
                            saved=dict(
                                year=2026,
                                month=10,
                                day=6,
                                weekday=2,
                                hour=17,
                                minute=30,
                                second=0,
                                speed=2,
                            ),
                            forced_night=False,
                            current_clock_verified=True,
                        )
                else:
                    r["issue"] = "hardware_rtc_unresolved"
                    if saved[0]:
                        r["hardware"] = dict(
                            offset=dict(days=1, hour=saved[0], minute=40, second=50),
                            last_update=dict(days=100, hour=8, minute=0, second=0),
                            offset_valid=True,
                            last_update_valid=True,
                        )
                return r

            def respond(route):
                req = route.request.post_data_json
                requests.append(req)
                command, payload = req["command"], req["payload"]
                if command == "state":
                    data = dict(catalog=catalog, save=None)
                elif command == "world":
                    data = WORLD
                elif command == "species":
                    data = dict(
                        species=catalog["species"][payload["id"] - 1],
                        evolutions=[],
                        learnset=[],
                        encounters=[],
                        origins={},
                    )
                elif command == "acquisition":
                    data = dict(target=payload, sources=[], partial=True, clock=None)
                elif command in ("sprite", "map_image"):
                    data = dict(url="")
                elif command == "open_save":
                    saved[0] += 1
                    data = dict(save, counter=saved[0])
                elif command == "clock_query":
                    data = report(payload)
                elif command == "clock_rtc_preview":
                    assert (
                        payload["expected_rom_md5"] == catalog["profile"]["md5"]
                        and not mercury
                    )
                    off = payload["offset"] or dict(
                        days=1, hour=saved[0], minute=40, second=50
                    )
                    r = payload["rtc"]
                    s = (
                        r["hour"] * 3600
                        + r["minute"] * 60
                        + r["second"]
                        - off["hour"] * 3600
                        - off["minute"] * 60
                        - off["second"]
                    ) % 86400
                    data = dict(
                        rom_md5=catalog["profile"]["md5"],
                        source="rtc_scenario",
                        input=r,
                        offset=off,
                        offset_source=(
                            "scenario" if payload["offset"] is not None else "save"
                        ),
                        local_time=dict(
                            days=1, hour=s // 3600, minute=s // 60 % 60, second=s % 60
                        ),
                        current_clock_verified=False,
                        weekday=None,
                        period=None,
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

            page = browser.new_page(viewport=dict(width=1100, height=780))
            page.add_init_script("localStorage.setItem('gen3.locale','en')")
            page.on("pageerror", lambda e: errors.append(str(e)))
            page.route("**/api", respond)
            page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
            page.get_by_role("button", name="ROM reference", exact=True).click()
            page.get_by_role("button", name="Game time", exact=True).click()
            panel = page.locator(".clock-panel")
            expect(panel).to_be_visible()
            if not mercury:
                expect(panel).to_contain_text(
                    "SAV fields alone cannot determine its current time"
                )
                panel.get_by_label("Simulated RTC date", exact=True).fill("2026-10-06")
                panel.get_by_label("Simulated RTC time", exact=True).fill("14:20:30")
                panel.get_by_label("Days", exact=True).fill("1")
                panel.get_by_label("Hours", exact=True).fill("1")
                panel.get_by_label("Minutes", exact=True).fill("40")
                panel.get_by_label("Seconds", exact=True).fill("50")
                panel.get_by_role(
                    "button", name="Calculate with this ROM", exact=True
                ).click()
                expect(panel.locator(".clock-projection")).to_contain_text("12:39:40")
                expect(panel.locator(".clock-projection")).to_contain_text(
                    "Simulated condition"
                )
                expect(panel.locator(".clock-projection")).to_contain_text(
                    "not verified current game time"
                )
                # Changing input discards an in-flight result, rather than relabeling it.
                hold[0] = True
                panel.get_by_role(
                    "button", name="Calculate with this ROM", exact=True
                ).click()
                page.wait_for_timeout(100)
                assert len(delayed) == 1
                panel.get_by_label("Hours", exact=True).fill("2")
                route, data = delayed.pop()
                route.fulfill(
                    content_type="application/json",
                    body=json.dumps(dict(ok=True, data=data)),
                )
                page.wait_for_timeout(100)
                expect(panel.locator(".clock-projection")).to_have_count(0)
            with page.expect_response(
                lambda r: r.request.method == "POST"
                and r.request.post_data_json.get("command") == "clock_query"
            ):
                with page.expect_file_chooser() as choice:
                    page.get_by_role(
                        "button", name="Open save", exact=True
                    ).first.click()
                choice.value.set_files(
                    dict(
                        name="synthetic.sav",
                        mimeType="application/octet-stream",
                        buffer=b"fixture-only",
                    )
                )
            if mercury:
                expect(panel).to_contain_text("Saved virtual clock")
                expect(panel).to_contain_text("Tuesday")
                expect(panel).to_contain_text("17:30:00")
                panel.get_by_label("Weekday", exact=True).select_option("3")
                expect(panel).to_contain_text("Simulated condition")
                expect(panel).to_contain_text("Wednesday")
                expect(panel).not_to_contain_text("17:30:00")
                panel.get_by_label("Query hour", exact=True).select_option("4")
                expect(panel).to_contain_text("Morning")
            else:
                panel.get_by_label("Offset source", exact=True).select_option("save")
                panel.get_by_role(
                    "button", name="Calculate with this ROM", exact=True
                ).click()
                expect(panel.locator(".clock-projection")).to_contain_text("12:39:40")
                with page.expect_response(
                    lambda r: r.request.method == "POST"
                    and r.request.post_data_json.get("command") == "clock_query"
                ):
                    with page.expect_file_chooser() as choice:
                        page.get_by_role(
                            "button", name="Open save", exact=True
                        ).first.click()
                    choice.value.set_files(
                        dict(
                            name="synthetic.sav",
                            mimeType="application/octet-stream",
                            buffer=b"fixture-only",
                        )
                    )
                expect(panel.locator(".clock-projection")).to_have_count(0)
                panel.get_by_role(
                    "button", name="Calculate with this ROM", exact=True
                ).click()
                expect(panel.locator(".clock-projection")).to_contain_text("11:39:40")
            page.get_by_role("button", name="简体中文", exact=True).click()
            expect(panel).to_contain_text("游戏时间与查询情景")
            expect(panel).to_contain_text(
                "明确输入 RTC 的模拟情景" if not mercury else "星期"
            )
            page.set_viewport_size(dict(width=720, height=740))
            expect(panel).to_be_visible()
            assert panel.evaluate("(e)=>e.scrollWidth <= e.clientWidth + 1")
            if key in ["BW", "MERCURY12"] and os.environ.get("GEN3_UI_ARTIFACTS"):
                directory = Path(os.environ["GEN3_UI_ARTIFACTS"])
                directory.mkdir(parents=True, exist_ok=True)
                panel.screenshot(path=str(directory / f"clock-{key}.png"))
            assert not any(
                r["command"] in ["action", "export_save", "save_bytes"]
                for r in requests
            )
            assert not errors, errors
            page.close()
            print(
                key,
                "bilingual native-clock scenario/SAV refresh/compact/read-only fixture passed",
                flush=True,
            )
        browser.close()


if __name__ == "__main__":
    main()
