# Fixed encounter members / 定点相遇成员

This increment follows the exact native `0xB6` setup command across five ROM
fingerprints. Source parameters and generated opponent fields are verified in
bounded isolated scenarios; capture, full battle startup, access and receipt
remain unverified. ROM-only queries never generate Pokémon in a SAV.

## Native differences / 原生差异

| ROM / MD5 | Handler offset | Instruction bytes | Species inputs | Opponent slots |
|---|---:|---:|---|---|
| Dark Phantom BW / `0d9b129f7dd76895f79bb47ad7dec2fe` | `0x9B674` | 6 | literal | 0 |
| Dark Phantom DP / `cb2940215f4dafb1bef133c3af379f44` | `0x9B674` | 6 | literal | 0 |
| Spanish Rocket / `59c658a1081f542086de1060bb65f0b3` | `0xD15D4` | 11 | literal | 0; second nonzero species uses slot 3 |
| Ultimate / `17ce9785b33319b3dbda9a5d37c57ec1` | `0x9B674` | 6 | literal | 0 |
| Mercury 1.2 / `f323df1792ac68462a34b42fe8571533` | `0x6C368` → current ROM hook | 6; 18 for raw sentinel `0xFFFF` | native VarGet | 0; sentinel pair uses slots 0 and 1 |

All lengths include the opcode. Level is a literal byte and held item a literal
halfword. Rocket always consumes its second operand group, even if its second
species is zero. The second group's nonzero species selects its two-opponent
constructor; it has its own level and held item. Mercury checks the raw sentinel
before resolving species; its paired operands start at byte offsets 7 and 13.
Ignored sentinel-prefix/separator bytes are independently varied in the verifier.

The old shared Rocket reader consumed 11 bytes but emitted only the first
member. It now reads both and keeps duplicate species as separate members.
Mercury's existing pair reader gains the same typed group context. Unknown
species variables remain unknown in the group's second member rather than being
invented or discarded. Current native command-table bindings are checked before
reading either the instruction width or source fields; a mismatch stays partial.

西班牙火箭队的原生指令支持同场两只，旧解析仅保留第一只；本次补齐第二只的
种类、等级和携带物。两只同种宝可梦仍分别表示，不按名称合并。水银的双对手
指令共用组信息，但保留其独立的哨兵、变量解析与长度规则。

## Query, map and planning / 查询、地图、规划

`PokemonSource.battle_members` keeps all members of a pair, with unknown values
explicitly nullable. Species acquisitions link companions and held items to
current-ROM reference queries. The shared map popup displays the group once per
instruction, with species/item navigation. Collection facts and standalone HTML
include both members and the same capture/access warning; HTML escapes all ROM
names. Levels, encounter-slot probabilities and held-item probabilities remain
separate. A pair is not two random encounter slots or a claimed chance.

A setup command is not a guarantee that both opponents can be captured, a
successful gift, a completed event or proof of current accessibility. These
sources retain `partial`, unknown receipt/repeatability and no fabricated
encounter probability. No party generation, Dex completion or story write is
triggered by a query or collection report.

**Actual reference survey:** the current bounded map-script walker reports **zero
paired member rows in each of the five exact inputs**. This verifies that the
new decoder does not invent double encounters from adjacent script bytes; it
does not establish absence. Unknown native calls, unparsed scripts and other
battle setup paths remain gaps. The existence of a native two-opponent routine
must not be advertised as a currently located in-game encounter.

当前五份 ROM 的有界地图引用扫描均未解析出双对手记录；指令支持不等于已找到
游戏内相遇，也不能据此断言本作不存在。地图脚本、原生调用及特殊战斗仍有缺口。

## Evidence / 证据

- `scripts/verify_static_battles.py`: **486 complete native setup calls** on mGBA
  ARM7: BW 54, DP 54, Rocket 108, Ultimate 54, Mercury 216. The handler and
  all called constructors/getters run unchanged, without injected fields or
  native-function replacements. Literal/variable Mercury inputs, single/pair
  branches, identical species, two seeds, levels 1/21/99 (second 2/22/100), and
  held items 0/13/255 (second 1/14/256) are covered.
- Native script pointer consumption, every occupied opponent slot, species,
  level and held item match independently specified scenarios. Entire player
  party bytes remain unchanged; getters preserve all 600 opponent bytes. Input
  ROM hashes are checked in `finally`. No SAV is opened. This is isolated setup,
  not full-frame battle startup, capture or arbitrary live battle context.
- Higher-level native generation exceeded the old probe's one-million step
  allowance; an explicit 20-million instruction budget completes all cases.
  `GEN3_NATIVE_MAX_STEPS` changes only the fixture execution limit; the shared
  probe retains its original one-million default. No ROM byte is patched.
- `local_static_battle_sources_match_complete_native_setup` compares the
  production reader with all 486 native vectors and surveys actual referenced
  sources. Public five-adapter scenarios verify both source queries, same tile,
  companion cross-links, unknown input, duplicate members, dispatch rejection
  and read-only ROM data.
- `scripts/test_static_battle_ui.py` covers source → companion → map target →
  return at a compact viewport, Chinese/English labels, one group on the map,
  held-item display and read-only API calls. The shared collection browser
  fixture includes a paired task and checks escaped standalone HTML in both
  languages. Synthetic UI fixtures do not prove real-game occurrence.

Core regression: **136 public tests passed**, **53 opt-in tests ignored** by
default; the five-fingerprint acquisition/collection regression and desktop
compilation passed. Formatting, production UI and release-workflow checks are
recorded alongside the browser/native evidence.

Overall static acquisition, map access, story and collection coverage remains
**P**, not fully verified. The version stays held; no package or release is
created by this increment.

## Reproduce / 复现

Provide five private `GEN3_ROM_*` files and compile the existing read-only native
probe with `-DGEN3_NATIVE_MAX_STEPS=20000000`. Supply mGBA headers/library as in
[breeding verification](breeding-20261006.md).

```sh
python3 scripts/verify_static_battles.py --mgba-probe /private/native.dylib \
  --output /private/static-battles.json
GEN3_STATIC_BATTLE_PROBES=/private/static-battles.json cargo test -p gen3-core \
  local_static_battle_sources_match_complete_native_setup -- --ignored --nocapture
```

Run Vite and the two browser scripts with Playwright/Chrome. Vectors, screenshots,
ROMs and saves stay local; they are not runtime catalogs or release inputs.
