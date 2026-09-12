# Encounter table selection / 相遇表与时间条件

Verified on 2026-09-12 against user-supplied, unmodified ROMs:

| Profile | MD5 |
| --- | --- |
| Dark Phantom 5.0EX+BW | `0d9b129f7dd76895f79bb47ad7dec2fe` |
| Dark Phantom 5.0EX+DP | `cb2940215f4dafb1bef133c3af379f44` |

## 结论与显示范围

上述两个版本的普通相遇表没有早晨／白天／夜晚分表。实际选表代码按地图组和
地图编号选择记录；草丛、水面、碎岩和三档钓竿的槽位抽取均不读取时钟。
这不是根据“表里没找到时间字段”做出的推断：已检查调用者，并执行原始
Thumb 选表和抽签函数，限制其只能读取已核对的非时钟输入。

因此本次不向修改器添加虚构的昼夜条件，也不将所有脚本事件统一标为“全天”。
同一地图、同一方式、同一物种的多个条目仍表示槽位／等级分布，不能据此
推断为不同草丛或不同时间段。后续聚合显示应保留每个等级的权重和原始偏移。

原野区的两个具体核对结果：

- `26-13` 的优雅仙子是同一草丛表中的 Lv.36（4%）和 Lv.39（1%），合计 5%。
- `26-1` 的胡说树为 Lv.31（1%），没有夜间限定。

此结论不表示游戏没有时间事件。洞窟变量、群聚、游走宝可梦、钓鱼特殊点、
设施配表以及脚本定点事件需要分别分析；剧情或地图可达性也不由槽位表证明。
本次没有完整验证每日群聚更新、全部脚本条件或手机模拟器实机运行。

## Active engine evidence

All addresses below are **file offsets**; add `0x08000000` for GBA ROM addresses.
Function names are descriptive matches to the upstream implementation, not
symbols embedded in the hack. Evidence comes from the supplied bytes.

| Offset | Observed behavior |
| --- | --- |
| `0xEA2D34` | Active array of 20-byte wild headers: map group/number and land, water, rock, fishing pointers at +4/+8/+12/+16. 175 records precede the terminator; 167 distinct map IDs. |
| `0xB4CF8` | Header lookup compares group/number with SaveBlock1 +4/+5 and returns the first match. Ordinary maps read no other save fields. |
| `0xB4D28` | Only special header-selection branch compares the map with `0x6A18`, meaning group 24, map 106 (Altering Cave). |
| `0xB4D30` | Calls `VarGet(0x403E)` for that cave. Values 0–8 add an index to the first cave header; values above 8 use zero. This is a save variable, not a clock read. |
| `0xB4D48` | Literal bytes `34 2D EA 08` reference the active relocated wild table. |
| `0xB4AC8` | Land selector uses `Random() % 100`; weights are 20,20,10,10,10,10,5,5,4,4,1,1. |
| `0xB4B84` | Water/rock selector uses weights 60,30,5,4,1. |
| `0xB4BD8` | Fishing selector branches on rod argument 0/1/2, with weights 70/30; 60/20/20; 40/40/15/4/1. |
| `0xB4F4C` | Ordinary generator receives a table pointer; checks lead ability influences and indexes that table. No morning/day/night branch. |
| `0xB5012` | Reads the selected species directly from the chosen row at +2, then calls the wild constructor. |
| `0xB5024` | Fishing generator selects within the supplied fishing table; reads species at `0xB5048`. |
| `0xB550C` | Rock Smash entry calls header lookup, loads header +12, then runs encounter/generation checks. |
| `0xB5588` | Sweet Scent calls the same header lookup. Ordinary grass loads header +4 at `0xB564A` onward and calls the generator at `0xB5686`; water uses +8. |
| `0xB5734` | Fishing entry handles the special fishing-spot check, otherwise calls header lookup at `0xB5766` and loads header +16. |
| `0x6F5CC` | Random advances the LCG in RAM at `0x03005D80` and increments a counter at `0x020249C0`. Neither is an RTC read. |

The ordinary walking path also calls this lookup at `0xB52A6`. Its grass and
water branches load +4 and +8 respectively before invoking the same generator.
The reviewed module at `0xB4880..0xB5B3C` is byte-identical in BW and DP, with
SHA-256 `2e71a4b6ff571679e4165d5e2061c85f253ae02e0067a12328b0dae6fdcfc698`.

Incoming direct Thumb BL references to header lookup were inspected at
`0xB52A6`, `0xB550E`, `0xB5588`, `0xB56FE`, `0xB5766`, `0xB57A8`, and `0xB582E`.
The final three include availability/ambient-selection helpers. A byte-pattern
scan is only a way to find candidate references; the conclusion rests on the
decoded call paths and actual memory-read checks, not on an absence of matches.

## Other conditions are separate

- The generator checks Magnet Pull and Static, and level/repel-related rules.
  These can affect encounters but are not time-of-day filters.
- `0xB50DC` checks outbreak data in SaveBlock1 (`+0x2B90` species,
  `+0x2B92/+0x2B93` map, `+0x2BA1` probability). The ordinary encounter entry can
  take this alternative path. Its daily update lifecycle is outside this audit.
- Roamers and special fishing spots can bypass normal slot selection. Facility
  rooms can provide separate tables when the ordinary map lookup fails.
- General Pokémon construction has hack-specific hooks, including shiny logic;
  a table audit is not a proof of every subsequently generated Pokémon field.

Do not generalize this result to an unknown ROM by reusing its table offsets.
A new adapter needs a separate audit of table selection and runtime conditions.

## Reproducible validation

`scripts/verify_encounter_selection.py` requires Python Unicorn. It reads only
user-supplied ROMs and executes their actual Thumb instructions in isolated RAM:

```sh
GEN3_ROM_BW='/path/to/BW.gba' GEN3_ROM_DP='/path/to/DP.gba' \
  python3 scripts/verify_encounter_selection.py
```

For **each** ROM, the verification completed:

1. 167 distinct map lookups plus one missing-map lookup.
2. 11 Altering Cave variable cases, including out-of-range values.
3. Every roll from 0 to 99 for land, water/rock, and each of the three rods:
   500 executed slot selections. The unmodified ROM RNG is seeded to produce
   each roll; the selector and RNG instructions are not replaced by stubs.
4. An input-read allowlist: header lookup can read the map location and, only
   for Altering Cave, its variable; slot selectors can read only the RNG state
   and call counter, in addition to ROM literals and their stack. Unexpected
   RAM or hardware/RTC reads fail the test. ROM memory is read-only.
5. The selected Safari slots resolve to the expected Sudowoodo and Lilligant
   levels and frequencies in the actual ROM tables.

This is exhaustive for the selectors' bounded roll domains and tested map
inputs. It is not a 24-hour emulator playthrough or an audit of every game event.
The script also checks the exact ROM fingerprint and reviewed module hash.

## Reference sources

- [Upstream encounter functions](https://github.com/pret/pokeemerald/blob/master/src/wild_encounter.c)
  supplied semantic names for the disassembled code and comparison points.
- [Upstream time-based events](https://github.com/pret/pokeemerald/blob/master/src/clock.c)
  distinguish daily systems from per-encounter table selection. Their presence
  upstream alone is not evidence that every corresponding hack event is active.

## English summary

The exact supported BW and DP ROMs do not select ordinary wild encounter tables
by morning, day, or night. Native header lookup is map-based, with one verified
save-variable exception for Altering Cave. Native land, water/rock and rod
selectors have no clock inputs. No time-of-day UI field was added. Duplicate
species rows describe separate weighted level slots, not separate time periods.
Script conditions and daily outbreak lifecycle remain distinct research scopes.
