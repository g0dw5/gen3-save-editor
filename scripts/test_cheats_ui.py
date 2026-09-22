"""Cheat window smoke/regression tests with public synthetic API fixtures.

Run against Vite using GEN3_UI_URL; never opens a user's ROM or save.
"""
import copy
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_reference_navigation import CATALOG, WORLD, species

UE = "17ce9785b33319b3dbda9a5d37c57ec1"
LINES = ["270AABF9 EF4D3B91", "05EFAF30 6F13BEC4"]
text = lambda zh, en: {"zh": zh, "en": en}
recipe = dict(id="disable-input-peeking", title=text("关闭 AI 窥屏 · 全模式", "Disable AI input peeking · all modes"),
              summary=text("测试说明", "Fixture summary"), scope=text("两条一起启用", "Enable both lines"),
              steps=[text("战斗外保存", "Save outside battle")], limitations=[text("手机未实测", "Mobile untested")],
              verification=[text("测试范围", "Test scope")], formats=["gameshark_v1_v2"])
cheats = dict(rom=dict(md5=UE, label="Ultimate Emerald 5.5", editor_supported=False), entries=[recipe])
code = dict(rom_md5=UE, cheat_id=recipe["id"], format="gameshark_v1_v2", lines=LINES,
            compact_lines=[s.replace(" ", "") for s in LINES])


def main():
    requests, errors = [], []
    editor = copy.deepcopy(CATALOG)
    editor["profile"]["md5"] = "editor-fixture"
    mode = {"editor": False, "invalid": False, "delay": False}
    pending = []

    def respond(route):
        req = route.request.post_data_json
        requests.append(req)
        command, payload = req["command"], req["payload"]
        if command == "state":
            data = {"catalog": editor if mode["editor"] else None, "save": None}
        elif command == "species":
            data = {"species": species(payload["id"]), "evolutions": [], "learnset": [], "encounters": []}
        elif command == "sprite":
            data = {"url": ""}
        elif command == "world":
            data = WORLD
        elif command == "cheats":
            assert payload["expected_rom_md5"] == editor["profile"]["md5"]
            data = {"rom": {"md5": editor["profile"]["md5"], "label": "Editor ROM", "editor_supported": True}, "entries": []}
        elif command == "open_cheat_rom":
            if mode["invalid"]:
                route.fulfill(json={"ok": False, "error": {"code": "unsupported_rom", "detail": "fixture"}})
                return
            data = cheats
        elif command == "cheat_code":
            assert payload == dict(expected_rom_md5=UE, cheat_id=recipe["id"], format="gameshark_v1_v2")
            if mode["delay"]:
                pending.append(route)
                return
            data = code
        else:
            raise AssertionError(req)
        route.fulfill(json={"ok": True, "data": data})

    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True)
        context = browser.new_context(viewport={"width": 1440, "height": 1000}, permissions=["clipboard-read", "clipboard-write"])
        page = context.new_page()
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.route("**/api", respond)
        for locale in ("zh", "en"):
            page.add_init_script(f"localStorage.setItem('gen3.locale','{locale}')")
            page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
            title = "金手指" if locale == "zh" else "Cheats"
            page.get_by_role("button", name=title, exact=True).click()
            dialog = page.get_by_role("dialog", name=title, exact=True)
            open_name = "选择金手指 ROM" if locale == "zh" else "Choose cheat ROM"

            def load():
                with page.expect_file_chooser() as fc:
                    dialog.get_by_role("button", name=open_name).click()
                fc.value.set_files({"name": "fixture.gba", "mimeType": "application/octet-stream", "buffer": b"fixture"})

            load()
            expect(dialog.locator("pre")).to_have_text("\n".join(LINES))
            expect(dialog).to_contain_text(UE)
            expect(dialog).to_contain_text("仅支持金手指" if locale == "zh" else "cheat-only support")
            copy_button = dialog.get_by_role("button", name="复制整组代码" if locale == "zh" else "Copy both lines")
            copy_button.click()
            assert page.evaluate("navigator.clipboard.readText()") == "\n".join(LINES)
            dialog.get_by_role("checkbox").check()
            expect(dialog.locator("pre")).to_have_text("\n".join(code["compact_lines"]))
            with page.expect_download() as dl:
                dialog.get_by_role("button", name="导出代码与说明" if locale == "zh" else "Export codes and guide").click()
            body = open(dl.value.path()).read()
            assert UE in body and "GameShark Advance V1/V2" in body
            assert code["compact_lines"][0] in body
            dialog.locator("summary").filter(has_text="模拟器验证" if locale == "zh" else "Emulator verification").click()
            expect(dialog).to_contain_text("测试范围" if locale == "zh" else "Test scope")
            search = dialog.get_by_role("textbox")
            search.fill("no-such-cheat")
            expect(dialog.locator(".cheats-list button")).to_have_count(0)
            search.fill("")
            expect(dialog.locator(".cheats-list button")).to_have_count(1)
            # Resize the actual floating panel, exercising its container query.
            dialog.evaluate("e => e.style.width = '490px'")
            assert dialog.locator(".cheats-detail").evaluate("e => e.scrollWidth <= e.clientWidth")
            mode["invalid"] = True
            load()
            expect(dialog.get_by_role("alert")).to_be_visible()
            expect(dialog.locator("pre")).to_have_count(0)
            mode["invalid"] = False
            # A code response arriving after closure must not re-open or repopulate the next window.
            mode["delay"] = True
            load()
            expect(dialog.locator(".cheats-code button").first).to_be_disabled()
            page.wait_for_timeout(100)
            assert pending
            dialog.get_by_role("button", name="关闭" if locale == "zh" else "Close", exact=True).click()
            pending.pop().fulfill(json={"ok": True, "data": code})
            mode["delay"] = False
            page.get_by_role("button", name=title, exact=True).click()
            expect(page.get_by_role("dialog", name=title).locator("pre")).to_have_count(0)
            page.get_by_role("dialog", name=title).get_by_role("button", name="关闭" if locale == "zh" else "Close", exact=True).click()
        # Existing editor ROM's empty state and reference-window entry.
        mode["editor"] = True
        page.goto(os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173"))
        page.get_by_role("button", name="ROM reference", exact=True).click()
        page.get_by_role("dialog").get_by_role("button", name="Cheats", exact=True).click()
        expect(page.get_by_role("dialog", name="Cheats", exact=True)).to_contain_text("No verified cheats for this ROM yet")
        assert not errors, errors
        assert not any(r["command"] in ("action", "open_save", "open_rom", "export_save", "patch_rom") for r in requests)
        browser.close()
    print("cheat UI: bilingual copy/export, import failures, stale responses, narrow panel and editor isolation passed")


if __name__ == "__main__":
    main()
