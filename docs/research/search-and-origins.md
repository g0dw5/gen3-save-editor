# ROM name search and candidate origins / ROM 名称搜索与相遇来源

## Search / 输入搜索

Editable species, held items, moves, balls and bag/PC item selectors accept typed
queries. Labels and search use names and IDs from the loaded ROM. Move options
retain the ROM's category, type and power; TM items also match their associated
ROM move name. Enter or a pointer selection commits an existing ROM ID. Typing,
Escape or leaving the field does not commit; IME composition does not select.

Version 0.1.5 removes the official-name alias data and lookup. Switching the UI
language does not translate ROM names. No name mapping or text-data download is
required for search.

宝可梦、携带道具、招式、精灵球及背包／电脑道具支持输入搜索，只使用当前 ROM
的名称和编号。技能机器也能按对应的 ROM 招式名查找。招式分类、属性和威力仍然
显示。输入后点击或回车选择，退出搜索不会修改字段。

0.1.5 已移除官方译名映射及数据；切换界面语言不会翻译 ROM 名称。

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
- `python3 scripts/test_search_origins.py` against Vite: ROM-only names, rejection of removed aliases, IDs, TM/PC search, keyboard and IME, cancellation, branch
  filtering, current-value preservation, hatch selection and free editing.
- Existing editor navigation tests retain coverage for tabs, independent drafts
  and fixed controls across desktop/minimum/mobile layouts.
