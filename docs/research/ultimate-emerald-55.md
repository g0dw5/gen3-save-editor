# Ultimate Emerald 5.5 / 究极绿宝石 5.5 适配记录

Supported input: **究极绿宝石 V5.5-失落之古遗**, 33,554,432 bytes, MD5
`17ce9785b33319b3dbda9a5d37c57ec1`. The adapter is tied to this exact
fingerprint; another Ultimate Emerald patch needs separate verification.

支持的输入为 **究极绿宝石 V5.5-失落之古遗**，大小 33,554,432 字节，MD5 如上。
其他究绿版本需单独核对地址与规则，文件名相同也不会通过校验。

## ROM reference / ROM 资料

Names, stats, abilities, moves, items, evolution entries, learnsets, encounter
tables, maps, trainer records and graphics are read from the supplied ROM at
runtime. The adapter contains addresses and verified format rules, not a copy
of those tables. Its catalog exposes 1,199 nonzero-numbered species slots, 938 move
slots, 800 item slots, 922 map headers and 1,336 nonempty trainer records.
One species slot (`412`) is an empty ROM entry. Eevee (`133`) uses a separate
ten-entry evolution table; the usual five-entry table would omit several
branches. Battle-only transformations are shown separately from permanent
evolution. Trainer enhanced-party entries expose the ROM's template for IV,
nature and ability; actual EVs and levels may change with difficulty, player
party and battle context, so the reference page labels them as dynamic.
The trainer view has one selector for the four difficulty settings. Each trainer
header has one party pointer rather than four mode-specific party tables, so
the selector retains the source roster and describes known generation limits.
For an explicit scenario, the trainer view now executes the loaded ROM's Thumb
opponent EV constructor (`0x09F042BC`) in a bounded ARMv4T sandbox. It first
executes the ROM's player Speed and role classifiers and party-summary sort,
then supplies the selected difficulty and either the opened save's current
party or a manually entered party. The page shows the resulting six IVs and
EVs for each enhanced-template Pokémon, including both possible values when
the constructor uses a random bit. The ordinary-party preview now resolves
levels from the ROM's constructor branches at `0x09F04C76–0x09F04C9C` and
`0x09F04E48–0x09F04E58`: a raw level of zero uses the player's highest party
level, and Lunatic raises lower raw levels above one. With no save, fixed-level
Standard/Challenge entries need only the ROM; other entries accept the player's
highest level as one number. Their level-up moves are read for the resolved
level. Untemplated ordinary parties use the native `CreateMon` call at
`0x09F04CB4` with fixed personality and IV zero, retaining zero EVs; their
nature, ability slot and gender therefore follow that personality and the
ROM species data. The preview assumes an ordinary
single or double trainer battle and excludes scripted substitutions, facility
constructors and the separate flag-`0x268` forced-level-50 path. Enhanced IV/EV
simulation still accepts a manual level override for special scenarios.

Charizard `6` has two Mega edges and an item-702 edge to species `252`.
Item `702` is named 许愿星块 and describes battle Dynamax; species `252` has
a distinct Gigantamax-style front sprite despite sharing the name 喷火龙.
The UI labels this edge as a Gigantamax appearance. This table and artwork
do not by themselves prove that the full Dynamax battle mechanic is usable.

名称、数值、图片、地图和各类条目在打开 ROM 后读取。适配层只记录地址和已核实
的格式。资料包括 1,199 个非零编号槽位、938 个招式槽位、800 个道具槽位、
922 张地图头和 1,336 条非空训练家记录。编号 `412` 在 ROM 内为空。
伊布 `133` 有独立的十条进化表；战斗变身与永久进化分开展示。强化训练家
显示 ROM 培养模板；实战努力值、等级可能随难度和战斗状态改变，不冒充固定值。
训练家页提供四档难度选择。同一训练家头只有一份队伍指针，没有四套并列的
模式队伍表；切换难度保留源队伍并说明已知约束。剧情脚本可改用其他训练家
编号。现在可按具体情景执行当前 ROM 的原生对手努力值例程，同行信息优先取当前
存档，也可手动设置；六项努力值和个体值按所选难度显示。若原生随机分支产生两种
结果，页面会并列标出。普通训练家队伍预览现会按本 ROM 的等级分支计算实战等级，
并按该等级读取升级招式。标准／挑战模式中等级固定的普通队伍只需打开 ROM；
原始等级为零或疯子模式的等级追赶需要玩家同行最高等级，可以从存档读取，也可
手填一个数字。没有强化模板的普通队伍保留原生创建流程赋予的固定性格、特性槽位、
性别及零个体值、零努力值。
强化模板仍可逐只指定模拟等级。强制 50 级标记、剧情换队和设施生成器不在此预览内。
喷火龙 `6` 除两条 Mega 关系外，还有携带 `702`“许愿星块”指向 `252`
的关系；`252` 的立绘是超极巨化外观，但进化表和立绘尚不足以证明完整的
极巨化战斗机制可用。

The full image scan found eight map headers in group 36 whose secondary
tileset points to bytes without the expected compressed header, plus species
slot `1199` with an invalid front-image pointer. One object sprite (`62`) has
fewer image bytes than its declared dimensions. Their ROM entries remain
listed; those images cannot currently be rendered reliably. The map reference
shows an explicit unavailable-image message for those entries while keeping
their location and trainer links visible. The map reference
also flags two ROM headers with malformed event pointers instead of treating
their event bytes as valid NPCs or pickups. These are input-ROM findings; the
editor does not replace the missing artwork with bundled assets.

全量渲染检查发现第 36 地图组有八张图的第二图块集指向非标准压缩数据，
以及编号 `1199` 的正面图片指针无有效压缩头。NPC 图片编号 `62` 的图像字节
少于声明尺寸。条目仍可查，相关图片当前
不能可靠绘制；地图页会明确提示，仍保留地点和训练家链接。另有两张地图的事件指针
不完整，页面会提示该地图的 NPC／
拾取标记不可确认，而不会把随机字节当成事件。

## Save editing / 存档修改

The game uses 128 KiB battery saves with Emerald-style sector rotation, but
the Pokémon payload is stored in plain canonical order. Its native Pokémon
checksum field and sector checksum fields are not used as in stock Emerald.
This adapter preserves the unused Pokémon word, writes the game's constant
sector marker, and still validates record bounds, supported IDs and save
layout. It reads the hidden-ability bit, mint nature and six independent Hyper
Training flags. It preserves original IVs when changing effective trained
stats. Inventory pockets stored across sector extension tails and the game's
shifted Pokédex bits are handled by the shared editor transactions.

本作同样使用 128 KiB 电池存档和轮转扇区，但个体记录明文顺序保存，原版的
个体校验和及扇区校验和已不按原版规则使用。修改器保留未使用的校验字，写入
本作扇区标记，并继续核对结构、编号和边界。隐藏特性、薄荷性格、六项极限
训练标记分别解析；极限训练只影响计算用个体值，不覆盖原始个体值。扩展扇区
里的背包和偏移后的图鉴位也纳入共用事务、撤销、导出流程。

## Cheats / 金手指

The existing two-line all-mode AI no-peek set remains. A two-line accuracy
correction swaps the mistaken Casual/Lunatic bonus side without changing the
multiplier. Seven other recipes are enabled for this exact ROM: portable
Pokémon PC, paused walking encounters, guaranteed wild capture, faster party
egg hatching, specified
wild species/level, ordinary wild shiny encounters, and map teleport. Each
recipe validates its original instruction bytes before encoding GameShark
Advance V1/V2 lines. The encounter and shiny hooks use Ultimate-specific code
locations and retain the ROM's native generation routines. See
[formats and instructions](../cheats.md).

原有两行全模式去窥屏继续保留。两行命中修正代码交换养生／疯子加成写反的阵营，
保留原倍率。另新增随身电脑、暂停走路遇敌、野生必捕、
同行蛋快速孵化、指定遇怪／等级、野生闪光和地图传送。生成前会核对原指令；
指定遇怪与闪光使用本作独立的挂钩，保留原生个体生成流程。

## Verification and boundaries / 验证及边界

- Exact BW, DP, Rocket and Ultimate ROM regression tests pass, including
  catalog loading, evolution and cheat-context isolation. Ordinary core tests
  pass independently of private ROM files.
- 675 synthetic Pokémon across nine species, 25 natures and three training
  patterns were compared with the unmodified ROM's native getters and stat
  routine: 10,125 getter comparisons and 675 stat comparisons agreed. A
  generated save edited at the last box slot, last pocket slots and final
  Pokédex bit was loaded, saved and reopened by a fresh mGBA core.
- Final encoded cheat groups were decoded and toggled in mGBA. Common recipes
  exercised native catch, egg and walking branches; parameterized recipes
  exercised 384 wild species/level/shiny combinations, 32 nonwild controls,
  256 lead-ability cases and five complete map transitions. The portable PC
  was tested through deposit, box movement, normal save, fresh-core reload
  and withdrawal. Fishing tiles for four seeds matched the ROM's own routine.
- A real progressed Ultimate Emerald user save was unavailable for these
  checks; tests used a disposable game-generated battery save. Mobile
  emulators and every story-dependent trainer or facility override have not
  been fully simulated. Trainer EV previews execute the ROM's native constructor
  for the stated inputs, not a complete battle forecast.

四个版本均通过精确 ROM 回归。究绿的 675 组个体与原生取值和能力计算核对，
合计 10,125 项字段、675 组能力一致；模拟器重新载入、正常存盘并二次读取了
编辑过的临时存档。金手指按最终编码在 mGBA 验证启停、遇怪、孵蛋、捕获、
闪光、传送及电脑存取流程。四个钓点种子与原生函数一致。当前没有用户
实际长期游玩的究绿存档用于验证；手机模拟器、全剧情地图与特殊设施的
实战队伍尚未穷尽。努力值预览严格对应页面给定的情景，不等于完整对战预测。
