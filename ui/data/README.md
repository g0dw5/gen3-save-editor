# Official numeric reference / 官方数值参照

`official-stats.json` is a comparison dataset, not a ROM catalog. The application
continues to read the selected hack's names, stats, evolutions and artwork from
the supplied ROM. This file contains only official names, form labels and six
numeric base stats. No artwork, ROM bytes or wiki articles are included.

Source: **52Poké Wiki (神奇宝贝百科) contributors**, [Generation IX base stats](https://wiki.52poke.com/wiki/种族值列表（第九世代）),
with absent entries filled from its Generation VIII and VII lists. Each row
records its source generation; the JSON includes permanent revision IDs and
source HTML SHA-256 hashes. Generation IX follows that page's current coverage,
including forms from Legends: Z-A; it is not restricted to Scarlet/Violet.

The source is attributed under **CC BY-NC-SA 3.0**:
<https://creativecommons.org/licenses/by-nc-sa/3.0/>.
This data is separate from the project's MIT-licensed code. Changes: extraction
of numeric rows, conversion to compact JSON, reordering stats to Gen III order,
and merging older missing entries. Attribution and this license notice must
remain when redistributing the dataset or a build containing it.

Rebuild with `python3 scripts/update_official_stats.py`; HTML caches remain under
`.local/`, outside release inputs. Use the recorded revision URLs to recover a
specific source snapshot. Validate with `python3 scripts/test_official_stats.py`.
Rows are `[dex, name, form, generation, hp, attack, defense, speed, spAttack, spDefense]`.
Matching never assumes a hack's reused National Dex slot is an official identity.

这是独立的官方对照资料，并非内置 ROM 图鉴。以神百第九世代种族值列表为先，缺项
补用第八／第七世代，每条记录保存来源世代，来源页保存修订号和摘要。六围顺序为
HP、攻击、防御、速度、特攻、特防。仅包含名称、形态名和数值，不含图片或文章。
神百资料注明为 CC BY-NC-SA 3.0，本文件按该许可单独署名，不属于项目 MIT 代码许可。
改动为提取、转为 JSON、调整六围顺序及补充缺项。修改器右列始终使用当前 ROM 数值。
