# Map rendering and trainer display / 地图与训练家显示

2026-09-19. Rocket 2.1 Chinese MD5: `59c658a1081f542086de1060bb65f0b3`.
Addresses are ROM file offsets; extracted assets and user saves remain private.

## Map format / 地图格式

The earlier all-map decode check established bounds, but did not establish
correct pixels. It missed three incompatible assumptions inherited from BW/DP:

| Format | BW / DP | Rocket 2.1 |
| --- | --- | --- |
| Primary / secondary tile boundary | 512 | 640 |
| Primary / secondary metatile boundary | 512 | 640 |
| Entries per metatile | 8 (two layers) | 12 (three layers) |
| Primary + secondary palette banks | 6 + 7 | 7 + 6 |

Native evidence:

- `0xBFB90`: compares the metatile against `0x27F`, subtracts 640 for the
  secondary table, and multiplies the local index by 24 bytes.
- `0xBFBFC`: entries 0–3, 4–7 and 8–11 go to BG3, BG2 and BG1 buffers;
  native execution verifies all twelve entries, including the third layer.
- `0xBE8F0`: loads 640 primary tiles and 384 secondary tiles starting at 640.
- `0xBE8D0/0xBE8E0`: palette loads are 224 + 192 bytes. `0xBE808` uses
  secondary source offset `0xE0`; the first backdrop color is black.
- Map `0-0` primary tileset `0x7C0300`: compressed tiles expand to `0x5000`
  bytes; metatiles `0x76251C` through attributes `0x76611C` span 640 × 24 bytes.

The renderer now takes these rules from each profile. Synthetic pixel tests
exercise both boundaries, all layers, transparency, XY flipping and palette
ownership for all three profiles. All 1,363 Rocket maps decode again; maps
0-0 through 0-3 were visually inspected, alongside BW samples. This is a static
map preview, not a simulation of animated tiles, weather or runtime map edits.

中文：黑洞、错位来自把西火三层格式按漆黑两层格式读取，且主／副图块与调色板
边界不同。配置现已分离；测试增加了实际像素断言，不再只验证“能输出 PNG”。
静态地图不会播放水面动画，也不承诺模拟剧情运行时替换的全部图块。

Map 42-5 was separately investigated after a scrambled-preview report. Its
native layout/grid/layer and palette outputs match the editor; see the
[focused evidence and limitations](rocket-map-42-5.md).

## Trainer abilities and gender / 训练家特性与性别

Constructor `0x4D3F0` uses deterministic PID generation for party formats 0/2.
Formats 1/3 roll PID with `0x9D40C` until the stored nature matches, without
constraining gender or ability bits. Species gender ratios still apply.
The constructor writes moves, held item, EVs and stats; its inspected caller
`0x4B7B4` does not replace abilities. The subsequent `0x9C610` routine returns
for trainer battles. Ability getters are `0x98884/0x988F0`.

All 13 Anya rows were constructed with 32 fresh RNG seeds: **1,696 Pokémon**.
Heracross in row 59 produced both abilities 62 and 153, and both genders.
Ursaring has ability 62 in both ordinary slots; Weavile has 151 in both.
Duplicated slots therefore mean one guaranteed ability, not two simultaneous
abilities. Slot equality can preserve an ability-dependent build despite PID
randomness. Other distinct slots genuinely introduce uncertainty; the UI cannot
choose the strategically preferable ability and present it as guaranteed.
These checks concern initial teams; temporary battle transformations and move
or ability effects can subsequently change the battle state.

中文：安雅的多个定制队伍在本场队伍生成时随机性别／特性，性格、努力值与配招
仍是预设。圈圈熊双槽都是“毅力”，玛狃拉双槽都是“穿透”，显示两次属于 UI
缺陷。现在候选去重，并标注“固定特性”或“生成队伍时随机选择其一”。

## Readable references / 可读资料

Evolution method semantics are a separate adapter format, independent of the
species struct. BW/DP's dispatch at `0x6D16A` and Rocket's `0x9A37C` distinguish
level, item, trade, gender, move, party and location conditions. Item, move,
species and region parameters resolve through the selected ROM. Unknown methods
remain explicitly unresolved. Battle-form methods remain separate.

Move effects show the ROM's description; numeric effect/target/flag values stay
in collapsed developer evidence. Inventory categories resolve through profile
pockets; fishing rod names use profile item IDs (BW/DP 262–264; Rocket 866–868).
Story encounter variants get readable labels while preserving raw selector data
in developer evidence. Fairy type, percentages and Rocket's fixed 60-power
Hidden Power are displayed correctly. The optional ROM patch form uses named
ability and growth-curve choices instead of raw enum numbers.

中文：进化显示“达到 90 级时升级进化”“使用某道具”等条件；招式显示 ROM 自带
效果说明，不臆造统一效果编号字典。普通资料不再显示 AI 位标志、剧情变量编号、
海拔数值等内部字段，开发者仍可展开依据查看原始信息。中英文界面均有回归测试。
