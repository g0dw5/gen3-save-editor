# Ultimate native ability items / 究极绿宝石原生特性道具

This increment extends the shared read-only Training reference to the exact
Ultimate Emerald 5.5 fingerprint `17ce9785b33319b3dbda9a5d37c57ec1`. It lists the two
runtime handlers/variants, supports stored or simulated individuals, shows actual
abilities/PID/nature/native party stats, and links item → acquisition/conditions →
map/entrance → back. The overall training workflow remains **P**.

究极绿宝石的胶囊与膏药使用同一菜单入口，通过当前 ROM 的道具变体区分。
膏药可从隐藏特性返回普通特性，并随机选择普通槽位；PID 保持不变。部分胶囊
情景会进入成功分支却不改变个体，页面明确提示，不把成功当作特性已经改变。
本次只读，不改 SAV、不扣道具、不生成宝可梦，不写 ROM。

## Corrected native chain / 原生调用链修正

The earlier “relocated ability handler” description was inaccurate. The ROM has
menu wrappers, a RAM-stored callback pointer, an event script and ROM-native code;
there is no evidence of executing relocated ability code in RAM on this path.

1. Item field-use handler `0x08F7F111` stores callback `0x08F7F12B` at RAM
   `0x0203A0F4` and invokes ordinary item-menu setup. Callback initializes script
   `0x08F7F0D0` through `0x08098EF8`; the initial script opens party selection,
   compares selected index with the cancellation value, then transfers control.
2. Script `0x0986419F` calls native trampoline `0x08C60D29`, which branches to
   ROM routine `0x09F00DD0`. The result variable drives ROM-script messages.
3. The preview sets selected index `0x020375E0` to 0 and selected item
   `0x0203CE7C`, with the individual in disposable party slot 0. It executes the
   native prefix. Native getter `0x080D7644` reads the loaded item's variant:
   1 = ordinary switch, 2 = hidden toggle. Unsupported variants fail rather than
   inheriting another item or ROM's behavior.
4. Native `GetMonData` field 46 returns 0/1 for normal slots and 2 for a hidden
   individual, independent of the retained normal bit. Target values 2 and 3
   both set the header hidden flag; the normal bit remains stored beneath it.
5. For the ordinary capsule, the native table comparison uses current slot XOR 1.
   For hidden individuals this can select 3 and reach acceptance without changing
   persistent data. The query executes this behavior, rather than adding a
   Rocket-style hidden-slot rejection or silently promising a new ability.
6. For the patch, ordinary → hidden compares hidden/current IDs and toggles the
   hidden bit. Hidden → normal uses native RNG `0x0806F5CC` to select a normal
   slot. The result depends on the explicit seed; PID is retained.
7. Acceptance boundary `0x09F00E20` precedes native `SetMonData` field 46.
   Production previews execute **one complete prefix** to post-setter boundary
   `0x09F00E2C`, or rejection join `0x09F00E3E`. They do not run the guard and then
   repeat its random choice. Execution stops before `RemoveBagItem` and message
   processing; source files and SAV bytes stay unchanged.

The selected-target IDs come from the current species' runtime ability table.
The hidden header bit and canonical normal bit are distinct. Runtime invariants
allow only canonical ability-bit ownership and header byte 30 bit 0, retaining
PID, OT, unrelated header bits (including Hyper Training), nature override and
all unrelated canonical data. Rejected operations must preserve all 100 bytes.
An observation failure is an error, never interpreted as rejection.

## Independent evidence / 独立证据

The public read-only mGBA verifier now includes Ultimate, alongside the previous
Rocket/Mercury profiles. Earlier Rocket/Mercury vectors compare unchanged with
the first independent runs.

| Ultimate native stage | Count | Meaning |
|---|---:|---|
| Guard scenarios | 9,592 | 1,199 runtime species entries × four underlying normal/hidden bit combinations × two items |
| Full persistent scenarios | 528 | Eleven selected species records × four bit combinations × six seed/identity scenarios × two items |
| Accepted and changed | 336 | 84 capsule / 252 patch |
| Accepted and unchanged | 132 | Capsule hidden-flag scenarios; successful branch alone proves no ability change |
| Rejected and unchanged | 60 | 48 capsule / 12 patch |

Guard sweeps include duplicate, unused and unavailable records; acceptance does
not establish obtainability or live menu eligibility. Rich samples have nonzero
EVs/IVs, friendship, status/HP, condition/origin history, effective-nature override,
reserved header bits and Hyper Training flags. The native function retains party
stats rather than recalculating them. Synthetic party stats are deliberately
stored separately from decode estimates; this test does not certify those
synthetic values as a newly generated in-game party.

The production sandbox compares guard targets/decisions and complete persistent
bytes/RNG state with independent mGBA execution. The shared seeded-input contract
covers both PID generation (Mercury) and ordinary-slot selection (Ultimate),
without equating them. All **468** accepted results match existing 80-byte box edits exactly;
PID, sampled nature/gender/shiny state and unrelated fields stay unchanged.
Ten parsed item-source/map references are linked, without proving access. No new
full party-recalculation or edited-SAV emulator round-trip claim is made by this query increment.

## Integration and limits / 集成与边界

- The shared bilingual UI shows normal/hidden item variants, native rejection,
  acceptance with no change, actual PID/abilities/nature/native stats, and an
  explicit seed for the patch. It never claims a next-live-choice prediction.
- Item/acquisition/map/back retains inputs; SAV refresh, input changes and ROM
  switches invalidate stale results and remove other-ROM controls. Existing
  Rocket/Mercury and all five EV profiles stay covered by regression.
- Runtime names, descriptions, abilities and tables come from the loaded ROM;
  only addresses, formats, field ownership and verified engine semantics are
  configured. No item-name catalog or extracted assets are added to releases.
- Menus, bag consumption, live RNG/facility state, current access, crown services,
  complete NPC training and unverified other-ROM handlers remain incomplete.
  Empty BW/DP ability lists mean no verified adapter, not proven absence.

Reproduce with the updated `scripts/verify_training_abilities.py` and private
`GEN3_ROM_*` paths, then pass its private output through
`GEN3_TRAINING_ABILITY_PROBES` to
`local_training_ability_items_match_native_and_keep_profiles_isolated`.
The CLI remains `training-items ROM` and
`training-preview ROM REQUEST.json [SAVE]`; an item whose runtime offer has
`requires_rng_seed=true` requires a non-null unsigned 32-bit `rng_seed`.
No real ROM/SAV/vectors enter Git or release inputs. Version stays unchanged;
this increment does not package or publish.

Integration validation: **121 public core tests passed, 38 opt-in ignored**.
The three local native training regressions passed, covering five-ROM EV effects,
Rocket mints and all three ability adapters. The five-profile bilingual browser
fixture passed, including Ultimate seed/no-change behavior and ROM-switch resets.
Workspace/CLI compilation, TypeScript/Vite production build, Rust/Prettier
formatting, version consistency and five release-workflow tests passed.
The existing Vite chunk-size advisory remains. Four explicit simulated CLI cases
cover both Ultimate items from normal and hidden starting states.
