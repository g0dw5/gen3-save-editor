# Search aliases and candidate origins / 搜索译名与相遇来源

## Search / 输入搜索

Editable species, held items, moves, balls and bag/PC item selectors accept typed
queries. Search matches the ROM name, ROM ID and mapped Simplified Chinese,
Traditional Chinese or English names. Move options retain the ROM's category,
type and power; TM item options also match their associated move. Enter or a
pointer selection commits an existing ROM ID. Typing, Escape or leaving the field
does not commit a value; IME composition does not trigger selection.

宝可梦、携带道具、招式、精灵球及背包／电脑道具支持输入搜索，匹配 ROM 名称、
编号及已映射的简中、繁中、英文名称。技能机器也能按对应招式查找。选择框同时显示
ROM 原名与当前界面的译名，写入的始终是 ROM 编号；招式分类、属性和威力以 ROM
为准。输入后通过点击或回车选择，退出搜索不会修改字段。

### Data provenance and adapter boundary

`ui/data/dark-phantom-aliases.json` maps ROM IDs to canonical identities and short
localized names: 456 moves, 337 items and 411 species. It is enabled only for the
BW/DP fingerprints listed in the README. Unknown profiles and unmapped custom
entries retain their ROM names without inferred aliases. It ships no ROM text
messages, descriptions, sprites, maps or binary data.

The identity map was reviewed against the supported ROM catalog (names, move
parameters/descriptions, item use and species national-index data), using
`pret/pokeemerald`'s item/move constants as the original-generation reference.
It is **not** a blanket correspondence with original-game IDs. For example:

| ROM ID | ROM name | Canonical identity |
| --- | --- | --- |
| Move 29 | 思念头槌 | 意念头锤 / Zen Headbutt (428) |
| Move 463 | 头槌 | 头锤 / Headbutt (29) |
| Move 206 | 刀背打 | 点到为止 / False Swipe (206) |
| Move 230 | 香甜花蜜 | 甜甜香气 / Sweet Scent (230) |
| Item 178 | 剧毒珠 | 剧毒宝珠 / Toxic Orb (PokeAPI item ID) |

Names come from the community-maintained PokeAPI CSVs at commit
`88f332f7a68a77162c64a48c230b19420e1ed3be`: `move_names.csv`, `item_names.csv`,
and `pokemon_species_names.csv`. The full notice is in
[licenses/PokeAPI.txt](../../licenses/PokeAPI.txt), also included in desktop bundles.
`scripts/update_official_names.py --check` downloads pinned, SHA-256-verified
sources and checks the text. Omit `--check` to refresh names while preserving the
reviewed canonical IDs. New profiles require their own reviewed identity mapping;
never assume shared indexes merely because they are Gen III hacks.

这里的官方译名来自社区维护的译名数据，并非官方在线服务。尚未映射的自创招式、
特殊道具继续显示 ROM 原名。新版本必须重新核对编号对应关系，不能直接套用原版
偏移或编号。

## Origins / 来源筛选

`Rom::ancestors` follows incoming evolution edges to include the selected species
and its pre-evolutions, terminating even if data contains cycles. Capture options
use the region IDs of parsed encounters for that set. Sibling branches are
excluded: Vaporeon includes Eevee's encounters, but not Flareon's. The species
reference window's direct encounter list remains specific to that species.

Breeding eligibility is separate. The evolution family is expanded to discover
breedable parents for baby species. A family is a candidate if a member has valid
egg groups other than Ditto (13) or Undiscovered (15). Hatch locations come from
map region IDs, not wild encounter tables. In Gen III the met-level field records
zero after hatching, so choosing the hatch source sets that field to zero; the UI
locks it while that source is selected in standard editing. Cancel restores both
source and location. Ditto and Mewtwo are excluded in both exact-ROM regressions.

普通编辑的捕获地点只列本体及进化前形态的已解析地点，不包含旁支进化。选择
“孵蛋获得”后，列出 ROM 地图中的孵化地点，相遇等级设为 0；宝宝宝可梦通过
进化后的可繁殖亲代判断。自由编辑允许完整地点列表及自定义来源。

These are **candidate origins**, not proof of legality or current-story access.
Breeding compatibility, incense and special breeding scripts are not fully
simulated. Encounter data can carry unresolved event/time conditions and does not
cover every scripted gift. Unverified existing locations remain visible as a
read-only current value, so opening or editing another field never silently
rewrites provenance. These UI suggestions do not add a new CLI legality gate.

候选列表不等同于合法性证明：没有模拟全部繁殖、熏香或特殊脚本条件，也不按当前
剧情过滤。事件、时间条件和部分赠送来源仍可能待核实。已有存档的不明地点保留为
不可选择的当前值，避免打开页面就覆盖原记录。

## Verification / 验证

- Synthetic Rust tests: branching evolutions, cycle termination, baby breeding,
  Ditto/Undiscovered exclusion and ancestor-only encounter slots.
- BW/DP local regression: Eevee/Vaporeon ancestry, Mewtwo and Ditto exclusions.
- `python3 scripts/test_search_origins.py` against Vite: Chinese/Traditional/English
  aliases, replacement IDs, TM/PC search, keyboard and IME, cancellation, branch
  filtering, current-value preservation, hatch selection and free editing.
- Existing editor navigation tests retain coverage for tabs, independent drafts
  and fixed controls across desktop/minimum/mobile layouts.
