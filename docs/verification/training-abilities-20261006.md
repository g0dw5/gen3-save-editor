# Ability-item native previews / 特性道具原生预览

The shared read-only Training reference now reads recognized item handlers from
the loaded ROM, links acquisition → conditions → map/entrance → back, and shows
native acceptance plus actual ability, PID, effective nature and six-stat results.
Stored party/boxed individuals or an explicit simulated individual are supported.
No SAV record is changed, no item is consumed, and no ROM is written.

统一“培育资料”页新增特性道具的获取跳转、原生允许／拒绝判断和实际前后结果。
道具、特性、宝可梦名称及说明来自当前 ROM。水银的预览必须指定随机种子情景，
不预测游戏下一次生成的 PID。整体培育流程仍为 **P**，并非所有机制都已追齐。

## Exact scope / 精确范围

| Fingerprint | Runtime offers | Verified bounded stage | Remaining gaps |
|---|---:|---|---|
| Rocket `59c658a1081f542086de1060bb65f0b3` | 2 | Ordinary capsule and hidden-slot toggle guards; confirmed persistent changes | Menu/setup/access, consumption, complete NPC services |
| Mercury 1.2 `f323df1792ac68462a34b42fe8571533` | 1 | Native capsule guard, seeded PID selection and persistent change | Live RNG, menus/setup/access, other handlers/services, all PID-linked appearance cases |
| Dark Phantom BW `0d9b129f7dd76895f79bb47ad7dec2fe` | 0 configured | Existing EV previews preserved | No verified ability-item adapter; not proof of absence |
| Dark Phantom DP `cb2940215f4dafb1bef133c3af379f44` | 0 configured | Existing EV previews preserved | No verified ability-item adapter; not proof of absence |
| Ultimate Emerald 5.5 `17ce9785b33319b3dbda9a5d37c57ec1` | 2 (later increment) | [Native script prefix, variants and seeded ordinary-slot choice](training-ultimate-abilities-20261006.md) | Menus/access/consumption and complete services remain incomplete |

Offers are identified by runtime field-use handlers, never by a bundled name list
or presumed item number. Necessary native boundaries are adapter metadata;
species ability entries and item names remain current-ROM data. Empty results
mean no verified adapter, not an absent game mechanic. The native guard tests
include unused/duplicate table entries; they do not establish obtainability.

## Rocket native flow / 西班牙火箭队

- Ordinary handler `0x081361F1`: initializer `0x08203398` stops before UI at
  `0x082033FA`. Native task state records current slot XOR 1. Callback
  `0x0820318C` accepts at `0x08203238` or rejects at `0x082031FA`.
  It rejects missing/equal normal abilities, invalid species and targets beyond
  normal slots. In particular, a hidden-slot individual is not automatically
  accepted just because its species has two normal abilities.
- Hidden-toggle handler `0x0813620D`: initializer `0x08203648` stops at
  `0x082036AC`; target is slot 0 from hidden slot 2, otherwise slot 2. Callback
  `0x08203430` accepts at `0x082034E8` or rejects at `0x082034AC`. It checks that
  the hidden ability exists and differs from **both** normal abilities.
- Confirmed task state 5 runs the actual native callback. Observations stop after
  `SetMonData` field 46, before bag consumption: `0x0820336A` / `0x0820361A`.
  Rocket stores an independent two-bit ability slot and retains PID.

These are complete native prefixes at verified boundaries, not replacements of
native eligibility with “there are two entries” heuristics. A missing boundary,
unexpected return, invalid memory access or execution limit is an error, never
silently interpreted as item rejection.

## Mercury native flow / 水银

Handler `0x09D58019` selects the capsule workflow. The preview sets selected-item
RAM `0x0203AD30` and runs native selector `0x09D561E0` to `0x09D56246`; register r4
contains the selected **ability ID**, with zero indicating rejection. It compares
native current/first/second/hidden ability getters; the current slot alone is
insufficient to reproduce decisions, especially with duplicate ability IDs.

For acceptance, the complete persistent prefix `0x09D55FB4` → `0x09D56006` clears
the hidden flag, selects a new PID through the loaded ROM's RNG and constraints,
writes native individual data and recalculates stats. Ordinary ability selection
is PID-parity based; the query does not substitute Rocket's independent slot.
`rng_seed` is an explicit unsigned 32-bit scenario. `rng_after` is evidence, not a
claim about the emulator's current RNG. The native result may reroll PID.

The sampled native changes retained nature, gender and shiny status. This does
**not** certify every species/form/pattern constraint. The UI shows actual before/
after PID and warns about PID-linked appearance. Existing Mercury SAV editing
uses its own deterministic constraint solver; no byte-exact equivalence to every
seeded native reroll is asserted, and this query adds no SAV-writing operation.

水银的普通特性依赖 PID 奇偶，不能沿用西班牙火箭队的独立槽位逻辑。页面会明确
展示 PID 前后值。测试中的性格／性别／闪光保持不变，仍不把这个有限样本扩大为
所有形态或斑纹都不变的保证，也不把现有确定性存档修改宣称为某次原生随机结果。

## Independent verification / 独立验证

`scripts/verify_training_abilities.py` runs the exact ROMs in the read-only mGBA
fixture, without function hooks, source SAV access or ROM writes. The production
runner is independently compared with those private vectors:

| Profile | Native guard cases | Persistent cases | Accepted / unchanged cases | Exact existing editor matches |
|---|---:|---:|---:|---:|
| Rocket | 8,364 | 360 | 120 / 240 | 120 accepted cases match both 100-byte party and 80-byte box edits |
| Mercury 1.2 | 4,659 | 180 | 84 / 96 | No byte-exact editor claim; all 180 previews match mGBA |

Guard sweeps cover three starting slots for every runtime species-table entry,
for each supported item. Guard execution preserves all 100 input bytes. Rich
persistent cases use dual-ability examples plus single/duplicate/hidden examples,
six seeds, nonzero EVs/IVs, friendship, conditions/origin history, status and
alternating shiny/non-shiny identities. These samples are not exhaustive PID or
live-menu coverage.

- All **13,023** accept/reject and target results match independent native runs.
- All **540** persistent outputs match mGBA, including PID/RNG effects and native
  party stat bytes. Rocket changes preserve PID; Mercury changes rerolled PID in
  84 cases. No sampled nature/gender/shiny differences occurred.
- Runtime invariants compare canonical persistent data and unrelated headers,
  allowing only the verified ability field and, for native PID-selection rules,
  PID to differ. OT/species are retained. Unexpected changes fail the query.
- Item query/map closure: Rocket 6 and Mercury 23 parsed source/map references.
  These are qualified references, not proof of current access, payment or receipt.
- Five-profile catalog regression retains 12 EV offers per ROM, 21 Rocket mint
  entries, and no cross-ROM leakage of mint/ability adapters. Previous native
  EV (660 effects) and mint (1,512 scenarios) regressions are rerun.
- Bilingual browser fixtures cover normal/hidden selection, native rejection,
  PID display, seed bounds, acquisition/map/back input retention, SAV-refresh
  invalidation and stale-result rejection after input changes or ROM switches.
  These are synthetic UI tests, separate from exact-ROM execution evidence.

Integration checks pass: 121 public core tests (38 opt-in ignored), the three
local native training regressions, four native-runner safety tests, workspace/CLI
compilation, TypeScript/Vite production build, Rust/Prettier formatting, manifest
version consistency and five release-workflow tests. The Vite large-chunk advisory
remains. CLI catalogs and explicit simulated ability previews were checked across
the five profiles. The strengthened independent verifier was repeated and its
vectors compared with the first run.

No new edited-SAV simulator load/save/re-read round trip is claimed. Menus,
consumption, live battle/facility context, actual current access, complete training
services remain unresolved. Ultimate's two item variants are covered by the later linked increment.

## Reproduction / 复现

Set the five exact private `GEN3_ROM_*` paths. Compile the read-only
`scripts/native/breeding_probe.c` against mGBA with its `callchoice` observation API:

```sh
/usr/bin/python3 scripts/verify_training_abilities.py --gen3 /path/to/gen3 \
  --mgba-probe /path/to/readonly-probe.dylib --output /private/abilities-native.json
GEN3_TRAINING_ABILITY_PROBES=/private/abilities-native.json cargo test -p gen3-core \
  local_training_ability_items_match_native_and_keep_profiles_isolated \
  -- --ignored --nocapture
```

Developer CLI: `gen3 training-items ROM`,
`gen3 training-preview ROM REQUEST.json [SAVE]`. A simulated individual can supply
`ability_slot` when supported by the exact codec. PID-generating previews require
an explicit `rng_seed`; invalid/nullable non-optional integers are not coerced.
All runtime catalogs, native vectors, binaries and real files remain private,
excluded from Git and release inputs. Version remains unchanged; no new package
or release is created by this increment.

The [Ultimate extension](training-ultimate-abilities-20261006.md) adds 9,592 guard and 528 persistent cases, with runtime variant/seed contracts and successful-but-unchanged results. Its callback lives in ROM; the earlier relocation hypothesis is corrected. Previous Rocket/Mercury counts above remain scoped to their original evidence.
