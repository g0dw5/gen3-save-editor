# Trainer lookup and generated attributes / 训练家查找与生成属性

Verified 2026-09-07 on the exact BW/DP profiles in `profile.rs`.

## 查找路径

ROM 资料 → 对手训练家 → 一周目联盟，按以下顺序展示：

| 顺序 | ID | 地图 | 战斗指令 |
| --- | --- | --- | --- |
| 1 花月 | 261 | 16-0 | 0x227F7B |
| 2 芙蓉 | 262 | 16-1 | 0x2281E2 |
| 3 波妮 | 263 | 16-2 | 0x228480 |
| 4 源治 | 264 | 16-3 | 0x22870A |
| 5 米可利 | 335 | 16-4 | 0x228A51 |

可搜索“一周目 米可利”“一周目联盟”“摩天楼 米可利”，也可搜索地图、
宝可梦或训练家编号。场景分组与排序是经过版本核对的适配元数据；姓名、
队伍、招式、图片仍从用户 ROM 读取。994–998 单列为联盟强化队伍，尚不推断
具体触发条件，尤其不自动认为玩家的当前存档一定会遇到这些强化队伍。

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

- 24 Rust tests, including opt-in exact-ROM regressions, generated name-sum/IV/
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

The trainer browser now supports ordered first-League lookup, same-name context,
and context/map/species search. Trainer cards display generated gender, ability,
nature, six IVs and creation-time EVs. Raw quality parameters are evidence only.
Dynamic levels show the rule and optionally the loaded party's highest level.
The supported ROMs' actual constructors independently validate the derived data.
