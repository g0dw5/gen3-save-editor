# Referenced fixed-encounter items / 定点携带物反查

Item queries now include the explicit held-item inputs of referenced ordinary
fixed-encounter commands, and link item → Pokémon → event tile → shared entrance
navigation. Collection suggestions and standalone HTML use the same sources.
The native setup field is verified; obtaining the item is not implied.

## Actual ROM evidence / 实际 ROM 证据

| Exact input / 精确版本 | Parsed fixed member rows | Unique setup inputs | Nonzero held-source rows | Complete native calls |
|---|---:|---:|---:|---:|
| Dark Phantom BW / `0d9b129f7dd76895f79bb47ad7dec2fe` | 58 | 45 | 0 | 90 |
| Dark Phantom DP / `cb2940215f4dafb1bef133c3af379f44` | 58 | 45 | 0 | 90 |
| Spanish Rocket / `59c658a1081f542086de1060bb65f0b3` | 28 | 15 | 0 | 30 |
| Ultimate / `17ce9785b33319b3dbda9a5d37c57ec1` | 1,199 | 266 | 186 | 532 |
| Mercury 1.2 / `f323df1792ac68462a34b42fe8571533` | 115 | 78 | 18 | 156 |

Rows may repeat a command across map roots or branch guards; they are **not
unique items, obtainable Pokémon or guaranteed physical events**. Unique inputs
use (command offset, species, level, held item, member). The nonzero held inputs
have 62 such unique records in Ultimate and 11 in Mercury. Zero rows in BW/DP/
Rocket mean no nonzero held input in this bounded ordinary-command scan, not
absence of every fixed encounter, held item or other native mechanism.

`scripts/verify_referenced_static_items.py` reads the CLI's current ROM world at
runtime, then executes **898 complete native `0xB6` calls** from the original
ROM operand addresses: two seeds for all 449 unique parsed inputs. There are
zero skipped/unresolved input records in this scan. Native species, level and
held item match every known parsed input. Script-pointer consumption is checked;
whole player-party bytes are unchanged and native getters preserve all 600
opponent bytes. Exact MD5 and original SHA-256 are checked; ROM bytes are also
compared after execution. No SAV is opened or written, no ROM bytes patched,
no functions/instructions replaced.

Mercury resolved species variables are **explicit isolated scenarios** populated
through native setvar. This reproduces the recorded command inputs, not the
whole preceding script, current player/story state or the event's activation.
The 20-million native probe limit is the one documented in
[fixed-member verification](static-battles-20261006.md); default fixtures retain
their original budget. Complete setup/getter calls do not replay full battle
startup, capture, theft rules, post-start overrides or current access.

## Shared queries and presentation / 共用查询与展示

Previously the reverse item index inspected NPC trades but omitted ordinary
fixed-encounter held fields, despite displaying those Pokémon sources. A
`static_held` source now retains the same map coordinates, script input, guards,
level and related species. NPC trade semantics stay separate. Source quantity
is one item on that recorded member; it is not a proven collection quantity.

The UI says “Explicit setup item / 生成时指定携带”, with the current ROM's item
name and an explicit capture/taking-move/access boundary. It shows no invented
encounter-slot probability, random-held probability or generic 100% acquisition
chance. Unknown guards remain unresolved. A failed verified prerequisite may be
shown blocked, but neither satisfied guards nor a current bag/individual proves
receipt. These sources retain partial status, no receipt flag/evidence and
unknown repeatability.

Single fixed members on the map gain the item link and the same explanation;
paired members retain the shared pair panel. Species, item, map and return flows
remain in the existing read-only ROM popup. Chinese/English labels and HTML use
shared helpers; no names, encounter catalogs or extracted art are bundled.

Collection now includes these item IDs when **including unconfirmed sources**.
With that option off they are excluded even when a saved prerequisite is false:
a partially understood source is not promoted by having one known condition.
No Pokémon is generated, no Dex/story bit written, and no source save edited.
The suggestions are not a globally shortest route or proof of availability.

## Regression evidence / 回归证据

Public core suite: **136 passed, 54 opt-in ignored**. The five exact-ROM
acquisition/collection regressions and native parity are run separately from
these synthetic tests.

- `local_referenced_static_held_items_match_native_setup_and_collection` checks
  all native vectors against production input reads and all **204** real held
  source rows against reverse item queries, coordinates, companion/item links,
  quantity, receipt/probability boundaries and original ROM bytes. Distinct NPC
  tiles that reuse the same map script remain separate query occurrences.
- Public five-adapter fixtures verify item/species/tile round trips, explicit
  fields, unchanged SAV bytes, and inclusion/exclusion of these planning sources.
- Bilingual browser fixtures cover item → species → map → item → return, single
  and paired map members, compact rendering, and absence of invented percentage
  labels. Shared collection fixtures check English/Chinese standalone HTML with
  fixed-held sources and escaped ROM text. These are synthetic UI responses,
  separate from the actual native/reference checks.

Acquisition, collection, story conditions and current map access remain **P**
across all profiles. Native setup fields are verified only within the stated
boundary. Gifts and custom/special encounters do not inherit this evidence.
Version remains held; there is no public release. Local test packaging is only
performed when explicitly requested.

## Reproduce / 复现

Provide private five-ROM `GEN3_ROM_*` inputs and the read-only mGBA ARM7 probe
compiled with `-DGEN3_NATIVE_MAX_STEPS=20000000`. See the native probe build
instructions linked above.

```sh
python3 scripts/verify_referenced_static_items.py --gen3 /path/gen3 \
  --mgba-probe /private/native.dylib --output /private/static-items.json
GEN3_REFERENCED_STATIC_PROBES=/private/static-items.json cargo test -p gen3-core \
  local_referenced_static_held_items_match_native_setup_and_collection \
  -- --ignored --nocapture
```

Run Vite and `scripts/test_static_battle_ui.py` / `test_query_collection_ui.py`
with Playwright/Chrome. Private vectors and assets are evidence, never release
content or runtime lookup catalogs.
