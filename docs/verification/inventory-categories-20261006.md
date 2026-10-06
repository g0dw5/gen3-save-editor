# Native inventory categories / 原生背包分类核对

Five exact fingerprints from the [capability matrix](../capability-matrix.md).
ROM/SAV files and native/UI result fixtures stay private. The current version
is unchanged; this increment does not package, publish or edit original inputs.

## Bug and resulting behavior / 问题与修正

Dark Phantom BW/DP's existing pocket offsets and capacities were correct, but
their configured ROM category numbers for berries and key items were reversed.
The native Emerald item categories are **berries 4, key items 5**. The former
configuration used 5 and 4 respectively. Consequently:

- Reading existing slots still returned their stored IDs/quantities correctly.
- Standard `edit_bag` rejected a correctly placed berry or key item, and could
  accept the opposite category in that pocket.
- `EventSnapshot::normal_bag_item` searched the other pocket for these categories,
  affecting ordinary holdings and resource-condition projections.
- ROM-reference category labels derived from the adapter were reversed.

Correct the shared BW/DP category metadata without changing field offsets,
quantities or storage. The other adapters retain their independently verified
definitions. The UI uses the currently loaded ROM item category and adapter
pocket metadata to filter normal item choices. PC storage and free editing keep
the complete catalog. A current mismatched ID remains visible and gets a
bilingual explanation; it is never silently moved, deleted or normalized.

| Profile / 版本 | Berries / 树果 | Key items / 重要道具 |
| --- | ---: | ---: |
| Dark Phantom BW / 漆黑 BW | 4 | 5 |
| Dark Phantom DP / 漆黑 DP | 4 | 5 |
| Spanish Rocket / 西班牙火箭队 | 5 | 8 |
| Ultimate Emerald 5.5 / 究极绿宝石 | 4 | 5 |
| Mercury 1.2 / 水银 1.2 | 5 | 2 |

此前保存闭环主要修改普通道具数量，并不能证明重要道具／树果的分类关联正确。
本次在五指纹中补充分类、错误栏位、持有量与保存验证。已有错放的道具保留原槽位
并提示；修复配置不会自动修正过往存档中的错放数据。

## Independent evidence / 独立证据

`scripts/verify_inventory_categories.py` boots the unmodified ROM and source
battery file through the full mGBA frame loop. It observes the initialized bag
pointer/capacity pairs and security key, and copies EWRAM/IWRAM into a separate
mGBA ARM7 function probe. Category representatives come from the ROM's own
getter, not a shipped item list. Disposable RAM tests cover **28 native bag
pockets × 36 quantity/request cases = 1,008 vectors**:

- Empty, zero, one, seven, duplicate matching slots and 16-bit boundary quantities.
- Requested counts 0, 1, 7, 8, 256 and 65535 are supplied through native variable
  `0x8005`, then the complete script holdings-check function executes normally.
- BW/DP/Rocket/Ultimate script requests use the verified u8 reduction and
  accumulation; Mercury keeps u16 requests and its first-match behavior.
- No native function is replaced or bypassed. RAM fixtures may contain abnormal
  quantities for arithmetic boundaries; they are not legitimate gameplay claims.
- Source ROM/SAV hashes remain unchanged, including failure-path checks.

The opt-in Rust parity test compares every native pocket's identity/category,
capacity and native holdings vectors with the adapter and production save
reader. The public test additionally proves standard wrong-pocket rejection is
atomic and that PC/wrong-pocket copies cannot satisfy ordinary bag checks.

`scripts/test_inventory_categories_ui.py` reads real current catalogs and saves
through the production CLI, then intercepts browser APIs as a read-only fixture.
All five cases verify berry/key/PC choices, free editing, retained mismatched
IDs, unchanged initial form state, English/Chinese warnings and no mutation or
export requests. The mismatch is an HTTP fixture only; no user file is changed.

All five profiles also pass a fresh production edit → normal-key game Save →
reboot run, extending the
[full-frame verifier](save-roundtrip-20261006.md) with standard berry/key pocket
actions. Native party/storage bytes and **2,658** inventory slots are checked at
both loads; native counters increase exactly once and snapshots preserve the
requested values. Any introduced item in a disposable fixture is a structural
test, not proof of story eligibility or a user's real acquisition history.

## Results and limits / 结果与边界

- **128 public core tests pass; 47 opt-in tests ignored by default.**
- Five-profile native pocket/holdings parity passes; existing native resource
  vectors and actual-ROM acquisition/collection regression pass separately.
- Five-profile bilingual inventory browser fixtures pass.
- TypeScript/Vite, desktop compilation, formatting and five release checks pass.

This does not establish alternate facility-bag state from a SAV, full native PC
menu/deposit/consumption rules, custom inventory mechanisms or legitimacy of all
free edits. Resource checks do not prove payment or receipt. Acquisition, maps
and full collection/story coverage remain partial in the matrix.

## Reproduction / 复现

Use the frame probe from [save-roundtrip](save-roundtrip-20261006.md) and the
mGBA ARM7 function probe from `scripts/native/breeding_probe.c`, compiled with
matching mGBA headers/library. Python must match the dylib architecture.

```sh
python scripts/verify_inventory_categories.py \
  --frame-probe PRIVATE/frame.dylib --native-probe PRIVATE/native.dylib \
  --scenario PRIVATE/normal-boot-scenarios.json --output PRIVATE/new-vectors.json
GEN3_INVENTORY_CATEGORY_PROBES=PRIVATE/new-vectors.json \
  cargo test -p gen3-core local_inventory_categories_match_booted_native_pockets_and_checks \
  -- --ignored --nocapture
GEN3_INVENTORY_UI_SCENARIO=PRIVATE/normal-boot-scenarios.json GEN3_CLI=TARGET/debug/gen3 \
  python scripts/test_inventory_categories_ui.py
```

Set all five `GEN3_ROM_*` paths for Rust parity. Scenario format follows the
full-frame verifier; boot steps must have no captures here. UI tests require
Vite, Playwright and Chrome, but no live development bridge. Private scenarios,
JSON, screenshots and actual game files do not enter Git or release inputs.
