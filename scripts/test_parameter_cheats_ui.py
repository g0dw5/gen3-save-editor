"""Parameterized cheat UI regression; synthetic choices, no user's files."""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from test_cheats_ui import recipe, text, UE
from test_reference_navigation import CATALOG
import copy


def main():
    entries = [dict(recipe, id="specified-wild-encounter", parameters="encounter", title=text("指定野生宝可梦与等级", "Choose wild Pokémon and level")),
               dict(recipe, id="teleport-to-map", parameters="teleport", category="travel", title=text("传送到指定地图", "Teleport to a chosen map")),
               dict(recipe, id="shiny-wild-encounters", title=text("普通野生遭遇必定闪光", "Shiny ordinary wild encounters"))]
    options = dict(species=[dict(id=25, name="皮卡丘"), dict(id=400, name="胡说树")], maps=[
        dict(id="1-0", group=1, number=0, region=10, name="测试城", code="01 00", landings=[dict(id=0,x=3,y=4),dict(id=2,x=7,y=8)]),
        dict(id="1-1", group=1, number=1, region=10, name="测试城", code="01 01", landings=[]),
        dict(id="2-0", group=2, number=0, region=20, name="测试洞窟", code="02 00", landings=[dict(id=1,x=5,y=6)])])
    catalog = dict(rom=dict(md5=UE,label="Synthetic ROM",editor_supported=True),entries=entries,options=options)
    editor = copy.deepcopy(CATALOG)
    editor["profile"].update(id="ultimate-emerald-55", md5=UE, label="Synthetic ROM")
    requests, pending, errors = [], [], []
    mode = dict(delay=False)

    def result(payload):
        params = json.loads(json.dumps(payload.get("parameters"), sort_keys=True))
        value = params.get("level", params.get("warp_id", 0)) if params else 86
        lines = [f"00000000 {value:08X}"] * (86 if not params else 1)
        return dict(rom_md5=UE,cheat_id=payload["cheat_id"],format="gameshark_v1_v2",parameters=params,lines=lines,compact_lines=[s.replace(" ","") for s in lines])

    def respond(route):
        req = route.request.post_data_json
        requests.append(req)
        if req["command"] == "state": data=dict(catalog=editor,save=None)
        elif req["command"] == "cheats":
            assert req["payload"]["expected_rom_md5"] == UE
            data=catalog
        elif req["command"] == "cheat_code":
            if mode["delay"]:
                pending.append((route,result(req["payload"])))
                return
            data=result(req["payload"])
        else: raise AssertionError(req)
        route.fulfill(json=dict(ok=True,data=data))

    with sync_playwright() as p:
        browser=p.chromium.launch(channel="chrome",headless=True)
        page=browser.new_page(viewport=dict(width=1440,height=1000),permissions=["clipboard-read","clipboard-write"])
        page.on("pageerror",lambda e: errors.append(str(e)))
        page.route("**/api",respond)
        for locale in ("zh","en"):
            page.add_init_script(f"localStorage.setItem('gen3.locale','{locale}')")
            page.goto(os.environ.get("GEN3_UI_URL","http://127.0.0.1:5173"))
            page.get_by_role("button",name="金手指" if locale=="zh" else "Cheats",exact=True).click()
            dialog=page.get_by_role("dialog")
            expect(dialog).to_contain_text("Synthetic ROM")
            copy_button=dialog.locator(".cheats-code button").first
            expect(copy_button).to_be_disabled()
            combo=dialog.get_by_role("combobox")
            combo.fill("400")
            page.get_by_role("option",name="#400 胡说树").click()
            expect(copy_button).to_be_enabled()
            expect(dialog.locator("pre")).to_have_text("00000000 00000005")
            level=dialog.get_by_role("spinbutton")
            level.fill("")
            expect(copy_button).to_be_disabled()
            level.fill("101")
            expect(copy_button).to_be_disabled()
            # Delayed response must never copy a previous target/level.
            mode["delay"]=True
            level.fill("17")
            expect(copy_button).to_be_disabled()
            page.wait_for_timeout(80)
            level.fill("18")
            page.wait_for_timeout(80)
            assert len(pending)==2
            pending.pop(0)[0].fulfill(json=dict(ok=True,data=result(dict(cheat_id=entries[0]["id"],parameters=dict(kind="encounter",species=400,level=17)))))
            expect(copy_button).to_be_disabled()
            route,data=pending.pop();route.fulfill(json=dict(ok=True,data=data));mode["delay"]=False
            expect(dialog.locator("pre")).to_have_text("00000000 00000012")
            dialog.locator(".cheats-list button").filter(has_text=entries[1]["title"][locale]).click()
            expect(copy_button).to_be_disabled()
            region=dialog.get_by_role("combobox").nth(0)
            region.fill("测试城")
            page.get_by_role("option",name="测试城 · #10").click()
            maps=dialog.get_by_role("combobox").nth(1)
            maps.click()
            blocked=page.get_by_role("option").filter(has_text="01 01")
            expect(blocked).to_have_attribute("aria-disabled","true")
            page.get_by_role("option").filter(has_text="01 00").click()
            expect(copy_button).to_be_enabled()
            expect(dialog.locator(".cheats-map-code")).to_contain_text("01 00")
            landing=dialog.get_by_role("combobox").nth(2)
            landing.click();page.get_by_role("option",name="#2 · (7, 8)").click()
            expect(dialog.locator("pre")).to_have_text("00000000 00000002")
            with page.expect_download() as dl:
                dialog.locator(".cheats-code button").nth(1).click()
            body=Path(dl.value.path()).read_text()
            assert "1-0" in body and "01 00" in body and "warp 2 (7, 8)" in body
            # Region change invalidates the map, landing and generated code together.
            region.fill("测试洞窟");page.get_by_role("option",name="测试洞窟 · #20").click()
            expect(copy_button).to_be_disabled()
            maps.click();page.get_by_role("option").filter(has_text="02 00").click()
            expect(copy_button).to_be_enabled()
            if os.environ.get("GEN3_UI_SHOTS"):
                out=Path(os.environ["GEN3_UI_SHOTS"]);out.mkdir(parents=True,exist_ok=True)
                page.screenshot(path=str(out/f"parameter-map-{locale}.png"))
            dialog.locator(".cheats-list button").filter(has_text=entries[2]["title"][locale]).click()
            expect(dialog.locator("pre")).to_contain_text("00000000 00000056")
            expect(dialog.locator("pre")).to_have_css("max-height","280px")
            dialog.evaluate("e=>e.style.width='490px'")
            assert dialog.locator(".cheats-detail").evaluate("e=>e.scrollWidth<=e.clientWidth")
        assert not errors,errors
        assert not any(r["command"] in ("action","export_save","patch_rom") for r in requests)
        browser.close()
    print("parameter cheat UI: bilingual ROM-name search, bounded levels, stale responses, two-level maps/codes, disabled landings, export and narrow layout passed")


if __name__=="__main__":main()
