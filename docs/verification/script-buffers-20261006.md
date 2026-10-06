# Bounded native text buffers / 有界原生文字填充

This increment covers all five exact fingerprints in the
[capability matrix](../capability-matrix.md). ROM inputs stay read-only. The
verifier opens no SAV, patches no ROM and runs complete native routines on an
independent mGBA ARM7 core. Version remains unchanged.

## Behavior and scope / 行为与范围

Previously, species/item/move/number/literal text-buffer commands discarded
assigned item quantities, warp coordinates and guards as unknown side effects.
Qualified commands now preserve those values in the shared event walker used by
references, map entrances and collection planning/HTML. This change preserves
analysis state; it does not reconstruct or expand dialogue text.

The reader checks the loaded ROM's dispatcher, handler header, source stride and
string-buffer destination table against verified adapter semantics. Supported
strings must terminate within 14 bytes; destination slots are limited to the
three verified buffers. Name IDs must be known and in bounds. Numeric operands
must follow the adapter's native variable ranges. Literal strings must point into
the current ROM. Unknown operands, invalid buffers, unsupported formatting and
long/unterminated strings keep conservative unknown boundaries.

Species and move sources follow the table loaded by the native formatter, even
when the editor's ordinary name reader uses another expanded table. Mercury's
item formatter has a runtime custom-berry branch: its compared ID is read from
the ROM instruction and is excluded from preservation. A default-context native
call does not establish a bound for custom berry data. No names are embedded in
configuration or shipped as an extracted catalog.

经核对的短名称／数字填充不再丢掉奖励数量、入口格位与条件。仍不声称对话文字完整
展开，也不据此断定奖励可领取或地图当前可达。水银的自定义树果名称依赖运行时
数据，继续作为未知处理；其他长字符串、非法参数及未核对例程同样保守停止。

## Evidence / 证据

`scripts/verify_script_buffers.py` executes **4,473 complete native calls**:

| Exact adapter | Calls |
| --- | ---: |
| Dark Phantom BW | 628 |
| Dark Phantom DP | 628 |
| Spanish Rocket | 1,171 |
| Ultimate Emerald 5.5 | 1,048 |
| Mercury 1.2 | 998 |

All item IDs are tested in a default native context. Selected species/moves,
numeric boundary values through 65,535 and ROM literals cover all three slots,
literal/variable operands and table edges. Mercury's context-dependent item is
explicitly marked excluded when comparing the production classifier. Each call
checks output termination, consumed script operands and return value. Entire
EWRAM is compared allowing only output bytes and the script-PC field; IWRAM below
the CPU stack region is unchanged. All input SHA-256 hashes are rechecked.

The opt-in Rust test
`local_script_buffers_match_complete_native_state_preservation` compares the
production classifier with this independent evidence, including exact MD5,
SHA-256, handler and destination identity. Disposable ROM decoding contexts exist
only in memory and are not treated as referenced gameplay scripts.

The public guarded fixture covers five commands across five adapters, retaining
item quantity 7, assigned destination coordinates, player-gender guards and SAV
projections without changing save bytes. Negative controls cover invalid slots,
unknown/out-of-bounds IDs, invalid destination tables and unterminated strings.
This is a complete query-to-map flow within the fixture, not live-game proof of
all rewards or entrances. The public suite passes 130 tests with 49 opt-in tests
ignored by default.

## Reproduction / 复现

Supply private `GEN3_ROM_*` files and build the existing
`scripts/native/breeding_probe.c` library. Runtime profiles are obtained privately
from the loaded ROM; do not commit ROMs, profiles or output catalogs.

```sh
python3 scripts/verify_script_buffers.py --mgba-probe /path/to/probe.dylib \
  --profiles /path/to/runtime-profiles.json --output /fresh/buffers.json
GEN3_SCRIPT_BUFFER_PROBES=/fresh/buffers.json cargo test -p gen3-core \
  local_script_buffers_match_complete_native_state_preservation -- --ignored
cargo test -p gen3-core
```

Full event execution, object movement/tasks, special calls, dialogue expansion,
custom berry names and current access remain partial or unknown. No new save edit,
ROM write or release claim is introduced by this analysis increment.
