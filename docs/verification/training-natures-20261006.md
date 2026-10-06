# Mint persistent effects / 薄荷持久化效果

The shared Training reference now lists runtime-recognized Rocket mint items,
links each to acquisition/conditions/maps/entrances, and previews its effective
nature and six stats for a stored or explicit simulated individual. The query is
read-only. This is a bounded addition to partially verified training workflows,
not a complete training catalog or a new save-writing operation.

培育资料新增西班牙火箭队薄荷的获取跳转与效果预览。目标性格、道具名及说明均来自
当前 ROM。支持存档个体或明确模拟初始实际性格；保留实际性格与 PID 性格的区别。

## Exact-ROM scope / 精确 ROM 范围

| Registered fingerprint | Mint persistent stage | Other training mechanisms |
|---|---|---|
| Rocket `59c658a1081f542086de1060bb65f0b3` | 21 runtime item entries; verified bounded scenarios below | Complete menu/access/consumption, ability changes and services remain partial/unverified |
| Dark Phantom BW / DP | No verified mint handler configured | Not a claim that a mint or nature service is absent |
| Ultimate Emerald 5.5 | No verified mint handler configured | Existing override editing is separately supported; item/NPC equivalence not established here |
| Mercury 1.2 | No verified mint handler configured | Custom ability/EV handlers and services remain unresolved |

All five fingerprints retain **P** for the overall training workflow. Empty
`nature_items` means no verified handler was configured, not a verified absence.

## Native control flow / 原生执行链

1. Current item record field-use handler `0x08136DDD` selects the mint workflow.
2. Initialization at `0x08208350` reads the current effective nature through
   `0x0809A334(..., 1)` and the selected item's target through `0x0810FA0C`.
   The target getter reads the current item record's byte at `+40` after the native
   item-index check; no target list is embedded in the editor.
3. Task callback `0x08208160`, state 0, compares those effective natures and
   rejects a matching target. PID nature alone does not determine this rejection.
4. Confirmed state 5 writes field 89 through `0x08097E2C`, then recalculates stats
   with `0x080967B4`. Field 89 changes canonical byte 9 bits 5–7 and byte 10 bits
   0–1, retaining PID, OT identity and all unrelated persistent fields.
5. The callback subsequently consumes an item and returns through the menu. The
   reference preview does **not** execute that stage or certify eligibility,
   item possession/payment, live menu state or current access.

The preview executes the complete native setter and stat calculator, in disposable
RAM; it does not replace them with an official formula. The equal-nature branch
is reproduced from separately observed native comparisons. No SAV item is consumed,
no individual is inserted and no Pokédex/story field is changed.

## Independent verification / 独立验证

`scripts/verify_training_natures.py` runs the exact ROM in the read-only mGBA
fixture, without function hooks or ROM writes:

- All 21 native item-target getters match the current ROM's item field. Each real
  initialization callback is observed before its UI stage and stores that target
  without changing the individual.
- 1,512 cases cover all 21 targets, 24 PID substructure permutations, six sampled
  species, levels 5–28, nonzero EVs/condition/history/friendship, a status condition,
  and zero/partial/full current HP. Complete native setter/calculator calls match
  the real task callback observed immediately before bag consumption.
- Native effective-nature equality branches are observed independently: 792 normal
  no-effect scenarios retain every byte; 720 changing scenarios produce the
  expected persistent result. Unrelated canonical bytes, PID/OT headers and status
  remain unchanged, and source ROM/cartridge bytes stay unchanged.
- The opt-in Rust test matches all 1,512 previews byte for byte. In the 720 changing
  scenarios, the **existing** SAV editor's nature-override patch matches the native
  party result (100 bytes) and boxed persistent result (80 bytes) exactly. This
  verifies sampled persistent-operation equivalence, not consumption, inheritance
  or every possible individual. Disposable source SAV data stays unchanged.
- Five-profile EV-item regression remains byte-exact for all 660 earlier vectors.
  The public core suite passes 120 tests (37 opt-in ignored); the new native test
  and previous EV test were separately run. Workspace/CLI compilation and the
  TypeScript/Vite production build pass.

No new edited-SAV emulator load/save round trip is claimed by this read-only
increment. Existing editor functionality remains subject to its previous evidence.

## Reproduction / 复现

Set the exact private `GEN3_ROM_*` paths for the five profiles. Run:

```sh
/usr/bin/python3 scripts/verify_training_natures.py --gen3 /path/to/gen3 \
  --mgba-probe /path/to/readonly-probe.dylib --output /private/natures-native.json
GEN3_TRAINING_NATURE_PROBES=/private/natures-native.json cargo test -p gen3-core \
  local_training_nature_items_match_native_and_existing_save_editor -- --ignored --nocapture
```

Runtime item lists/native vectors/ROMs/SAVs are private test inputs, not committed
catalogs or release assets. The shared developer CLI remains `training-items ROM`
and `training-preview ROM REQUEST.json [SAVE]`. Simulated requests may provide
`nature_override` (0–24, or 26 to use PID nature) only where the codec supports it.
Unsupported mechanisms retain their explicit limits instead of inheriting another
ROM's rules.

Native six-stat displays read the actual disposable party bytes, rather than
substituting decode-time calculations. A runtime invariant rejects mint outputs
that change unrelated persistent/header data. The 21 entries have 69 parsed
acquisition/map-navigation references; those references do not prove access or
current availability.

The five-profile bilingual browser fixtures pass item → acquisition → map → back,
saved-individual selection/reload, simulated effective-nature input, normal
unchanged-nature rejection and compact layout. A fixture deliberately makes
native party stats differ from decode estimates to check the displayed source.
Both Rocket and Mercury switch-ROM checks discard delayed responses, clear SAV
context and remove old nature controls. These synthetic UI checks are separate
from exact-ROM CPU evidence. Formatting and manifest-version checks also pass;
this increment keeps the current unreleased version and does not package/release.
