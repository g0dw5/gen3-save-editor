# Edited SAV → native save → reload / 修改存档的游戏内保存与回读

Scope: all five registered fingerprints, mGBA **0.10.5**, 128 KiB battery
files. This checks the tested individual/inventory operations, not every save
field, emulator, gameplay condition or in-game equivalent editing mechanism.
Real ROMs, SAVs, key scenarios, snapshots, screenshots and result JSON remain
private. User input files were not changed. No package or release was built by
this increment; the current development version remains unchanged.

## Method / 方法

`scripts/native/save_roundtrip_probe.c` uses the full mGBA frame loop, interrupts
and normal keys. Its exposed controls are load/reset, key/frame execution,
read-only memory observation, capture and exclusive new-file battery export.
There are no cheats, RAM writes, native-function calls/replacements or savestate
loads. Source battery data is read into memory with no writable attachment.

`scripts/verify_save_roundtrip.py` applies actions through the production CLI's
standard transaction policy into a fresh test folder, then checks:

1. Exact ROM fingerprint and unchanged SHA-256 of both input files, also on
   failure. It records the CLI/probe hashes and source revision.
2. Actual native party count and every occupied 100-byte party record against
   the edited file's active logical section, before and after reboot.
3. The entire **33,744-byte native storage block**, including unused records,
   box names and wallpapers, against logical sections 5–13 at both loads.
4. Every native bag descriptor, capacity, item ID and visible quantity, plus
   PC storage at its configured SB1 field. Encrypted quantities use the native
   SB2 key; Mercury uses its plaintext expanded bag. Empty slots have no visible
   quantity. This does not establish full PC menu or item-consumption behavior.
5. The game's Save menu and overwrite confirmation via normal keys. An exported
   native flash image must have **exactly one** additional save counter and
   identical complete individual and inventory snapshots.
6. A fresh core and normal Continue flow load that native flash image. The second
   battery export is byte-identical, with no game save performed during reload.
7. All ROM-mapped bytes except GPIO offsets `0xC4..0xC9` match the input ROM.
   mGBA represents native RTC data/direction/control in those six bytes; their
   values are recorded separately. The original ROM file hash must still match.
8. Negative controls reject a shifted party address, shifted storage-pointer
   address and deliberately incorrect quantity expectation. Exporting again to
   an existing test file must fail and leave that file's hash unchanged.

背包采用游戏初始化后的栏位指针和容量进行独立核对，而不是再次按修改器的 SAV
偏移读取。原生背包指针顺序、物品表分类和界面显示顺序不能混用。同行与盒子使用
完整原始字节核对，不只比较名字、PID 或校验和。每一步重新开机，不靠即时存档。

## Results / 结果

Each fingerprint passed a baseline and an expanded edit/save/reload scenario.
The expanded cases include simultaneous speed IV/EV changes and markings.
The first four also batch-mark two existing individuals, exchange occupied
party/box slots, and move an existing boxed individual to an empty box slot.
No individual was created, copied or deleted; no Pokédex or story flag was edited.

| Exact profile | Individuals retained | Native inventory slots | Native counter | Expanded actions |
| --- | ---: | ---: | --- | ---: |
| Dark Phantom BW / 漆黑 BW | 233 | 236 | 120 → 121 | 7 |
| Dark Phantom DP / 漆黑 DP | 2 | 236 | 120 → 121 | 7 |
| Spanish Rocket / 西班牙火箭队 | 13 | 966 | 9 → 10 | 6 |
| Ultimate Emerald 5.5 / 究极绿宝石 | 2 | 412 | 2 → 3 | 7 |
| Mercury 1.2 / 水银 1.2 | 1 | 808 | 3 → 4 | 2 |

BW and Rocket use private gameplay inputs. Mercury uses the opening Totodile
input with six Potions; its disposable edited copy has seven. DP and Ultimate
use earlier developer fixtures containing existing party/box records, not a
claim about a player's current progress. Ultimate tests PC Potion quantity;
the other quantity changes use existing ordinary items. All **2,658** inventory
slots are observed at both loads, not only edited entries.

最终扩展用例在五个指纹均通过：15 个错误期待被拒绝，5 次覆盖已有导出文件被拒绝，
原文件哈希不变。场景中 IV／EV 的修改用于验证数据持久化，不据此宣称保持原个体
历史的 IV 改动等同于这些游戏中的某项正规操作。

The Mercury fixture has no occupied box slot. It proves current party fields,
expanded-bag quantities and preservation of the entire empty native storage
block, **not** this increment's occupied-box move or exchange. Earlier bounded
Mercury codec/storage evidence remains in the
[correction record](mercury-display-storage-20261005.md).

## Limits / 边界

- Not proof of all edits, in-game legitimacy, NPC delivery, daycare, mail,
  battle/facility transient data, corruption recovery or hardware compatibility.
- The probe deliberately rejects files with RTC trailers; 128 KiB + trailer
  round trips are not established here. Current fixture files are exactly 128 KiB.
- Frame/key sequences depend on the fixture's title/menu/story state. Rocket
  reaches Continue earlier than BW; Mercury uses a carousel and opening dialogue.
  A screenshot of an overworld alone does not prove the intended save loaded.
- No new frontend drag gesture, backup/undo or source-conflict test is implied.
  Native scenarios use the same production transaction actions; existing
  [transaction/UI evidence](../research/save-integrity.md) remains separate.
- Dynamic story access, full collection planning, maps and trainer generation
  are not upgraded by this save validation. Their matrix rows remain partial.

## Reproduction / 复现

Compile against the same mGBA source/build headers and dynamic library. Including
generated `mgba/flags.h` is required: mismatched feature flags change `mCore`
callback offsets and invalidate the harness. Use Python with the dylib's CPU
architecture; this run used arm64 `/usr/bin/python3`.

```sh
cc -dynamiclib -O2 -Wall -Wextra -Werror \
  -I MGBA_SOURCE/include -I MGBA_BUILD/include \
  scripts/native/save_roundtrip_probe.c \
  -L MGBA_BUILD -lmgba -Wl,-rpath,MGBA_BUILD -o PRIVATE/probe.dylib
python scripts/verify_save_roundtrip.py \
  --probe PRIVATE/probe.dylib --cli TARGET/debug/gen3 \
  --scenario PRIVATE/scenario.json --output PRIVATE/new-run-directory
```

The scenario is keyed by `BW`, `DP`, `ROCKET`, `ULTIMATE`, `MERCURY12`; each entry
supplies `rom`, `save`, production `actions`, `boot`, `save_steps`, `reload` and
native `ram` addresses (`party`, `count`, `storage_pointer`). Step rows contain
`frames`, optional GBA `keys` and optional unique `capture` names. The output
directory must be new. Fixtures and scenario paths are not public release inputs.


## Compact Mercury storage follow-up

The earlier empty-box Mercury fixture checks the contiguous storage block but
does not certify individual PC records. That limitation is superseded by the
[25-box native-converter and populated-SAV increment](mercury-cheats-storage-20261006.md).
The verifier now also checks every native compact-box pointer against all four
SAV backing regions. Earlier 80-byte created-box round-trip claims are withdrawn.
The first-four record codecs remain unchanged.
