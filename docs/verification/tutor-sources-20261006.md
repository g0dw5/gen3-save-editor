# Tutor sources and native move lookup / 招式教学来源与原生取招

Increment after `3748fa4`, version unchanged. ROMs and SAVs stay read-only.
This adds move → teacher offer → NPC tile → static exterior entrance → back, and
corrects a previously misread Dark Phantom tutor table. It is not a complete
teaching/payment/receipt or breeding-mechanism implementation.

## Native semantics / 原生语义

| Exact fingerprint | Special table / teaching special | Native menu entry | Parameter | Native move lookup | Parsed map-script references |
|---|---|---|---|---|---:|
| BW | `0x1DBA64` / `0x1DD` | `0x1B892C` | `0x8005`, indexed | `0x1B2360`, bounded 32 | 10 |
| DP | `0x1DBA64` / `0x1DD` | `0x1B892C` | `0x8005`, indexed | `0x1B2360`, bounded 32 | 10 |
| Rocket | `0x22B620` / `0x1DD` | `0x2070D8` | `0x8005`, direct move ID | Native selected-mon path reads u16 directly | 10 |
| Ultimate | `0x1DBA64` / `0x1DD` | `0x1B892C` | `0x8005`, indexed | `0x1B2360`, bounded 127 | 13 |
| Mercury 1.2 | `0x15FD60` / `0x18D` | `0x12781C` | `0x8005`, indexed | `0x120BA8`, bounded 154 | 170 |

The shared `script_teaching.rs` reader requires a resolved script parameter and
verifies the native dispatch target. Indexed selectors are converted to u8 as in
the native selected-mon path, bounded, and passed into the current ROM's getter
in isolated GBA RAM. This executes redirects and Mercury special-case code;
it does not ship extracted move catalogs. Invalid/unresolved/native calls retain
script stop evidence. The sandbox rejects writes to ROM and unmapped memory and
bounds instruction execution. Addresses/counts/input semantics are adapter
configuration, distinct from names, artwork and move values.

Dark Phantom's native getter reads the current table at `0x1CA00C0`, not the legacy
`0x61500C`. **Six of its 32 tutor slots differ**, independently in BW and DP. The
ordinary learnset reader now uses the native table. Compatibility bitsets are
unchanged; TM/egg/level readers are not substituted. Ultimate ordinary tutor
entries and Mercury's first 145 entries match their existing runtime tables.

Mercury lookup indices 145–153 execute dedicated native cases. These offers are
now resolvable; their individual eligibility and special payment/selection rules
are **not** inferred from the ordinary 145-entry compatibility bitset. At menu
entry, the full u16 selector changes callback/message above 153; it is not a
rejection of the tutor action. Both paths use action 12; the later move lookup
uses a byte selector. No unsupported restriction is derived from this branch.

Rocket's selected-mon path reads `0x8005` as the actual move ID, passes it to the
native compatibility routine (`0x9B80C`) and knows-move routine, and stores it as
the proposed learned move. Reusing an Emerald tutor-index table would be wrong.

The [Emerald tutor scripts](https://github.com/pret/pokeemerald/blob/master/data/scripts/move_tutors.inc)
and [FireRed party menu](https://github.com/pret/pokefirered/blob/master/src/party_menu.c)
provide naming/orientation only; exact native ROM instructions/execution establish
the adapter semantics. No original-game tutor locations are copied into the editor.

## Shared query/UI workflow / 共用查询与界面

- Runtime map-script teaching references retain NPC/map coordinates and branch
  conditions; map-level references without a verified tile remain unplaced.
- Move queries show physical teacher references before broad compatible-species
  rows, so the default compact result page exposes map links. Teaching does not
  masquerade as an item reward, Pokémon gift or a receipt flag.
- Map NPC details link to the current ROM move query. Map search includes offered
  move names. Exterior entry chains and back navigation use the shared map/history
  flow; static connections do not prove current reachability.
- SAV conditions may show a known missing prerequisite. They never turn a teaching
  offer into “completed” based on an NPC flag, move ownership or bag contents.
  Otherwise the status stays unknown; repeatability is unknown. Eligibility,
  payment and one-time limits are explicitly separate in both languages.
- Existing collection planning remains focused on Pokémon/items. A user can follow
  evolution move requirements into this query, but there is no new move-goal or
  fully solved breeding/teaching dependency DAG in this increment.

## Independent evidence / 独立证据

`scripts/verify_tutor_sources.py` uses five fingerprint-verified private ROMs,
Unicorn and synthetic RAM. **345 indexed native getter vectors** are compared with
Rust's independent ARMv4T execution and ordinary runtime tables. **19 menu cases**
intercept only the UI initialization call, checking tutor action/message and full
selector branch. Rocket also runs **61 native compatibility cases** for a synthetic
species-1 individual, with its current ROM list and native knows-move routine.
No menu renderer, teaching/save mutation or payment operation is asserted from
these injected-UI cases. ROM RAM bytes and all input file hashes remain unchanged.

Public fixtures cover all five adapters: script guards, native lookup, dispatch
mismatch, index bounds/byte conversion, positioned/unplaced sources, unknown
receipt/repeatability and unchanged ROM/SAV query data. The opt-in cross-ROM test
validates the 345 getters, corrects the six BW/DP mismatches, and checks each of
213 parsed references against its acquisition query. Counts are script references,
not unique or currently reachable teachers; retained maps and unresolved native
menus can affect coverage.

```sh
cargo test -p gen3-core -p gen3-cli --locked
# Set all five GEN3_ROM_* variables to private exact files.
uv run --with unicorn python scripts/verify_tutor_sources.py > /private/tutors.json
GEN3_TUTOR_PROBES=/private/tutors.json cargo test -p gen3-core local_teaching_sources -- --ignored --nocapture
# With Vite running; synthetic read-only API data, not user files.
uv run --with playwright python scripts/test_tutor_sources_ui.py
```

Current regression run: **98 public core tests passed, 23 opt-in tests ignored**;
the five-ROM teaching parity and acquisition/collection/navigation checks pass.
Teaching, trade, NPC-receipt and query/planning browser fixtures pass without
mutation calls. The learning-row jump now records reference history, so returning
from a teacher/entrance can reach the original Pokémon page. TypeScript,
production Vite and formatting/version checks pass. Clippy retains the six known
baseline warning sites; Vite retains its large-chunk advisory. No strict-clean
lint or comprehensive feature-completion claim is made.

## Remaining work / 剩余缺口

Native menu lists with unresolved dynamic parameters, complete current access,
payment/discount/currency consumption, successful learning, receipt/reset flags,
selected-individual eligibility and Mercury special teaching compatibility are
still pending. Ordinary learnset lists are bounded parsed coverage, not proof of
all obtainable moves. This increment does not establish all training mechanisms,
all story conditions or any new edited-SAV emulator save/re-read evidence.
No new app/installer or public release is produced during this continuation.

本轮修复漆黑 BW／DP 的旧教学表读取，并将已解析教招报价贯通招式、NPC 格位、
外部入口和返回。水银特殊取招通过原生代码执行，但资格、费用和次数规则未自动
推定。345 个原生取招向量、19 个选队菜单情景与西班牙火箭队 61 个兼容情景提供
有界证据；213 条引用不代表 213 位可达老师。保持当前版本号，不重复打包。
