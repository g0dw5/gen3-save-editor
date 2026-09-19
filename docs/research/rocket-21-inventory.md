# Team Rocket 2.1 data inventory / 西班牙火箭队数据规模

Read-only assessment: 2026-09-19. ROM MD5:
`59c658a1081f542086de1060bb65f0b3` (32 MiB, Chinese five-arc release).
These findings describe this exact binary, not all Team Rocket releases.
No ROM, save, full text table, or graphics are distributed here.

Follow-up: [battle forms and persistence](rocket-21-battle-forms.md) verifies
native Mega restoration and Z-move eligibility, and records the limits of the
Dynamax/Tera evidence. 后续验证已从表存在推进到部分原生执行路径。

## Counts / 数量口径

| Dataset | Observed count | Meaning / 解释 |
|---|---:|---|
| Nonzero species records | 1,394 | Includes regional, battle and original forms; not distinct obtainable species / 包含各种形态，不是可捕获种类数 |
| Distinct nonzero national-dex mappings | 944 | Mapping coverage, not a verified completion target / 图鉴映射去重数量，不代表已确认的全图鉴目标 |
| Unmapped species/form records | 40 | National-dex conversion returns zero / 全国编号转换返回零 |
| Ordinary move records | 754 | Excludes move zero / 不含空招式 |
| Additional Z-move battle records | 35 | Data presence does not prove player access / 有数据不等于玩家可使用 |
| Used elemental types | 18 | Includes Fairy; reserved type ID 9 is not a nineteenth type / 包含妖精，保留编号不算新属性 |
| Mega target records | 56 | 55 item-based associations and one move-based association / 55 条道具关联及 1 条招式关联 |
| Primal-style target records | 3 | Kyogre, Groudon and a custom Ghost form / 盖欧卡、固拉多及原创幽灵形态 |
| Map groups | 56 | Native map-bank table / 地图分组表 |
| Map headers | 1,363 | Includes rooms, floors and story variants / 包括房间、楼层及剧情变体 |
| Distinct layout pointers | 1,251 | Pointer deduplication, not visual or geographic deduplication / 按指针去重，不等于独立地点数 |

The first 898 species IDs cover the base species through Calyrex. The remaining
496 records include forms, original experiments and selected later-generation
species. Not all Generation IX species are present. The dex mapping contains
944 distinct nonzero values, reaches 950, and omits 937–942. Forty records map
to zero. Neither 1,394 nor 944 establishes how many species the player can obtain.

中文：前 898 个编号对应至蕾冠王的基础种类，后面 496 条混合了形态、原创实验体及
部分后续世代宝可梦。可获得性还要核对相遇、赠送、进化、事件及剧情条件。

## Stat examples / 种族值举例

Comparison uses ordinary forms in the current
[Pokémon Showdown data](https://github.com/smogon/pokemon-showdown/blob/master/data/pokedex.ts),
retrieved 2026-09-19. Historical generation differences can also produce a mismatch;
the examples below describe actual numeric differences, not their development history.

| Species / 宝可梦 | Reference BST | ROM BST | Selected reference → ROM changes / 变化举例 |
|---|---:|---:|---|
| Castform / 飘浮泡泡 | 420 | 550 | HP 70→95; Attack 70→105; Speed 70→105 |
| Sharpedo / 巨牙鲨 | 460 | 540 | Attack 120→140; Defense 40→70; Speed 95→115; Sp. Atk 95→75 |
| Camerupt / 喷火驼 | 460 | 540 | Sp. Atk 105→145; Defense 70→100; Speed 40→20 |
| Absol / 阿勃梭鲁 | 465 | 545 | Speed 75→115; Sp. Atk 75→95 |
| Gengar / 耿鬼 | 500 | 552 | HP 60→70; Sp. Atk 130→140; Sp. Def 75→97 |
| Flygon / 沙漠蜻蜓 | 520 | 550 | Speed 100→110; Sp. Atk 80→100 |
| Blastoise / 水箭龟 | 530 | 550 | HP 79→99; Defense 100→120; Speed 78→58 |
| Shuckle / 壶壶 | 505 | 505 | HP 20→30; Attack and Sp. Atk 10→5 |

Examples of additional Mega associations include Flygon (BST 650), Milotic
(670), Dusknoir (635), Starmie (620) and Noivern (635). These are actual special
evolution associations, not an inference from duplicate names or graphics.
This does not establish availability of every required stone or story unlock.

中文：不是所有宝可梦统一增强，也存在降速、重新分配点数及总和不变的调整。
Mega、地区形态、原创实验体应作为不同的数据概念展示。

## Binary evidence / 二进制证据

All addresses below are ROM file offsets.

- Species names: `0x59F9F0`, stride 11, ending at `0x5A35E1`.
  Species base data: `0x5B4764`, stride 36, ending at `0x5C0B90`.
  Both independently bound 1,395 records including the zero record.
- National-dex mapping: `0x5B1608`, u16, indexed by species minus one.
  Native converter: `0x9ABB0`. Executing it in Unicorn matched all 1,394
  nonzero-species table lookups.
- Move battle data: `0x5ACD5C`, stride 20. IDs 1–754 are ordinary moves;
  755–789 have the 18 type-based plus 17 signature Z-move records. The next
  structure begins at `0x5B0B14`. Ordinary names use stride 13; do not extend
  that name indexing into the Z-move range.
- Evolution data: `0x5F96D4`, ten 8-byte slots per species, ending at
  `0x614AC4`. Special methods: `0xFFFF` (55), `0xFFFE` (1), `0xFFFD` (3).
  Rayquaza's move-based entry requires move 620. Native item-form lookup
  `0x67614` explicitly checks methods `0xFFFF` and `0xFFFD`; executing it
  matched all 58 item-based target associations.
- Map-group table: `0x9F4F40`, 56 pointers. Map-pointer lists occupy
  `0x9F39F4..0x9F4F40`. Native lookup `0xBA0B8` references the group table
  through its literal at `0xBA0CC`. Executing it matched all 1,363 headers.

Native checks ran with the ROM mapped read-only from the source file into an
isolated emulated address space. The user's ROM and save were not edited.
These inventory checks do not establish support for safe save editing, Z-move
activation, Dynamax, Terastallization, or every encounter and story path.
