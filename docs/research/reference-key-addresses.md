# ROM reference research addresses / 资料页逆向关键地址

The four advanced tabs (event clues, collection planning, game clock, training)
and their dedicated product implementations have been removed. This document
preserves the investigation entry points, not a built-in game catalog or a claim
of complete game behavior. Historical verification documents describe the former
implementation; removed CLI commands and UI tests are not current instructions.

按用户要求，四个高级资料页及专用实现已撤去。本文件保留后续逆向的切入点，
不包含提取后的任务、名称、地图、图片或队伍目录。历史验证记录用于了解证据与
边界，其中已移除的 CLI 命令和页面步骤不能视为当前使用说明。

## Address conventions and exact inputs / 地址约定与指纹

`ROM offset` starts at file byte zero; `CPU address` includes the GBA mapping.
An ordinary ROM CPU address is `0x08000000 + offset`; bit 0 in a function pointer
selects Thumb and is not part of the file offset. RAM addresses below are transient
execution state, not direct offsets in a `.sav`. Save offsets refer to reassembled,
validated logical blocks, not blindly to the beginning of a raw battery save.

| Adapter / 适配 | Exact ROM MD5 |
|---|---|
| 漆黑的魅影 BW | `0d9b129f7dd76895f79bb47ad7dec2fe` |
| 漆黑的魅影 DP | `cb2940215f4dafb1bef133c3af379f44` |
| 西班牙火箭队 2.1 汉化 | `59c658a1081f542086de1060bb65f0b3` |
| 究极绿宝石 5.5 | `17ce9785b33319b3dbda9a5d37c57ec1` |
| 宝可梦水银 FC 1.2 | `f323df1792ac68462a34b42fe8571533` |

These addresses are fingerprint-specific. Similar names, engine families or
version labels do not establish compatibility. ROMs remain read-only; isolated
native verification modifies disposable RAM only. There is no ROM writer/export.

> Mercury version note (2026-10-09): active support is FC 1.33. The Mercury
> addresses recorded on this page are **FC 1.2 historical research** unless
> explicitly stated otherwise. Use [FC 1.33 addresses](mercury-fc-133.md) for
> current code; FC 1.2 is no longer registered.
>
> 水银已推进到 FC 1.33；本页原水银地址保留为 **1.2 历史研究**，不能直接用于
> 当前版本。新版本地址及验证边界见上述独立文档。

## Event clues and prerequisite writes / 事件线索与前置标记

| Adapter | Script command table (ROM offset) | Trainer-battle handler (ROM offset) |
|---|---|---|
| BW / DP | `0x1DB67C` | `0x9B5D0` |
| 西班牙火箭队 | `0x22B218` | `0xD1530` |

The currently retained exact rules for all five fingerprints live in
[`event_dependencies.rs`](../../crates/gen3-core/src/event_dependencies.rs) and
the adapters' `event_state` layouts. This shared reader still serves map passages,
trainer locations and expandable reward conditions. The general event-search UI,
its search API and collection prerequisite DAG builder have been deleted.

Useful script operations: `0x16..0x1A` variable writes/arithmetic, `0x29/0x2A`
flag writes, `0x0F` standard calls and the adapter-qualified battle instruction.
Walk **referenced** map/NPC/trigger roots, follow bounded calls/branches, retain
unknown operand or native-special stops, and validate dispatcher pointers before
interpreting a byte as an opcode. A matching flag value is not proof of task
completion; inventory absence is not evidence of a missing receipt.

解析应从地图／NPC／触发引用出发，不把扫描到的无引用记录当作必然可达的任务。
写标记、显示 NPC、成功赠送和领取标记是不同协议。静态脚本连接不证明当前可达，
未知动态布局和原生特殊调用须保留未知。

Evidence: [event effects](../verification/event-clues-20261006.md),
[reward conditions](../verification/event-dependencies-20261006.md),
[trainer references](../verification/trainer-locations-20261006.md),
[script passages](../verification/script-warps-20261006.md).

## Collection research / 收集规划逆向切入点

The planner, saved-parent suggestion cache, prerequisite route aggregation,
regional UI and standalone collection-HTML exporter have been deleted. Their
building blocks still used by the six core pages remain separate:

- [`dex.rs`](../../crates/gen3-core/src/dex.rs): exact native Pokédex validity,
  split-bank/history reads. A historical capture record is not a currently owned
  individual. Do not create Pokémon or write story flags to fill a route.
- [`acquisition.rs`](../../crates/gen3-core/src/acquisition.rs): referenced
  encounters/rewards/shops and reverse evolution resources from the loaded ROM.
- [`navigation.rs`](../../crates/gen3-core/src/navigation.rs): static and guarded
  script passage links, bounded exterior approaches; no shortest-route claim.
- [`breeding.rs`](../../crates/gen3-core/src/breeding.rs): existing per-species
  daycare information/isolated offspring preview, shared with acquisition queries.
  This is distinct from the removed all-box parent-pair planner.

Previously observed offspring-selection checkpoints (CPU addresses):
BW/DP `0x080708E8`, 西班牙火箭队 `0x0809E8D0`, 水银 `0x09D1B68C`.
These stop before creation; a selected species alone does not certify the full
egg receipt, inheritance, menu, delivery or current access. The old planner
sampled four RNG/PID scenarios and at most 2,048 parent pairs; omitted suggestions
never proved breeding impossible.

Evolution entry points: BW/DP selector `0x0806D098`, Eevee hook offset
`0x1196250`, seven-row alternate table `0x1196300`; 水银 item-gender compare
offset `0x1D2870E`, native gender getter `0x0803F721`.
See [complete selector evidence and limits](../verification/item-evolutions-20261006.md).
Read evolution rows/conditions at runtime; keep permanent evolution, form changes
and battle transformations separate.

## Clock research / 时钟切入点

| Adapter | Validation / difference / getter (ROM offsets) | Logical SAV block 0 |
|---|---|---|
| BW / DP / 究极绿宝石 | `0x2F2FC` / `0x2F504` / `0x2F588` | Offset `0x98`, last checkpoint `0xA0` |
| 西班牙火箭队 | `0x441A0` / `0x443A8` / `0x4442C` | Offset `0x98`, last checkpoint `0xA0` |

The offsets/checkpoints are not the live RTC. The removed projection feature
subtracted an explicitly supplied RTC using native borrowing; it did not infer
the effective clock or weekday from the host computer.

水银 1.2 uses a saved virtual clock, not these RTC routines:

| Entry | Address / layout |
|---|---|
| Restore / persist (ROM offsets) | `0x1D5AA94` / `0x1D5A65C` |
| Period predicates (ROM offsets) | `0x1D20DE0`, `0x1D20DF8`, `0x1D20814` |
| Month-length table (ROM offset) | `0x1DDEDF8` |
| Extension block enable | byte `0x146`, mask `0x20` |
| Forced-night state | byte `0xE8`, mask `0x02`; native flag `0x1041` |
| Packed time words | `0x5DE`: year, month/day, hour/minute, second/weekday |
| Speed word | `0x5E6` |

Verified period boundaries are 04:00 / 08:00 / 17:00 / 20:00, with a separate
forced-night state. Weekday is stored independently, not recomputed from the date.
The snapshot can become stale while playing. The encounter query retains its
compact hour filter and read-only snapshot, but the standalone clock page,
RTC-input form and projection API/code are gone.

Evidence: [Mercury clock](../verification/mercury-clock-20261005.md),
[RTC projections](../verification/hardware-clock-20261006.md).

## Training research / 培育机制切入点

Dedicated item classification/effect simulations, crown menus and service
previews have been deleted. These are research addresses, not available editor
operations; existing safe individual SAV edits are separate from these removed
previews. Do not equate similar item names with identical field semantics.

| Adapter | Item classification / application (CPU addresses) | Notable path |
|---|---|---|
| BW / DP | `0x081B7CEC` / `0x0806BD04` | EV medicine handler `0x080FDEA1`; reduction `0x080FDEBD` |
| 西班牙火箭队 | `0x082064AC` / `0x08098EB0` | EV handler `0x081361D5`; mint handler `0x08136DDD` |
| 究极绿宝石 | Same medicine entry points as verified Emerald adapter | Ability handler pointer `0x08F7F111` |
| 水银 | `0x08126C68` / `0x08042414` | EV handler `0x080A16E1`; ability pointer `0x09D58019` |

西班牙火箭队 mint target getter `0x0810FA0C`, SetMonData `0x08097E2C`
(field 89), recalculate `0x080967B4`. The observed mint preserves PID and trainer
identity. Ordinary ability callback `0x0820318C` (applied `0x0820336A`), hidden
toggle callback `0x08203430` (applied `0x0820361A`).

水银 ability selection `0x09D561E0` → checkpoint `0x09D56246`;
application `0x09D55FB4` → `0x09D56006`. RNG can reroll PID, so ability slot
editing is not automatically equivalent to this item operation.

究极绿宝石 ability variant getter `0x080D7644`; effect entry `0x09F00DD0`,
accepted `0x09F00E20`, rejected `0x09F00E3E`, applied `0x09F00E2C`.
The observed path preserves PID and can accept without changing an ability.

| Crown service | Script/menu entry (ROM offsets) | Native effects (CPU addresses) |
|---|---|---|
| 究极绿宝石 | Root `0x1812758`, level check `0x181278F`, menu `0x181279A`, choices `0x1700000` | Level `0x098126D8`, mark `0x098126FC`, mask `0x09812720` |
| 水银 | Root `0x7B05BA`, entry `0x7B000A`, level checks `0x7B07B0` / `0x7B099B`; stat dispatch `0x7B0229` | Level `0x0896A730`, base-IV setter `0x09D5C1D8` |

究极绿宝石 marks effective training without replacing base IVs or immediately
refreshing saved party stats. 水银 changes base IVs and recalculates stats.
Its silver branch checks at `0x7B00C0` but the observed removal at `0x7B019C`
attempts gold payment before the single-stat menu. This is a native-command
observation, not a complete transaction/access guarantee.

Evidence: [EV items](../verification/training-items-20261006.md),
[mints](../verification/training-natures-20261006.md),
[abilities](../verification/training-abilities-20261006.md),
[crowns](../verification/training-crowns-20261006.md),
[service previews](../verification/training-service-previews-20261006.md).

## Ultimate trainer difficulty / 究绿训练家难度说明

The retained trainer page uses one difficulty selector and automatically reads
the current saved party. It no longer has manual roster, seed or level overrides.
Unknown dynamic values remain unknown without a SAV; ordinary fixed records can
still be previewed with ROM alone. Random native outputs are labelled samples or
alternatives, not guarantees about the next battle.

The verified generation path adjusts **levels / IVs / EVs**, not the species
base-stat table. The party header does not contain four independent mode parties.
Do not label EV templates, IV-quality bytes or resulting battle stats “base stats”.
This is the only currently verified difficulty-dependent trainer adapter in the
supported set; other hacks' unverified difficulty mechanisms are not inferred.

Key CPU entries: EV constructor `0x09F042BC`, player speed `0x09F03F0C`,
player role `0x09F0229C`, summary sort `0x09F03E6C`.
Template table ROM offset `0x1F0AF60` (8-byte records).
Keep the native routines and the shared preview APIs used by the trainer page.
Current scene/player party, battle flags and random branches limit exactness.

See [Ultimate native generation and boundaries](ultimate-emerald-55.md) and
[`ultimate_ev.rs`](../../crates/gen3-core/src/ultimate_ev.rs).
