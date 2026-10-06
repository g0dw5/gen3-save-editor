# Trainer script references / 训练家战斗引用

This increment is a read-only flow: **trainer record → qualified script reference
→ conditions and NPC visibility → static tile → exterior entrance → return**.
The same reference opens its event context, with a link back to the current-ROM
trainer name. Coverage remains **partial for every fingerprint**.

## Native evidence

`scripts/verify_trainer_script_commands.py` executes the loaded ROM's 0x5C handler
in isolated mGBA RAM. Three literal trainer IDs are checked for each type 0–16;
Mercury also checks its hook context byte at values 1 and 2. No user SAV is loaded.
All source files and emulated ROM bytes remain unchanged. The consumed boundary
comes from the native return-to-field script pointer, not the wrapper pointer
written back into the command context.

| Exact ROM | Native executions | Qualified record references | With static coordinates |
|---|---:|---:|---:|
| Dark Phantom BW | 51 | 1,789 | 1,784 |
| Dark Phantom DP | 51 | 1,789 | 1,784 |
| Spanish Rocket 2.1 | 51 | 2,592 | 2,561 |
| Ultimate Emerald 5.5 | 51 | 4,001 | 3,803 |
| Mercury 1.2 | 85 | 2,810 | 2,717 |

The cross-ROM opt-in test compares all **289 native executions** with adapter
boundaries and qualified opponent-field roles, then checks **12,981 emitted
record references** against actual opcodes, literal operands and map roots.
Of these, 12,649 have in-bounds static coordinates. These counts include shared
actors and repeated references; they are not distinct trainers or available battles.
ROM-only checks remain unknown. Index identity and replacement are checked through
the same App. Native vectors and private input files are not release inputs.

Mercury type 10 consumes **20 bytes including the opcode**, rather than the shared
14-byte layout. Types 13–15 also have verified extended boundaries. Mercury types
11/16 do not supply a verified continuation in these contexts and are stopped.
Other four adapters stop types 13–16. Their default native behavior is not treated
as proof these unsupported formats are applicable.

## Semantics and limits

Primary literal record references, rematch base records and opponent setup records
are separate roles. Common types 9/12 do not retain their literal operand as the
opponent in the tested ordinary context; the qualified index omits them. Mercury
has different roles and layouts. The older broad map-reference list remains a
less qualified fallback and is not evidence of final battle selection.

The index retains the branch guards before the command separately from NPC
visibility. SAV checks use a fresh immutable snapshot. Passing guards, visible
actors, ROM references and coordinates do not establish access, defeat state or
battle availability. No completed/available task status is inferred.

Unparsed commands, native battle activation, rematch selection, multi-opponent
setup, script replacements, movement and dynamic layouts remain unresolved.
The constructor preview concerns the selected record/scenario, not a proof of
that particular script's final party. Setup validation covers disposable contexts,
not full battles. No new edited-SAV emulator round trip is claimed here.

## Reproduction

Run the native script with `--mgba-probe` and `--output`, plus the five private
`GEN3_ROM_*` paths. Pass its private JSON as `GEN3_TRAINER_SCRIPT_PROBES` to:

```sh
cargo test -p gen3-core local_trainer_reference_formats_match_native_boundaries_and_current_roots -- --ignored --nocapture
```

Public tests exercise all adapters with shared NPC roots, separate actors,
branch guards, map scripts without coordinates, trigger positions, visibility,
SAV reload, stale/malformed requests, changed dispatch pointers and whole-ROM/SAV
preservation. Browser fixtures cover trainer/event/tile/entrance/back and bilingual
labels, without opening or editing real files.

## 中文说明

战斗引用从当前 ROM 的实际地图脚本读取，不内置训练家或任务目录。原生命令核对
验证的是参数边界和记录关系，不是完整剧情激活；地图格位是静态位置，不证明当前
能进入或对战。再战基础记录、第二对手准备指令与最终实战队伍不能混为一谈。
水银扩展类型 10 的边界已修正，未核实的类型停止并保留证据。读取 SAV 只投影
当前条件，NPC 可见性单独展示，不据此标记任务完成。所有 ROM／SAV 输入保持不变。

Build checks: 118 public core tests pass; 34 private opt-in tests are excluded
from that public count. Four relevant five-ROM opt-in regressions were rerun:
trainer references, event search, native persistent effects and acquisition/
collection/navigation. TypeScript, formatting, editor/reference browser regressions
and five release-workflow checks pass. Strict Clippy with the installed Rust 1.98
still reports six existing warnings in emergency/form/Ultimate helpers and a
legacy world expression; this increment does not claim a clean strict-Clippy run.
