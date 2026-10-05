# Script resource prerequisites / 脚本资源前置条件

Increment after `b59b3c1`; version unchanged. Read-only acquisition queries now
retain bounded native item/money
checks through result copies, comparisons and conditional branches. This is not
a complete quest/access, spending, teaching or receipt implementation.

## Exact native rules / 精确原生规则

| Fingerprint | Script table | `checkitem` | Script quantity | Normal-bag duplicate slots | `checkmoney` |
|---|---|---|---|---|---|
| BW | `0x1DB67C` | `0x99A6C` | u8 after VarGet | Accumulate | `0x9B4C0` |
| DP | `0x1DB67C` | `0x99A6C` | u8 after VarGet | Accumulate | `0x9B4C0` |
| Rocket | `0x22B218` | `0xCF95C` | u8 after VarGet | Accumulate | `0xD1420` |
| Ultimate | `0x1DB67C` | `0x99A6C` | u8 after VarGet | Accumulate | `0x9B4C0` |
| Mercury 1.2 | `0x15F9B4` | `0x6A6E4` | u16 after VarGet | First matching slot only | `0x6C18C` |

`script_resources.rs` verifies the native dispatch pointer before accepting a
predicate. The shared walker retains that Boolean predicate in VAR_RESULT and
copy aliases, handles all six compare operators (including inverted failure
branches and Boolean-vs-constant branches), and invalidates overwritten results.
Money requirements/actuals are u32; persistent event variables remain native u16.
`checkmoney`'s nonzero ignore operand leaves the old RESULT and comparison intact;
it must not fabricate a new money condition. A check is not a payment operation.

Normal bag reads execute the current ROM's item-ID sanitizer before reading its
pocket and this adapter's save-pocket configuration. The bag loop still matches
the original requested ID. Ultimate's empty slot 377 becomes index 0 for the
category lookup; using its raw pocket byte would give a different result.
The other registered in-range IDs match their raw indices in these five ROMs.
This resource lookup does not classify every raw catalog row as game-obtainable.
PC and wrong-pocket entries do not count. Mercury's native check
fails on an insufficient first match even when later duplicate slots have enough.
A requested quantity of zero still needs a matching item record, including a
native record whose count is zero; an absent item does not automatically pass.
Zero-count reader fixtures do not relax the editor's prohibition on creating
nonempty zero-quantity slots.

**Facility context remains unresolved for BW/DP/Rocket/Ultimate.** Their native
helper selects a separate bag through map state or temporary flag `0x4004`.
Synthetic native probes show RESULT switching from 1 to 0 while the ordinary bag
still holds ten items. SAV alone does not preserve every RAM/map selection.
Those four adapters therefore display the normal-bag effective count as a
reference, with unknown satisfaction and an explicit explanation. Mercury's
verified check has no such alternate-bag branch. This increment does not infer
normal context from a static map record or silently count ordinary items for a
facility check.

Checks after bag/money mutations, shops, standard scripts, battles or operations
whose resource effects are unqualified are labeled as runtime-dependent instead of comparing against the initial SAV.
Earlier branch guards keep their original predicate. Unknown-effect commands
invalidate cached resource results, and later variable operands cannot reuse
unqualified values merely because the command width was decoded. Unresolved operands and changed
native dispatch targets retain stop evidence. New resource commands are not
added to the receipt-proof whitelist: a resource guard cannot certify an award
or successful receipt by itself.

The [Emerald script commands](https://github.com/pret/pokeemerald/blob/master/src/scrcmd.c)
and [FireRed script commands](https://github.com/pret/pokefirered/blob/master/src/scrcmd.c)
provide orientation only. Exact current ROM instructions and independent execution
establish the widths, bag differences and no-op behavior above.

## Query closure / 查询闭环

Source conditions are shared by rewards, shops, scripted Pokémon and teacher
offers; required items are cross-links to their runtime acquisition pages. The
UI shows names, required holdings, known current values and missing/unknown
results. Raw IDs/offsets remain in expandable evidence. Map markers and unplaced
sources show human-readable conditions without inventing SAV state. Query and
collection panels and escaped standalone HTML use the same bilingual formatter.

Resource checks that are known false can block a source. Enough holdings do not
prove payment, delivery or current access: these sources remain partial, and an
otherwise available source stays unknown until that broader access is established.
Completed receipt evidence is independent of present resources and is preserved.
The queries never edit a bag, wallet, individual, event flag, ROM or SAV.

## Independent evidence / 独立证据

`scripts/verify_resource_checks.py` runs five fingerprint-verified ROMs in Unicorn
with synthetic RAM. Bag descriptor pointers/counts, slots, encrypted wallets and
key pointers are injected; native pocket selection, quantity reading/decryption,
script-width conversion and actual comparison/conditional handlers execute without
bypass. A zero map header and unset temporary flags define the normal-bag scenario;
separate facility-flag cases establish that this is not a full context proof.
No user SAV is opened; ROM files are never written. It is not a complete map-load,
field-interaction, cost/discount, learning or save-edit emulator test.

- **690 holdings vectors:** 48 item and 90 money/no-op cases per fingerprint.
  Empty/zero/duplicate slots, 255/256/257/65535 requests, 32-bit wallets and
  requirements, encrypted key and nonzero ignore operands are covered.
- **240 native copy/compare/goto cases:** two Boolean values, four constants and
  six operators per fingerprint. Rust's symbolic guards are compared to the
  independently executed branch outcomes, including always/never branches.
- **3,227 native category lookups:** all registered item IDs, independently compared
  with the native sanitizer/table path and Rust execution. Ultimate slot 377's
  conversion is reproduced; no extracted category catalog ships with the app.
- **8 facility-flag cases:** two native contexts per alternate-bag adapter.
- Public synthetic script/query fixtures cover dispatch mismatch, unresolved
  operands, alias/no-op preservation, opaque-effect invalidation, mutations, required-item links, PC/wrong-pocket
  exclusion, zero-record readers and absence of implicit receipt/save mutations.
- Browser fixture covers prerequisite → item → focused map → back, Chinese/English
  conditions, read-only collection planning and safe, human-readable HTML export.

```sh
cargo test -p gen3-core -p gen3-cli --locked
# Set all five private GEN3_ROM_* paths; never put these inputs into Git.
uv run --with unicorn python scripts/verify_resource_checks.py > /private/resources.json
GEN3_RESOURCE_PROBES=/private/resources.json cargo test -p gen3-core local_resource_guards -- --ignored --nocapture
# With Vite running; public synthetic API data, no real ROM/SAV opened.
uv run --with playwright python scripts/test_resource_conditions_ui.py
```

Current regression: **101 public core tests pass; 24 opt-in tests are ignored by
default**. Native holdings/category/branch parity and five-fingerprint
acquisition/collection/navigation and teacher-source regressions pass. Resource,
query/collection, tutor, trade and NPC-receipt browser fixtures pass. Five-ROM
pickup and NPC receipt parity is also rechecked after conservative resource invalidation. TypeScript,
production Vite, formatting and version checks pass. All-target Clippy retains
six pre-existing warning sites; the Vite chunk-size advisory remains. No new
edited-SAV emulator round trip or public release is claimed. User-requested local
app packaging and launch evidence is recorded separately in its accompanying BUILD-INFO.txt.

Full quest dependencies, complete access, special currencies, dynamic item counts,
alternate facility bag restoration, actual consumption/payment and receipt/reset
semantics remain pending. These guards are one verified part of the requested
cross-ROM collection workflow; they do not establish full support for any game.

本轮把脚本道具／金钱检查贯通来源、所需道具跳转、地图、收集建议与独立 HTML。
水银保留 16 位数量且只检查首个匹配槽位；其他四份脚本截成 8 位并累计普通背包
槽位，但设施临时状态仍不足以从 SAV 确定，数量仅供参考，满足状态保持未知。
持有量不是费用或已支付证明；新增检查不扩大领奖证明。普通背包、原生分支和
金钱规则提供有界验证，不宣称完整剧情、可达性或所有培育机制已经完成。
