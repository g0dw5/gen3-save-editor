# Multi-ROM adapters / 多 ROM 适配

Updated in place for **0.2.0**, 2026-09-19. Dark Phantom BW/DP and Team Rocket
2.1 Chinese use the same editing workspace. ROM reference windows are read-only;
there is no separate Rocket inspection-only Pokémon page.

Ultimate Emerald 5.5 joins the same workspace in 0.3.0. Its independent
[adapter record](research/ultimate-emerald-55.md) documents exact-ROM evidence
and remaining dynamic-gameplay limits.

## Coverage / 覆盖范围

| Feature | Dark Phantom BW / DP | Team Rocket 2.1 Chinese | Ultimate Emerald 5.5 |
| --- | --- | --- | --- |
| Exact MD5 and length check | Required | Required | Required |
| Pokémon, party, all boxes, drag/copy/swap, create/import/export | Read/write | Read/write | Read/write |
| Player, money, coins, box names, inventory, Pokédex | Read/write | Read/write | Read/write |
| Undo/redo, changes, atomic export, conflict detection | Shared | Shared | Shared |
| Inventory | PC + five bag pockets | PC + eight bag pockets | PC + five expanded bag pockets in sector tails |
| Level / experience | 1–100, ROM table | 1–150, ROM table | 1–100, ROM table |
| Nature / abilities | PID nature, two slots | Optional nature override, three slots | Mint override, hidden ability, Hyper Training |
| Level-up / machines / tutors / egg moves | Parsed | Parsed | Parsed, including separate Eevee evolution table |
| Maps, encounters, trainers, item/NPC layers | Parsed | Parsed: 1,363 maps, 2,558 trainers | Parsed: 922 map headers, 1,336 trainer records; some ROM graphics/event pointers are invalid |
| Individual artwork | Unown, Spinda, shiny | Also female artwork/palettes | Native front artwork; one invalid ROM sprite pointer |
| Hidden Power | IV type; power 30–70 | IV type; power 60 | IV type; power 60 |
| Mega / primal | Not enabled | Source/target/trigger references; separate from evolution | Permanent and battle transitions separated |
| Feebas seeded fishing tiles | Verified native rule | No rule enabled without evidence | Verified native rule |
| Z / Dynamax / Tera save fields | None | No invented persistent fields or ordinary Z move slots | No invented persistent battle-form fields |

中文：所有常规编辑操作共用同一套 UI、事务和导出流程。版本差异由字段格式、
表配置及明确的游戏规则提供；没有遗留临时只读页面。地图与训练家资料、图片、
相遇及教学表均来自当前 ROM，不打包游戏资源。仅支持 README 中列出的精确 MD5。

## Architecture / 组合方式

1. Identity and table locations: MD5, size, offsets, bounds and strides.
2. Independent ROM formats: species, moves, learnsets, trainers, event opcodes
   and object graphics IDs. A future hybrid can reuse individual readers.
3. Pokémon codec: shared format selection for native encryption/permutation or
   Ultimate's plain records, with bit ownership
   for ball, PP bonuses, experience, nature, ribbons, ability and header flags.
4. Save layout: sector payloads, party/storage, logical multi-sector inventory,
   pocket categories and encrypted quantities, dex block and mirrors.
5. Game rules: level/experience, appearance, Hidden Power, persistent item/move
   forms and temporary battle transformations. Changes trigger only the relevant
   persistent form rule; unrelated edits never normalize the species silently.
6. Capability checks remain at public read/write boundaries for future adapters.
   They do not select a second Pokémon editor. Free editing cannot bypass binary
   widths, checksums, linked-mail restrictions or cross-ROM import identity.

中文：相同格式可以只配置偏移量；不同位布局、指令格式或战斗生命周期需要独立
适配组件。不能把究极绿宝石简单继承成“西火同款”。切换 ROM 清除旧草稿与资料，
图片缓存及异步响应按 ROM 身份隔离。

## Limits / 资料边界

A parsed source is not proof of current story accessibility. Trainer custom
parties can generate random genders and either ordinary ability: the UI shows
these choices instead of pretending there is a fixed individual. Machine/tutor
compatibility does not establish which NPC currently offers a move. Two alternate
species have null compatibility pointers; this is reported without inventing
inherited compatibility. Remaining unresolved scripts retain offsets; native
special routines are not treated as fully interpreted event code.

中文：相遇、道具标记和训练家引用表示 ROM 中存在该来源，不承诺当前剧情可达。
未解析的脚本、原生特殊函数及空教学表会明确保留诊断，不把“未找到”解释成“不可能”。
Mega／Z 的使用条件仍受战斗状态影响，不能把预览关系当成必定可发动；没有擅自增加
极巨化、太晶化或战斗使用次数的存档字段。

## Regression contract / 回归约束

Generated fixtures run without copyrighted data: all 24 PID permutations,
bit ownership, ribbon/ability separation, every inventory slot, all 420 box slots
across physical sector rotations and both banks, transaction rollback, undo/redo,
save export/conflict handling and profile composition. ROM access is read-only.
Browser tests cover BW → Rocket → DP → Ultimate, editable controls, stale request isolation,
drag races, tab retention, field search, origins, Hidden Power and navigation.

Local tests open all four exact ROMs. Rocket additionally renders every map,
every species front sprite, all trainer portraits and referenced static NPC
sprites; reads all learnsets; and checks disposable save edits/reopening.
Native ARM probes validate decoded fields/setters/stats, all four trainer party
formats, 955 dex bits, 6,786 compatibility queries, 5,576 palette selections and
4,096 Hidden Power vectors and 1,910 encounter-header selections. A disposable
edited save also completed a VBA-M load/save/reopen round trip with all decoded
Pokémon, inventory and dex values preserved. Tests never overwrite a supplied save.
Ultimate's plain Pokémon format and sector-extension inventory have separate
native getter/stat and mGBA battery save/reload checks. See its adapter record
for the tested cases and the malformed ROM graphic entries.

```sh
GEN3_ROM_BW='/path/BW.gba' GEN3_ROM_DP='/path/DP.gba' \
GEN3_ROM_ROCKET='/path/Rocket.gba' GEN3_SAVE_ROCKET='/path/Rocket.sav' \
GEN3_ADAPTER_PROBES='/private/local/probes.json' \
  cargo test -p gen3-core -- --include-ignored
python3 scripts/verify_adapter_native.py --rom '/path/Rocket.gba' \
  --probes '/private/local/probes.json'
gen3 world '/path/Rocket.gba' > /private/local/world.json
gen3 catalog '/path/Rocket.gba' > /private/local/catalog.json
python3 scripts/verify_rocket_parity.py --rom '/path/Rocket.gba' \
  --world /private/local/world.json --catalog /private/local/catalog.json
python3 scripts/test_adapter_ui.py  # Vite running; generated API fixtures
```

See [Rocket parity evidence](research/rocket-21-parity.md),
[initial format assessment](research/rocket-21-compatibility.md), and
[battle lifecycle evidence](research/rocket-21-battle-forms.md).
