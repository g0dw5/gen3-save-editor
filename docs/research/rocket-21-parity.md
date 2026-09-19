# Rocket 2.1 adapter parity / 西火适配验证

2026-09-19, MD5 `59c658a1081f542086de1060bb65f0b3`, 32 MiB.
All addresses below are ROM file offsets unless prefixed with a RAM address.
This report supersedes the initial read-only assessment. No game assets or saves
are distributed. Version remains **0.2.0**.

| Area | Evidence | Implementation |
| --- | --- | --- |
| Bag | Native initializer `0x10E8D0` | PC 10, items 255, medicine 67, balls 16, battle items 130, berries 68, special items 102, machines 254, key items 64 |
| Bag offsets | SB1 `498/4C0/F04/9BC/1010/DF4/1218/9FC/8BC` respectively | Logical sections 1–4; quantities XOR key except PC |
| Pokédex | Getter/setter `0xF7C60`; seen SB1 `2EE4`, owned `2F5C` | 955 bits each; no Emerald SB2 mirrors |
| Experience | `0x5B3484`, six 604-byte rows | Native thresholds through level 150; 23-bit stored experience |
| Ribbons | Native getter `0x976D0`, fields 50–54, 67–80 | Cool/Beauty are one-bit; other contest ranks three-bit; ability is separate in byte 47 |
| Machines | Item IDs 592–845; moves `0xCF8C54` | 246 TMs + 8 HMs; item names from ROM |
| Teaching compatibility | `0x616090` pointer table, native `0x9B80C` | Per-species zero-terminated u16 list; shared machine/tutor compatibility |
| Egg moves | `0x61DAC4`, native `0x9E234` loop bound `0xFDB` | 4,060 words; species marker 20,000 + ID |
| Descriptions | Move pointers `0xD045EC` | 754 ordinary moves, excluding move 0 |
| Maps | `0x9F4F40`, 56 groups | 1,363 headers, all rendered |
| Trainers | `0x586A18`, 2,559 × 40 including record 0 | 2,558 nonempty teams |
| Trainer generation | `0x4D3F0` | Types 0/2: 8-byte members; types 1/3: 24-byte members, explicit EVs/nature, random PID |
| Trainer pictures | `0x55E2F4` + palettes `0x55EA9C` | 245 portraits |
| NPCs | `0xB64C38`, 542 pointers, palettes `0xB654CC` | 16-bit object graphics ID at template +2 |
| Female artwork | Flag table `0x54CBE0`; native `0x49680`, `0x9C31C` | Front `0x56B420`, regular palette `0x55716C`, shiny `0x55BA44` |
| Hidden Power | Type `0x53C88`; power `0x6354C` | Same IV parity/type formula, fixed 60 power |
| Persistent forms | `0x6193AC`; native `0x9D210/0x9D224` | Trigger 1 after held-item changes; trigger 3 after move changes |

## Cross-sector inventory / 跨扇区背包

The original inventory reader added every offset to physical section 1. That is
valid for BW/DP's small pockets, but Rocket medicine crosses `0xFF4`; battle and
special pockets start after it. The new reader/writer concatenates logical SB1
payloads and regenerates each affected checksum. Tests exercise **every slot**,
including medicine slot 60 and the boundary-adjacent slots, with independent
physical bank data and XOR quantity expectations. The inactive bank stays intact.

中文：不是“把校验和修好就算完成”。跨扇区的地址必须先转换为逻辑块位置，否则
可能读到扇区元数据或其他逻辑区。三版全部道具格均有读回与无关字节保留测试。

## Encounter selection / 相遇选表

Main headers start at `0xBE8A70`. Native selector `0xEC170` returns the **first**
matching map header. Later duplicate map headers are not additional simultaneous
encounters. The one explicit exception is map **51-106**: variable `0x403E`
selects first-header + value for 0–8; values above 8 select variant 0. The selector
is called by the ordinary grass/surf, fishing and rock-smash paths. These nine
variants have readable story-dependent labels; raw selector conditions remain
in developer evidence rather than being merged.
This inspected selector has no RTC/time-of-day branch.

中文：相遇概率是对应表内的槽位权重。保留重复宝可梦槽位的贡献，但过滤游戏不会
选到的后续同地图表；51-106 的九组资料按剧情变量标注，不当成九片草丛。31 个源表
空种类槽位不会伪装成宝可梦或导致其他权重重新归一。设施中的程序生成、群体出现
等额外运行时规则不由普通地图表证明，资料不承诺当前存档的完整遭遇模拟。

## Scripts / 脚本

Native dispatch is `0x22B218`. Rocket uses Emerald operand widths plus opcodes
`DD–EA`; BW/DP retain their own walker rules. Teleport/warp commands `39/3A`
suspend the field script and reload the map, so trailing movement/text bytes are
not parsed as more instructions. Remaining unparsed paths retain offsets. One
ordinary map script on 3-41 reaches `0x496771` without an interpretable continuation;
it is surfaced rather than scanned byte-by-byte for plausible trainer numbers.
Native special routines are not claimed as fully interpreted reward scripts.

## Validation / 验证

Public fixtures cover all codecs and all 24 permutations, ribbon bit ownership,
all bag slots, all box slots with physical sector rotations, form triggers,
transaction rollback and bounded ROM patches. Native probes execute the actual
ROM getters/setters/stat routines and compare the edited bytes. Additional probes
cover all 955 dex bits, four trainer party formats, every palette choice at two
PID/gender extremes, machine/tutor queries and 4,096 Hidden Power inputs. Level
100/101/127/150 records are checked for every growth curve assigned to a species
in this ROM (five curves; none uses curve 1).

All map images, species fronts, trainer portraits and referenced static NPC
sprites are decoded locally. This establishes bounded decoding, not that every
map or NPC is reachable in a normal playthrough. An edited disposable gameplay
save is exported and reopened; the user's source file is never overwritten.
On 2026-09-19, the disposable export was also loaded in VBA-M 2.2.3: Zubat
displayed level 15, Adamant and Infiltrator after editing. An in-game save advanced
the counter from 3 to 4; reopening it preserved every decoded Pokémon, inventory
entry and dex flag exactly, with both save banks valid. The medicine test writes
slot 66 across the first main-block sector boundary. The original save SHA-256
remained unchanged. This checks a real gameplay round trip, not every feature.
Native comparisons are not a claim that every battle mechanic or emulator has
been tested through a complete gameplay session.

Validation totals for this update: 52 public core tests and four private-ROM
regressions pass; six browser workflows pass. Native comparisons include 173
records, 3,287 field reads, 144 byte-exact setters, 173 stat calculations, 260
persistent-form choices and 1,910 encounter-header choices, in addition to the
palette, teaching, trainer, dex and Hidden Power checks above.

中文：模拟器按键无响应的排查已写入
[存档研究流程](../../skills/gen3-rom-research/references/saves.md#emulator-input-preflight)。
先检查英文输入法、窗口焦点与实际映射，再用单次输入确认响应；本次通过菜单虚拟
按钮完成验证，不能据此断定输入法是唯一原因。

Map pixel correctness and Anya random-generation follow-up: see
[map and trainer display](rocket-map-and-trainer-display.md). The earlier bounded
map decode checks alone did not establish visually correct rendering.
