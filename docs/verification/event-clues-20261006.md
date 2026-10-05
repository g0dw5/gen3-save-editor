# Searchable event clues / 可搜索事件线索

This increment adds a shared, read-only flow across the five registered
fingerprints: **ROM reference → Event clues → search ROM text or map name →
map-filtered reference → conditions/prerequisite trace → exact static tile →
exterior entrance → back**. A map page can open the same map-filtered search.
Return preserves the search, map filter, selection and page. All event/story
coverage remains **partial**, not a complete mainline/sidequest dependency graph.

## Content and identity

The lazy `event_dependencies::Index` retains its current-ROM map-root references.
Search reads the existing bounded bank-zero text contexts and persistent effects;
there is no extracted dialogue, task-name or map catalog. Referenced NPCs, tile
triggers, signs and map scripts remain distinct, including actors sharing a root.
Map-level roots sharing one map header also have distinct identities: map ID,
reference kind, source offset **and script root**. Valid-looking unreferenced bytes
are excluded. Out-of-layout coordinates have no precise-tile link.

A response pages 32 references and includes at most 64 potential writes per
reference with truncation disclosed. A selected ID may be resolved outside the
current page, but only inside the current text/map filter. Text search is bounded
to 512 UTF-8 bytes; map/selected IDs are bounded. The existing root, text and script
walk bounds are documented in [prerequisite tracing](event-dependencies-20261006.md).
Empty traces with neither readable text nor effects are not invented tasks.
The coverage object concerns parsed roots, not exhaustive searchable game dialogue.

The command rejects stale ROM fingerprints, wrong-ROM index identity, malformed
payloads and invalid offsets. Indices contain no saved progress. Every query
projects the current immutable SAV afresh. UI requests are debounced and cancelled
on changed query/ROM/SAV revision, so old responses cannot replace current results.
No ROM bytes, SAVE bytes, Pokédex records or story flags are written.

## Progress semantics

- NPC visibility is displayed separately from branch prerequisites.
- Each potential write has its own guarded conditions, using the same human
  labels, required-item links and prerequisite tracing as other reference pages.
- An **observed match** compares a known write result to a saved flag/variable.
  It is not receipt evidence: another event could write the same value.
- ROM-only and unresolved values remain undetermined. Matching values, passed
  guards, map references and NPC visibility never establish task completion or
  current access. There is intentionally no fabricated `completed`/`available`
  task status derived from these writes.

The page explicitly states task completion is undetermined. Root text is context,
not a quest title or proof this branch delivers that dialogue. Native/custom
messages, custom writes, entry selectors, dynamic activation/layout, facilities,
movement and exact storyline progression remain unresolved. A missing search
result does not establish absence. Recurring resets remain outside this evidence.

## Evidence

| Exact ROM | Searchable references checked |
|---|---:|
| Dark Phantom BW | 3,885 |
| Dark Phantom DP | 3,885 |
| Spanish Rocket 2.1 | 8,976 |
| Ultimate Emerald 5.5 | 5,971 |
| Mercury 1.2 | 5,605 |

The five-ROM opt-in test checks **all 28,322 emitted references** against current
map names, event-table script pointers (or map roots), static bounds and current
ROM string pointers. It checks unique reference identities, search/detail lookup,
ROM-only unknown observations and cache replacement through the same App.
A real Mercury SAV overlay keeps both file bytes and in-memory bytes unchanged.
This is reference evidence, not emulator proof of accessibility or completed tasks.

Public fixtures exercise all five adapter configurations: 70 actors sharing one
root, multiple map roots sharing a header, text-only scripts, unreferenced bytes,
pagination, off-page selection, branch guards, hidden actors, explicit synthetic
snapshot changes, arithmetic/write bounds, stale/malformed command rejection and
whole ROM/SAV preservation. The prior independent **360 native command vectors**
and 16,680 sampled prerequisite queries are rechecked; search does not change
command semantics or the shared walk state.

Bilingual Playwright fixtures check dialogue search, exact map filtering, page
selection, escaped `<script>` text, unknown task status, prerequisite expansion,
map/tile/entrance/back, returned search/filter/selection/page, snapshot reload,
no-results recovery and map-to-clue navigation. Visual inspection exposed floating windows remaining
offscreen after viewport resize; shared floating windows now clamp to actual
dimensions on viewport/native size changes and drag. A compact-window boundary
assertion checks all edges, not merely element visibility. They also retain the existing
query, collection, breeding, safe standalone HTML and compact-window flows.
The query/clue fixture never issues an edit/export-SAV action. The separate
reference-tab/late-response fixture and synthetic editor tab/draft/multi-field
fixture also pass after the shared floating-window change. The older reference
fixture now uses the same 30-second asynchronous assertion budget as the query
fixture; its prior 5-second wait expired before synthetic details appeared.
Its ID, matching-detail and obsolete-response assertions are unchanged. Text is React-rendered, not HTML.

117 public core tests pass (33 opt-in ignored); TypeScript/Vite production build
and formatting pass. Clippy retains six earlier warnings and adds none.
The developer CLI `event-search ROM QUERY.json [SAVE]` shares the same reader.
Its real Mercury query returned 32 of 5,605 references, rejected a stale fingerprint,
and preserved both ROM/SAV SHA-256 hashes.
No new edited-SAV in-game save roundtrip is claimed for this read-only increment.
Private logs, screenshots, ROMs and SAVs stay excluded from Git and bundles.

```sh
cargo test -p gen3-core
# QUERY.json: {"search":"your ROM dialogue", "map_id":"0-0"}
gen3 event-search ROM QUERY.json [SAVE]
cargo test -p gen3-core local_event_clue_search_cross_rom_queries -- --ignored --nocapture
# Existing five-ROM variables and private native command vector file:
GEN3_EVENT_EFFECT_PROBES=/private/native.json cargo test -p gen3-core \
  local_event_effects_match_native -- --ignored --nocapture
uv run --with playwright python scripts/test_query_collection_ui.py
```

本轮新增“事件线索”搜索，以当前 ROM 文字和地图引用为起点，可追查条件并跳转
地图／外部入口，返回后恢复搜索、地图筛选、条目和分页。NPC 可见性、写入结果与
存档一致、任务已完成、当前可执行始终分开；不从标记值拼造已完成任务。五份指纹
共核对 28,322 条可搜索引用，整体仍是部分解析。没有完整任务树或动态可达证明。
版本号保持不变；本轮未打包、未发布。此前交付的本地 App 不含本次增量。
