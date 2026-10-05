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
| Entrances, connections, current reachability / 入口与可达性 | P | P | P | P | P | `navigation.rs`, shared map UI and five exact-ROM tests. Static entrance alternatives, focus and return work; current access and dynamic destinations remain unresolved. |
| Pickup, hidden items, dialogue rewards / 拾取与奖励 | P | P | P | P | P | [map events](research/map-events.md): BW/DP 707 maps, 122 balls, 112 hidden items. Bounded scripts report stop offsets; FireRed command semantics and reward guards need further validation. |
| Random encounters, probability, time tables / 随机相遇 | P | P | P | P | P | `world.rs`; [time selection](research/encounter-time-selection.md), Mercury native selector. Slot weights are separate from encounter frequency; no full weekday/clock-state validation. |
| Static, gifted, traded Pokémon / 定点、赠送、交换 | P | P | P | P | P | Bounded `script_report`; [Rocket reward egg](research/rocket-reward-egg.md). Native calls, roamers and custom exchanges are not comprehensively indexed. |
| Breeding, evolution, form rules / 孵蛋进化形态 | P | P | P | P | P | `rom.rs`, `forms.rs`, `relations.rs`; [evolution tree](research/runtime-data-and-evolution-tree.md). Egg-group candidates are not proof of all incense/baby/parent requirements. Permanent evolution and battle forms have separate readers. |
| Item shops, wild held items, teaching sources / 道具与培育来源 | P | P | P | P | P | `acquisition.rs` reads held-item fields, reverse learnsets and bounded shop scripts at runtime, with map/target links. Custom shops and full receipt/condition semantics remain partial. No verified probability under all ability modifiers. |
| SAV receipt flags, story dependencies / 领取状态与剧情依赖 | P | P | P | P | P | [Persistent ranges and hidden protocols](verification/event-state-20261005.md) and [qualified ordinary pickup protocols](verification/pickup-receipts-20261005.md) verified across five fingerprints. [Bounded NPC success/receipt protocols](verification/npc-receipts-20261006.md) cover 80 reward rows. Compound/custom pickups, unqualified NPC protocols and complete story dependencies remain unknown. NPC visibility and bag absence are not receipt evidence. |
| Effective game clock, weekday, next event / 有效时钟与刷新 | U | U | U | U | P | Mercury virtual SAVE clock, native weekday, speed, forced-night state and jump menu verified within [bounded scope](verification/mercury-clock-20261005.md); hardware RTC and all weekday refresh remain unresolved. No device-time assumption. |
| Ordinary trainer construction / 普通训练家实战值 | P | P | P | P | P | [trainer generation](research/trainer-search-and-generation.md): BW/DP 20 parties, 106 mons match native constructor; Rocket expanded EV fields and random ability/gender are distinguished. Mercury executes its native constructor for explicit zero-context scenarios; independent CPU comparisons are recorded below, while full setup remains unknown. |
| Difficulty/player-dependent trainer generation / 难度动态队伍 | U | U | U | P | U | [Ultimate](research/ultimate-emerald-55.md), `ultimate_ev.rs`, `ultimate_battle.rs`: bounded native execution with party/scenario. Script overrides and full facilities are not exhaustively replayed. U does not assert other games lack difficulty mechanics. |
| Edit, drag/swap, batch, undo/export safety / 编辑事务 | V | V | V | V | V | `session.rs`, [save integrity](research/save-integrity.md). Checksums, record preservation, rollback, backup and source conflict checks; linked mail changes blocked. Scope is represented fields, not all in-game legitimacy rules. |
| Edited SAV in-game save and re-read / 模拟器再次保存 | P | P | P | P | V | Mercury 1.2 mGBA party/box core fields and corrected expanded-bag editing/re-save verified; earlier vanilla-offset inventory claims were withdrawn. [Correction evidence](verification/mercury-display-storage-20261005.md). Other individual emulator experiments do not certify every edit or format. |
| Missing collection, regional planning, HTML / 缺失与路线规划 | P | P | P | P | P | `collection.rs` and shared Collection planning UI: read-only SAV missing goals, regions, static entrances and standalone HTML. Full task DAG/access/time remain partial. Mercury uses existing individuals; expanded Dex flags stay disabled. |

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
