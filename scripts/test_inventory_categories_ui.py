"""Real-ROM catalogs/SAV inventory in a read-only browser fixture.

GEN3_INVENTORY_UI_SCENARIO supplies private verifier cases; production CLI reads
catalogs and saves. All browser API calls are intercepted, including mutations.
The mismatched-slot case alters only an HTTP fixture; originals remain unchanged.
"""

import copy
import hashlib
import json
import os
import re
import subprocess
from pathlib import Path
from playwright.sync_api import sync_playwright, expect


def main():
    expect.set_options(timeout=60000)
    cases = json.loads(Path(os.environ["GEN3_INVENTORY_UI_SCENARIO"]).read_text())
    cli = os.environ["GEN3_CLI"]
    url = os.environ.get("GEN3_UI_URL", "http://127.0.0.1:5173")
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(channel="chrome", headless=True)
        for key, case in cases.items():
            paths = [Path(case["rom"]), Path(case["save"])]
            hashes = [hashlib.sha256(p.read_bytes()).hexdigest() for p in paths]
            catalog = json.loads(subprocess.check_output([cli, "catalog", case["rom"]]))
            original = json.loads(
                subprocess.check_output([cli, "inspect", case["rom"], case["save"]])
            )
            snapshot = copy.deepcopy(original)
            page = browser.new_page(viewport={"width": 1100, "height": 800})
            page.add_init_script("localStorage.setItem('gen3.locale','en')")
            errors, requests = [], []
            species_cache = {}
            page.on("pageerror", lambda e: errors.append(str(e)))

            def route(api):
                request = api.request.post_data_json
                requests.append(request)
                if request["command"] == "state":
                    data = dict(catalog=catalog, save=snapshot)
                elif request["command"] == "species":
                    species_id = request["payload"]["id"]
                    if species_id not in species_cache:
                        species_cache[species_id] = json.loads(
                            subprocess.check_output(
                                [cli, "species", case["rom"], str(species_id)]
                            )
                        )
                    data = species_cache[species_id]
                elif request["command"] == "sprite":
                    data = dict(
                        url='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="8" height="8" fill="blue"/></svg>'
                    )
                else:
                    raise AssertionError(request)
                api.fulfill(json=dict(ok=True, data=data))

            page.route("**/api", route)
            page.goto(url)
            page.get_by_role("button", name="Items", exact=True).click()
            editor = page.locator(".bag-layout aside")
            choices = editor.get_by_role("combobox", name="Items", exact=True)
            for pocket, label in [
                ("berries", "Berries"),
                ("key_items", "Key items"),
                ("pc", "PC items"),
            ]:
                page.get_by_role("button", name=label, exact=True).click()
                category = next(
                    p["category"]
                    for p in catalog["profile"]["save"]["pockets"]
                    if p["id"] == pocket
                )
                selected = next(
                    e["item"]
                    for e in snapshot["bag"]
                    if e["pocket"] == pocket and e["slot"] == 0
                )
                choices.click()
                expected = [
                    i
                    for i in catalog["items"]
                    if not i["id"]
                    or i["id"] == selected
                    or category == 0
                    or i["pocket"] == category
                ]
                expect(page.get_by_role("option")).to_have_count(len(expected))
                choices.press("Escape")
                expect(
                    editor.get_by_role("button", name="Apply changes", exact=True)
                ).to_be_disabled()
            # Free editing deliberately retains the complete current-ROM catalog.
            page.get_by_role("button", name="Berries", exact=True).click()
            page.get_by_label("Free editing", exact=True).check()
            choices.click()
            expect(page.get_by_role("option")).to_have_count(len(catalog["items"]))
            choices.press("Escape")
            page.get_by_label("Free editing", exact=True).uncheck()
            # Preserve a pre-existing wrong-category item; never normalize the source.
            bad = next(
                i
                for i in catalog["items"]
                if i["id"]
                and i["pocket"]
                == next(
                    p["category"]
                    for p in catalog["profile"]["save"]["pockets"]
                    if p["id"] == "key_items"
                )
            )
            entry = next(
                e
                for e in snapshot["bag"]
                if e["pocket"] == "berries" and e["slot"] == 0
            )
            entry.update(item=bad["id"], quantity=1)
            page.reload()
            page.get_by_role("button", name="Items", exact=True).click()
            page.get_by_role("button", name="Berries", exact=True).click()
            expect(editor).to_contain_text(
                "This item's ROM category does not match this pocket"
            )
            expect(choices).to_have_value(re.compile(re.escape(bad["name"])))
            expect(
                editor.get_by_role("button", name="Apply changes", exact=True)
            ).to_be_disabled()
            page.get_by_role("button", name="简体中文", exact=True).click()
            expect(editor).to_contain_text("分类与当前栏位不符")
            page.get_by_role("button", name="English", exact=True).click()
            assert not errors, errors
            assert not any(
                r["command"] in ["action", "export_save", "save_bytes"]
                for r in requests
            ), requests
            assert hashes == [hashlib.sha256(p.read_bytes()).hexdigest() for p in paths]
            page.close()
            print(
                key,
                "real category choices, free editing and bilingual mismatch preservation passed",
                flush=True,
            )
        browser.close()


if __name__ == "__main__":
    main()
