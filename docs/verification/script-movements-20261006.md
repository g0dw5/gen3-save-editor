# Referenced actor actions / 地图角色行动引用

This increment identifies native actor-action and waiting commands in referenced
map scripts for all five exact fingerprints. It makes the map's initial-tile
boundary visible; it does not simulate arbitrary action bodies or certify current
NPC positions, completion, collision, map entry or reachability.

## Native boundary / 原生边界

| ROM | MD5 | apply / apply-at / wait / wait-at handlers (ROM offsets) |
| --- | --- | --- |
| Dark Phantom BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | `0x9A5E8`, `0x9A62C`, `0x9A698`, `0x9A6EC` |
| Dark Phantom DP | `cb2940215f4dafb1bef133c3af379f44` | Same offsets, independently tested |
| Spanish Rocket | `59c658a1081f542086de1060bb65f0b3` | `0xD0530`, `0xD0574`, `0xD05E0`, `0xD0634` |
| Ultimate Emerald 5.5 | `17ce9785b33319b3dbda9a5d37c57ec1` | `0x9A5E8`, `0x9A62C`, `0x9A698`, `0x9A6EC`, independently tested |
| Mercury 1.2 | `f323df1792ac68462a34b42fe8571533` | `0x6B200`, `0x6B244`, `0x6B2B0`, `0x6B304` |

Commands `0x4F`, `0x50`, `0x51`, `0x52` consume 7, 9, 3, 5 bytes respectively,
including the opcode. The actor operand goes through native VarGet; the engine's
actor lookup consumes its low byte. Apply stores the original resolved halfword
as its last actor. Wait with zero retains that live last-actor state; it is not
an instruction to move NPC #0. The two *-at forms include explicit map-group and
map-number bytes. They refer to an actor on that map; they are **not warps or
navigation edges**. The ordinary forms use the live current-map state.

Native apply returns without awaiting background motion. Native wait advances
the script pointer, installs a completion callback and switches the context to
waiting mode. The tested callback returns 0 while its matching task is pending
and 1 after the native task consumes an end marker; missing-actor/task cases can
also finish. Consequently, reading an instruction or a pointer is insufficient
to claim the player can reach the next reward or that its actor has arrived.

## Shared reader and UI / 共用解析与界面

The reader checks each current ROM's native command-table binding against the
adapter's verified handlers and records actor references, explicit map operands,
ROM movement-script pointers and branch guards. Unknown variables remain unknown;
RAM-only movement bodies are not presented as ROM offsets. Referenced content is
read at runtime; there is no bundled movement/script/location catalog.

Map markers retain **initial ROM coordinates** and include `scripted_movements`.
Map-level referenced actions appear separately as `unplaced_movements`; they do
not acquire invented tiles. Movement-only tile events are retained as map events.
The selected marker shows a Chinese/English explanation, with raw action evidence
inside the existing expandable details. A map-level action section provides the
same initial-position boundary. Source → map tile → entrance → return navigation
uses the existing shared interface and remains read-only.

此事件含角色行动或等待指令时，地图小人的位置仍是 ROM 初始格位，界面明确提示
实际位置、动作完成和碰撞条件要在游戏内确认。地图级行动单独展示，不凭脚本
编号猜格位；带明确地图参数的行动也不会被误当成传送入口。变量未解出的角色
编号保持未知，等待编号零不会被误解为“NPC 0”。

The bounded native fixture does not prove that every arbitrary background action
preserves story/script data. The walker therefore retains its conservative
variable/resource invalidation and marks these paths partial; movement commands
cannot qualify a complete receipt proof merely from their widths. This increment
adds actionable map context, not an automatic current-position projection or a
new source of completed/available task assertions.

本次不放宽动作后的数据失效与领取证明边界，不把异步任务当成无操作。带动作的
路径保留部分解析提示；没有根据这些测试将完整剧情或当前可达性升级为已验证。
ROM 和原始 SAV 均保持不变。

## Evidence / 验证

- `scripts/verify_script_movements.py`: **640 complete native lifecycle
  scenarios**, 128 per exact ROM. Both apply forms and both wait forms, active
  and absent object fixtures, literal/variable operands, low-byte actor handling,
  special result-variable input and high-byte map operands are covered.
- Complete native apply, wait setup, pending completion callback and native
  end-only task/completion calls run on mGBA ARM7. No instruction/function is
  replaced or skipped inside these calls. Native object/task addresses are derived
  from the loaded executable. Script pointers, callback/mode and cached comparison
  are checked; all 16 neighboring special variables and whole SaveBlock1/2 bytes
  remain unchanged in these fixtures. Original ROM SHA-256 is checked in `finally`.
  No SAV is opened. This is an **end-only task fixture**, not walking animation,
  collision, an arbitrary action-body or a full-frame overworld proof.
- `local_script_movement_operands_match_native_dispatch_and_lifecycle` compares
  production operand/dispatch reads against every recorded scenario and fingerprint.
  Public all-adapter tests preserve initial marker coordinates, separate map-level
  actions, last-actor reuse and unknown RAM bodies; reject wrong dispatch bindings;
  retain unknown gift receipt and avoid fabricating navigation edges.
- Five-fingerprint actual-ROM acquisition/collection regression passes. Shared
  browser fixtures follow an item source to its map target, check Chinese/English
  initial-position notices and map-level actions, then retain entrance/back,
  prerequisite refresh, collection and safe HTML workflows. Synthetic fixtures
  open no real files and send no save mutation.
- Public core suite: **135 passed**, **52 opt-in tests ignored** by default.
  Formatting, TypeScript/Vite, desktop compilation and release-workflow checks
  accompany the increment. Version stays held; no package or public release.

## Reproduce / 复现

Supply the five private `GEN3_ROM_*` paths. Build the existing ARM7 probe from
`scripts/native/breeding_probe.c`; see other native verifier build instructions.

```sh
python3 scripts/verify_script_movements.py --mgba-probe /path/native.dylib \
  --output /fresh/native-movements.json
GEN3_MOVEMENT_PROBES=/fresh/native-movements.json cargo test -p gen3-core \
  local_script_movement_operands_match_native_dispatch_and_lifecycle -- --ignored
```

Run Vite and `scripts/test_query_collection_ui.py` with Playwright/Chrome for the
synthetic bilingual navigation workflow. Private output is evidence, never a
release input or a substitute for runtime ROM data.
