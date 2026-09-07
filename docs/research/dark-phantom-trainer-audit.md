# Dark Phantom trainer reference audit / 训练家引用核查

Verified on 2026-09-07 against user-supplied ROMs; no ROM content is distributed.
All offsets below are file offsets unless prefixed as a GBA address.

| Version | MD5 |
| --- | --- |
| 5.0 EX+ BW | `0d9b129f7dd76895f79bb47ad7dec2fe` |
| 5.0 EX+ DP | `cb2940215f4dafb1bef133c3af379f44` |

## 结论

此前按原始等级字节不在 1–100 范围筛出的 96 条训练家记录，全部找到
地图脚本或活跃复战表引用：93 条有地图脚本战斗指令，另 3 条有复战表引用。
因此这批记录没有一条能根据“未找到引用”判定为无引用脏数据。

这不等于已经证明正常游戏流程可触发全部战斗。地图本身的入口、剧情条件、
复战阶段和当前存档状态仍可能限制可达性；静态引用与游戏流程可达性必须区分。

| 训练家 ID | 地点 | 证据类型 |
| --- | --- | --- |
| 681 阿元·阿初 | 流星瀑布 `24-1` | 地图脚本 |
| 683、684、685 阿元·阿初 | 流星瀑布 `24-1` | 与 681 同组的复战表 |
| 903 小遥、904 丛树 | 变化洞窟 `24-106` | 菜单及性别分支后的战斗脚本 |
| 905–978、980–992（87 条） | 魅影摩天楼，多张内部地图 | 地图脚本 |
| 993 库鲁玛 | 魅影摩天楼 `36-46` | 原生函数调用后的战斗脚本 |
| 999 小智、1000 三人组 | 不存在之所 `37-38` | 音乐指令后的战斗脚本 |

还有一个独立结论：92 条记录使用原始等级 101，不能再统称为等级错误。
该版本的创建宝可梦代码会把不在 1–100 范围内的参数替换为同行六个槽位的
最高等级，最低取 1。这是反汇编确认的运行规则，尚未作为模拟器实战验证。

另外 4 条（681、683、684、685）读出 244，存在队伍格式疑点：头部 flags=0
使引擎按 8 字节步长读取，但相邻数据看起来按带技能的 16 字节记录排列。
682 也属于同组格式疑点，碰巧读出的等级在范围内，没有进入这 96 条名单。
244 同样会经过动态等级转换，但这不能证明错位读取的其他字段正确，不能
自动修改 flags 或删除记录。当前 API 保留原始参数和诊断；UI 已显示动态等级规则并可按当前同行预览。
复战关系索引尚未实现，不能把原始值当作实战等级。

## Reference evidence

The original walker linked 88 of the 96 headers. Four missing opcode handlers
accounted for five more: `callnative` (0x23, 5 bytes), `fadenewbgm` (0x36,
3 bytes), `multichoice` (0x6F, 5 bytes), `checkplayergender` (0xA0, 1 byte).
Native addresses are not treated as bytecode roots. Native calls invalidate
known script variables; menus and gender queries invalidate VAR_RESULT.

Instruction formats were checked against the upstream
[command table](https://github.com/pret/pokeemerald/blob/master/data/script_cmd_table.inc)
and [event macros](https://github.com/pret/pokeemerald/blob/master/asm/macros/event.inc),
then followed from map roots in both exact ROMs. Nearby matching bytes alone
were not counted as a map association.

| ID | Map root / branch | Battle command |
| --- | --- | --- |
| 903 | `24-106`: `0xDFBC40` → `0xDFBCC0` → `0xDFBF00` | `0xDFBF00` |
| 904 | `24-106`: `0xDFBC40` → `0xDFBCC0` → `0xDFBF10` | `0xDFBF10` |
| 993 | `36-46`: `0xE02400` | `0xE0241C` |
| 999 | `37-38`: `0xE069C0` | `0xE069E7` |
| 1000 | `37-38`: `0xE068D0` | `0xE068F7` |

681 has direct battle commands at `0x22C540`, `0x22C589`, `0x22C5A4`, and
`0x22C5ED` on map `24-1`.

### Indirect rematch references

The active rematch table starts at `0x5500A4`, with a 16-byte stride. Row 22 at
`0x550204` contains five little-endian u16 trainer IDs `[681, 682, 683, 684, 685]`,
map group 24 and map number 1. The engine literal at `0x0B1EBC` points to GBA
address `0x085500A4`; code at `0x0B1E94` indexes it with a four-bit left shift.
The row is within the engine's main 65-row iteration, not merely in adjacent data.
These values are identical in BW and DP.

The upstream [rematch structure](https://github.com/pret/pokeemerald/blob/master/include/battle_setup.h)
explains the five IDs and map fields; the hack's actual table address and stride
were verified in its own code. This proves a reference from an active table,
not that every rematch stage is triggerable in a particular save.

### Runtime level rule

The trainer party constructor at `0x0386E6` selects its record stride from
header flags, reads the level as a byte at record +2, and calls the Pokémon
constructor at GBA address `0x08067B4C`.

That constructor branches via the literal at `0x067B64` to GBA address
`0x09B037BC` (file `0x1B037BC`). The hook:

1. Preserves a supplied level in the inclusive range 1–100.
2. Otherwise initializes the result to 1 and scans six bytes at
   `0x02024540 + 100 * slot`, retaining the maximum.
3. Stores the result in the constructor's level local and returns to
   GBA address `0x08067B68`.

The player party base is `0x020244EC`: the save routine at `0x076D8C` copies six
100-byte records from this address to SaveBlock1 +0x238. Level is at record +0x54,
matching `0x02024540`. The hook and its entry/return literals are identical in
BW and DP. It scans all six slots without checking party count or egg status;
“highest party level” assumes ordinary valid slot contents.

This explains the 101 parameters without inventing a fixed replacement level.
The separate hook at `0x33E552` reads record +3 for personality-related behavior;
that adjacent byte must not be consumed as the high byte of a u16 level.

## Verification and limits

- Updated walker: 609 distinct trainer IDs, 653 map associations, 2,658 encounters
  extracted from supported sources, on each ROM. Associations retain command offsets.
- Exact-ROM regression checks all five recovered map/battle associations, the
  96-header count, the three remaining indirect IDs, and their rematch row.
- Generated fixtures check instruction boundaries, native-call variable
  invalidation, and that native addresses are not parsed as script roots.
- There are still 550 maps with at least one unresolved script path. An absent
  map link elsewhere must remain “unresolved,” not “unused.”
- No full game-flow reachability proof, emulator battle replay, or automatic
  party-format repair is claimed. The public location index currently contains
  direct script associations only; the three rematch findings above are research
  evidence, not yet links shown by the UI.

## English summary

All 96 previously flagged trainer headers have references: 93 through map battle
scripts and three through an active rematch table. None can be classified as
unreferenced on this evidence. This does not prove normal gameplay reachability.
Of these, 92 use raw level 101, which the ROM's runtime converts to the highest
level among the six player party slots. Four raw-244 records still need a
separate party-layout investigation. Preserve raw parameters, distinguish dynamic
rules from malformed layouts, and never equate parser gaps with unused data.

## Follow-up: first League and generated attributes

The subsequent [trainer UI/generation audit](trainer-search-and-generation.md)
corrected music command lengths (0x31 and 0x33), recovering the first League map
links. Current totals are 636 distinct trainer IDs / 680 direct map associations,
2,659 extracted encounters, and 536 maps with an unresolved path. The historical
counts above describe the earlier four-opcode audit. The 96-header conclusion
and three indirect rematch IDs are unchanged.
