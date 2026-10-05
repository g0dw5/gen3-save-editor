# Ordinary production checks / 普通产蛋检查

Read-only increment after `86b778d`. All five profiles retain partial breeding
coverage. Production and receipt are different scenarios; neither predicts the
next real egg or edits the daycare, parents, bag, flags or ROM on disk.

## Native boundaries / 原生边界

| Exact profile | Ordinary step | Observe before comparison | Roll starts | Modifier |
|---|---|---|---|---|
| Dark Phantom BW | `08070AC4` | `08070B2C` | `070B1E` | Runtime immediate at `310EA2` |
| Dark Phantom DP | `08070AC4` | `08070B2C` | `070B1E` | Runtime immediate at `310EA2` |
| Spanish Rocket | `0809EAAC` | `0809EB1C` | `09EB0E` | Runtime operand at `09F35C` |
| Ultimate Emerald | `08070AC4` | `08070B2C` | `070B1E` | No modifier check in this ordinary branch |
| Mercury 1.2 | `080462C4` | `0804632C` | `04631E` | Runtime shifted immediate at `1D1BF0C` |

Current-ROM code supplies compatibility, item-category/presence checks and
threshold adjustment. The shared sandbox observes the complete step just before
its native comparison; it does not replace calls or patch instructions. The
ordinary bag projection preserves ordered slots, duplicates, capacities and
native quantity encryption/plaintext. An explicit modifier override operates
only in disposable RAM; insertion requires an empty simulated slot. Item names
and acquisition links come from the current ROM, not a bundled charm catalog.

The verified roll instructions zero-extend a 16-bit draw, multiply by the ROM's
immediate scale and divide by its ROM literal. The native unsigned comparison
passes when the adjusted threshold exceeds the result. We count all 65,536
draws using these validated parameters. Unknown instruction sequences and zero
parameters fail rather than returning an official-game formula.

This is a theoretical per-check chance over uniform 16-bit draws, not a forecast
of the current RNG sequence. The UI rounds the percentage; exact numerator,
denominator, threshold and scenario roll remain in expandable evidence. Because
the divisor is 65,535 in these inputs, nominal percentages need not be exact
multiples of 1/100 (for example threshold 20 gives 13,107/65,536).

## Native differences / 版本差异

- All five tested ordinary steps check at a specific low-byte parent-step phase,
  repeating every **256 steps**. This is not a claim that the next check is 256
  steps away; deposit/step phase and live state are not restored.
- BW/DP adjust a zero compatibility threshold to 20 when the verified modifier
  is present. Independent complete steps also produce a pending marker. The UI
  explains this native branch and does not declare a normally obtainable child
  from incompatible parents. Rocket/Mercury leave zero unchanged.
- For tested base thresholds 20/50/70, BW/DP/Rocket/Mercury produce adjusted
  thresholds 40/80/88 with the verified item. Ultimate's tested ordinary branch
  keeps its original threshold; this is not evidence of game-wide item absence.
- Mercury's patched generator `09D1BB90` sets the flag operand read at
  `1D1BB9C`, while its old 16-bit pending-personality field stays zero. The flag
  getter supplies the independent full-step outcome; zero personality alone is
  not a no-egg test. The query clears this availability flag only in isolated RAM
  to establish its explicit no-pending scenario. Full custom receipt/live daycare
  storage and egg identity are not decoded by this increment.
- Mercury's per-step wrapper `080463B8` calls the ordinary step and is referenced
  by field code at `06D704` and a pointer at `1D4E328`. This does not identify a
  reachable daycare NPC or prove all custom service setup.

## Workflow / 使用流程

Pokémon acquisition → expand native daycare preview → choose parents → choose
ordinary SAV-bag projection (or empty bag without SAV), with-item simulation or
without-item simulation → preview → inspect production probability/assumptions
→ follow the ROM-named modifier item → acquisition source/map/exterior entry →
back. The offspring/move/service-map workflow remains available in both languages.
ROM/SAV changes invalidate prior scenario results; no save action is issued.

The scenario assumes two deposited parents, ordinary bag context and no pending
egg. It does not establish current access, live facility flags, whole-game setup,
production refresh, complete inheritance/forms or hatching. Rocket/Mercury
receiving-service references remain unresolved. Collection planning still does
not treat simulated children as owned and no source is promoted to reachable.

## Evidence / 验证

`scripts/verify_breeding_production.py` uses the RAM-only mGBA probe, without
opening a user SAV or replacing native functions:

- **327,680 native roll results**: every possible 16-bit draw for each fingerprint.
- **270 complete native production steps**: five parent combinations, two
  security-key contexts, with/without the verified item where applicable, and
  low/middle/high forced first draws obtained through the inverse native LCG.
  Whole-step pending/availability outcomes agree with the observed comparison.
- **30 gate cases**: non-check phases, the next check phase and a preexisting
  legacy pending marker. These do not certify complete custom pending state.
- Rust verifies every native roll, all 270 synthetic SAV projections and 240
  explicit item overrides, retaining input ROM and SAV bytes. No real SAV is
  edited; synthetic SAV construction is test-only.
- The updated mGBA observer reproduces every prior **210 receipt/compatibility
  vector, 171 generated individuals and 120 CPU microcases** exactly. It retains
  coherent board memory/timing caches when restoring registers after a stop
  inside ROM; a stale BIOS fetch cache was rejected during harness development.
- **109 public core tests passed**, with 28 opt-in tests excluded. The browser
  fixture checks probability, explicit-item invalidation, item/query/back,
  offspring/map/back, bilingual labels/IVs and absence of save/edit calls.
- Five-ROM source/collection/navigation, teaching, resource checks, NPC trades,
  qualified rewards and held-item distributions passed. Mercury storage and
  Rocket/Ultimate adapter regressions passed. **1,484 Mercury trainer scenarios**
  are exactly unchanged from the preceding native verification after the shared
  observer refactor. This does not prove all live-battle context or a new edited
  SAV emulator re-save.

Compile the existing `scripts/native/breeding_probe.c` against mGBA 0.10.5 as
described in [the receipt record](breeding-20261006.md), then use the private
five-ROM environment variables:

```sh
python3 scripts/verify_breeding_production.py \
  --mgba-probe /private/breeding-probe.dylib --output /private/production.json
GEN3_PRODUCTION_PROBES=/private/production.json cargo test -p gen3-core \
  local_breeding_production_matches -- --ignored --nocapture
cargo test -p gen3-core
uv run --with playwright python scripts/test_breeding_ui.py
```

Native vectors, ROMs, SAVs and native libraries remain private and outside release
inputs. This increment keeps the current version, does not package or publish,
and does not claim a new edited-SAV emulator save/re-read.

普通产蛋情景已对照原生分支、完整抽值与普通背包投影；实际寄养状态、自定义服务、
完整遗传／孵化及当前可达性仍有缺口。各版本的总体孵蛋能力继续标为“部分解析”，
不会把概率预览或亲本兼容性包装成已获得、可立即领取或完整支持。
