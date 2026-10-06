# Mercury native Dex / 水银原生图鉴

Exact ROM: Mercury FC 1.2, MD5 `f323df1792ac68462a34b42fe8571533`.
This increment enables read-only historical-Dex collection planning. It does not
mark the Dex, generate Pokémon, change story flags or authorize Dex edits.

## Native structure / 原生结构

The loaded ROM's getter entry `0x88E74` branches through `0x1CCE5F4` to the
complete native routine at `0x1D6680C`. Upstream
[CFRU symbols](https://github.com/Skeli789/Complete-Fire-Red-Upgrade/blob/master/BPRE.ld)
help locate that original entry; upstream structures are not used as layout proof.
The exact ROM's executable and native comparisons establish these SaveBlock1
fields, assembled from the active save's logical sections:

| Numbers | Seen | Caught | Initialization |
| --- | --- | --- | --- |
| 1–905 | `0x310` | `0x38D` | none |
| 906–1027 | `0x40C` | `0x41C` | word `0x40A` must equal `0x11DE` |

Each bank indexes its own first number as bit zero. The native range limits and
literals are inspected by the verifier. Invalid numbers 0, 1028 and 65535 return
false. The native count loop also confirms the exclusive upper bound 1028.
These are native flag numbers, not a claim of 1,027 distinct obtainable species.

If the extended marker differs, the native getter writes the marker and clears
`0x40C..0x42C` before reading. The editor projects that result as unseen/uncaught
and explicitly reports the uninitialized range. It preserves the marker and raw
flag bytes in the SAV. Nonzero stale bytes cannot masquerade as captured entries.

The reusable read-only bank model is separate from editable legacy layouts.
Mercury still has no editable Dex layout/capability. Existing BW/DP, Rocket and
Ultimate readers and edit rules remain unchanged. Names, forms and
species-to-national-number relationships continue to come from the loaded ROM.
Shared Dex numbers establish historical species ownership, not possession of
all separately stored forms or presently usable breeding parents.

## User workflow / 使用流程

Open ROM and SAV, open Collection planning, and choose Pokédex ownership or
current individuals. Historical records and healthy non-egg stored individuals
are separate inputs. The read-only explanation and uninitialized-range notice
appear in Chinese/English and the standalone HTML. Source/map/prerequisite links
use the existing shared planning flow. Acquisition and current access remain
partial and route ordering remains a suggestion.

打开 ROM 和 SAV 后，在收集规划选择图鉴捕获记录或现有个体。水银的两段图鉴
现已按原生例程核对；扩展段未初始化时会解释实际读取结果，不写入存档。历史
图鉴不能证明当前持有每种形态，也不能当作可用的孵蛋亲本。默认仍通过正常捕捉、
孵蛋和进化补齐。资料关联、区域建议与 HTML 延用统一流程，不宣称完整任务依赖
或当前可达性；图鉴编辑仍关闭。

## Evidence / 验证

- `scripts/verify_mercury_dex.py`: **16,432 complete synthetic getters**, covering
  all 1,027 numbers, independent seen/caught bit patterns 00/FF/A5/5A, initialized
  and uninitialized extended state. Whole SaveBlock1 comparisons allow only the
  exact native lazy-initialization writes; other queries leave it unchanged.
  Invalid-number controls also leave the block unchanged. No native routine is
  hooked or replaced and no ROM write interface exists.
- The verifier loads the actual battery with normal Continue keys on a full-frame
  mGBA core, then clones its native RAM to an independent ARM7 core for **2,054
  complete getters**. The current fixture's extended bank is uninitialized.
  A disposable 128 KiB flash copy omits the 16-byte emulator RTC trailer for the
  loader; no effective-clock claim is made. Source ROM/SAV SHA-256 hashes are
  checked in `finally`, and original files are never modified.
- `local_mercury_dex_matches_native_split_banks_and_booted_save` compares every
  native result against production reads and the actual SAV, including fingerprint
  and SHA-256. Public tests contrast historical ownership with current individuals,
  verify initialization projection and Standard/Free write rejection, and preserve
  legacy reads/edits across all five adapters. Public suite: **132 passed**, **50
  opt-in tests ignored** by default.
- `scripts/test_query_collection_ui.py` covers the read-only Dex option despite a
  disabled write capability, bilingual initialization explanations, escaped HTML,
  switching back to individual planning and existing source/item/map/back flows.
- The five-fingerprint actual-ROM acquisition/collection regression includes
  Mercury historical planning with this current SAV. TypeScript/Vite, formatting,
  desktop compilation and release-workflow checks are recorded independently.

## Reproduce / 复现

Provide private `GEN3_ROM_MERCURY12` and `GEN3_SAVE_MERCURY12`. Build the existing
native ARM7/frame probes; supply a normal-key Continue sequence for that save.

```sh
python3 scripts/verify_mercury_dex.py --mgba-probe /path/arm7.dylib \
  --frame-probe /path/frames.dylib --boot /private/continue.json \
  --output /fresh/native-dex.json
GEN3_MERCURY_DEX_PROBES=/fresh/native-dex.json cargo test -p gen3-core \
  local_mercury_dex_matches_native_split_banks_and_booted_save -- --ignored
cargo test -p gen3-core
python3 scripts/test_query_collection_ui.py
```

Private ROM/SAV files, native result arrays and extracted names are excluded from
Git and release inputs. This increment holds the current version and does not
create a new package or public release.
