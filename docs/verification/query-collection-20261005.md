# Query and collection increment / 查询与收集增量验证

Scope: the five exact fingerprints in [the capability matrix](../capability-matrix.md).
The application version stays unchanged. No installer, ROM, save or extracted game
content is added to Git. These are bounded functional increments, not full story or
all-mechanism certification.

范围：能力矩阵中的五份精确 ROM。版本号保持不变，未打包或发布，未提交游戏
文件及提取素材。本轮交付查询和规划闭环，不能据此宣称全剧情完整支持。

## Delivered flows / 可使用流程

1. Pokémon/item/move reference → runtime acquisition source → target map tile →
   static exterior entrance chain → previous reference. Map rewards link back to
   item reference. Existing NPC, pickup, hidden and reward layers remain independent;
   entrances have a separate layer and focused targets cannot be blocked by item pins.
2. Open SAV → choose Dex ownership/current individuals → optional permanent family
   grouping → regional missing goals and unclaimed/unknown rewards → standalone HTML.
   Unknown rewards require opt-in. Collection is a read-only query; receipt flags are
   not inferred from bag contents. Permanent form references override exclusion from
   battle-only collection goals when a permanent source is actually parsed.
3. Mercury ROM → trainer → native ordinary-party scenario → generated nature,
   gender, ability, IVs, EVs, moves, item and level. Explicit simulation seeds are
   separate from actual game RNG. This does not emulate the complete battle setup.
4. Mercury acquisition → saved virtual time or explicit simulated hour → native
   period and next boundary. Virtual SAVE time is now decoded by the later
   [clock increment](mercury-clock-20261005.md); no device-clock fallback.

以上流程均为共用页面与查询模型。地图静态关系不保证现在可达；收集路线按区域
合并，仅作建议，不宣称最短。家族模式表示至少有一个成员，不表示分支已全部完成。
水银训练家预览为剧情状态清零的普通生成情景，不保证等于当前存档下一场实战。
时钟后续增量已验证虚拟保存时间和跳时持久化，保留明确模拟条件；硬件 RTC 和完整
星期刷新仍待验证，详见时钟增量记录。

## Evidence / 证据

- Core: **83 passed, 16 opt-in ignored**. Safety tests cover every PID permutation,
  all box slots, drag/swap preservation, combined IV/EV/ability edits, rollback,
  undo/redo, backup and external export conflicts. New tests reject native sandbox
  ROM writes and unmapped accesses and enforce instruction bounds.
- Two `local_query_` tests passed with all five exact ROMs, including bounded
  topology, acquisition lists, shop records and invalid destinations. Private BW,
  Rocket and Mercury SAV planning preserves in-memory save bytes. DP and Ultimate
  were ROM-only in this increment; no fresh emulator round trip is claimed for them.
- `verify_query_rules.py`: 56 independent native flag/variable reads for each of
  BW/DP; 72 Mercury native time predicates, all 24 hours with forced night unset.
- `local_native_trainer_mercury`: 742 trainers × 2 seeds = **1,484 scenarios**,
  **3,784 Pokémon**. `verify_native_trainers.py` independently runs Unicorn and
  checks PID, level, stats, species, item, experience, moves, IVs, EVs and ability;
  **71,896 getter comparisons**. ROM remains byte-identical. This establishes
  matching constructor output for this scenario, not full live battle accuracy.
- Browser fixtures: source/tile/entrance/back, reward/item, planner/HTML escaping,
  compact window; retained editor tabs and controls; BW→Rocket→DP→Ultimate stale
  world isolation; native seed changes and stale native-response isolation.
- Real-ROM browser smoke passed for all five loaded fingerprints: runtime sources,
  map navigation and Mercury native scenario UI. No SAV is opened in that smoke.
- TypeScript and production web build passed. No desktop package was produced.
- Strict Clippy identified six existing baseline lints (constant chunks, redundant
  cast, identical form branches, argument count, two test initializations). They are
  recorded separately from functional results; strict Clippy is not claimed clean.

`scripts/test_exact_rom_query_ui.py` requires all five `GEN3_ROM_*` variables,
a token-authorized local development bridge and Vite. It opens actual ROMs only.
Python probes use `uv run --with unicorn`; browser fixtures use
`uv run --with playwright python scripts/test_*.py` with Vite running. Native
probe data is generated under a private, ignored directory, never bundled.
Use an absolute `GEN3_NATIVE_TRAINER_PROBES` output path for the Cargo local test.

## Remaining boundaries / 剩余边界

- Full primary/side-quest dependency graphs, native reward calls and all custom
  gifts/trades/roamers are not indexed. Native stops remain visible as uncertainty.
- Only BW/DP have native-verified SAV flag/variable addressing. Rocket, Ultimate
  and Mercury receipt results stay unknown. NPC visibility is not receipt evidence.
- FireRed/custom hidden-item quantity/index packing needs exact native proof;
  item identity and tile are shown, quantity and collection flag are unknown.
- Complete training-mechanism catalogs and their unlock/acquisition paths are not
  certified. Existing per-ROM editor rules do not prove mint/cap/tutor availability.
- Shop prerequisites, repetition, held-item probabilities with ability modifiers,
  all baby/incense/parent rules and weekday refresh remain partial or unverified.
- Static topology does not resolve script-swapped layouts, random facilities,
  one-way mechanics or story access. Invalid destinations are diagnostics.
- Mercury ordinary constructor executes `0x09D0B150` with zero context, first
  trainer and opponent-side arguments. Its full setup `0x09D0D514`, nonzero context,
  player-party dependencies and facilities remain unverified. Two seeds do not
  describe the complete distribution. Other ROM preview rules remain separate.
- Mercury expanded Dex flags, hardware RTC mode and complete weekday refresh
  remain unverified. Virtual-clock, jump-time and forced-night SAVE conditions
  have since been validated in the clock increment. No unsupported mechanism is
  inferred absent merely because the current parser does not expose it.

未完成项保留为明确缺口；不得把读取表格、生成情景或打开页面等同于完整支持。
