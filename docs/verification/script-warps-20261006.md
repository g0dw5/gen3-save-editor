# Referenced script passages / 已引用脚本通道

Map reference windows now combine ordinary map-table connections with bounded,
referenced script transitions. This increment covers all five registered exact
fingerprints; overall map access remains **P**, not fully verified.

## User workflow / 查询流程

Follow a Pokémon, item, move or event reference to its map. **Entrances and
connected maps** now includes script passages, positioned NPC/trigger sources,
incoming references and conditional exterior chains. Select a source to locate
its tile; follow a destination or return to the previous reference. The entrance
layer also displays positioned script sources. An NPC position locates the
actor; it does not mean stepping onto that tile initiates a transition.

Each passage retains NPC visibility and script branch guards. ROM-only queries
show unknown saved state. Opening/updating a SAV rechecks persistent conditions
against a fresh snapshot; the ROM-bound script cache contains no SAV eligibility.
Satisfied guards do not establish activation, story access, collision permission
or a return route. Links are directed; no reciprocal edge is invented.

地图页新增已引用的脚本通道、NPC／触发格位、入口来源及带提示的外部入口链；
支持定位、跳转及返回。NPC 格位用于找人，不表示踩上该格就会传送。打开或更新
SAV 后仅叠加已知持久条件的结果；触发方式、当前可达性与返程仍需验证。

## Native operand evidence / 原生参数证据

`scripts/verify_script_warps.py` runs unmodified native script handlers through
independent mGBA ARM7 execution in disposable RAM. No SAV is opened, no ROM byte
is written and no native function is replaced.

| Fingerprint | Native cases | Referenced passages in first 50 map records |
|---|---:|---:|
| Dark Phantom BW | 189 | 84 |
| Dark Phantom DP | 189 | 84 |
| Spanish Rocket 2.1 Chinese | 189 | 89 |
| Ultimate Emerald 5.5 | 189 | 83 |
| Mercury FC 1.2 | 162 | 4 |

The **918** cases check literal/variable operands, 127/128, 255/256, negative
results, map/warp byte narrowing, consumed operand widths and RAM neighbors.
For immediate instructions, execution stops naturally at `SetWarpDestination`
and checks all five arguments. Complete `setwarp` calls check the eight-byte RAM
record, unchanged padding/neighbors and false return. Reading two-byte coordinate
operands does **not** imply two-byte effective coordinates: `VarGet` is followed
by signed eight-bit narrowing. The shared decoder matches every independent
vector. The map counts are reference samples, not accessible passage counts.

Engine rules validate loaded-ROM dispatch before decoding. Ordinary warp,
silent, door, teleport and spin prefixes are covered across all five fingerprints;
the gym-warp prefix is additionally covered in BW/DP/Rocket/Ultimate. Mercury's
gym opcode is not assumed to share those semantics. `setwarp` and dynamic/dive/
hole/escape destination setters do not create immediate passage edges.

坐标先经本作 `VarGet`，再缩为有符号一字节。918 个原生用例及共享解析器逐例
一致；立即传送只验证到设置目的地之前，`setwarp` 则完整执行。上述地图数量是
前 50 条地图记录中的引用数量，不等于游戏中当前可用的入口数量。

## Regression and reproducibility / 回归与复现

- Public synthetic regression covers all five adapters: branch/NPC visibility,
  destination setters and unreferenced bytes excluded, warp-index target tiles,
  signed coordinates, unresolved variables, fresh SAV guards, directed edges,
  stale-ROM rejection and input preservation.
- Opt-in `local_script_warp_operands_match_native_all_profiles` compares the
  independent CPU vectors, builds each exact-ROM reference index, and samples
  its first 50 map records. Original ROM memory/file hashes remain unchanged.
- `GEN3_UI_SCRIPT_WARPS=1` extends `test_collection_source_facts_ui.py`: shared
  five-profile bilingual map/source/marker/back workflow and SAV condition refresh.
  UI content is synthetic; it is not evidence of in-game access.
- Public core suite: **125 passed, 45 opt-in ignored**. Workspace check,
  TypeScript/production UI build, formatting and five release-workflow checks pass.
  The five-profile UI suite passed; a separate 900×640 Mercury fixture verifies
  wrapped conditions and the movable reference window with synthetic content.

Run the native verifier with the five `GEN3_ROM_*` variables, the RAM-only mGBA
probe and a private output path. Then set `GEN3_WARP_PROBES` to that output and
run the ignored Rust parity test. Private fixtures/output are not release inputs.
The read-only helper source is `scripts/native/breeding_probe.c`, built against
mGBA with its `calluntil`, `resetfixture`, RAM-write and observation interfaces.

## Remaining limits / 未完成范围

This is a bounded script-reference graph, not a reachability solver. Hole warps,
extended/native/special transitions, live trigger selectors, moving actors,
dynamic layouts and random facilities remain incomplete. Unknown variables,
negative/dynamic destinations and invalid target records are retained as
unresolved and do not enter resolved entrance chains. Script stops and technical
evidence are expandable. Existing collection HTML entrance suggestions still
use the static map-table graph; integrating guarded script alternatives into
that planner is further work. No shortest-route or full-map-loading claim is made.

整体入口能力仍为部分解析；未知脚本、动态布局、随机设施和完整地图切换尚未穷尽。
本轮没有修改用户 SAV、ROM、剧情或图鉴，也没有打包或发布。
