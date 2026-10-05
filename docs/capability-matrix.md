# Capability audit / 实际能力审计

Audit baseline: 2026-10-05, `f6f347a`. This matrix describes tested scope, not
promises implied by an API, readable table or working screen. Local binaries,
saves and extracted artwork are excluded from Git and releases.

审计基线：2026-10-05，`f6f347a`。接口存在、能读取表、页面能打开不代表完整支持。
ROM 和 SAV 仅用于本地验证。新增实现的证据在本文后续记录中追加。

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
| Entrances, connections, current reachability / 入口与可达性 | U | U | U | U | U | Baseline `Map` omits warps/connections; a map reference is not proof of current access. |
| Pickup, hidden items, dialogue rewards / 拾取与奖励 | P | P | P | P | P | [map events](research/map-events.md): BW/DP 707 maps, 122 balls, 112 hidden items. Bounded scripts report stop offsets; FireRed command semantics and reward guards need further validation. |
| Random encounters, probability, time tables / 随机相遇 | P | P | P | P | P | `world.rs`; [time selection](research/encounter-time-selection.md), Mercury native selector. Slot weights are separate from encounter frequency; no full weekday/clock-state validation. |
| Static, gifted, traded Pokémon / 定点、赠送、交换 | P | P | P | P | P | Bounded `script_report`; [Rocket reward egg](research/rocket-reward-egg.md). Native calls, roamers and custom exchanges are not comprehensively indexed. |
| Breeding, evolution, form rules / 孵蛋进化形态 | P | P | P | P | P | `rom.rs`, `forms.rs`, `relations.rs`; [evolution tree](research/runtime-data-and-evolution-tree.md). Egg-group candidates are not proof of all incense/baby/parent requirements. Permanent evolution and battle forms have separate readers. |
| Item shops, wild held items, teaching sources / 道具与培育来源 | P | P | P | P | P | Held-item fields and learnsets read from ROM; shop/receipt/location closure missing. No verified probability under all ability modifiers. |
| SAV receipt flags, story dependencies / 领取状态与剧情依赖 | U | U | U | U | U | Marker visibility is not a universal reward receipt; bag absence is not evidence of non-receipt. No general SAV condition overlay at baseline. |
| Effective game clock, weekday, next event / 有效时钟与刷新 | U | U | U | U | P | Mercury has verified four encounter periods, but virtual-clock/jump-time save state still needs native proof. No device-time assumption. |
| Ordinary trainer construction / 普通训练家实战值 | P | P | P | P | U | [trainer generation](research/trainer-search-and-generation.md): BW/DP 20 parties, 106 mons match native constructor; Rocket expanded EV fields and random ability/gender are distinguished. Mercury raw records are not final generated values. |
| Difficulty/player-dependent trainer generation / 难度动态队伍 | U | U | U | P | U | [Ultimate](research/ultimate-emerald-55.md), `ultimate_ev.rs`, `ultimate_battle.rs`: bounded native execution with party/scenario. Script overrides and full facilities are not exhaustively replayed. U does not assert other games lack difficulty mechanics. |
| Edit, drag/swap, batch, undo/export safety / 编辑事务 | V | V | V | V | V | `session.rs`, [save integrity](research/save-integrity.md). Checksums, record preservation, rollback, backup and source conflict checks; linked mail changes blocked. Scope is represented fields, not all in-game legitimacy rules. |
| Edited SAV in-game save and re-read / 模拟器再次保存 | P | P | P | P | V | Mercury 1.2 mGBA load, game save and re-read of disposable edited party/box/items verified. Other individual emulator experiments do not certify every edit or format. |
| Missing collection, regional planning, HTML / 缺失与路线规划 | U | U | U | U | U | Earlier private reports are manual research outputs, not a reusable released planner. Mercury expanded Pokédex remains unavailable; existing individuals can be inspected. |

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
