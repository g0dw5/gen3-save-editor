# Reference cleanup verification / ROM 资料精简验证

Scope: remove the four advanced reference tabs and dedicated implementation,
preserve shared six-tab queries and safe SAV editing, simplify trainer scenarios.
Version remains the existing unreleased version; no application installer or
release was produced for this change. ROM/SAV inputs and screenshots stay local.

本次按新要求调整产品范围：删除事件线索、收集规划、游戏时钟、培育机制四页，
不以隐藏页面或另一个入口保留。旧研究记录不再代表当前产品能力。
已验证的地址与限制转入[逆向地址文档](../research/reference-key-addresses.md)。

## Removed and retained boundaries / 删除与保留边界

Removed UI: the four panels, visited/hidden-page state, tab routes, event-page
jumps, collection HTML/preparation/prerequisite helpers, training individual/
service widgets, orphan entrance-planning helpers, exclusive styles and labels.

Removed backend/CLI: collection planner and breeding-pair planning cache,
training item/service/individual simulation modules, RTC projection implementation,
event-search report builder, dedicated commands and app APIs. Unknown-command
tests verify that calls to all ten withdrawn app commands neither succeed nor
change ROM/SAV bytes. Removed-feature tests/drivers are withdrawn rather than
left executable against nonexistent screens or endpoints.

Retained shared code: map topology/passages, trainer references, acquisition and
reverse evolution resources, receipt-condition readers, per-species daycare
information, virtual-clock/SAV RTC snapshots and encounter-hour filtering,
native trainer generation, individual SAV fields and transactions. Removing
services does not remove already-supported nature/ability/IV/EV SAV editing.

保留六个核心页：宝可梦、招式、道具、特性、地图、对手训练家。技能表、地图训练家／
相遇列表、连接、领取条件和技术证据按需展开；来源列表分批显示，仍可继续加载和
搜索。训练家取消重复地图列表，配队优先显示。原生随机字段保留样例说明。

Ultimate uses one difficulty selector and automatically uses the current SAV
party. No manual roster, random seed or per-mon level overrides remain. Without
a SAV, unresolved enhanced-template IVs/EVs display `?`, not raw template values.
Ordinary fixed native records remain available with ROM alone. The verified
generation path adjusts levels/IVs/EVs; species base stats are a separate ROM table.
Facilities, scripted roster substitution and full live battle state remain outside
this bounded preview; the cleanup does not extend its verification claims.

## Checks / 验证

| Check | Result and scope |
|---|---|
| Public Rust suite | **124 passed**, 42 intentionally ignored local-input cases; parser checks and independent SAV transaction, drag/swap, batch, history, field ownership and integrity tests remain. Removed-only feature tests are not counted as current coverage. |
| Five exact ROM acquisition regression | BW, DP, 西班牙火箭队, 究极绿宝石, 水银 1.2 passed `local_query_acquisition_all_profiles`. Runtime evolution-resource reverse links, actual encounter/map sources, teaching, shops, held-item context and immutable query overlays checked. This does not prove complete map accessibility or every source. |
| Ultimate native trainer tests | `ultimate_ev::tests`: three native execution tests; `ultimate_battle::tests`: two ordinary level/ROM-only tests. Native numeric behavior retained; the synthetic browser tests below do not independently establish numerical parity. |
| Six-tab browser regression | Five adapters switch in one page, English/Chinese and 720-pixel compact layout; no removed tab, manual trainer field or dedicated command is used. Automatic scenarios use the current SAV speed/party and generated levels; no-SAV templates stay unknown. |
| Existing browser regressions | Human-readable references, native trainer stale responses, five-adapter evolution cross-links, item/required-resource/map/back, trades, NPC receipts, wild held odds, adapter switching and editor tab/header/footer preservation passed. No write commands are issued by read-only reference tests. |
| Build/style/package audit | TypeScript and Vite production build, `cargo check --workspace`, formatting and five release-input audit tests passed. Vite retains its existing >500 kB chunk warning. No packaging performed. |

Public reproduction: `cargo test -p gen3-core`, `npm run build`,
`npm run format:check`, `python3 scripts/test_release.py`. With Vite running and
Chrome/Playwright installed, run `scripts/test_reference_scope_ui.py` and the
shared reference/editor UI drivers. Set `GEN3_ROM_BW`, `GEN3_ROM_DP`,
`GEN3_ROM_ROCKET`, `GEN3_ROM_ULTIMATE`, `GEN3_ROM_MERCURY12` to private exact ROMs
for the ignored acquisition test; optional `GEN3_SAVE_*` supplies read-only SAV
overlays. No test binary or extracted game data is a release input.

Private results: `.local/analysis/reference-cleanup-public.txt` and
`.local/analysis/reference-cleanup-20261006/`. These local artifacts are not Git
content. Previous simulator save/reboot evidence remains historical evidence
for the unchanged save writer, not a newly repeated full gameplay test here.
