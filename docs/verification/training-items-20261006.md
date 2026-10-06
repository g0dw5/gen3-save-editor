# Ordinary EV-item reference / 普通努力值道具资料

This increment adds a usable read-only training → item acquisition → conditions
→ map/entrance flow, with native field-effect scenarios for stored or simulated
individuals. It does not certify a complete breeding/training mechanism catalog.

本轮支持培育道具 → 获取途径 → 条件 → 地图／入口，以及同行、盒子或模拟个体的
原生战斗外道具效果预览。菜单资格、消耗、付款和特殊设施并未纳入模拟。

## Source and native evidence / 来源与原生证据

`training.rs` identifies medicine/reduction dispatch entries from the current ROM
item records, then runs that ROM's effect classifier. Category values 12–17
correspond to Attack/HP/Sp.Attack/Sp.Defense/Speed/Defense in verified switch cases.
No item names, descriptions, prices or item catalog are built in. Adapter
configuration contains exact native addresses, handler values and the context-
dependent Enigma item exclusion. Other/custom handlers remain outside this scope.

| Fingerprint family | Native classifier | Native field effect | Verified offers |
|---|---|---|---|
| BW / DP | `0x081B7CEC` | `0x0806BD04` | 12 each: medicine/reduction dispatch |
| Rocket | `0x082064AC` | `0x08098EB0` | 12: ordinary medicine dispatch |
| Ultimate | `0x081B7CEC` | `0x0806BD04` | 12: medicine/reduction dispatch |
| Mercury 1.2 | `0x08126C68` | `0x08042414` | 12: ordinary medicine dispatch |

`scripts/verify_training_items.py` uses independent raw individuals and mGBA,
executing complete native routines in disposable RAM. **229 item classifications**
and **660 complete application cases** cover zero/90/99/100/251/252/253/255 EVs,
510/511 total EVs and several friendship values. The opt-in core test compares
all 660 resulting 100-byte individuals byte for byte, including native no-effect
returns, identity preservation and unchanged original ROM/SAV bytes.

The public API rejects stale fingerprints, malformed/null/out-of-range fields and
unsupported items. Source files remain read-only. Simulated individuals are temporary
RAM scenarios; nothing is inserted into a SAV or marked in the Pokédex.

Upstream [party-menu code](https://github.com/pret/pokeemerald/blob/master/src/party_menu.c)
was a semantic guide. Category/application evidence comes from each exact ROM,
not a blanket claim that every engine follows Emerald's vitamin formula.

## Important limits / 重要边界

- Classification is a use category, not a proven amount, eligibility or consumption.
  Preview shows actual EV/friendship before and after, never assumed +10 or -10.
- Mercury's successful ordinary application vectors report success without changing
  EVs in the supplied zero persistent context. A preserved instruction at
  `0x0804292A` branches to `0x08042A3A`, skipping the normal HP-EV mutation block.
  Other live/menu/state dependencies are unresolved. This is **not** proof that
  EV training is globally unavailable or broken in the game.
- Stored party data is preserved as scenario input. Box data is converted to a
  full-HP party scenario in disposable RAM. Persistent context may use current SAV
  blocks, but battle/menu/facility state is zeroed; this is not live emulation.
- Native item effects do not validate menu-level Shedinja checks, pay/consume an
  item, open an NPC service or prove current access. A native success result alone
  is insufficient to claim EV gain or full legitimate-operation equivalence.
- Custom reduction handlers in Rocket/Mercury, complete mint menus, ability changes, Hyper
  Training, complete NPC services and subsequent native/save round trips remain
  gaps. Existing editor fields retain their separately documented validation.

中文：五份 ROM 共验证 660 次原生效果，保留 PID、训练家身份及源文件。水银情景
返回成功却不增加努力值，因此页面显示实际前后数值并提示依赖未确认，不宣称
正常游戏中无法培养。未列出的培育方式表示未验证，不能标成“本作不存在”。

## Reproduction / 复现

Run the public verifier with exact private `GEN3_ROM_BW/DP/ROCKET/ULTIMATE/MERCURY12`
paths, a built developer CLI and the read-only mGBA fixture library:

```sh
/usr/bin/python3 scripts/verify_training_items.py --gen3 /path/to/gen3 \
  --mgba-probe /path/to/readonly-probe.dylib --output /private/native.json
GEN3_TRAINING_PROBES=/private/native.json cargo test -p gen3-core \
  local_training_classification_and_effect_match_native_all_profiles -- --ignored --nocapture
```

Private vectors, catalogs and source files are not committed or release inputs.

## Integration checks / 集成验证

- Public core suite: 120 passed, 36 opt-in tests ignored; the independent five-ROM
  native EV-item test was separately run and passed all 660 application vectors.
- Five exact-ROM acquisition/collection queries passed, with runtime item offers
  and CLI scenarios; original ROM bytes remained unchanged.
- The bilingual training-page browser fixtures passed for all five profiles:
  item acquisition → map → back retains inputs; SAV reload clears prior results;
  input edits and ROM switches discard stale responses and stored-individual
  context. Compact layouts have no horizontal overflow. These are synthetic UI
  fixtures, separately qualified from native execution evidence.
- Existing reference navigation and editor retained-tab/draft/multiple-field/fixed
  operation-area regressions passed. Workspace compilation, TypeScript/Vite build,
  formatting, version consistency and five release-workflow tests passed.

No new edited-SAV emulator round trip is claimed for this read-only increment.

A subsequent [mint increment](training-natures-20261006.md) adds Rocket runtime targets and verified persistent-stage previews. It supersedes the unverified mint statement only within its documented scope.

A subsequent [ability increment](training-abilities-20261006.md) verifies bounded Rocket/Mercury guards and persistent effects. Other handlers and complete training services remain unresolved.
