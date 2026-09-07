# Trainer lookup and generated attributes / 训练家查找与生成属性

Verified 2026-09-07 on the exact BW/DP profiles in `profile.rs`.

## 查找路径

ROM 资料 → 对手训练家：按身份、地点筛选。选择“彩幽市联盟”得到以下五条记录，
再选择“四天王”或“联盟冠军”缩小范围；界面不再提供专门的一周目入口。

| 顺序 | ID | 地图 | 战斗指令 |
| --- | --- | --- | --- |
| 1 花月 | 261 | 16-0 | 0x227F7B |
| 2 芙蓉 | 262 | 16-1 | 0x2281E2 |
| 3 波妮 | 263 | 16-2 | 0x228480 |
| 4 源治 | 264 | 16-3 | 0x22870A |
| 5 米可利 | 335 | 16-4 | 0x228A51 |

可搜索“冠军 彩幽市”“馆主 绿岭市道馆”“摩天楼 米可利”，也可搜索地图、
宝可梦或训练家编号。条目显示标签，详情标签可点击继续筛选，身份与地点
支持组合，更多筛选提供单打/双打与预设/动态等级。切换界面语言保留筛选。
未关联地图的记录标注“地点待解析”，不能据此认定为无引用数据。

身份直接读取 ROM 的称号表：0x119A000，共 83 条、每条 13 字节，代码指针
位于 0x183B4、0x6F0AC。旧表 0x30FCD4 已没有这些引用。31/74 均为“四天王”，
38/75 均为“联盟冠军”，筛选按称号合并；不同训练家 ID 的队伍仍各自保留。
超出已知表范围的称号保留原始 ID 并显示未知，不影响查看队伍。

地图地区名来自 ROM，关联来自战斗脚本。“道馆／联盟”是按地图 ID 核对的
少量用途配置，目前覆盖丰缘八座道馆与彩幽市五间联盟战斗房间。
配置不含训练家编号，也不为没有脚本证据的强化队伍补写出现地点。
994–998 等记录仍可通过姓名/身份检索，具体触发条件尚未解析。

### 剧情与设施的后续模型

标签是训练家检索的基础。剧情导航可作为叠加视图，节点应引用战斗记录、
条件和前置事件，不把一次出现等同于这个人物的所有队伍。分支和复战需要
条件图，单一顺序树无法表达全部情况。

随机设施应使用“设施 → 模式 → 挑战阶段 → 候选训练家/配队池与生成规则”。
绿宝石原版的 `SetNextFacilityOpponent` 按挑战次数随机选择，并避免本轮重复
对手；这仅是适配研究的参照，尚未验证漆黑的魅影的完整对战塔实现。
魅影摩天楼已解析的固定队伍与动态等级，也不能等同于对战塔的随机配队。
目前应用不显示尚未实现的候选池或预测下一场随机对手。

Source: [pokeemerald battle_tower.c](https://github.com/pret/pokeemerald/blob/master/src/battle_tower.c).

先前的地图解析将 0x31（playfanfare）和 0x33（playbgm）误当作单字节指令。
它们实际分别为 3 和 4 字节；修正后可从上述地图根脚本追踪到战斗指令。
公开生成样本测试包含看起来像战斗指令的音乐参数，防止误扫描。

## 宝可梦卡片

主卡片显示性别、特性、性格、等级/动态等级规则、持有物、招式来源，
以及按 HP/攻击/防御/速度/特攻/特防排列的六项 IV 和六项 EV。
原始强度参数移入“原始参数与解析依据”；不再将 255/255 标成个体值。

示例：三春 #74 的第三只大奶罐，Lv.26、雌性、厚脂肪、害羞、无携带道具，
六项 IV 均为 31，创建时六项 EV 均为 0。训练家本人是男性，不影响大奶罐
这个固定雌性种族的性别。特性可点击查看 ROM 中的说明，便于判断迷人等招式。

动态等级由当前同行最高等级预览；没有打开存档则明确显示动态规则。
若自动生成的招式依赖动态等级，UI 不将按原始 101 参数推导的列表冒充实战招式。

## Engine evidence

`CreateNPCTrainerParty` at GBA 0x080385E8 accumulates the encoded trainer-name and
species-name bytes, adding the trainer name again for each party member. The sum
is not a Unicode string hash. The hook at ROM 0x33E552 constructs the personality
from this cumulative sum and record byte +3. With zero +3, its low byte is 0x88
for male trainers, 0x78 for female trainers, or 0x80 for double battles. A nonzero
parameter uses 400 × sum + parameter, optionally adding 25 to force parity.

The shiny-generation hook at ROM 0x311000 preserves nature, gender and parity for
fixed-personality callers even if it changes the complete PID. The adapter therefore
exposes derived attributes, not a promised exact PID or random OT ID.

At ROM 0x038758 and the corresponding branches for the other party layouts, the
u16 quality parameter is converted to a byte using `(quality * 31 / 255) & 255`.
The Pokémon constructor writes that byte to all six IVs if it is at most 31;
otherwise IVs are random. Quality 255 means six 31s; quality 250 means six 30s.

`CreateBoxMon` starts from cleared data, writes IVs at ROM 0x067D64 onward, and
selects the second ability with PID parity only when a second ability exists
(ROM 0x067E50 onward). The ordinary party constructor does not assign EVs, so
creation-time EVs remain zero. This describes the created team, not subsequent
native/script mutations or temporary battle effects.

These formulas require an engine-specific adapter. Moving table offsets alone
cannot describe personality hooks, level scaling, or script argument formats.
BW and DP share the verified implementation; a new ROM family needs its own
verification before it can use these rules.

## Reproducible validation

- 25 Rust tests, including opt-in exact-ROM regressions, generated name-sum/IV/
  gender fixtures, and music instruction-boundary checks.
- `scripts/verify_trainer_generation.py`: requires Python Unicorn, a compiled
  `gen3` CLI, and `GEN3_ROM_BW` / `GEN3_ROM_DP`. `GEN3_CLI` selects the executable.
  The CLI fingerprints the ROM before execution. No ROM or generated Pokémon
  is saved by the script.
- Executes the original Thumb constructor in isolated RAM for IDs 74, 261–264,
  335, 957, 993, 999 and 1000 on both ROMs: 20 parties / 106 Pokémon. Compares
  species, gender, nature, ability, IVs, EVs, level and explicit moves. Encrypted
  Pokémon checksums are verified independently.
- This is CPU-level constructor verification, not an emulator playthrough or a
  complete battle replay. Map/plot reachability and later party overrides remain
  separate concerns.

## English summary

The trainer browser uses composable role, location, battle-format and level-rule
facets, with clickable detail tags and bilingual controls. Roles come from the
ROM's relocated 83-entry class-name table, not hardcoded trainer ID lists. Map
purposes supplement script references; currently the verified metadata covers
the eight Hoenn gyms and five Ever Grande League rooms. Unresolved locations
remain searchable and do not imply unused records. Story progress is not inferred.

A future story view should reference encounters and conditions. Random facilities
need candidate pools, challenge stages and generation rules; the current app does
not claim to resolve Dark Phantom's entire Battle Tower or predict its next team.
Trainer cards retain generated gender, ability, nature, six IVs and creation-time
EVs. Dynamic levels show the rule and optionally the loaded party's highest level.
The supported ROMs' actual constructors independently validate the derived data.
