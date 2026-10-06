# Evolution requirements and collection locations / 进化条件与收集地点

The read-only reference viewer now closes species → required item/move →
referenced evolution → species, and decoded location → map → entrance → return.
Collection preparation and escaped standalone HTML use the same location facts.
These are table-backed references, **not proof of current evolution eligibility**.

## Exact-ROM scope / 精确 ROM 范围

| Input / 输入 | MD5 | Item references | Move references | Companion references | Distinct resources |
|---|---|---:|---:|---:|---:|
| Dark Phantom BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | 39 | 0 | 0 | 15 |
| Dark Phantom DP | `cb2940215f4dafb1bef133c3af379f44` | 39 | 0 | 0 | 15 |
| Spanish Rocket | `59c658a1081f542086de1060bb65f0b3` | 227 | 9 | 1 | 25 |
| Ultimate 5.5 | `17ce9785b33319b3dbda9a5d37c57ec1` | 95 | 13 | 1 | 48 |
| Mercury 1.2 | `f323df1792ac68462a34b42fe8571533` | 142 | 20 | 1 | 61 |

These counts record the reader before the subsequent native-selector correction;
BW/DP's alternate table was **not yet configured** in this revision. See
[the correction and native evidence](item-evolutions-20261006.md). The earlier
claim below that the BW/DP alternate table was already read is withdrawn.

Counts are decoded resource references, not distinct evolutions, obtainable
species or completed tasks. A compound rule can reference two items. Zero move/
companion references in this BW/DP scan do not prove absence of every custom
progression mechanic. Runtime rows stay bound to the opened ROM; no names, rules
catalog or images are bundled.

## Shared model and navigation / 共用模型与跳转

`acquisition.rs::evolution_targets` supplies both relation-edge links and the
new `AcquisitionReport.evolution_uses`. Resource use is kept separate from
`sources`: an evolution needing a stone does not supply that stone. The queried
resource is not repeated among its other requirements. Source and result species
remain independent links, including required companion/trade species. Battle
transformations are excluded by the existing native-format reader. Unsupported
methods and raw level/type/flag/map numbers do not become item or move IDs;
zero is not a required resource.

`EvolutionRuleDetails` is shared by the tree, acquisition queries, reverse uses,
collection tasks and preparation. It resolves current-ROM names and opens maps
through the existing guarded navigation/return flow. Optional serialized links
keep older synthetic/browser responses usable without guessing resources in UI.

The tree previously deduplicated by source, result, method and main parameter,
but ignored auxiliary and compound requirements. The key now retains different
held items and region/map/weather/time constraints; equivalent requirement order
still merges duplicated rows. Species cards remain unique by internal ID. Name
associations and battle forms do not become ordinary evolution edges.

## Location boundaries / 地点边界

Only positive decoded `region`/`map` requirements select map records. A decoded
region parameter and explicit extra conditions intersect; `outside_region`
restricts a positive condition but never lists every map on its own. Unresolved
raw map methods/parameters are not guessed into bank/number coordinates. Missing
matches do not imply impossibility. Matching map names and IDs are read from the
current world, not from a built-in route list.

The collection backend adds entrance suggestions for positive evolution-location
maps, in addition to capture/reward maps. A referenced map without any parsed
entrance receives no invented path. Shared UI/HTML show named maps, exterior
chains where parsed, and access/activation/dynamic-layout limits. No exact
interaction tile or current eligibility is inferred from a region/map tag.

The existing permanent-evolution decoders are not newly certified as complete
native behavior. This increment validates references and query/plan projection;
it does **not** execute every evolution selector, weather/RTC branch, species
exception, item consumption, animation or live map replacement. Full native
eligibility remains **P/U**. No new evolution or edited-SAV emulator round trip
is claimed.

地点关联只使用已解析的正向地区／地图条件；反向地区限制不独立展开全部地图。
未解析的参数不会猜成地图编号。收集建议会附带对应地图的入口记录，但没有入口
证据时不生成路径，也不承诺当前可以进入。HTML 同样显示这些限制。

## Evidence / 验证

- Public five-adapter binary fixtures exercise ordinary item, level, move,
  compound held-item, unknown and battle-form rows. They check API/tree resource
  links, reverse uses separate from acquisition, unchanged SAV/ROM bytes,
  conjunctive location filtering and full read-only collection entrance output.
- `local_query_acquisition_and_collection_all_profiles` reads all exact-ROM
  evolution rows, checks main operand/result bytes, all distinct resource reverse
  references, API projections and relation links. Existing wild-held/source/
  map/clock/planning checks remain included. This is a runtime-reader regression,
  **not independent native evolution execution**.
- `test_evolution_graph.mjs` covers same-target auxiliary differences, different
  location requirements, equivalent requirement ordering, unique cards, cycles,
  form families and immutable input.
- `test_evolution_links_ui.py` uses five synthetic adapters on one browser page,
  with real file-picker ROM switches. It checks item/move → reverse use → species,
  tree resources, map → exterior → return, EN/ZH, compact width and no old-ROM
  names. These fixtures test presentation; they are not game-play evidence.
- `test_query_collection_ui.py` checks preparation location names/entrances and
  EN/ZH standalone HTML, escaping and the existing native-Dex/source/prerequisite/
  navigation/refresh workflows. Public core suite: **137 passed, 54 opt-in ignored**.

All listed core/query and final browser checks passed. Formatting, production
TypeScript/Vite build, desktop compilation and five release-workflow tests also
passed. A browser toolbar click was initially blocked by the movable reference
window; the fixture now drags the live window away instead of force-clicking or
closing its state traces. The saved-state refresh assertions remain in place.

No ROM/SAV is added to Git or releases. Query and planning do not create Pokémon,
light the Dex or write story flags. Version stays held; this increment does not
build or publish another package.

```sh
cargo test -p gen3-core
# Provide all five private GEN3_ROM_* inputs; optional SAV fixtures remain private.
cargo test -p gen3-core local_query_acquisition_and_collection_all_profiles \
  -- --ignored --nocapture
node scripts/test_evolution_graph.mjs
# Start Vite; use an environment with Playwright and Chrome.
python3 scripts/test_evolution_links_ui.py
python3 scripts/test_query_collection_ui.py
```
