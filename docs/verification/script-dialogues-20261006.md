# Script dataflow through field dialogue / 对话后的入口坐标

The bounded script walker discarded temporary coordinates after `callstd`,
close-message and delay commands. Referenced maze scripts assign coordinates
before a yes/no prompt and use them in a later warp. Those coordinates were
therefore shown as unresolved even though the script contains their constants.
This affected map references and their shared collection/HTML entrance graph.

问题不是从 SAV 中漏读坐标：这些脚本使用临时变量，先赋固定值，再询问玩家是否
传送。旧解析器在对话后清空了已知值。此次保留经原生验证未被改写的参数；不从
SAV 猜临时变量，不把尚未作出的选择标成已满足。

## Runtime behavior / 实现

`script_presentation.rs` validates native command dispatch against adapter
addresses, reads standard table bounds from native literals and reads the actual
standard body from the current ROM. Only short linear bodies consisting of
verified message, wait, close, delay, yes/no and return instructions are summarized.
Unknown native calls, writes, branches, malformed pointers and overlong bodies
retain conservative invalidation. No body, name, location list or image is bundled.

Native yes/no creation writes `VAR_RESULT = 255`; selection writes 1 for yes,
0 for no or B cancellation. An existing menu can leave the result unchanged.
The static query does not assume any of those live outcomes. It invalidates the
result's known value/predicate while retaining unrelated coordinates/resources
and previously computed script comparisons. Dialogue does not establish receipt
state or current access. Unknown standard bodies no longer qualify a receipt
proof solely because they use a familiar presentation index.

An out-of-range ordinary standard index is a native no-op after consuming its
operand: `gotoStd` must continue rather than return from the caller. A valid
linear standard body ending in return follows the existing caller stack.

Ultimate's native standard-call dispatcher has a message-preparation gate.
It admits indices through an inclusive native limit (currently 11), while the
ordinary standard-jump dispatcher uses table-end bounds (currently 11 entries).
The gate prepares selected message indices before entering their bodies. The
adapter reads its native compare operands and excludes those additional effects
from summaries. It does not use another ROM's count/semantics or assume all
standard-call implementations are identical. Mercury's custom message dispatcher
is also verified separately.

究极绿宝石的额外分派是适配规则，不是内置对话目录。常量坐标变得可读，不代表
玩家可进入对应区域、NPC 当前可见、传送可触发或可以原路返回。未知选择、完整
地图切换、背景任务及完整现场交互仍未验证。

## Evidence / 证据

`verify_standard_dialogues.py` runs unmodified native functions in mGBA ARM7
with disposable synthetic RAM, using each exact fingerprint. It opens no SAV,
replaces no native functions, skips no instructions within a function call and
writes no ROM bytes. It checks all 16 neighboring special variables, persistent
flag/variable ranges and cached comparisons in the presentation fixtures.

- **2,560 native standard dispatch cases:** both opcodes, all 256 indices, all
  five ROMs. Native valid calls push/return to the expected caller; invalid
  indices consume their operand without jumping. The extra expanded gate is
  exercised rather than treated as ordinary table bounds.
- Five complete message/wait/close/yes-no/delay command calls per profile;
  30 native delay-callback ticks per profile finish at the expected frame.
- Four complete native yes/no callbacks per profile: waiting, A on yes, A on
  no, and B. Synthetic key/menu/task RAM drives the actual callback and input/
  cleanup routines. Another fixture confirms existing-menu preservation.
- ROM hashes and emulated ROM bytes remain unchanged. These are native field
  effects, not screenshots, complete text rendering, a full NPC transaction or
  arbitrary asynchronous task scheduling.

Public fixtures run all adapters with a relocated runtime table and a prompt
in slot 7. They cover known coordinates after a prompt, both unknown choice
branches, preserved prior flag comparisons, standard jump/return versus invalid
no-op, item/quantity operands, absent receipt proof, changed body pointers,
unknown bodies/dispatches, stale-index rejection and unchanged ROM/SAV inputs.

The opt-in native-parity test reads each real ROM and the independent CPU vectors,
checks adapter dispatch and standard classification, then scans only referenced
map script edges. Coordinate values remain the same when a synthetic SAV is
added; runtime choices and access remain unknown.

| Exact profile | Referenced script edges before → after | Variable-coordinate references | Unresolved before → after |
|---|---:|---:|---:|
| BW | 8,735 → 8,736 | 12 | 12 → 0 |
| DP | 8,735 → 8,736 | 12 | 12 → 0 |
| Rocket | 7,036 → 7,036 | 12 | 12 → 0 |
| Ultimate | 15,727 → 15,727 | 12 | 12 → 0 |
| Mercury 1.2 | 4,383 → 4,383 | 0 | 0 → 0 |

Multiple roots can reference the same instruction; **48 references do not mean
48 physical doors**. The extra BW/DP reference has not been independently
confirmed as a live transition. Counts are a runtime query survey, not a shipped map
catalog, complete transition audit or reachability assertion.

The public core suite passes **127 tests / 46 opt-in ignored**. The five-profile
collection/prerequisite route regression remains bounded and passes with
synthetic SAV scenarios. Five-profile bilingual collection/map/back/HTML
fixtures pass, including pending ROM/SAV responses, marker focus, independent
route conditions and refreshed SAV guards. Workspace and desktop checks,
production TypeScript/Vite build, formatting and five release-workflow checks
pass separately. All map/acquisition/story/collection matrix rows remain P.
No new SAV mutation, edited-SAV emulator round trip, version bump or package is
part of this increment.

## Reproduction / 复现

```
python scripts/verify_standard_dialogues.py --mgba-probe PROBE.dylib --output PRIVATE.json
GEN3_DIALOGUE_PROBES=PRIVATE.json cargo test -p gen3-core \
  local_standard_dialogues_match_native_and_resolve_referenced_coordinates -- --ignored --nocapture
cargo test -p gen3-core
cargo test -p gen3-core local_collection_prerequisite_routes_all_profiles -- --ignored --nocapture
GEN3_UI_SCRIPT_WARPS=1 GEN3_UI_PREREQUISITES=1 GEN3_UI_PENDING=1 \
  uv run --with playwright python scripts/test_collection_source_facts_ui.py
```

Native runs require all five `GEN3_ROM_*` paths and the probe built from
`scripts/native/breeding_probe.c`; use a Python architecture matching that
library. Real files and private vectors are excluded from Git and release inputs.
UI fixtures require the local Vite server and Chrome and contain synthetic data.
