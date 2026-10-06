# Player references and adventure guide / 玩家资料与冒险攻略验证

Developer record only. Excluded from player documentation and application inputs.
ROMs, saves, parsed scene output and screenshots remain in private local analysis.
Version held; no application package or release produced for this increment.

本记录只面向开发者，不附给玩家。数据来自当前精确 ROM 的运行时读取，不新增提取
后的名称、地图、素材或任务目录。原有四个已删除的高级页和专用 API 没有恢复。

## Boundaries

- Move references retain native egg-move offspring and positioned tutors, and
  follow the actual machine item through ordinary pickups, shops and rewards.
  Level/machine/tutor compatibility lists remain on Pokémon pages, with source,
  category, type and name filters and progressive row loading.
- Map encounters group by method, time table and selector. Repeated species
  slots merge only within that group; rates stay conditional on the table.
  Current-time filtering uses the existing ROM-specific saved-clock reader.
- Reserved-item presentation uses runtime references, names and
  native item-handler fields. Maps use known native layout mismatches and the
  decoded incoming topology. Lack of a parsed reference is not proof of absence.
  Hidden entries can be revealed or opened through direct links; SAV-held items
  remain visible. No ROM record is deleted or rewritten.
- Official references supply reviewed identity/comparison captions and stats.
  Native evolution, form-family and battle-transformation tables alone determine
  ancestry. No similar-name link, manual stats-reference override or mapping
  table is exposed in the player page.
- Developer JSON, offsets, actor indices and verification details are removed
  from reference widgets. Cheat verification is excluded from serialized player
  metadata and exported text. Player distribution staging copies only user
  guides, changelog, licenses and attribution notices.

## Guide scope

The separate guide is **partial**, not a complete walkthrough. It reads referenced
scene dialogue, NPC coordinates, reward receipt conditions and candidate setters
of persistent prerequisites. Lists of writers are alternatives, not a required
chronological sequence. Bounds, cycles, unknown guards and map access remain
unresolved. Dialogue can span other choices/stages in the scene; undecoded control
text is omitted rather than presented as a quest instruction.

Rocket's exact verified main-story variable is `0x40f7`. The current local SAV
contains state 19. The referenced scene on map `1-34` includes the matching native
upper-bound branch and a write of state 20; the guide offers this as a next-action
clue. It does not use `current + 1`, mark smaller stages complete, or declare every
condition satisfied merely because the branch exists. ROM and SAV hashes were
unchanged after the read. Other ROMs' overall main-story state remains unverified.

水银目前能搜索已识别的 NPC 奖励、对白和前置候选；这不是全部支线任务的完整树。
领取完成只认已验证的领取标记，既不从背包持有推断，也不从主线序号大小推断。
“已知条件满足”不等同于已经证明当前可达或能实际完成。所有新入口只读。

## Checks

| Check | Result and practical limit |
|---|---|
| Public Rust | **127 passed**, 43 opt-in ignored. Includes state/receipt distinction, current-branch selection, stale-ROM rejection and unchanged SAV transaction/drag/batch/history regressions. |
| Exact-ROM increment | BW, DP, Rocket, Ultimate, Mercury 1.2 passed `local_player_references_and_adventure_are_read_only_across_profiles`. Machine physical sources preserved, no broad compatibility rows, valid task/dependency IDs, immutable ROM/SAV overlays and shared ROM-bound cache identity. Optional current SAVs used for BW, Rocket and Mercury. |
| Native graph regression | Ordinary ancestry and native form membership stay separate; name similarity/dex-number collisions do not create ancestry. Existing synthetic core cases and the shared graph/UI drivers pass. |
| UI | Five ROM-scoped English/Chinese scenarios pass filters, compact time tables, hidden/reveal lists, no raw traces, illustrated guide, receipt/search/dependency/back and map focus. Native data correctness is checked separately; synthetic browser fixtures are not game evidence. |
| Existing flows | Configured direct/comparison/none references, same-page five-adapter switching, trainer scenarios, resource→item→map/back, cheat code copy/export/race isolation and editor tab/fixed action area passed. |
| Build and staging | TypeScript/Vite, formatting, workspace/tests check and five release audits pass. Existing Vite chunk-size warning remains. No installer or app was produced. |

Native private inputs produced these **scene/prerequisite candidates**, not these
numbers of proven obtainable tasks or complete quest coverage:

| ROM | Runtime candidates | Main-state transitions | Linked prerequisite groups | Tested machine source routes | Hidden items | Hidden maps |
|---|---:|---:|---:|---:|---:|---:|
| BW | 523 | 0 | 360 | 10 | 10 | 65 |
| DP | 523 | 0 | 360 | 10 | 10 | 65 |
| Rocket | 1393 | 291 | 1290 | 11 | 40 | 74 |
| Ultimate | 893 | 0 | 930 | 75 | 31 | 78 |
| Mercury 1.2 | 1363 | 0 | 739 | 13 | 0 | 26 |

Mercury contained no safely classified nonzero reserved item; no item was forced
out merely to make the filter appear effective. Counts of hidden maps are an
uncertainty/readability filter, not certification of unreachable maps.

## Reproduction

Run `cargo test -p gen3-core`, `npm run build`, `npm run format:check`,
`python3 scripts/test_release.py` and `node scripts/test_evolution_graph.mjs`.
With Vite, Chrome and Playwright, run `scripts/test_player_references_ui.py`,
`test_configured_reference_ui.py`, `test_evolution_reference_ui.py`,
`test_reference_scope_ui.py`, `test_resource_conditions_ui.py`,
`test_cheats_ui.py` and `test_editor_navigation.py`.

The ignored exact-ROM case requires `GEN3_ROM_BW`, `GEN3_ROM_DP`,
`GEN3_ROM_ROCKET`, `GEN3_ROM_ULTIMATE`, `GEN3_ROM_MERCURY12`; optional
`GEN3_SAVE_*` inputs supply read-only overlays. Local logs are under
`.local/analysis/player-reference-*`. No private binary or extracted asset is
committed. The save writer is unchanged; earlier emulator save/reboot evidence
remains bounded historical evidence, not a newly repeated gameplay test here.
