# Native item-evolution references / 原生道具进化引用

Two reader gaps were found by executing the complete original item-evolution
selectors. The production viewer remains read-only and reads the opened ROM;
native calls below run only in disposable local test RAM.

## Exact inputs and bounded evidence / 精确输入与验证范围

| Game / 游戏 | MD5 | Species with ordinary item rows | Selector calls | Extra gender/selector pairs |
|---|---|---:|---:|---:|
| Dark Phantom BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | 26 | 240 | 0 |
| Dark Phantom DP | `cb2940215f4dafb1bef133c3af379f44` | 26 | 240 | 0 |
| Spanish Rocket | `59c658a1081f542086de1060bb65f0b3` | 199 | 1,680 | 0 |
| Ultimate 5.5 | `17ce9785b33319b3dbda9a5d37c57ec1` | 43 | 376 | 0 |
| Mercury 1.2 | `f323df1792ac68462a34b42fe8571533` | 92 | 808 | 1,024 |

The baseline scans the current tables, including configured alternate rows.
For each species with method 7, it tests every distinct nonzero item parameter
and item 0, PIDs 42/255 and selector modes 2/3. Inputs are synthetic level-50
party records with friendship 220, held item 0 and zero map/weather/party RAM.
Native growth-table experience thresholds are used. It is not a menu interaction
or a full gameplay state. Table references are not proof of obtainability.

There are **4,368 complete selector executions**, plus **1,024 complete native
gender getter executions**. No routine is skipped, patched or replaced. A
20-million-instruction mGBA ARM7 fixture budget is used. Source fingerprints and
ROM memory are checked unchanged; no user SAV is opened. Ultimate's selector
changes byte 30 of some synthetic inputs; this is recorded, not applied to a
stored Pokémon or claimed as a completed evolution.

## BW/DP alternate rows / 漆黑扩展表

The native entry is `0x6D098`. Its item-loop hook at `0x6D3A4` enters
`0x1196250`. For species 133, the hook uses a biased table pointer at `0x119626C`
and permits row indices through 6. Applying the native species stride gives
`0x1196300`, seven rows of eight bytes. The first five rows match the ordinary
table in both exact inputs; the last two are additional item routes.

Previously the BW profile configured no alternate table, so both BW and the
inheriting DP adapter omitted those last two routes. The existing shared
`evolution_overrides` reader now uses all seven current-ROM rows. Only layout
configuration is added: no item names, target catalog or extracted rows are
bundled. Ultimate retains its distinct ten-row table; Rocket/Mercury have no
new table override.

All 240 observations per BW/DP input return the corresponding first matching
item row or zero, and preserve the synthetic individual bytes. The two added
rows are checked through item reverse uses, target acquisition and connected
relations. Collection uses this same index; its general cross-ROM workflow is
tested separately. The prior statement that BW/DP's alternate table was already
read in [the resource-link record](evolution-links-20261006.md) is withdrawn.

## Mercury gender gate / 水银性别限制

The original `0x42EC4` entry redirects to `0x1D286A4`. In the item branch,
`0x1D2870E` compares the used item against an immediate ROM operand. A matching
item also compares the row's auxiliary halfword with the native gender result.
The native call at `0x1D28C14` loads its gender getter through `0x1D28C80`.
Gender codes are 0 male, 254 female and 255 genderless.

Ordinary method 7 alone therefore does not express the full condition. The
adapter records the verified comparison-instruction address; the reader checks
its opcode and obtains the item operand from the current ROM. Matching item
rows carry an additional typed gender requirement read from that row. Unknown
codes remain labelled unresolved. Neither species names nor an official item
catalog determine this condition.

Both affected ordinary source rows are tested across every PID low byte
0–255, in both modes: **1,024 selector/gender pairs**. The target is returned
only when the native gender matches the row. This explains four zero results
among the baseline's nonzero-item scenarios, rather than inventing missing
routes or assuming an unconditional stone evolution. Other contextual methods,
held-item interactions and complete native eligibility remain partial.

The shared human-readable evolution label displays the requirement in English
and Chinese in the tree, acquisition, reverse uses and collection preparation.
Standalone HTML uses the same label and escapes it. Existing-individual counts
in preparation describe possession, not a promise that each counted individual
can execute every step now; the preparation remains explicitly partial.

## Verification and limits / 验证与边界

- Public byte-mutating fixtures cover seven-row runtime reads, neighboring
  species, isolated alternate layouts, item operand changes, auxiliary gender
  changes and rejecting an unexpected comparison opcode. Other profiles do not
  inherit Mercury's requirement.
- `local_item_evolutions_match_native_selectors_and_reference_closure` compares
  every baseline scenario with the actual reader, checks native-selected targets
  are represented and validates all additional Mercury gender cases. Native
  outputs, not interface availability, are the comparison source.
- `local_query_acquisition_and_collection_all_profiles` regresses all five
  exact fingerprints, sources, preparation, maps, clocks and read-only SAV
  planning. The query overlay does not rewrite ROM facts or SAV progress.
- Five synthetic adapters on one browser page cover ROM switches, resource
  reverse uses, tree/back, map/entrance/back, compact layout and bilingual gender
  labels. Shared planner/HTML fixtures check both language exports and escaping.

The BW/DP resource-reference counts are now 41 item references and 15 distinct
resources per input (previously 39/15). These are not unique obtainable species
or proof of completed evolution. Other three adapters retain their prior
resource-reference counts.

Public core suite: **139 passed, 55 opt-in ignored**. Both native parity and
cross-ROM query tests, bilingual browser/HTML workflows, tree graph checks,
formatting, production build, desktop compilation and five release-workflow
tests passed. The first HTML assertion checked Chinese text in an English
export; the final test checks each actual selected language separately.

Full evolution access, item consumption, animations, friendship/time/weather/
location selection, all custom methods and post-evolution individual bytes are
**not certified by these observations**. No new edited-SAV game-save/reboot
claim is made; previous evidence retains its own bounded scope. Evolution and
collection remain **P**, with verified subfeatures described above.

The initial parity coverage check caught a verifier mistake: Mercury's 1,435
valid species count was treated as its upper internal-ID bound. Its table has
1,554 slots. The final verifier scans that actual bound and checks its scenario
set against the reader. An initial gender probe also used an incorrect adjacent
literal; the final getter is read from the actual PC-relative target. Failed
diagnostics are not verification evidence.

只验证道具判定及这些情景的性别限制，没有宣称完整进化过程已复刻。原始 ROM 与
用户存档均未修改；版本保持当前未发布版本，本轮不另行打包或发布。

```sh
# Provide all five private GEN3_ROM_* inputs. Output stays in ignored local data.
python3 scripts/verify_item_evolutions.py --mgba-probe /private/native-probe.dylib \
  --output /private/item-evolutions.json
GEN3_ITEM_EVOLUTION_PROBES=/private/item-evolutions.json \
  cargo test -p gen3-core local_item_evolutions_match_native_selectors_and_reference_closure \
    -- --ignored --nocapture
cargo test -p gen3-core local_query_acquisition_and_collection_all_profiles \
  -- --ignored --nocapture
cargo test -p gen3-core
node scripts/test_evolution_graph.mjs
# Start Vite; Playwright and Chrome are required for presentation fixtures.
python3 scripts/test_evolution_links_ui.py
python3 scripts/test_query_collection_ui.py
```
