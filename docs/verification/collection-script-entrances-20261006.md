# Guarded collection entrances / 收集规划脚本入口

This increment connects referenced script passages to collection sources,
directed-evolution preparation, prerequisite writers and standalone HTML across
BW/DP/Rocket/Ultimate/Mercury 1.2. All relevant matrix rows remain **P**: an
indexed entrance does not establish current access or a complete task sequence.

## Behavior / 行为

The map window, collection API and read-only CLI share `Index::navigation_graph`.
The index is bound to the loaded ROM's fingerprint **and** immutable input Arc;
SAV observations are rebuilt for each query. No saved eligibility is cached.
The app reuses its existing ROM-bound index; opening another ROM invalidates it.

Each exterior approach is a separate alternative, with source/target map names,
known grid positions, script guards and an activation/access warning. The live
panel initially shows three chains and can expand all retained alternatives;
HTML retains all returned chains. Unknown destination positions stay in a
separate incoming-reference section. Finding an NPC's tile does not establish
that stepping onto it activates a passage. Edges are directed; no return is
fabricated. Bounded searches retain their warnings and coverage evidence.

入口按独立备选路线展示，不将不同路线的条件合成必做清单。NPC 格位用于找人，
不冒充可踩的传送点；目标格位无法确定时单独保留来源。即使已解析条件满足，
仍明确提示触发方式、当前可达性与返程未知。界面与 HTML 使用相同的数据。

Persistent guards on goal entrances are linked to affected collection tasks.
Unmet writer guards and writer-entrance guards are recursively traced with the
existing depth/report/map caps. `requires` describes a writer's own guards;
`entry_requires` keeps each approach separate. Goal association/cycle detection
is a **potential clue graph**, not an AND schedule or proof of impossibility.
Already-satisfied conditions stop writer/entrance expansion. Native/runtime guards
and omitted records remain unknown; no receipt state is inferred from an entrance.
HTML guard links point to traced condition anchors; all ROM text/JSON is escaped,
with a non-executable CSP and no ROM artwork or SAV bytes embedded.

## Evidence / 证据

- Public fixtures cover all five adapters: collection/map approach parity,
  missing/met SAV conditions, guarded interior writers, cycles, met-condition
  expansion stopping, input preservation and stale-ROM rejection.
- An independent route test keeps separate unmet/met/unknown entrance alternatives
  out of the writer's mandatory guard list. The public core suite has **126
  passing tests / 45 opt-in ignored**; the later guarded-writer fixture also passes.
- The existing **918 independent mGBA native operand cases** verify the unchanged
  script-warp decoder, not full map loading or access. See
  [native scope](script-warps-20261006.md).
- `local_collection_prerequisite_routes_all_profiles` reads all five real ROMs
  with synthetic SAV scenarios, compares goal entrances to map-window chains,
  validates per-alternative prerequisite associations and preserves input hashes.

| Exact profile | Selected runtime goals | Condition reports | Candidate writers | Truncated |
|---|---:|---:|---:|---|
| BW | 3 | 18 | 378 | yes |
| DP | 3 | 18 | 378 | yes |
| Rocket | 3 | 3 | 69 | yes |
| Ultimate | 3 | 21 | 342 | yes |
| Mercury 1.2 | 3 | 8 | 28 | no |

These counts describe selected references, not accessible tasks or exhaustive
storyline coverage. Private ROM/SAV files are not repository or release inputs.

A local real Mercury SAV was queried through `gen3 collection-plan`, using
existing-individual/family goals and including unknown rewards. It produced
767 missing-family goals, 116 regions, 452 entrance reports, 753 guarded approach
chains and 14 unresolved incoming references. Script-index coverage was
4806/4806 roots with no failed root walks; **this does not mean every script's
semantics is understood**. The prerequisite appendix reached its 128-report cap
and marked truncation. ROM/SAV SHA-256 hashes were unchanged. The CLI took about
46 seconds locally; this is a reference-data scenario, not an in-game route or
edited-SAV emulator test.

The five-profile bilingual browser fixture passes source/target/map/back,
per-route unmet/unknown/met displays, expanded alternatives, unresolved incoming
references, SAV refresh, HTML escaping and stale ROM/SAV responses. An additional
Mercury fixture confirms HTML guard-anchor links and refreshed collection guards;
it also passes at **900×640**, with the panel later resized to 720 pixels wide.
Captured native-styled panel screenshots were inspected for readable wrapping.
These are synthetic presentation scenarios, not gameplay/access evidence.
Workspace checking, Rust/Prettier/Black formatting, TypeScript/production UI build
and five release-workflow checks pass.

## Reproduction / 复现

```
cargo test -p gen3-core
cargo test -p gen3-core local_collection_prerequisite_routes_all_profiles -- --ignored --nocapture
GEN3_UI_SCRIPT_WARPS=1 GEN3_UI_PREREQUISITES=1 GEN3_UI_PENDING=1 \
  uv run --with playwright python scripts/test_collection_source_facts_ui.py
# Optional compact presentation run: add GEN3_UI_COMPACT=1
gen3 collection-plan ROM SAVE QUERY.json
```

The ignored test requires the five `GEN3_ROM_*` paths. UI fixtures require the
local Vite server and Chrome; data are synthetic, with no real saves or assets.
The query file can contain `{"basis":"individuals","families":true,
"include_unknown_rewards":true}`. Outputs belong in private analysis locations.

## Remaining limits / 剩余边界

Full activation selectors, moving NPCs, collision permission, dynamic layouts,
random facilities, extended/native transitions and complete task dependencies
remain incomplete. Depth is bounded to three prerequisite expansions, 128 reports,
256 writer-map entrance suggestions and the existing directed approach limits.
No global shortest route, current access, automatic Pokémon creation, Pokédex
completion, story editing or new SAV write operation is claimed. This increment
has no new edited-SAV emulator round trip and no package/public release.
