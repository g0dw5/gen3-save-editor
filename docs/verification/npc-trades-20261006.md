# NPC trade quotes / NPC 交换报价

Increment after `3ea4127`; development version unchanged. This closes the bounded
trade quote → required Pokémon / held item → NPC tile → entrance → planning HTML
workflow. It does not certify every exchange or complete delivery/story semantics.

## Runtime sources / 实时来源

`PokemonScriptRules.trade` separates native dispatch/layout configuration from
game content. The shared reader verifies the information-special target, reads
the table pointer from its current literal pool and bounds the indexed record.
Species and held items are validated through current ROM tables. Only a parsed
map-script invocation with a resolved index contributes a source; unreferenced
table records do not become acquisition locations. Map-level invocations without
a verified tile remain unplaced. Names, pictures, offers and NPC catalogs are
not embedded in source or release inputs.

| Fingerprint | Information special | Dispatch table | Native information code | Runtime table-pointer literal | Bounded records | Parsed map quotes |
|---|---|---|---|---|---:|---:|
| BW | `0xFF` | `0x1DBA64` | `0x7E73C` | `0x7E774` | 4 | 4 |
| DP | `0xFF` | `0x1DBA64` | `0x7E73C` | `0x7E774` | 4 | 4 |
| Rocket | `0xFF` | `0x22B620` | `0xB3D1C` | `0xB3D54` | 4 | 4 |
| Ultimate | `0xFF` | `0x1DBA64` | `0x7E73C` | `0x7E774` | 8 | 8 |
| Mercury 1.2 | `0xFC` | `0x15FD60` | `0x53A9C` | `0x53AD4` | 9 | 14 |

Rocket uses an Emerald-indexed trade group in its hybrid dispatch table. FireRed's
usual `0xFC`/`0xFD` targets are other functions in this exact ROM; engine ancestry
alone is not adequate evidence. The 60-byte ordinary record stores received
species at +12, held item at +40 and requested species at +56. Offsets/widths were
checked against the actual native information and generation routines.
The [Emerald](https://raw.githubusercontent.com/pret/pokeemerald/master/src/trade.c)
and [FireRed](https://raw.githubusercontent.com/pret/pokefirered/master/data/specials.inc)
decompilations provide naming/orientation only; exact ROM execution is authoritative.

## Independent native parity / 独立原生对照

`scripts/verify_npc_trades.py` opens five exact, hash-verified ROM inputs, executes
native quote and ordinary generation entry points in synthetic RAM and then
queries the generated record with native getters. No SAV is opened or written.
The selected party member's **level getter is injected** at levels 1, 50 and 100;
two RNG seeds are used per level. Creation, setters, stat calculation and final
species/level/held-item getters execute native code. This tests how the generator
uses a selected member's level, not party eligibility, capacity or UI selection.

29 table records × six cases = **174 native cases**. The independent Rust reader
matches requested species, received species and held item in every row; native
received level equals the injected offered level in each case. Native emulated
ROM bytes and input-file hashes remain unchanged. No fixed IV/nature/ability or
post-trade evolution guarantee is inferred from this narrow parity check.

Mercury's creation entry is hooked to a dispatcher at `0x1D0DF7C`. Native flag
`0x15F8` chooses a custom constructor at `0x1D0DDE8`; unset follows the ordinary
table path. One additional branch probe verifies this dispatch and stops at the
custom constructor. Its generated output is **not** certified. Ordinary quote
sources retain this mode guard; explicit prior script writes set/clear it in
path context instead of misreading it as an unchanged SAV prerequisite.

## Shared workflow / 共用查询流程

- ROM-only quotes show the exact requested Pokémon, same-as-offered level rule,
  held-item reference and unresolved delivery/access notice. Required species and
  held items link back to their own acquisition queries.
- SAV overlay looks for exact-species non-egg individuals. Party candidates show
  their levels; box candidates explicitly require moving into party; absence
  adds a missing prerequisite. Having a donor never proves access or completion.
- The collection planner preserves trade source/guard/donor context. Independent
  HTML uses the same bilingual explanation and escapes all ROM names and evidence.
  It never creates Pokémon, executes exchanges or changes story/dex flags.
  Trade-held item goals require the explicit unknown-reward option; a missing
  donor does not turn unknown exchange receipt state into a known unclaimed item.
- Quote references can occur in retained/unused maps or scripts; static entrance
  chains do not prove that a map or NPC is currently reachable. No claim of one-time
  availability or reset semantics is made from object visibility.

## Checks / 检查

Public fixtures cover all five adapters: dispatch mismatch, runtime table changes,
index bounds, NPC tiles, known/unknown alternate-mode guards, read-only donor
selection, party/box separation, egg exclusion, held-item crosslinks and planning.
The opt-in `local_npc_trades_match_native_quote_and_generation` checks all native
vectors and the 34 parsed map quote references. Browser regression verifies source
→ donor → NPC tile → back → bilingual planning/escaped HTML with no mutation calls.

The current check run passes 97 public core tests (22 opt-in tests ignored by
default), the five-ROM acquisition/collection and entrance regressions, and the
strengthened native-trade parity test that also queries each referenced received
Pokémon and nonzero held item. Trade browser navigation and bilingual HTML pass;
TypeScript, production Vite and formatting/version checks pass. Clippy still
reports the six existing baseline warning sites recorded in the prior query
verification; no new trade-path warning was reported. Vite retains its existing
large-chunk advisory. These checks do not establish every remaining capability
in the project matrix.

```sh
cargo test -p gen3-core -p gen3-cli --locked
# Five exact private paths in GEN3_ROM_*; parity file stays outside release inputs.
uv run --with unicorn python scripts/verify_npc_trades.py > /private/trades.json
GEN3_TRADE_PROBES=/private/trades.json cargo test -p gen3-core local_npc_trades -- --ignored --nocapture
uv run --with playwright python scripts/test_npc_trade_ui.py
```

本轮闭环覆盖已解析的普通交换报价，不是所有交换的完整支持。五份精确 ROM 共
29 条表记录经 174 个原生生成情景对照；34 个地图引用不等于 34 个可达 NPC。
存档仅叠加对应非蛋候选及其所在位置，不按背包、图鉴或 NPC 消失推断已交换。
水银特殊模式、实际交换动画／后续进化、领取／刷新标记、完整任务依赖和当前
可达性仍待验证；不宣称新的编辑存档模拟器再次保存证据。用户随后要求本机
测试包，构建及启动证据随该测试目录的 BUILD-INFO.txt 保存；版本号不变，不公开发布。
