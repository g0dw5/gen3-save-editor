# Editor field semantics and trainer art / 编辑字段与训练家图像

Verified against the exact BW and DP profiles on 2026-09-07. No game graphics,
ROMs or extracted catalogs are included in the distribution.

## 宝可梦编辑

- 能力页按性格显示 ↑10% / ↓10%，无修正性格显示“能力无加减”；HP 不受性格
  修正。只读能力数值在应用修改后更新。应用修改保留当前页签。
- 同行 100 字节记录的前 80 字节就是盒子记录。两者都保存四项当前 PP，
  以及一个字节内的四项 PP 提升次数（每项两位，0–3）。最大 PP 不另存，
  使用 `基础 PP × (5 + 提升次数) / 5` 向下取整。同行追加的 20 字节存储
  状态、等级、邮件、HP 和能力，没有另一份 PP。
- 招式页明确显示当前 PP、提升次数与置灰的最大 PP。更换招式或修改提升次数
  会回满该招式 PP；自由编辑可保留超出常规上限的当前 PP。
- 来源页显示原训练家姓名、性别、公开 TID / 秘密 SID、相遇地点、等级、
  来源版本、捕获球及宝可梦语言标记。组合 OT ID 只读，等于 `SID << 16 | TID`。
  语言标记不等于界面语言；汉化 ROM 沿用原版编号。地点使用当前 ROM 的
  地区表解释，外来版本的同号地点不能据此视为相同地点。
- 地区表为 0x5A1480，共 213 条，每条 8 字节，文字指针位于条目开头。
  不扫描表后的数据作为地名；未知已有值保留并明确标识。253/254/255 是
  特殊蛋、游戏内交换、命运的相遇标记。
- 标记低四位依次为圆形、方形、三角形、心形，是整理用标签，不影响能力。
- 宝可病毒高四位为病毒株，低四位为剩余传染天数。未感染值为 0；已痊愈
  保留高四位而天数为 0。通常初始天数为 `(病毒株 % 4) + 1`，1–4 天。
  UI 使用状态、病毒株、天数控件；未知已有组合不会被无关修改自动规范化。
- 缎带低 15 位由五个三位的华丽大赛最高等级组成（0–4），不是 15 个独立
  勾选项。15–26 位是十二项独立缎带；27–30 位只读保留；31 位是命运相遇
  标志，在来源页单列。UI 修改采用无符号掩码，保留其他位。
- PID 默认只读，由性格、性别和闪光联动求解。自由编辑可指定 PID，但不能
  与尚未应用的性格/性别/闪光修改混用。校验结果与缎带保留位始终只读。

## Trainer image evidence

The relocated battle portrait table is at `0x1198000`, 203 entries of 8 bytes;
compressed palettes are at `0x1199000`. Code references at `0x5DF78` and `0x5DF80`
point to these tables. Entries render on a 64×64 4bpp canvas. Resource 165 has only
63 tiles and is not referenced by any parsed trainer: it is reported unavailable
rather than filled with guessed pixels. Resource 195 has an extra tile outside
the 64×64 canvas. All 202 complete portraits are tested on both ROMs.

Map actors retain their local object ID, 16-bit graphics ID and script pointer.
A trainer is associated only with objects whose own scripts reach its battle;
sharing a room is insufficient. Shared scripts can correctly link both twins.
The noninteractive Wallace scene at battle `0x228A51` has a reviewed adapter entry
for local actor 1. No name-to-image inference is performed for unlinked records.

The graphics hook at `0x11960E4` selects a bank by the high byte. Bank pointers
at `0x1196184` select `0x505620` and `0x1197000`. Bank zero IDs 240–255 require
runtime variable resolution and are not treated as static pictures. Each graphics
info record supplies width/height, palette tag and a frame table pointer at +28.
The first frame uses 4bpp tiles. Palette tags are resolved through the 35-entry
raw palette table at `0x50BBC8`. The UI displays a battle portrait and the distinct
verified map sprites, with map/object evidence available in a disclosure.

## Validation / 验证

- Core regressions cover both exact ROMs, the complete portrait resources, original
  and extended map graphics banks, and actor associations for the twins and Wallace.
- Generated fixtures check PP preservation across 80/100-byte records, tile order,
  transparency and resource bounds. Existing Pokémon sprite regressions also exercise
  the shared tile decoder.
- UI regressions cover nature markers, current/max PP, translated origin choices,
  TID/SID composition, markings, infection/recovery, contest rank, ribbon bit 31
  and preservation of reserved bits 27–30, both languages, and trainer pictures.
- Existing storage, undo/redo, drag-and-drop, export and small-window checks remain.

## English summary

Numeric enums now use named controls; calculated and reserved values are disabled.
Markings use four symbols, Pokérus uses state/strain/days, contest ribbons use rank
selectors, and other ribbons use checkboxes. Party and box Pokémon share the same
PP storage. Applying an edit preserves the active tab. Trainer portraits and map
characters are extracted on demand from the user's fingerprinted ROM; unresolved
actors and incomplete resources remain explicitly unavailable.

Primary references:
[pokemon.h](https://github.com/pret/pokeemerald/blob/master/include/pokemon.h),
[pokemon.c](https://github.com/pret/pokeemerald/blob/master/src/pokemon.c),
[global.fieldmap.h](https://github.com/pret/pokeemerald/blob/master/include/global.fieldmap.h),
[global constants](https://github.com/pret/pokeemerald/blob/master/include/constants/global.h),
[decompress.c](https://github.com/pret/pokeemerald/blob/master/src/decompress.c).
