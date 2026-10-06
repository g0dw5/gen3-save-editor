# Native Dex read checks / 原生图鉴读取校验

This increment corrects read-only historical collection judgments for the four
legacy exact fingerprints. Mercury 1.2's separate split-bank reader is retained
and cross-checked against its existing native evidence. It does not automatically
repair a save, mark entries or generate Pokémon.

## Exact native behavior / 精确原生行为

| ROM / MD5 | Native getter (ROM offset) | Effective seen | Effective caught |
| --- | --- | --- | --- |
| Dark Phantom BW `0d9b129f7dd76895f79bb47ad7dec2fe` | `0xC0664` | SaveBlock2 `0x5C` bit and SaveBlock1 `0x988`, `0x3B24` mirror bits must all be set | SaveBlock2 `0x28` bit and valid seen record |
| Dark Phantom DP `cb2940215f4dafb1bef133c3af379f44` | `0xC0664` | Same exact native checks, verified independently | Same exact native checks, verified independently |
| Spanish Rocket `59c658a1081f542086de1060bb65f0b3` | `0xF7C60` | SaveBlock1 `0x2EE4` bit | SaveBlock1 `0x2F5C` bit, independent of seen |
| Ultimate Emerald 5.5 `17ce9785b33319b3dbda9a5d37c57ec1` | `0xC0664` → `0x1257950` | SaveBlock1 `0x560` bit | SaveBlock1 `0x5D8` bit and seen bit |
| Mercury 1.2 `f323df1792ac68462a34b42fe8571533` | `0x88E74` → `0x1D6680C` | [Native split banks and initialization](mercury-dex-20261006.md) | Independently read from initialized native bank |

BW/DP and Rocket use `(number - 1)` as their bit index; Ultimate uses `number`.
Ultimate's native bit-mask table is read and verified by the native verifier,
including all eight mask values. The production reader uses the fingerprint's
verified bit layout and dependency rule. ROM names/content and species-to-Dex
relationships are still read from the loaded ROM.

The BW/DP getter clears inconsistent positive records in native RAM: GET_SEEN
clears seen and both mirrors on a mismatch; GET_CAUGHT clears caught, seen and
both mirrors when caught is set but the seen checks fail. If primary seen and
caught are both zero, mirror-only bits are not consulted as positive records.
Rocket and Ultimate getters leave these blocks unchanged. The editor projects
these Boolean results without performing the native invalidation writes.

配置的 416（BW／DP）、955（西班牙火箭队）、905（究极绿宝石）是本次核对的
图鉴标记索引范围，不代表可获得宝可梦数量。保留现有分配范围和显式编辑边界，
没有根据整字节容量制造新宝可梦。实际物种映射由 ROM 表决定；本地当前表的正向
最大编号分别为 411／411／950／905，存在共享编号或空映射。原生计数循环也不应
被当作“全部可获得物种”证明；此增量不缩减或扩张那些映射。

## User workflow / 使用流程

Open ROM and SAV, then choose historical Dex ownership in Collection planning.
Records that fail the current ROM's native read checks cannot remove a missing
goal merely because their raw caught bit is set. The planner, independent HTML
and legacy Dex editor explain the affected flag count and show up to 20 flag
numbers. Loading a different save replaces the notice. Healthy historical
ownership still does not establish possession of all separately stored forms or
usable breeding parents.

漆黑 BW／DP 的已见记录要与两个镜像一致，已捕获还需有效的已见记录；究极绿宝石
已捕获同样依赖已见，西班牙火箭队则独立读取。界面与 HTML 会提示未通过原生校验
的记录，不默默按原始捕获位算作完成，也不会自动修复或改亮图鉴。既有显式图鉴
编辑仍会同步相关标记；水银图鉴写入仍关闭。默认补齐方式仍为正常捕捉、孵蛋和
进化。完整来源、剧情依赖、时段和当前可达性仍是部分支持。

## Evidence / 验证

- `scripts/verify_dex_flags.py` executes **83,008 complete, unmodified native
  getters** through mGBA ARM7: BW 26,624; DP 26,624; Rocket 15,280; Ultimate
  14,480. Every configured flag number is tested with every independent
  caught/seen/mirror combination and both getter orders. Adjacent bits have
  alternating nonzero backgrounds. Whole SaveBlock1/2 comparisons permit only
  the exact BW/DP invalidation writes; IWRAM below the disposable call stack is
  unchanged. No native function is replaced, no SAV is opened and ROM SHA-256
  guards are checked in `finally`.
- `local_legacy_dex_matches_complete_native_getters` compares all **41,504
  paired native results** to production readers, checks MD5/SHA-256 and native
  layout/dependency fields, and preserves every synthetic save byte during reads.
- Public tests exercise first/last flags and byte boundaries across all legacy
  adapters, raw positive mismatches, native projections, rotated save sectors,
  explicit synchronized edits and reopening the resulting save. A planning test
  keeps a bad raw caught record in the missing goals and passes its notice along.
- Mercury's production reader is rerun against its separate **16,432 synthetic
  getter and 2,054 normally booted-save getter** evidence. Its read-only bank
  initialization and Standard/Free edit rejection are unchanged.
- Shared query/planning browser fixtures verify English/Chinese native-record
  notices, fresh-SAV notice replacement and standalone HTML. A separate real-App
  Dex fixture checks projected toggles, a bounded number list and notice removal
  after reloading a different save; it sends no save action.
- Public core suite: **134 passed**, **51 opt-in tests ignored** by default.
  Five-fingerprint actual-ROM query/collection, formatting, TypeScript/Vite,
  desktop compilation and release-workflow checks accompany this increment.

## Reproduce / 复现

Keep private ROMs out of source/release inputs. Supply `GEN3_ROM_BW`,
`GEN3_ROM_DP`, `GEN3_ROM_ROCKET`, `GEN3_ROM_ULTIMATE`; use the existing
`scripts/native/breeding_probe.c` ARM7 probe and its mGBA build instructions.

```sh
python3 scripts/verify_dex_flags.py --mgba-probe /path/native.dylib \
  --output /fresh/native-dex.json
GEN3_DEX_FLAG_PROBES=/fresh/native-dex.json cargo test -p gen3-core \
  local_legacy_dex_matches_complete_native_getters -- --ignored
```

For Mercury, follow its [separate reproducible verifier](mercury-dex-20261006.md).
Run Vite, then the synthetic `scripts/test_dex_read_ui.py` and
`scripts/test_query_collection_ui.py` browser tests with Playwright/Chrome.
The evidence verifies flag reads/projections, not complete collection access,
all in-game Dex screens, or an automatic save repair feature.
