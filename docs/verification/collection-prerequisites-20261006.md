# Collection prerequisite routes / 收集前置关联

This increment connects runtime persistent-condition clues to collection goals in
both the live editor and standalone reports. All five fingerprints remain **P**
for collection/story dependencies. It does not introduce a bundled task catalog,
a complete quest DAG, an access solver or a shortest-route claim.

## Shared model and workflow

`event_dependencies::Bundle.routes` links each condition report to exact
region/task indices in one immutable plan. A candidate references one guarded
writer in that report. Its `requires` list contains traced unmet conditions;
other unmet/unknown guards remain in `untraced_conditions`. Multiple candidates
are alternatives, while each candidate's guards are conjunctive. Unknown write
values, script stops, unavailable writers and truncated coverage remain qualified.

Potential cycles are reported per alternative. They do not make a goal
impossible: another candidate may have no recursive dependencies. Already-met
conditions remain facts but stop expansion into their writers' prerequisites.
A matched condition is not proof that a quest was completed or that a location
is currently reachable.

The read-only `collection_prerequisites` app request validates the current ROM
fingerprint and computes a fresh SAV snapshot with the existing acquisition/index
rules. The Collection planning page offers **Trace prerequisites**, regional
candidate groups, goal links, recursive-condition jumps, NPC/tile links and static
exterior entrance alternatives. Cards render on expansion and paginate candidates
and related goals. Reference navigation retains this panel's state; a new ROM/SAV
clears its snapshot and discards responses from the old request generation.

HTML keeps goal↔condition links, current condition status and recursive/unknown
notes. Names, script text and locations come from the opened ROM. Reports have
escaped text, no executable script, and no ROM artwork or raw SAV bytes.

Developer CLI (writes JSON to stdout only):

```sh
gen3 collection-plan ROM SAVE QUERY.json
```

`QUERY.json` uses the existing collection request, for example:

```json
{"basis":"individuals","families":true,"include_unknown_rewards":false}
```

## Evidence

- Public route tests cover alternatives, duplicate guards, cycles with a separate
  nonrecursive exit, satisfied guards and unresolved resource prerequisites.
- The app fixture verifies direct/transitive goal references, equal export/query
  snapshots, wrong-fingerprint rejection, cache reuse and ROM/SAV preservation.
  A generated flag-state change verifies that already-met conditions no longer
  propose their writers' prerequisite work. No production flag-edit API is added.
- `local_collection_prerequisite_routes_all_profiles` uses each exact local ROM's
  real guarded acquisition sources, with synthetic SAV state. Final results:

| Fingerprint | Selected goals | Condition routes | Candidate writers | Truncated |
|---|---:|---:|---:|---|
| BW | 3 | 3 | 70 | yes |
| DP | 3 | 3 | 70 | yes |
| Rocket | 3 | 3 | 69 | yes |
| Ultimate | 3 | 3 | 2 | no |
| Mercury 1.2 | 3 | 8 | 28 | no |

  Truncation includes writer pagination and bounded traversal; counts are indexed
  references, not independently playable quests. This increment reuses previously
  verified persistent/script readers; it does not newly execute every writer in-game.
- The fresh CLI read the current Mercury SAV with individual-family planning:
  767 missing goals, 102 regions, 59 condition routes and 390 candidate references.
  The prerequisite bundle was truncated; zero unsupported persistent conditions
  were skipped. Both input file hashes were unchanged. This is read-only planning,
  not edited-SAV emulator validation or proof of all suggested route accessibility.
- Browser verification extends `test_collection_source_facts_ui.py` with synthetic
  five-profile scenarios: alternatives, recursive navigation and reopening,
  pagination, source→map→back state, bilingual 720px layout, escaped HTML links,
  and delayed responses discarded after SAV reload and ROM switch. An additional
  Mercury fixture verifies that a known writer of an already-met condition is
  hidden from both actionable UI candidates and exported HTML. The existing
  query/entrance/back, reward and preparation workflow is also tested separately.

```sh
GEN3_UI_PREREQUISITES=1 GEN3_UI_PENDING=1 \
  uv run --with playwright python scripts/test_collection_source_facts_ui.py
cargo test -p gen3-core prerequisite -- --nocapture
cargo test -p gen3-core local_collection_prerequisite_routes_all_profiles \
  -- --ignored --nocapture
```

Public core regression: 124 passed, 44 local opt-in tests ignored by default.
Workspace/CLI builds, TypeScript, formatting and five release-workflow checks
passed; Vite retains its existing large-chunk advisory. No new emulator game-save
round trip is claimed by these tests.

Native/runtime conditions, actual access, custom script calls, complete weekday
refresh and the full main/side-quest graph still require further work. No version
bump, new package, publishing, SAV edit or ROM write occurs in this increment.

中文：新增“分析前置条件”，把可能解锁条件的事件按区域展示，并关联到收集目标、
NPC 格位和静态入口。候选事件是替代途径，不是全部必做任务；每个候选自身的
条件仍需同时满足。循环不代表目标无法完成，条件已满足不代表任务已完成。
五 ROM 检查使用真实脚本来源与合成存档状态；另有水银真实 SAV 只读规划，输入
未改变。遍历上限与未解析条件均保留提示，仍不宣称完整剧情依赖或当前可达性。
