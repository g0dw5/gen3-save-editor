# Native player conditions / 原生玩家条件

This increment covers the five exact fingerprints in the
[capability matrix](../capability-matrix.md). All ROM inputs remain read-only;
no SAV is opened by the native verifier. Version and release status are unchanged.

## Problem and behavior / 问题与行为

The script walker treated `checkplayergender` as an unknown, potentially mutating
native operation. That discarded unrelated assigned values, resource checks and
the cached comparison, and subsequent branches showed an unnamed temporary
variable rather than a player condition.

Each loaded ROM's dispatcher and complete Thumb reader are now validated. The
reader loads SaveBlock2, reads byte 8 and writes VAR_RESULT. Its literals must
match that adapter's verified SaveBlock2 and result pointers. The offset comes
from the validated load instruction; a changed dispatcher, executable or literal
keeps conservative unknown behavior. Only VAR_RESULT's prior value/origin is
replaced. Previously cached comparisons, unrelated assignments and checks remain.

Copied results retain their origin through `copyvar` and `setorcopyvar`. All six
comparisons, reversed operands, conditional jumps and conditional calls produce
player-gender conditions. The value is a **raw byte**, not a boolean: corrupt or
nonstandard values are not silently interpreted as boy/girl. Overwriting or
arithmetic invalidates the alias; unsupported commands retain existing unknown
boundaries.

ROM-only queries show the condition without inventing its result. SAV overlays
read the trainer block through the same verified rule. Shared condition rendering
shows boy/girl in English and 男孩／女孩 in Chinese, including negated conditions
and the current saved value. Acquisition, map details, preparation/prerequisites,
collection planning and standalone HTML use the same renderer. This does not
modify player identity or add story/receipt flag editing.

原先的性别读取会被当作未知指令，导致条件显示不清楚，并丢掉无关的临时数据。
现逐 ROM 核对完整原生读取例程，保留复制后的来源与六种比较；仅读 ROM 不代入
玩家身份，载入 SAV 后读取存档性别。技术证据保留原始字节，界面正常值显示为
男孩／女孩。例程不匹配、未知调用、未确认的进入条件仍保留未知。

## Evidence / 验证证据

- `scripts/verify_player_conditions.py` calls unmodified complete native readers,
  copy operations, comparisons and jump/call handlers on mGBA's ARM7 core, with
  disposable script contexts/RAM and no hooks or ROM writes. Across all five
  fingerprints: **1,280 raw-byte reads**, **40 copies**, **2,400 branches**.
  Reads cover all 256 byte values. Branches cover 0/1/2/255, five RHS values,
  all six selectors, both operand orders and both jump/call forms. Script
  contexts, persistent RAM and unrelated temporary variables are checked.
  The test SaveBlock2 is relocated away from the native temporary-variable
  area; an earlier overlapping Rocket fixture was corrected before accepting
  evidence. All source hashes are checked again in `finally`.
- `local_player_conditions_match_complete_native_reads_copies_and_branches`
  compares production saved-byte reads and emitted conditions with those
  independent native results, verifying exact ROM MD5/SHA-256 and addresses.
- The public fixture exercises 1,200 generated branch scripts / 4,800 saved-value
  projections across five adapters, aliases, preserved prior comparisons,
  overwritten origins and negative executable/literal controls.
- `scripts/test_resource_conditions_ui.py` verifies acquisition -> required item
  -> map detail -> back, bilingual collection conditions and escaped standalone
  HTML. Its synthetic read-only API fixture follows the current fingerprint-bound
  `collection_export` contract. No save action/export is issued.
- The final public suite passes **129 tests**; **48** actual-ROM opt-in tests are
  ignored by default. TypeScript/Vite build, formatting, desktop check and release
  workflow checks are recorded separately.

A read-only runtime search of referenced map roots finds these entries containing
at least one player-gender condition after parsing:

| Exact adapter | Search entries | Distinct referenced script roots |
| --- | ---: | ---: |
| Dark Phantom BW | 79 | 74 |
| Dark Phantom DP | 79 | 74 |
| Spanish Rocket | 68 | 64 |
| Ultimate Emerald 5.5 | 84 | 79 |
| Mercury 1.2 | 25 | 21 |

Counts include shared roots and effect/battle pagination. They are **not counts
of available rewards, distinct NPCs or complete events**. Examples are decoded
from the current ROM in the private survey, including Mercury's New Bark Town
scripts; none of the names/scripts/results is bundled as a catalog. Many such
roots still encounter battles, special/native calls or other unknown commands.
Ultimate retains a stopped position whose byte is A0; this survey does not
establish its cause or remove that stop.

## Reproduction / 复现

Provide the five `GEN3_ROM_*` inputs privately, then run with an independently
built `scripts/native/breeding_probe.c` library and a fresh JSON destination:

```sh
/usr/bin/python3 scripts/verify_player_conditions.py \
  --mgba-probe /private/path/native-probe.dylib \
  --output /private/path/player-conditions.json
GEN3_PLAYER_CONDITION_PROBES=/private/path/player-conditions.json \
  cargo test -p gen3-core local_player_conditions_match_complete_native \
  -- --ignored --nocapture
uv run --with playwright python scripts/test_resource_conditions_ui.py
```

## Limits / 边界

This verifies one reader and its dataflow, not full story completion, dialogue
access, changing player gender in game, all native/special effects, receipt
transactions or dynamic trainer construction. A satisfied saved condition does
not prove current reachability. No additional task catalog, ROM assets or save
fixture is shipped. Broader acquisition/story/map rows remain **partial**.

本轮不宣称完整剧情、所有奖励或当前可达性已验证；性别条件满足不代表其他条件满足。
保留四款游戏的部分解析范围，不把指令原生对照扩展成完整现场交互的证明。
