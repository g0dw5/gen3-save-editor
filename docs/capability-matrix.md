# Capability audit / 实际能力审计

Audit baseline: 2026-10-05, `f6f347a`. The table includes the increments recorded below and describes tested scope, not
promises implied by an API, readable table or working screen. Local binaries,
saves and extracted artwork are excluded from Git and releases.

审计基线：2026-10-05，`f6f347a`。接口存在、能读取表、页面能打开不代表完整支持。
ROM 和 SAV 仅用于本地验证。矩阵已包含下文增量，证据在本文后续记录中追加。

## Exact supported inputs / 精确支持范围

| Game / 游戏 | MD5 |
|---|---|
| Dark Phantom / 漆黑的魅影 BW | `0d9b129f7dd76895f79bb47ad7dec2fe` |
| Dark Phantom / 漆黑的魅影 DP | `cb2940215f4dafb1bef133c3af379f44` |
| Team Rocket / 西班牙火箭队 2.1 汉化 | `59c658a1081f542086de1060bb65f0b3` |
| Ultimate Emerald / 究极绿宝石 5.5 | `17ce9785b33319b3dbda9a5d37c57ec1` |
| Mercury / 宝可梦水银 FC 1.2 | `f323df1792ac68462a34b42fe8571533` |

Mercury 1.1 is rejected. Fingerprints establish input identity, not feature completeness.
水银 1.1 不再接受；指纹校验确认输入身份，不代表所有功能已经验证。

## Status / 状态

**V 已验证 / verified**: the stated bounded behavior has direct evidence.
**P 部分解析 / partial**: usable records exist but coverage or semantics are incomplete.
**U 待验证 / pending**: no adequate implementation/evidence for the requested behavior.
**N 本作不存在 / absent**: requires native evidence of absence; none is inferred merely
from missing code. A P row can contain individually verified subfeatures.

| Requested workflow / 功能 | BW | DP | Rocket | Ultimate | Mercury 1.2 | Evidence and limit / 证据与边界 |
|---|---|---|---|---|---|---|
| Map layouts, NPC art / 地图与 NPC | P | P | P | P | P | `graphics.rs`, `map_events.rs`, [map palettes](research/map-palettes.md), [Rocket maps](research/rocket-map-and-trainer-display.md), [Mercury](research/mercury-fc-11.md). Static initial layouts; no live movement/layout replacement simulation. |
| Entrances, connections, current reachability / 入口与可达性 | P | P | P | P | P | `navigation.rs`, shared map UI and five exact-ROM tests. Map windows now combine static links with [referenced script passages](verification/script-warps-20261006.md), guard overlays, source/target focus and return. 918 native operand cases cover the decoded handlers; activation, full map loading, dynamic destinations and current access remain unresolved. Collection UI/HTML now shares guarded script entrances; see [integration evidence](verification/collection-script-entrances-20261006.md). |
| Pickup, hidden items, dialogue rewards / 拾取与奖励 | P | P | P | P | P | [map events](research/map-events.md): BW/DP 707 maps, 122 balls, 112 hidden items. Bounded scripts retain stop evidence; ordinary/hidden and qualified NPC receipt protocols have five-fingerprint evidence below. Custom commands, complete resource/runtime guards and access remain partial. |
| Random encounters, probability, time tables / 随机相遇 | P | P | P | P | P | `world.rs`; [time selection](research/encounter-time-selection.md), Mercury native selector. Slot weights are separate from encounter frequency; no full weekday/clock-state validation. |
| Static, gifted, traded Pokémon / 定点、赠送、交换 | P | P | P | P | P | Shared typed map-script sources and [bounded NPC trade quotes](verification/npc-trades-20261006.md); [Rocket reward egg](research/rocket-reward-egg.md). Native calls, roamers, actual delivery and custom exchanges are not comprehensively indexed. |
| Breeding, evolution, form rules / 孵蛋进化形态 | P | P | P | P | P | `rom.rs`, `forms.rs`, `relations.rs`; [evolution tree](research/runtime-data-and-evolution-tree.md). [Native ordinary daycare scenarios](verification/breeding-20261006.md) execute compatibility/full receipt across five fingerprints with stored/simulated parents and offspring/NPC/map navigation. [Ordinary production checks](verification/breeding-production-20261006.md) execute native step/item branches with explicit bag scenarios. Rocket/Mercury service references and complete setup/inheritance/hatching/access remain unknown. Egg-group candidates are not proof of all incense/baby/parent requirements. Permanent evolution and battle forms have separate readers. |
| Item shops, wild held items, teaching sources / 道具与培育来源 | P | P | P | P | P | `acquisition.rs` reads held-item fields, reverse learnsets and bounded shop scripts at runtime, with map/target links. Custom shops and full receipt/condition semantics remain partial. Native ordinary single-wild item selection has five-fingerprint evidence, including tested lead abilities and exceptional layout branches; full encounter modifiers and facilities remain unresolved. See [wild-item evidence](verification/wild-held-20261006.md). |
| Nature, ability, EV and crown services / 性格特性努力值与王冠 | P | P | P | P | P | Shared read-only native training previews with current-ROM item/source/map links. Ordinary EV effects cover all five; bounded mint effects cover Rocket; bounded ability guards/effects cover Rocket, Mercury and Ultimate. Complete menus/consumption and most NPC services remain unresolved; Ultimate and Mercury now have positioned, qualified [crown-service references](verification/training-crowns-20261006.md). See [ability evidence](verification/training-abilities-20261006.md). |
| SAV receipt flags, story dependencies / 领取状态与剧情依赖 | P | P | P | P | P | [Persistent ranges and hidden protocols](verification/event-state-20261005.md) and [qualified ordinary pickup protocols](verification/pickup-receipts-20261005.md) verified across five fingerprints. [Bounded NPC success/receipt protocols](verification/npc-receipts-20261006.md) cover 80 reward rows. The [prerequisite trace](verification/event-dependencies-20261006.md) links potential referenced writers, guards, ROM text and maps to queries/HTML; it is not a full task DAG. Compound/custom pickups, unqualified NPC protocols and complete story dependencies remain unknown. NPC visibility and bag absence are not receipt evidence. |
| Item/money prerequisites / 道具与金钱前置条件 | P | P | P | P | P | [Native holdings and Boolean branch verification](verification/resource-conditions-20261006.md), required-item cross-links, readable query/map/planning/HTML conditions. Mercury ordinary checks use the first matching slot; other four retain unknown facility-bag context. Spending, complete eligibility and access remain unverified. |
| Effective game clock, weekday, next event / 有效时钟与刷新 | P | P | P | P | P | Mercury virtual SAVE clock, native weekday, speed, forced-night state and jump menu verified within [bounded scope](verification/mercury-clock-20261005.md); [Native RTC scenarios](verification/hardware-clock-20261006.md) verify offset subtraction for four hardware-clock profiles; actual RTC and complete weekday refresh remain unresolved. No device-time assumption. |
| Ordinary trainer construction / 普通训练家实战值 | P | P | P | P | P | [trainer generation](research/trainer-search-and-generation.md): BW/DP 20 parties, 106 mons match native constructor; Rocket expanded EV fields and random ability/gender are distinguished. Mercury executes its native constructor for explicit zero-context scenarios; independent CPU comparisons are recorded below, while full setup remains unknown. |
| Difficulty/player-dependent trainer generation / 难度动态队伍 | U | U | U | P | U | [Ultimate](research/ultimate-emerald-55.md), `ultimate_ev.rs`, `ultimate_battle.rs`: bounded native execution with party/scenario. Script overrides and full facilities are not exhaustively replayed. U does not assert other games lack difficulty mechanics. |
| Edit, drag/swap, batch, undo/export safety / 编辑事务 | V | V | V | V | V | `session.rs`, [save integrity](research/save-integrity.md). Checksums, record preservation, rollback, backup and source conflict checks; linked mail changes blocked. Scope is represented fields, not all in-game legitimacy rules. |
| Edited SAV in-game save and re-read / 模拟器再次保存 | P | P | P | P | V | Mercury 1.2 mGBA party/box core fields and corrected expanded-bag editing/re-save verified; earlier vanilla-offset inventory claims were withdrawn. [Correction evidence](verification/mercury-display-storage-20261005.md). Other individual emulator experiments do not certify every edit or format. |
| Missing collection, regional planning, HTML / 缺失与路线规划 | P | P | P | P | P | `collection.rs` and shared Collection planning UI: read-only SAV missing goals, regions, guarded entrance alternatives and standalone HTML. Full task DAG/access/time remain partial. Mercury uses existing individuals; expanded Dex flags stay disabled. |

## Reproducible baseline / 可重复基线

- Core public tests after Mercury 1.1 removal: **76 passed, 13 opt-in ignored**.
- Four older ROMs: **6 opt-in local tests passed** (table/artwork/adapter/save and cheat checks).
- Mercury 1.2: **5 opt-in tests passed**, including removed-1.1 rejection and native re-save.
- Mercury native individual routines: 840 records, 18,480 getters, 576 setter byte
  comparisons and 840 stat comparisons. Historical 1.1 evidence is not current support.
- TypeScript and Vite build passed. These tests do not prove full game progression,
  all map accessibility, or all trainer RNG/native overrides.

Relevant test commands and file prerequisites are in the individual research records;
private file paths are not release inputs. Missing evidence is retained as a gap.

## Work order / 增量实施顺序

1. Runtime map topology and source-to-map navigation; preserve unresolved destinations.
2. Shared acquisition index, with coordinates, conditions and explicit coverage limits.
3. Read-only SAV overlays and collection planning, with evidence and HTML export.
4. Native clock/reward/generation validation per fingerprint; expand only proven rules.

每阶段交付可操作的流程；未知脚本、动态布局和当前可达性始终保留提示。不存在
全局最短路线证明时仅称建议路线。默认不生成个体、不改图鉴和剧情标记。

## Query closure increment / 本轮闭环增量

Shared modules `navigation.rs`, `acquisition.rs`, `collection.rs`, `clock.rs`
add usable query → tile → entrance → return and SAV → regional suggestion → HTML
flows. Entrance/planning rows, originally U at the audit baseline, are now **P for all five**;
it does not certify current accessibility, complete tasks or all source types.
BW/DP receipt overlays are **P**: direct item-ball/hidden protocols and native
flag/variable addresses are verified, while generic NPC receipt inference is not.
Other ROM overlays were U at this increment; the later event-state increment below verifies persistent ranges and hidden-item receipts while leaving ordinary pickup/NPC protocols unresolved. Shop lists are
bounded, ROM-backed script records, not a claim about all native/custom shops.

| Exact input | Maps | Warps | Connections | Unresolved links | Maps with exterior chains | Positioned shop rows |
|---|---:|---:|---:|---:|---:|---:|
| BW | 707 | 1,808 | 204 | 58 | 474 | 415 |
| DP | 707 | 1,808 | 204 | 58 | 474 | 415 |
| Rocket | 1,363 | 3,790 | 445 | 69 | 1,053 | 987 |
| Ultimate | 922 | 3,023 | 292 | 63 | 628 | 1,200 |
| Mercury 1.2 | 871 | 2,866 | 244 | 61 | 535 | 1,470 |

Counts describe parsed references, not unique physical entrances or shops.
Ultimate reports three invalid static connection/event diagnostics and one
out-of-layout destination; unresolved links are retained. Paths are bounded,
cycle-safe static alternatives and make no single-direction/story-access claim.

`local_query_` opt-in tests passed on all five ROMs. Private BW, Rocket and
Mercury 1.2 SAVs also passed planning/read-only comparisons. DP and Ultimate had
ROM-only queries in this increment; no new edited SAV game round trip is claimed.
The browser fixture checks target tile, exterior entrance, back navigation,
map reward → item, planning, compact window and escaped standalone HTML export.
It caught and fixed a focus-marker hit-testing problem.

`scripts/verify_query_rules.py`: BW and DP each match **56 native flag/variable
reads**, with synthetic SB1 bytes unchanged. Mercury 1.2 matches **72 native
period-predicate results** across all 24 hours. These are query scenarios with
forced-night flag `0x1041` modeled unset, not effective-time decoding. A previous
Mercury research table described the 1.1 selector's literal pool at `0x1D65828`;
the 1.2 header-selection entry is `0x1D69AC0`, with predicates at `0x1D20DE0`,
`0x1D20DF8`, `0x1D20814`. Virtual-clock persistence and jump-time state were pending at this increment;
they are validated in the later clock increment below. Complete weekday refresh remains unresolved. The UI never substitutes the device's clock.

收集家族只用永久进化边，不用名称推测或战斗变身关联。按图鉴判断时，图鉴的
共同编号只能证明该物种曾获得，不证明每种独立形态都持有；按现有个体可区分
存档里的种类。已有家族成员不等于所有分支已完成。隐藏条件、交换、游走、原生
奖励、剧情依赖、跨地区解锁和设施仍有缺口；生成的 HTML 明示这些边界。

## Native trainer increment / 原生训练家增量

Mercury ordinary trainer construction advances U → **P**. The shared trainer
page executes its exact ROM constructor in bounded, isolated RAM; 742 trainers,
two seeds, 1,484 scenarios and 3,784 individuals match an independent Unicorn
execution, including 71,896 getter comparisons. This is a zero-context ordinary
scenario, not the entire battle setup, script replacements or facilities.
At this increment, FireRed/custom hidden-item quantity and receipt indexes were
left unknown. The later event-state increment validates Mercury packing and its
runtime region-dependent flag bases, rather than reusing Emerald packing.

The matrix above includes the current increments. Entrance, acquisition,
planning and Mercury trainer increments remain partial; no cell is promoted to full
coverage merely by an interface. See [the current verification and gaps](verification/query-collection-20261005.md).

水银普通配队由待验证提升为部分验证，展示原生构造器的情景结果。领取标记、
虚拟时钟、完整剧情依赖、特殊设施等缺口仍未关闭，详见增量验证记录。

## Mercury display/storage correction / 水银显示与背包修正

The opening-stage empty fixture and vanilla FireRed pocket reads did **not**
prove the expanded bag. The current reader uses native descriptors, section tails
and shared auxiliary flash sectors, with plaintext quantities. A real current SAV
matches all 778 native RAM bag slots; edited quantities survive an actual mGBA
Save-menu save and re-read without changing the Pokémon. Map/met labels use native
`GetMapName`, not the legacy town-map table. The extended metatile hook is
reproduced; map 1-0 also renders incorrectly on direct native load and has inbound
ROM references, so neither normal appearance nor unreachability is asserted.
Maps remain **P**. See [scope and evidence](verification/mercury-display-storage-20261005.md).

早期空背包／原版地址回读不构成扩展背包证明。当前背包已对照原生内存及游戏
再次保存核验。地图名称表与扩展图层已修正；1-0 有入口引用，在原生直接加载时
也错乱，不能宣称废弃或已还原正常游戏场景。地图总体仍为部分验证。

## Virtual-clock increment / 虚拟时钟增量

Mercury's time row stays **P**, with verified subfeatures expanded to its restored
virtual SAVE snapshot, native weekday/speed/forced-night fields and jump-menu
persistence. `clock.rs` reads ROM month parameters at runtime; acquisition and
collection/HTML share the snapshot. The four older profiles keep **U** effective-clock
status. Hardware RTC, all weekday event resets and full quest/reward semantics
are not inferred. See [native evidence and reproducible checks](verification/mercury-clock-20261005.md).

水银时钟总体仍为部分验证：虚拟保存快照和跳时持久化已有原生／模拟器证据，
并接入查询与收集 HTML；硬件 RTC 和完整星期事件刷新仍有缺口。其他 ROM 不继承
这些规则，领取状态／剧情依赖不会因时钟字段验证而自动升级。

Final regression for this increment: 88 public core tests passed (18 opt-in
ignored); both five-ROM local query checks and the five-ROM actual query/map UI
passed. Mercury's saved-clock English/Chinese planning/HTML flow and existing
inventory/location/map-warning UI regression passed without SAVE writes.

## Persistent event increment / 持久化事件增量

All five event/receipt rows are **P**: configured persistent flag/variable ranges
and native hidden-item protocols now have independent CPU evidence and shared
SAV queries. Mercury region-dependent hidden flags and packed quantities come
from runtime ROM records. At this increment pickup receipt was enabled only for BW/DP; NPC
visibility, full quest graphs and recurring-item resets remain separate gaps.
See [scope and evidence](verification/event-state-20261005.md).

五份 ROM 都能在已验证范围内读取剧情条件与隐藏道具领取状态；整体仍为部分验证。
水银按 ROM 区域列表选择隐藏道具标记基址，并展示脚下取物说明。普通拾取与 NPC
奖励领取协议、完整剧情依赖及刷新机制继续保留各自缺口。

## Ordinary pickup increment / 普通道具球增量

Native receipt control branches now qualify ordinary item-ball scripts in all five
profiles: BW 122, DP 122, Rocket 804, Ultimate 402 and Mercury 506 records.
The **3,912** isolated native cases inject bag-space outcomes and execute the
actual command/flag/object handlers; the independent Rust reader matches every
qualified record. Extra script effects, reassigned NPC identity, dynamic/zero
quantities and missing flags retain unknown receipt state. Collection filtering
uses proven receipts; no source promises single-time-only without reset evidence.
Native bag insertion, current map access, full quest logs and NPC receipts are not
certified by this experiment. All rows remain **P** within their documented scope.
See [verification and remaining boundaries](verification/pickup-receipts-20261005.md).

五份 ROM 的普通道具球已按独立原生证据接入领取查询与收集建议，判定领取标记
不再只依赖对象可见性。复杂事件、共享标记／剧情初始化、当前地图可达性和完整
任务依赖继续保留边界，未扩大为完整任务支持。


## NPC receipt increment / NPC 领奖增量

All five receipt rows remain **P**, with bounded per-reward protocol proofs added:
BW 18, DP 18, Rocket 15, Ultimate 28, Mercury 1.2 1 parsed reward row.
The 80 rows match 320 native caller/standard-script success, failure and already-set
flag cases. This is not comprehensive NPC coverage or a count of unique NPCs.
Qualified gifts carry their own flag/root/award/setter evidence, independent of
object visibility. The shared acquisition → map → back → planning → bilingual HTML
flow exposes the evidence and retains unqualified reward uncertainty. No SAV or
ROM write is performed. Mercury's many custom commands, native reward calls,
whole-game resets, quest DAGs and current accessibility are still gaps.
See [method, reproducible checks and limits](verification/npc-receipts-20261006.md).

五份 ROM 均新增有界 NPC 成功领奖协议，整体仍为部分验证。80 条记录经 320 个原生
分支对照，不是 80 个独立 NPC，也不代表所有赠送。按奖励单独保存证据，共用查询、
地图、返回、规划及中英文 HTML；未资格化奖励保持未知。水银的大量自定义指令、
原生奖励调用、完整重置／剧情依赖和当前可达性尚未补齐。

## Script Pokémon source increment / 脚本宝可梦来源增量

Script gifts, eggs and fixed encounters remain **P** across all five fingerprints.
Shared command decoding now distinguishes literal/variable operands and extended
single/double battle layouts. Parsed sources link to NPC/coordinate tiles or an
explicit unplaced list, acquisition and collection HTML. Runtime egg levels are
checked against native constructors; delivery, exchanges, custom native sources,
receipt state and full access prerequisites remain unresolved. See
[evidence and boundaries](verification/script-pokemon-20261006.md).

五份指纹的脚本宝可梦获取仍为部分解析；来源和格位关联不代表当前可领取。
交换、自定义原生来源、完整生成／发放结果、领取状态和剧情可达性仍有缺口。

## NPC trade increment / NPC 交换增量

All five source rows remain **P**. Native ordinary generation matches 29 offer
records / 174 synthetic cases. Parsed map references: BW 4, DP 4, Rocket 4,
Ultimate 8, Mercury 14. These are quote references, not a count of reachable NPCs.
Required Pokémon and held items share source/map/back links; current SAV donors
are checked by exact species, excluding eggs, with party/box context. Completion,
receipt/reset rules, trade animation/evolution and special branches stay unknown.
Mercury's alternate flag selects a different constructor; ordinary rules do not
apply to it. See [evidence and workflow](verification/npc-trades-20261006.md).

交换查询读取当前 ROM 的报价，不打包名称或目录。等级按交出个体变化，并非固定
等级；有候选也不意味着剧情和地图可达性已满足。原生对照不包括实际交换动画、
交换后进化或特殊生成；这些边界继续保留。

## Tutor query increment / 教招查询增量

Teaching rows remain **P** across all five exact fingerprints. Bounded native
lookup and map-script sources now complete move → teaching offer → NPC tile →
exterior entrance → back. Parsed references: BW 10, DP 10, Rocket 10, Ultimate 13,
Mercury 170. Independent evidence covers 345 native indexed getter vectors,
19 menu-selector scenarios and 61 Rocket native compatibility cases. BW/DP's
legacy tutor table was incorrect in six slots and is corrected to the runtime
native table. No location/name/move catalog is bundled. Eligibility, payment,
one-time receipt/reset rules, Mercury special compatibility and unresolved
native-menu sources remain unknown. See [scope and reproduction](verification/tutor-sources-20261006.md).

教招报价接入共用招式、地图和返回流程；查询保留有界条件，不将 NPC 可见性、
已有招式或背包内容当作教学完成证据。213 条引用不等于独立、可达或当前可教的
NPC；普通兼容表也不证明已完整覆盖所有特殊招式学习。整体仍为部分验证。

Current teaching increment regressions: 98 public core tests pass (23 opt-in ignored), plus independent native lookup/menu/compatibility checks, five-ROM source/query/navigation regression and bilingual browser workflows. No new edited-SAV emulator round trip or package is claimed.

## Resource prerequisites increment / 资源条件增量

All five fingerprints retain verified holdings predicates and symbolic Boolean
branches, with 690 native holdings vectors, 240 native copy/compare/goto cases,
3,227 native category lookups and 8 alternate-bag flag cases. Source → required item → acquisition → map/back,
collection and bilingual standalone HTML use shared conditions. Mercury ordinary
bag/money checks can be evaluated; other four normal-bag counts are projections
with unknown satisfaction until facility context is recovered. Checks after
resource changes stay unknown. No spending, access or new receipt proof is implied.
See [evidence and remaining scope](verification/resource-conditions-20261006.md).

五份指纹的道具／金钱条件均有原生验证并可跳转所需道具。水银普通背包条件可以
叠加存档；其他四份的设施临时情景仍未恢复，只显示普通背包参考数量。所有功能
行仍为部分覆盖；持有量不代表支付、领奖或当前可达。保持当前版本号。

Resource increment regression: 101 public core tests passed, 24 opt-in ignored;
native and five-ROM query/teaching parity plus bilingual browser workflows pass.
No new save-edit emulator round trip or package is asserted for this increment.

## Wild held-item increment / 野生携带道具增量

All five source rows remain **P**. Actual random encounter references close item
→ Pokémon → map/entry/back; encounter slot probability and native held-item
probability are separate. Time selectors stay distinct. With a SAV, the first
raw party individual supplies the native ability/egg context. No random reference
means an explicitly unplaced, unverified source, not guaranteed availability.
Independent CPU execution covers 1,090 contexts / 327,000 assignment cases and
327,680 native RNG outputs. Dynamic layouts, facilities, full encounter ability
modifiers, complete access and next-encounter prediction are outside this evidence.
See [scope and reproduction](verification/wild-held-20261006.md).

五份指纹仍为部分支持。原生概率验证不证明地图当前可达，不把静态或赠送引用
当作普通随机来源；收集建议与独立 HTML 保留相同的情景和未知边界。

## Native daycare increment / 原生寄养增量

The [existing-parent collection increment](verification/breeding-planning-20261006.md)
observes native ordinary offspring selection and links sampled party/box pairings
to preparation, service references, full receipt previews and HTML. Bounds and
unknown results are visible. Deposited-parent automatic search, complete offspring
coverage, inheritance, hatching and service access remain unresolved; all five
breeding/planning rows remain **P**.

The [saved ordinary daycare increment](verification/daycare-state-20261006.md)
adds native saved presence, availability, service status and next ordinary check
phase across five fingerprints, with deposited-parent scenario selection.
It does not resolve custom service access, live RNG or complete inheritance and
hatching. Overall breeding coverage remains **P**.

All breeding/acquisition rows remain **P**. Five fingerprints have 210 ordinary
compatibility/receipt scenarios (171 generated records) matching mGBA and 120 GBA
halfword microcases. Stored/simulated parents, child/move links and located
receiving NPC/map/back are a read-only shared workflow. BW/DP have two receiving
paths each, Ultimate six; Rocket/Mercury currently have no located service and
show explicit uncertainty. Complete production/charm context, live pending eggs,
complete inheritance/forms and full collection dependency planning remain gaps.
See [scope, CPU correction and evidence](verification/breeding-20261006.md).

本增量未将孵蛋候选或模拟后代视作可达、已捕获、当前有蛋可领。没有新增 SAV 编辑
动作；不同版本服务引用与完整产蛋／遗传／孵化依赖继续需要验证。本地测试包已按
用户明确请求单独交付，未公开发布。

## Ordinary production increment / 普通产蛋检查增量

All five breeding rows remain **P**. Native ordinary step comparisons, modifier
checks, ordinary keyed-bag projections and explicit item simulations now join
parent → production scenario → required item → acquisition/map/back. Independent
mGBA evidence covers 270 complete steps, 30 pending/step gates and 327,680 native
roll results; Rust verifies every draw and projected/override scenario. Mercury
uses a persistent availability flag rather than populating the legacy pending
personality slot. Full live daycare state, custom service setup, inheritance,
hatching and service access remain separate gaps. Ultimate's tested ordinary
branch lacks the modifier check; this does not assert game-wide absence.
See [scope and reproduction](verification/breeding-production-20261006.md).

界面区分配对兼容性与产蛋概率，普通背包投影与道具模拟不会改动 SAV。漆黑
BW／DP 的不兼容亲本道具分支按原生证据明示，不认定已获得或正常可获得后代。
本增量不生成个体，不修改图鉴或剧情，不递增版本，不另行打包或发布。

## Evolution preparation increment / 进化准备链增量

Collection rows remain **P**. Suggestions now expand one bounded directed
permanent-evolution chain, starting from actual non-egg individuals or a located
ancestor source, with resource and map/entrance/back links and bilingual HTML.
Dex history is never substituted for current parents. Possession does not certify
eligibility; gender/time/friendship, full breeding, joint branch quantities and
complete story/access dependencies remain unverified. Neither shortest paths nor
reverse-evolution breeding are asserted. See [evidence and scope](verification/evolution-preparation-20261006.md).

收集规划仍为部分支持。准备链区分实际非蛋个体与图鉴记录，展示进化条件、道具、
招式和前代来源地图；不把“已有前代”自动提升为当前可进化，也不把不同分支视为
只需一个个体。完整孵蛋、资格、剧情依赖及合并路线最优性尚未完成。

## Prerequisite trace increment / 前置线索增量

All five progression/query/collection rows remain **P**. The [native command and
workflow evidence](verification/event-dependencies-20261006.md) covers potential
persistent writers from referenced map roots, non-reward tile triggers, root text
contexts, guard expansion, map/entrance/back and standalone HTML links. Five
fingerprints have 360 independent native write cases, 25,106 referenced roots and
16,680 sampled effect queries. This is not proof of all activations or a complete
mainline/sidequest DAG; native writes, dynamic entry conditions and current access
remain unresolved. No receipt logic is replaced with item holdings or visibility.

五份指纹均保持部分支持。线索来自当前 ROM，被地图表引用不代表游戏中当前可达或
必定使用；NPC 可见和标记满足不代表领奖／完成。普通 SAV 检查不是模拟器实时状态
或全部设施情景。独立 HTML 附录有界，未知条件和循环不包装成无法完成。

## Event-clue search increment / 事件线索搜索增量

All five story/dependency rows remain **P**. The shared read-only event page now
searches current-ROM text/map references and links guarded changes to prerequisite
tracing, map tiles, exterior chains and return. All 28,324 searchable references
have current map/root/text evidence; this does not prove game activation, complete
story coverage or quest completion. Observed values and NPC visibility are
explicitly separate from receipt/task status. See
[scope and verification](verification/event-clues-20261006.md).

五份指纹共用事件线索页，支持文字／地图搜索、条件追查、格位／入口和返回；不内置
任务目录，不把标记值一致当作完成状态。完整剧情依赖、原生事件、动态激活和可达性
仍有缺口，相关行继续为部分解析。

## Qualified trainer references / 有条件的训练家引用

All five fingerprints remain **P** for trainer location/access coverage. The shared
trainer/event/map flow now retains native-format-qualified literal record roles,
branch guards, NPC visibility and static actor/tile links. Native mGBA checks cover
289 disposable command executions and cross-ROM tests check 12,981 references;
12,649 have in-bounds static coordinates. Rematches, setup/replacements and current
access remain unresolved. See [boundaries and evidence](verification/trainer-locations-20261006.md).

## RTC scenario increment / RTC 情景增量

Four older profiles now have partial clock workflows: saved offsets/checkpoints
and explicit native RTC scenarios, not recovered current time. Independent mGBA
vectors cover 3,456 projections and 24 full getter error-clock cases. Mercury
continues its separate virtual-clock path. See [evidence and limits](verification/hardware-clock-20261006.md).
This supersedes the earlier U classification only for these bounded additions.

## Native EV-item increment / 原生努力值道具增量

All five training/source workflows remain P. The shared read-only training page
links verified dispatch/classifier offers to acquisition, guards and maps, with
660 byte-exact native field-effect scenarios. Custom handlers, complete mint menus, ability
changes, Hyper Training services and complete normal-operation equivalence remain
unresolved. See [evidence and limits](verification/training-items-20261006.md).

## Native mint increment / 原生薄荷增量

All five overall training workflows remain **P**. Rocket now reads 21 mint targets
at runtime and previews effective-nature/stat changes, with acquisition/condition/
map/back links. Independent mGBA evidence covers 1,512 scenarios; 720 changing
cases also match existing party and boxed editor patches byte for byte. Other
profiles have no configured verified mint handler, not a proof of absence.
Menu eligibility, consumption/access, inheritance and ability/Hyper Training
services remain incomplete. See [evidence](verification/training-natures-20261006.md).

## Native ability increment / 原生特性道具增量

All five overall training workflows remain **P**. Rocket reads two item handlers;
Mercury reads one. Shared native previews cover **13,023** guard cases and **540**
persistent outcomes, with source/condition/map/back links. Rocket's 120 accepted
cases match existing party/box edits byte for byte. Mercury's 84 changing cases
reroll PID; sampled nature/gender/shiny state is preserved, without certifying all
PID-linked appearances or equating the existing editor solver to a native RNG
outcome. Explicit seed scenarios and actual before/after data are shown.
No verified adapter for the other three fingerprints is not a claim of absence.
Menus, access/consumption, live RNG, crown and
complete NPC services remain unresolved. See [evidence](verification/training-abilities-20261006.md).

## Ultimate ability extension / 究极绿宝石特性道具扩展

Ultimate now reads two variants of its item-menu/script/native ability workflow,
with **9,592** native guard cases and **528** persistent cases. This supersedes
its earlier unconfigured-handler status. The earlier relocation hypothesis was
incorrect: ROM code is reached through wrappers/scripts and a RAM callback pointer.
Native hidden-to-normal selection is seeded; PID stays unchanged. Successful but
unchanged capsule cases remain visible. Overall training is still **P** across
all five fingerprints; full menus, access/consumption, crown and NPC services
remain incomplete. See [evidence](verification/training-ultimate-abilities-20261006.md).


## NPC crown-service increment / NPC 王冠服务增量

Overall training remains **P** for all five fingerprints. Ultimate now reads one
referenced NPC service, seven runtime menu choices and both item/earned-credit
requirements, with read-only SAV overlays and NPC tile/item/map/back links.
Independent mGBA and the core runner agree on **48** selected-byte readers and
**10,752** native mask/mutation scenarios across six party slots and all header
bytes. Only intended training bits change; base IVs, identity, other data and
other party slots stay intact. This verifies persistent helper semantics, not a
complete NPC interaction or edited-SAV emulator round trip. Full menus, credit
acquisition, current access, immediate stat refresh and other-ROM crown services
remain incomplete. Mercury now has a separate base-IV service adapter, with
runtime unlock/level/menu and required-versus-payment records. Its native silver
branch checks silver but attempts to remove gold, before the single-stat
menu; field/payment probes do not certify the full interaction. BW/DP/Rocket
empty service results remain unknown, not absence.
See [evidence and scope](verification/training-crowns-20261006.md).

### Mercury base-IV training increment / 水银基础 IV 训练增量

Overall training remains **P** across all five fingerprints. The shared service
page distinguishes Mercury's direct base-IV/stat recalculation from Ultimate's
training flags. Mercury's positioned NPC, runtime seven choices, minimum level
and unlock flag link to shared item acquisition, prerequisites and map/entrance
navigation. SAV conditions are read-only and refresh with the session revision.
Independent complete mGBA calls and the core runner agree on **216** selected
level reads plus **432** script level branches, **1,512** IV/stat cases across
six slots, **4** invalid-slot guards, and **32** ordinary-bag check/removal cases.
The silver check/payment mismatch is displayed explicitly; native removal failure
does not by itself stop the next script instructions. Full menu/cancellation,
access, unlock progression and complete item transaction remain unresolved.
No new SAV editing or emulator export/reload guarantee is added by this increment.
See [Mercury evidence](verification/training-mercury-crowns-20261006.md).

### Read-only service effects / 只读服务效果

Ultimate and Mercury referenced services now preview stored or simulated
individual effects through native level-gated helpers. Mercury changes base IVs
and recalculates stats; Ultimate marks training flags and leaves saved party
stats unchanged. The API matches 252 Mercury independent field vectors and
56 Ultimate level/header/choice cases; the five-profile legacy training
regressions still pass. Parsed requirements and hypothetical effects stay
separate. Box/simulated inputs use a full-HP projection, not a verified PC
transfer. No items or SAVs are changed. Overall training stays **P**: full menu,
payment/cancellation, story access and other-profile services remain unresolved.
See [service-preview evidence](verification/training-service-previews-20261006.md).

### Service-menu cancellation / 服务菜单取消

Runtime menu flags now distinguish Mercury's cancellable initial menu from its
single-stat menu that ignores B after the payment attempt. Ultimate's training
choice menu allows B. Three parameter observations, 60 native input decisions
and eight cancel-routing commands match independent mGBA probes and the core
runner. Another 512 native flag-transfer slices verify the distinct Mercury
bit-mask and Ultimate full-byte rules. This corrects the earlier unsupported “cancellable stat menu” wording;
it does not certify rendered windows, party selection, transaction/refund or
current story access. Training remains **P** across all five fingerprints.
See [menu evidence](verification/training-service-menus-20261006.md).

### Party-only service selection / 服务同行选择

The positioned Ultimate and Mercury services use native party-menu action 11,
then separately check the selected level. B cancellation returns through each
game's own result convention (Mercury 7; Ultimate normalizes to 255). Runtime
special pointers and cancel comparisons are read and validated, not borrowed
across engines. Fainted and synthetic egg states are not filtered in this
selection step; that does not establish complete training eligibility. Editor
previews intentionally reject eggs. Box previews now instruct withdrawal first
and expose `withdrawal_required`; no PC transfer is replayed.

Independent evidence covers three script/launcher observations, 288 input/state
cases, 256 Ultimate return prefixes and 21 native script routes. Core comparisons
cover script dispatch, bounded Init argument prefixes, input decisions/selection
result fields, return normalization and routes. Full fade/overworld cleanup is
observed only in mGBA, not certified by the stricter core runner. Rendered menus,
cursor traversal/empty-slot reachability, complete transactions and current
access remain unresolved. Overall training stays **P** for all five profiles.
See [selection evidence](verification/training-party-selection-20261006.md).

## Collection source presentation increment / 收集来源展示增量

Collection remains **P** for all five fingerprints. Shared rows and standalone
HTML now display known acquisition type, quantity, level, encounter-slot
probability, periods and repeatability, with related-target navigation. Period
hours require current verified clock rules. No new acquisition/access/receipt
semantics or task-DAG completeness is implied. See
[source presentation evidence](verification/collection-source-facts-20261006.md).

五份 ROM 的收集规划仍为部分支持。本次补齐清单和 HTML 的来源事实及关联跳转，
未知规则不补猜；真实查询回归与合成界面场景的证据分别说明。没有新增完整任务
依赖、动态可达性或模拟器内路线验证。

## Collection prerequisite routes / 收集前置关联增量

Collection and story-dependency rows remain **P** for BW/DP/Rocket/Ultimate/Mercury.
Runtime report routes now connect guarded writer alternatives to plan goals and
other unmet conditions. The live regional panel and HTML expose this relation,
static entrances, cycles and unresolved guards; already-met conditions stop
prerequisite expansion. Reference/back preserves panel state, while ROM/SAV
reload invalidates it. The five exact-ROM scenarios and current Mercury SAV
read-only CLI run retain truncation/access limits; see
[route verification](verification/collection-prerequisites-20261006.md).

五款游戏的前置关联仍为部分支持。候选事件不是已确认任务目录，不保证完整任务
顺序或当前可达；条件已满足不等于任务已完成。五指纹真实脚本来源回归与合成
界面测试分别记录，水银另有真实 SAV 只读规划。没有新增存档写入或模拟器保存验证。

## Referenced script passages / 已引用脚本通道增量

Map access stays **P** for all five fingerprints. The live map window now joins
the static topology to bounded, referenced immediate script transitions, with
source/target tiles, branch/visibility guards, fresh SAV observations, entrance
markers and directed exterior-reference chains. Destination setters and
unreferenced bytes do not become passage edges. **918 independent native operand
cases** match the shared decoder; public core has **125 passes / 45 opt-in ignored**.
Five-profile bilingual UI closure passes. Full activation, movement, dynamic
layout/loading and current reachability remain unresolved. Collection HTML now shares the guarded graph in the later integration below. See [scope and reproduction](verification/script-warps-20261006.md).

五款游戏的地图通道仍为部分解析。脚本入口新增定位、条件叠加、跳转及返回；不自动
推导返程，不将设置目的地的指令当作通道。参数原生对照不等于完整地图加载或可达性
验证；独立收集 HTML 已在下文增量中共用脚本入口图。


## Guarded entrances in collection / 收集脚本入口整合

All five exact profiles remain **P** for map access, acquisition, story conditions
and collection planning. Collection sources, directed-evolution preparation,
prerequisite writers and standalone HTML now share the map window's ROM-bound
script graph. Entry alternatives are separate from writer guards. Fresh SAV
observations are not cached as eligibility; satisfied conditions stop recursive
writer/entrance tracing. Unknown destination tiles remain separate incoming
references. Coverage, cycles and bounded-search warnings are retained.

五指纹共用入口查询与 UI，按当前 SAV 重查条件；不将各条备选路线当成一组必做
任务，不将领取状态或可达性从入口推断出来。真实 ROM、原生参数对照、合成 SAV
及合成界面测试分别说明。[验证与未完成范围](verification/collection-script-entrances-20261006.md)。
