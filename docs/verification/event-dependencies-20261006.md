# Prerequisite clues / 前置线索追查

All five progression/planning rows remain **partial**. A map-referenced script
write is a potential clue, not a verified quest, reachable NPC, activation path or
completion proof. This increment does not construct a complete mainline/sidequest
DAG and does not turn unnamed event IDs into invented task names.

## Workflow and sources

Acquisition / map / collection / daycare prerequisite → expand **Find prerequisite
clues** → inspect potential persistent flag/variable writes, root-script text
references and their guards → follow another guard or a map/tile link → inspect
exterior entrance chains → return to the original reference. ROM-only checks stay
unknown; a SAV overlays its ordinary persistent state. Refresh re-reads the current
saved snapshot. Snapshot revision changes also invalidate open traces even if the
outer condition value is unchanged. A saved check is not emulator live state or facility context.

`event_dependencies.rs` indexes roots from the current ROM's map records,
including NPCs, non-reward coordinate triggers, signs and unplaced map-level
scripts. It does not scan arbitrary unreferenced bytes for supposedly obtainable
events. Map-table membership still does not prove a map is used/reachable in play.
Coordinates outside the static layout are not offered as precise tile targets.
NPC visibility guards are separate from receipt evidence and access. Coordinate
selectors, map-script entry selectors, collision, movement, dynamic layouts and
native/special writes remain unresolved.

The shared bounded script walker observes writes without adding data to its
control-flow state or changing receipt-proof state merging. Persistent ranges
come from verified adapters. Direct writes and known local operands are recorded;
unknown native/standard commands invalidate local knowledge in this observer.
Unresolved arithmetic/copies remain potential writes with no asserted resulting
value. Commands/paths that stop or merge cannot certify complete coverage.

The eight guarded dispatch entries per adapter are necessary engine configuration,
not content catalogs: `setvar`, `addvar`, `subvar`, `copyvar`, `setorcopyvar`,
`setflag`, `clearflag`, and message-context `loadpointer`. Changed targets fail
rather than borrowing another ROM's semantics. Text references are bounded ROM
bank-zero pointer contexts (up to 16 strings per root, 1,024 bytes each), not a
claim that this exact branch speaks those words. Direct/custom message paths are
not exhaustively indexed. Text is displayed/HTML-escaped, never executed.

| Exact fingerprints | Dispatch table | Native write cases | Referenced roots | Checked effect queries |
|---|---:|---:|---:|---:|
| Dark Phantom BW | `1DB67C` | 52 | 3,342 | 2,646 |
| Dark Phantom DP | `1DB67C` | 52 | 3,342 | 2,646 |
| Spanish Rocket 2.1 | `22B218` | 52 | 8,271 | 3,834 |
| Ultimate Emerald 5.5 | `1DB67C` | 100 | 5,345 | 3,412 |
| Mercury 1.2 | `15F9B4` | 104 | 4,806 | 4,142 |

The native oracle executes the real command handlers in independent mGBA ARM7
with synthetic RAM/context operands and no replacements. **360 writes** include
persistent range edges, extension/trainer flag blocks, copy sources, 16-bit wrap
and literals greater than `4000`. Neighbor bytes and ROM inputs are compared.
Five additional bank-zero loadpointer cases confirm the native context field.
Rust reader fixtures use the same operands and compare all resulting values.

This exposed two shared semantic errors: `copyvar` dereferences its source variable
pointer rather than interpreting IDs below `4000` as literals; `subvar` resolves
its operand with native `VarGet`, while `addvar` consumes an immediate. The shared
walker now follows these differences. Invalid variable destination/source pointers
stop the walk. The observation does not execute or write a user's script/SAV.

## HTML, caching and limits

The index is lazy and bound to the loaded ROM Arc, cleared on open-ROM. It stores
no SAV checks; every query projects the current immutable SAV anew. Requests are
fingerprint-guarded and expose no edit action. The app tests successive ROMs through
the same command/cache, including ROM-only unknown checks. The command-writing routines run only
in oracle disposable RAM. Runtime tracing is read-only and reuses the existing
isolated native item-validation helper when resolving resource operands.

Shared UI expands at most four levels and recognizes repeated condition keys.
Cycles are not labelled impossible. Each query pages 64 potential writers. The
index bounds 16,384 distinct referenced roots; each walk bounds 8,192 instructions,
16 return frames and 32 guards. Coverage/failures/truncation remain evidence,
never a completeness claim.

Standalone HTML export queries a new collection plan and dependency appendix in
one backend request against the same current ROM/SAV. Condition links point to
an escaped appendix containing source maps, coordinates, text contexts, further
guards and exterior chains. It bounds 128 conditions, three recursive expansions,
64 writers per condition and 256 entrance maps, with explicit partial/truncation
notes. Unverified persistent IDs are counted as skipped; bag/money conditions keep
their existing explanations. No JavaScript, ROM sprites, raw SAV records, egg,
Pokédex update or story edit is added. Changing ROM/SAV/query settings while export
is pending prevents the old response from being downloaded.

## Verification

- Public fixtures exercise guarded set/clear alternatives, non-reward trigger
  positions, unreferenced byte exclusion, unknown post-native results, changed
  dispatch rejection, per-query SAV refresh, malformed/stale request rejection,
  export/cache equality and whole ROM/SAV preservation.
- Native opt-in verification checks the 360 vectors, all 25,106 roots and 16,680
  sampled effect queries across five fingerprints. This proves bounded parsing
  and map references, not all game activations or a full task graph.
- Bilingual browser fixtures exercise lazy expansion, repeated guards, refreshing
  a snapshot, map/tile/entrance/back, safe ROM text and linked standalone HTML in
  compact windows. No edit/export-SAV action is issued. Open map traces are also checked across a
  synthetic SAV reload with unchanged outer conditions. Flag values use set/unset
  labels; raw IDs and command details remain expandable evidence.
- **116 public core tests pass** (32 opt-in excluded), along with TypeScript/Vite
  production build, format checks and five release-workflow tests. Clippy retains
  six existing warnings; this increment adds none.
- No new edited-SAV emulator round trip is claimed for this read-only increment.

```sh
python3 scripts/verify_event_effects.py --mgba-probe /private/probe.dylib \
  --output /private/event-effects.json
GEN3_EVENT_EFFECT_PROBES=/private/event-effects.json cargo test -p gen3-core \
  local_event_effects_match_native -- --ignored --nocapture
cargo test -p gen3-core
uv run --with playwright python scripts/test_query_collection_ui.py
# JSON: {"kind":"flag","id":11,"value":1,"comparison":1,"taken":true}
gen3 event-dependencies ROM CONDITION.json [SAVE]
```

Use the existing private five-ROM variables. Native probe/vector files, real ROMs,
SAVs and runtime extracted text are excluded from Git/release inputs. Version is
unchanged and this increment is not packaged/published.

本轮贯通“前置条件 → 可能改变它的脚本 → 上下文文字／其他条件 → 地图格位 →
外部入口 → 返回”，并在独立 HTML 中保留条件锚点和附录。五份指纹共同验证，不
内置任务或对话目录。领取证据、条件满足、NPC 可见、地图引用与当前可达仍分开。
整体为部分解析；完整任务树、动态激活、特殊设施状态和原生函数中的剧情写入尚有
缺口。没有找到来源不代表不可完成，循环依赖不代表不可能，不修改用户存档。
