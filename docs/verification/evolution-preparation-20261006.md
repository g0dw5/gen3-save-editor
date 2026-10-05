# Directed evolution preparation / 有向进化准备链

## Implemented workflow / 可用流程

Open ROM and SAV → Collection planning → missing Pokémon → evolution preparation
→ ancestor/reference source → item or move query / origin tile → exterior entrance
→ back. Standalone HTML includes the same ordered chain, source levels, slot
probability, time/resource overlays and entry names. ROM-only acquisition queries
retain item/move/species requirement cross-links; collection still requires a SAV.

五份精确指纹共用流程与数据模型。准备链只读，不产生个体、不改图鉴、剧情或 ROM。
页面与 HTML 展示起点、前代来源、格位、入口及每步进化条件；所需道具／招式可跳转。

## Evidence / 证据

`local_collection_preparation_all_fingerprints` uses the five current private ROMs
and **synthetic** SAVs with one healthy non-egg individual. It checks every proposed
step against `Rom::evolutions` for that exact loaded ROM, preserving the complete
record and offset, cross-links, chain direction, cycle absence, actual individual
counts and ancestor-source/entrance references. Complete ROM and SAV byte arrays
stay identical. These are table/reference consistency checks, not independent
execution of the native evolution eligibility routine.

| Exact ROM | Proposed chains | From actual individuals | Missing species goals |
|---|---:|---:|---:|
| BW | 189 | 2 | 410 |
| DP | 189 | 2 | 410 |
| Spanish Rocket | 459 | 2 | 1,335 |
| Ultimate Emerald | 473 | 2 | 1,143 |
| Mercury 1.2 | 287 | 2 | 1,318 |

Counts describe this synthetic query scenario, not all obtainable Pokémon or the
user's current missing collection. The opt-in test passes all **1,597** chains.
Existing `local_query_` tests also pass five ROM query/navigation checks and the
available Mercury SAV's read-only regional plan. The other four actual user SAVs
were not provided to that run; no real-file coverage is inferred from fixtures.

Public tests cover all five configured codecs with synthetic inputs: two-step
directed chains, compound item operands, current-individual versus historical Dex
separation, no reverse-evolution assumptions, unreferenced origins, cycles, egg
exclusion and whole-SAV preservation. Evolution cross-link tests distinguish item,
move and species operands from numeric levels, types, flags and unknown conditions.

The shared `held_day` / `held_night` decoded conditions previously did not match
acquisition's `held_item_day` / `held_item_night` checks. Cross-links are corrected
and centralized with gender-dependent, move-dependent and compound item operands.
The page does not reinterpret a level or map number as an item ID.

`scripts/test_query_collection_ui.py` verifies bilingual preparation text, item
query/back, origin tile/back, exterior entry, out-of-period indication, actual
ancestor-count display and escaped standalone HTML. The saved synthetic card was
visually inspected. No mutation, export-SAV or save-bytes request is sent. The
existing breeding and resource-query UI workflows also pass.

Final checks: **111 public core tests passed, 29 opt-in tests ignored**; the new
five-ROM opt-in chain test and existing two five-ROM query/navigation tests pass
separately. TypeScript/Vite production build, formatting, reference navigation,
five release-workflow tests and bilingual HTML/browser flows pass. Clippy adds no
new warning (six existing warnings remain outside this increment).

## Scope / 边界

- Only runtime **permanent** evolution edges are traversed. Battle forms are
  excluded by the existing evolution reader. Directed edges are never reversed
  to invent baby/parent or breeding rules.
- Missing goals can use historical Dex records; preparation can only count
  actual checksum-valid non-egg individuals. Eggs, past possession and a final
  stage cannot silently provide an earlier stage.
- One suggested chain is chosen: prefer an existing origin, then a referenced
  origin without a known failed source condition/time, then fewer evolution
  edges and stable IDs/offsets. This is not a cost, reachability or global
  shortest-route solver. The target's ordinary direct source remains visible.
- Search is cycle-safe and bounded to eight edges / 512 queued-and-examined
  paths. A selected result marks truncation if the bound is reached. No result
  does not prove that no chain exists outside the bound.
- Only ancestor sources with actual map references are proposed; a bare breeding
  candidate or unplaced table row is not an obtainable origin. Completed rewards
  without verified repeatability are not offered as a fresh ancestor source.
- Origin egg sources say to hatch first and retain unknown complete hatching
  conditions. Source receipt/access and dynamic map/script limitations remain
  those of the acquisition/navigation readers.
- Possession never promotes the final target to “currently available”. Current
  level, gender, friendship, beauty, move, held item, time, evolution location,
  trade restrictions and full native eligibility still require validation.
- Branches may consume the same actual individual. A joint minimum-individual,
  breeding and resource plan is not implemented; no single-individual completion
  guarantee is made.

准备链是准备建议，不能认证当前能进化、所有分支只需一个个体或地图当前可达。
完整孵蛋／资格执行、剧情依赖、分支数量与合并路线优化仍未完成；能力矩阵维持 P。

## Reproduction / 重现

```sh
cargo test -p gen3-core collection_preparation
cargo test -p gen3-core evolution_cross_links
# Supply five private GEN3_ROM_* paths; no real SAV is required or written.
cargo test -p gen3-core local_collection_preparation_all_fingerprints -- --ignored --nocapture
cargo test -p gen3-core local_query_ -- --ignored --nocapture
# Start Vite and provide Playwright / Chrome.
python scripts/test_query_collection_ui.py
```

Private logs/images are local verification inputs and never enter release assets.
No edited-SAV emulator round trip is asserted for this read-only increment. Keep
the current version; no additional package or public release is produced here.
