# Runtime data and evolution references / 实时数据与进化树

2026-09-19. Applies to all three fingerprinted profiles. No ROMs, saves, extracted
images or full wiki pages belong in release inputs.

## Data ownership audit / 数据来源审查

| Content | Source and retained configuration |
| --- | --- |
| Pokémon names, six stats, types, abilities, growth groups, dex mapping | Current ROM; profile table addresses/strides/counts |
| Moves, descriptions, power/category/PP, item/ability names and descriptions | Current ROM; text pointers and layout adapters |
| Evolutions, learning sources, encounters, trainer parties | Current ROM; parser rules and verified engine behavior |
| Maps, tiles/palettes, NPCs, trainer and Pokémon artwork | Current ROM; format boundaries and palette rules |
| Map items, NPC rewards, trainer locations | ROM events/scripts; evidence and unresolved branches remain visible |
| Fishing rod labels | Current ROM item names; profile-specific item IDs (BW/DP 262–264, Rocket 866–868); removed frontend's legacy ID fallback |
| Growth thresholds | Current ROM for all three profiles; BW/DP now use their table rather than formula-generated thresholds |
| Official comparison stats | Explicit external reference requested by the user; independent compact, attributed numeric dataset |

Retained constants are not all removable, and the application does not claim
zero ROM-specific configuration:

- `profile.rs` contains MD5s, addresses, format bounds/counts, semantic item IDs,
  save fields and supported capabilities. Map bank lengths are bounded profile
  metadata; they are not a list of map names or rendered maps.
- `script_actors` retains one verified BW/DP script-to-actor annotation;
  `map_groups` retains verified gym/league map IDs. Rocket does not inherit these
  annotations. They support search tags, not fabricated script evidence.
- `adapter.rs`, `pokemon.rs`, `world.rs` and `map_events.rs` encode binary field
  layouts, opcode widths and verified engine behavior (e.g. encounter slot
  weights and trainer generation). These rules cannot be replaced with a text
  lookup. Do not assume they apply to a new engine without native verification.
- `data/charmap.txt` is a 61,903-byte byte-to-Unicode mapping, not glyph artwork or
  a Pokémon/name database. ROM glyph pixels do not identify Unicode by themselves.
- UI translations include field labels and readable engine conditions. Nature
  and type names now come from the active ROM (corrected in 0.2.1; the original
  audit incorrectly treated these as UI enums). UI translations do not replace
  ROM content or add official-name aliases to editing searches.
- Test fixtures and assertions intentionally contain known values; they are not
  runtime fallbacks and are excluded from release binaries/assets.

No production ROM catalog, trainer-team snapshot, item-name list, extracted map
image, sprite bundle or save file was found. `include_str!` in the core loads only
the encoding map. Frontend catalog requests remain bound to the active ROM;
image cache keys include its MD5. The new official reference is 57,754 UTF-8 bytes
(about 21 KB gzip), covering 1,238 species/form rows and all 1,025 National Dex IDs.
No source HTML or images are shipped. See its [attribution](../../ui/data/README.md).

中文摘要：实际游戏内容仍从打开的 ROM 读取。保留的常量包括格式、偏移、边界、
编号语义、已验证的引擎规则和少量地图标签，不声称“零版本配置”。本次把漆黑
经验值阈值也改读 ROM，并去掉前端旧钓竿编号兜底。官方六围是用户明确要求的
独立对照资料，约 58 KB 原始 JSON；它不代替 ROM 图鉴，也不包含图片。

## BW/DP experience / 漆黑经验表

Both exact ROMs contain six growth rows at `0x31F72C`, each 101 `u32` values
(404 bytes). Native level routines load the pointer at `0x690B8`/`0x69124` and
use `growth * 404 + level * 4`. The first levels for medium-fast growth are
`0, 1, 8, 27, 64`. Level 1 is 1 in this table; the old generic formula gave 0.
Existing records with XP 0 are not rewritten on load. Level creation/editing now
uses the exact ROM table. Rocket continues to use its own level-150 table.

## Relation graph / 关系图

The shared core builds a connected component from incoming and outgoing ROM
evolution rows, battle transformations and form-family lists. Clicking any node
can return to previous stages or visit sibling branches. Browsing does not
perform devolution or mutate a Pokémon. The UI is shared across BW, DP and Rocket.

Rocket form-family pointer table: `0x617B00`, one `u32` pointer per species.
Lists contain `u16` species IDs terminated by `0xFFFF`. Native routine `0x9D19C`
returns a member by index (or the input species for a null pointer); `0x9D1C8`
searches the same list. The bounded reader rejects invalid IDs or missing
terminators. This table is separate from persistent item/move form transitions.

Water starter evidence in Rocket MD5 `59c658a1081f542086de1060bb65f0b3`:

| ID | Evidence | Display |
| --- | --- | --- |
| 7 → 8 → 9 | ROM ordinary evolution rows: levels 16 and 36 | Ordinary evolution |
| 9 → 980 | Evolution table special method `0xFFFF`, held item 297; shared form-family pointer `0x861766A`, list `[9, 980, 0xFFFF]` | Mega battle transformation |
| 899 水箭龟 Z | No ordinary incoming/outgoing evolution, null form-family/storage-form pointer; used by trainer records 1579/1583 | Name association, conversion method unverified |

The [experiment guide](https://teamrocketedition.com/pokemon-created-by-rocket-experiments/)
also describes Blastoise-Z as a Rocket experiment with the matching six stats.
That is corroboration, not proof of an in-game acquisition method in this exact
Chinese ROM. The parser does not claim it is obtainable or impossible to obtain.

Generic ROM-name suffix hints provide navigation for entries such as 899, with a
separate dotted section and explicit uncertainty. They are not evolution edges,
are not used by `ancestors()`/origin validation, and never permit ordinary creation
or collection completion merely because names look related. Neither the ID 899
nor a hardcoded “Blastoise-Z family” list is needed by the runtime graph.

980 是已证实的 Mega 水箭龟；899 的名称是水箭龟 Z，并没有在上述进化／形态表中
与 9 连接。它现在可通过“名称关联”反复跳转，但不会伪造进化条件或扩大合法来源。

## Official comparisons / 官方对照

The six rows display HP, Attack, Defense, Sp. Atk, Sp. Def and Speed, with totals
below. Official values are on the left; current ROM values on the right. A
latest-generation reference is filled from the most recent older list only when
absent. Every choice links to a source revision and shows its generation.

Ordinary entries match by exact normalized name. Verified Mega/primal edges can
select a unique official battle form; ambiguous X/Y branches and other uncertain
forms require manual selection. National Dex numbers alone never establish
identity: Rocket reuses slot 899 (officially Wyrdeer), and BW/DP also repurpose
slots. Old translations may therefore need a manual reference. Choices persist
per ROM MD5/species in local preferences and do not change the ROM or save.

## Verification / 验证

- Synthetic cross-profile tests change table bytes and verify that evolution
  graphs and XP reads change immediately; name hints never become ancestry.
- Private Rust regressions check BW/DP incoming starter evolutions and Rocket
  9/899/980 graphs, alongside existing read/write, transfer and checksum tests.
- `verify_form_families.py`: 3,285 native member comparisons across 451 species,
  143 unique families, and 943 null-pointer identity controls; no save writes.
- `test_evolution_reference_ui.py`: BW/Rocket/DP navigation, all six stat rows and
  totals, official Mega distinction, reused-dex rejection, manual-reference
  persistence and read-only API calls. Existing browser regressions also run.
- `test_official_stats.py`: full dex coverage, unique keys, numeric bounds, base
  versus Mega values, and revision/hash provenance. The extraction script checks
  each wiki row's six-value sum against the displayed total.

No full battle simulation or acquisition proof for experimental species is
implied by these tests. Windows packages are cross-compiled and payload-checked;
that does not establish Windows runtime behavior.
