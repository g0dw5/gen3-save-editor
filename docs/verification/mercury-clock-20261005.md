# Mercury 1.2 game clock / 水银 1.2 游戏时钟

Exact MD5: `f323df1792ac68462a34b42fe8571533`. These additions query SAVE
snapshots and never modify game time, ROM bytes, story flags or collection records.
The common acquisition and collection pages use fingerprint-scoped clock rules.
Other games retain their own unresolved clock status; no Mercury rule is reused.

范围是水银 1.2 的精确指纹。查询保存时的时间，不修改时钟、ROM、剧情标记或图鉴。
资料查询、收集建议和独立 HTML 共同展示时间依据；不把水银规则套给其他版本。

## Native data path / 原生数据路径

| Meaning / 内容 | Exact-ROM evidence / 原生证据 |
|---|---|
| Virtual-clock switch / 虚拟开关 | Flag `0x1335`; native `FlagGet` `0x6E6D0` → hook `0x1D5B4E0`; persisted extension byte `0x146`, bit 5 |
| Forced night / 强制夜晚 | Flag `0x1041`; same expanded flag hook; extension byte `0xE8`, bit 1 |
| Saved clock words / 保存字段 | Variables `0x51EF–0x51F2`; `VarGet` `0x6E568` → resolver `0x1D5B530`; extension `0x5DE–0x5E5` |
| Word packing / 打包 | Year; month/high + day/low; hour/high + minute/low; second/high + weekday/low |
| Clock speed / 速度 | Variable `0x51F3`, extension word `0x5E6`; native `0x1D5A6B0` accepts 1/2/5/10/30/60, otherwise normalizes to 1 |
| Restore and persist / 恢复保存 | `0x1D5AA94` and `0x1D5A65C`; runtime struct `0x03005EA0` |
| Month lengths / 每月天数 | ROM table `0x1DDEDF8`, 32-bit entries; native `0x1D5A618`, Gregorian leap/century rule |
| Advancement / 推进 | `0x1D5A9A8`, requires virtual flag and active-play gate `0x03000E7C`; subsecond counter `0x03005EA9` is not in the four persisted words |
| Jump menu / 跳时菜单 | `0x1D5ABE0`: select hour, or separately advance one day while retaining hour/minute/second |
| Encounter periods / 相遇时段 | Morning 04–07, day 08–16, dusk 17–19, night 20–03; forced-night predicate runs first |

The clock is in Mercury's existing segmented extension storage, not the base
FireRed variable block or an assumed RTC trailer. Weekday is a saved native value
(0 Sunday…6 Saturday), not recomputed from the date. A differing weekday and date
must not be silently corrected. No names, event catalog or clock table is bundled.

时间保存在水银扩展存储里，不是原版火红变量区，也不从 RTC 尾部猜测。星期是独立
保存的游戏值，不根据日期重算。月份天数仍实时读取当前 ROM，不内置内容表。

## Usable flow / 可使用流程

Open Mercury ROM and SAV → ROM reference → acquisition panel → **Use saved game
time**. The panel shows saved date, native weekday, time, virtual speed, period and
game-time interval to the next boundary. Choose an hour for a clearly labelled
simulation, or **Show all periods** to remove time annotations. Simulations do not
inherit the SAVE's forced-night override. A forced-night SAVE shows no promised
next period, because the flag must first clear through game behavior.

Collection suggestions prefer sources matching the saved encounter period when
comparing otherwise eligible sources, retain other-period sources as alternatives,
and label time mismatches. The plan and its standalone HTML state the saved game
time separately from the report-generation timestamp. This does not establish
current map access or complete prerequisite coverage.

打开 ROM 和 SAV 后，资料的获取途径默认按保存时段标注；可切换模拟小时或所有
时段。收集建议优先考虑符合保存时段的来源，保留并标注其他时段。HTML 将游戏
保存时间与报告生成时间分开。时间匹配不等于地图现在可达或其他前置条件都满足。

The game's menu jumps to 04:00 / 08:00 / 17:00 / 20:00. Choosing a target hour
less than **or equal to** the current hour advances the date and native weekday;
choosing a later hour stays on the same day. “Tomorrow” advances the day while
retaining time. Jumping hours clears minutes, seconds and the fractional counter;
the separate tomorrow operation retains hour/minute/second. This explains why
changing period can also change weekday.

游戏菜单分别跳到 04／08／17／20 点。目标小时小于或等于当前小时会推进日期和
星期；晚于当前小时则当天跳转。“明天”推进一天并保留时分秒。跳小时会清零分秒
与小数计数；单独推进明天保留时分秒。因此换时段也可能改变星期。

## Verification / 验证

- `scripts/verify_mercury_clock.py` independently runs original Thumb code in
  Unicorn: **336** hour × weekday × forced-night restore/selection/persistence
  vectors; **72** month/leap-year comparisons; **8** jumps covering equal hour,
  next day, leap day, century exception and year rollover; **9** speed cases;
  **9** invalid saved-clock vectors that enter the actual RTC fallback branch.
- `local_mercury_clock_matches_native_restore_and_period_selection` compares
  Rust output with all 336 private native vectors, on the current SAV and on a
  post-jump native re-save. It also checks exact SAVE byte preservation and rejects
  invalid dates. Public tests cover RTC/invalid/scenario separation, native month
  data ownership, forced-night behavior and isolation from the four older profiles.
- Real mGBA menu test, with a private copy: 120 active frames at speed 1 advance
  two seconds. From October 5, 10:24:23, morning/day choices advance to October 6
  04:00/08:00; dusk/night stay on October 5 17:00/20:00; tomorrow retains 10:24.
  A normal Save-menu save after the morning jump increments counter 3→4;
  re-reading its persisted clock gives **October 6, Tuesday, 04:00:10**.
- Original input is **October 5, Monday, 10:24:02**, speed 1, forced night unset.
  The editor reads a snapshot, so live seconds after loading need not equal this
  saved second. User originals stay unchanged; emulator tests use private copies.
- Core regression: **88 passed, 18 opt-in ignored**. The two five-ROM `local_query_`
  checks also passed. New public cases cover per-map/method base-table fallback,
  including a target species absent from the overriding period table.
- `test_saved_clock_ui.py` passed in English and Chinese: saved/simulated/all-period
  queries, collection planning, standalone HTML and a compact window. Each of the
  four older exact ROMs lost Mercury's previous SAVE clock and query controls.
  Query/navigation/HTML and native-trainer browser fixtures passed as well.
- The five exact-ROM query/map UI smoke and Mercury inventory/location/warning
  regression passed on the final sources. No SAVE mutation/export request was
  sent. User original ROM/SAV SHA-256 values still match untouched test inputs:
  `b98d9701f4b567810c70221564c348f4482791c614c3f1bb282e96678b7a0896` and
  `33e4ac873e411cdf214efb46efc881b1c2e5d176d65cfe28f417677569ee7f70`.
- TypeScript, production web build, formatting and diff checks passed. Clippy
  passed only with existing baseline lint exclusions (`const_is_empty`,
  `unnecessary_cast`, `if_same_then_else`, `too_many_arguments`,
  `field_reassign_with_default`, `chunks_exact_to_as_chunks`); this is not a
  strict-clean claim. No desktop package was built for this increment.

本轮公开核心测试 88 项通过、18 项本地文件测试默认忽略；五份精确 ROM 的查询
回归通过。中英文界面验证覆盖保存时间、模拟／全部时段、收集建议、独立 HTML 和
紧凑窗口，切换其他四份 ROM 不残留水银时间。未发布或打包本轮增量。

Reproduce with:

```sh
uv run --with unicorn python scripts/verify_mercury_clock.py \
  --rom /private/Mercury12.gba --save /private/Mercury12.sav \
  --output /private/clock-vectors.json
```

Then set `GEN3_ROM_MERCURY12`,
`GEN3_SAVE_MERCURY12`, and `GEN3_CLOCK_PROBES` to these private files and run
`cargo test -p gen3-core local_mercury_clock -- --ignored --nocapture`.
The read-only CLI also provides `gen3 game-clock ROM [SAVE]`.

## Boundaries / 未完成边界

Hardware RTC mode is still unresolved. When the virtual switch is off or the
saved virtual date is invalid, the native routine consults hardware RTC; these
SAVE words alone cannot determine its result. The UI reports this and permits an
explicit simulated hour. It never substitutes the computer's time.

The current snapshot cannot predict uncommitted emulator changes, future flag
changes, all weekday rewards or refresh/reset mechanisms. General Mercury reward
receipt flags, complete quest dependencies, expanded Dex flags and full trainer
battle setup remain separate gaps. This increment does not promote them to verified.

硬件 RTC 模式、所有星期奖励与刷新机制仍未完成；未保存的模拟器变化不在 SAV 中。
虚拟日期无效时不猜测重置结果。普通奖励领取、完整任务依赖、扩展图鉴和完整入战
设置仍是独立缺口；本轮时间验证不能证明这些机制已经支持。
