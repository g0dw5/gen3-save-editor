# Existing-parent collection preparation / 现有亲本收集准备

Read-only increment after `2c92457`. All five breeding/planning rows remain partial.
Suggestions use exact existing party/box individuals and their current held items,
not inferred reverse evolutions, simulated partners or newly generated SAV records.

## Native selection

| Exact profile | Constructor | Observe before creation |
|---|---|---|
| Dark Phantom BW / DP | `080708C8` | `080708E8` |
| Spanish Rocket | `0809E8B0` | `0809E8D0` |
| Ultimate Emerald 5.5 | `080708C8` | `080708E8` |
| Mercury 1.2 | `080460D4` → patched constructor | `09D1B68C` |

The ordinary native compatibility function runs first; accepted pairs execute the
constructor without replacing calls, stopping before the verified creation call
whose r1 contains species. This executes the ROM's parent choice, baby/item and
branch logic. Mercury's original species helper is insufficient because its
patched constructor has additional selection branches. Exact fingerprints and
checkpoint instruction guards fail rather than substituting a vanilla formula.

Four explicit scenarios sample seeds 42/1 and pending-personality values
24/`8018`. These values are query inputs, not live RNG or a species probability.
The planner checks up to 2,048 distinct unordered party/box pairs in storage order,
preserving record order within each pair; first observed routes represent an
alternative, not an optimal or exhaustive breeding plan. Duplicate locations are
never paired. Existing eggs are excluded. Already deposited daycare individuals,
changing held items to unlock additional babies, reversed parent order and all
possible RNG/form outcomes are not covered by this automatic search.

## Workflow

SAV → Collection planning → missing target → breeding/evolution preparation →
inspect exact parent locations/species/genders and held-item links → optionally
preview this pairing with the complete native ordinary receipt routine → follow
offspring or item references → receiving-service map/entrance when identified →
normally obtain and hatch the egg → directed evolution requirements. Navigation
returns to planning. Standalone bilingual HTML includes the same parent locations,
item names, service/map references, hatching and evolution instructions.

Sources with unresolved access stay unknown. Rocket/Mercury service locations
remain unresolved. Ordinary compatibility and sampled offspring do not prove
complete inheritance, hatching, live setup or access. A production check is not
an egg guarantee. Suggested offspring are not owned, missing goals remain missing,
and no egg, Pokédex bit or story flag is inserted or changed.

## Caching and safety

The runtime-only pairing cache belongs to the loaded acquisition index and is
bound to the current ROM Arc and SHA-256 of complete SAV bytes. SAV edits or input
changes invalidate results. Native writes occur only in disposable RAM; each
original parent record is compared after observation. Query errors are counted,
recorded as technical evidence and presented as unresolved, not impossible.
Coverage displays checked/total pairs and truncation. No-parent/unsupported rules
do not fall back to official breeding formulas or bundled catalogs.

## Evidence

`scripts/verify_breeding_selection.py` uses the existing independent mGBA 0.10.5
probe and complete native vectors from `verify_breeding.py`:

- **210 compatibility scenarios** across all five fingerprints, including accepted
  and rejected pairings; **171 selection checkpoints** exactly match the offspring
  species in complete native egg constructions. Parent records and ROM bytes remain
  unchanged. The test data includes native baby/item and pending-personality branches.
- Rust opt-in verification compares all native results and exercises synthetic SAV
  collection → exact stored pairing → full receipt confirmation, then removes
  synthetic parents to verify SAV-cache invalidation. Whole input SAV and ROM bytes
  are checked; no real user SAV is edited.
- **113 public core tests pass**, 31 opt-in excluded. Bilingual browser fixtures
  verify pairing/item/back navigation, full receipt request locations, bound and
  unresolved warnings, hatching/evolution text and escaped standalone HTML. Parent
  nicknames are treated as text; exported HTML has no executable scripts.
- TypeScript/Vite production build, format checks and release-workflow checks pass.
  No new edited-SAV emulator round trip or live daycare access is claimed.

```sh
python3 scripts/verify_breeding_selection.py \
  --mgba-probe /private/breeding-probe.dylib \
  --baseline /private/full-native-breeding.json --output /private/selection.json
GEN3_BREEDING_SELECTION_PROBES=/private/selection.json cargo test -p gen3-core \
  local_breeding_collection_matches -- --ignored --nocapture
cargo test -p gen3-core
uv run --with playwright python scripts/test_query_collection_ui.py
```

Use the private five-ROM environment variables from the existing breeding probe.
ROMs, SAVs, vectors and extracted assets stay outside Git/release inputs. The
current version remains unchanged; this increment does not package or publish.

本轮补齐“现有亲本 → 原生抽样后代 → 孵化 → 进化”的准备流程，并保留服务地图、
入口、道具和独立 HTML 的关联。整体仍为部分验证：自动配对有界，未覆盖全部亲本
顺序、道具变更和形态分支；已寄养亲本暂未纳入搜索。没有建议不表示不存在途径，
不把亲本兼容或一次构造结果当成当前可领蛋、完整遗传或已获得的证明。
