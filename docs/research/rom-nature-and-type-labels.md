# Runtime nature and type labels / 性格与属性实时读取

2026-09-20 · development version 0.2.1, not packaged for release.

## Findings and implementation

The previous frontend reused one translated list of nature names and one list
of type names for all ROMs. This hid Rocket's native “内敛”, “爽朗”, “慢吞吞” and
“温和”, and replaced BW/DP's own terminology. The earlier data-ownership audit
incorrectly classified these game strings as ordinary UI translations.

All three catalogs now read the game's active tables, on demand:

| Exact profile | Nature pointers (25 × u32) | Signed nature changes (25 × 5 bytes) | Type strings |
| --- | --- | --- | --- |
| Dark Phantom BW / DP | `0x61CB50` | `0x31E818` | `0x31AE38`, 18 × 7 bytes |
| Rocket 2.1 Chinese | `0xD052EC` | `0x5B335C` | `0x5A6480`, 19 × 10 bytes |

Addresses, widths and bounds are adapter configuration. No extracted list is
bundled. Native nature IDs are unchanged. BW/DP's strings include their own
parenthesized stat hints; preserve these instead of replacing or reconstructing
them. In particular, the old unreferenced strings around `0x61CAAC` are **not**
the source used by the current BW/DP summary screen.

Pointer literals referenced by the native UI establish the active tables:

- BW/DP: nature `0x73188`, type `0x166F4`, nature effects `0x6D914`.
- Rocket: nature `0xA1EA4`, type `0x16780`, nature effects `0x9AFF4`.

The catalog owns nature names/changes and type strings. Pokémon editor choices,
PID-nature sentinel text, ability-page markers, trainer parties, move prefixes,
Hidden Power and evolution conditions all consume that catalog. Switching UI
language translates controls, not ROM strings. Search uses the displayed ROM
names and numeric IDs; there is no official/legacy-name alias fallback.

## Calculation difference

Nature effects are ordered Attack, Defense, Speed, Sp. Attack, Sp. Defense.
The signed ROM values -1/0/+1 mean 90/100/110 percent, with no HP change.
Both games share the table reader and stat calculator, but the multiplication
width is a verified profile rule:

- BW/DP native routine `0x6D8D4` truncates the multiplication result to `u16`
  **before** dividing by 100 for increased/decreased stats.
- Rocket routine `0x9AFB4` divides the full product before returning `u16`.
- A neutral stat returns unchanged without multiplication. For an unmodified
  value of 609 with an increase, BW/DP return 14 and Rocket returns 669. This is
  a native high-stat edge case, not a recommendation to create such Pokémon.

This fixes the displayed calculation and derived party stats on an explicit
edit. Loading a save does not rewrite it. The nature-edit operation is unchanged:
Rocket uses its existing mint-equivalent override field; BW/DP use their existing
PID constraints. No new save fields or write ranges are introduced.

## Wider source audit

Reviewed the frontend name providers, bundled data imports, catalog readers and
profile-driven references as well as nature/stat calculation.

| Display content | Source after this change |
| --- | --- |
| Pokémon, moves, items, abilities, descriptions, trainer/class and location names | Active ROM tables/pointers; unchanged |
| Natures, type names, nature modifiers | Active ROM tables; corrected here |
| Move power/category/PP, six base stats, growth thresholds, evolutions and forms | Active ROM data plus verified adapter semantics |
| Fishing rods, balls, evolution items and learned-move references | ROM names resolved from profile/table IDs |
| Artwork, map/NPC/item placements and encounters | ROM graphics/events/scripts; no extracted asset bundle |
| Official stats, approved species mappings | Explicit user-authorized external reference/configuration; separate from ROM values |
| Form labels and bit/enum meanings | Reviewed identity/rules plus UI wording; no replacement species-name catalog |
| Stat labels, physical/special/status, gender, source-game/language IDs, ribbon/virus field explanations | UI/format descriptions; not fallback game-name databases |

This is an audit of the currently implemented displays, not a claim that every
unimplemented battle or script mechanic has been reversed. Do not copy these
profiles' table sizes, arithmetic width, ribbon semantics or special IDs into a
new adapter without verifying its engine.

## Validation

- Synthetic cross-profile tests mutate ROM strings and nature effects and verify
  that fresh catalogs and calculated stats follow the new bytes. Invalid nature
  IDs and pointers fail; no bundled name fallback is used.
- `scripts/verify_rom_labels.py` checks active native pointer literals, all 25
  names and type-table bounds, then executes 1,050 native nature-stat probes per
  ROM (3,150 total), including neutral HP and truncation boundary cases.
- `scripts/test_rom_labels_ui.py` uses catalogs from the exact private ROMs and
  synthetic saves only. It checks nature search, stat markers, ROM type labels,
  trainer names and both UI languages in BW → Rocket → DP order.
- Existing editor-navigation, cross-adapter and readable-reference UI regressions
  remain applicable. Core/save tests use generated records or in-memory copies;
  they do not overwrite the user's source files.

中文摘要：性格及属性属于 ROM 内容，不应归入修改器自带译名。本次移除两套固定
列表，按版本读取原生表；同时让能力修正、↑↓ 标识共用 ROM 数据，并复刻漆黑
特有的乘法截断。西火可直接搜索“内敛”，漆黑继续显示 ROM 内的“保守”等名称。
按钮和说明仍支持中英文，切换语言不会替换游戏原名。版本保持 0.2.1，未打发布包。
