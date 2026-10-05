# Mercury 1.2 map, location and inventory correction
# 水银 1.2 地图、地点与背包修正

Scope: exact MD5 `f323df1792ac68462a34b42fe8571533`; version remains unchanged.
All ROM/SAV files, RAM captures, screenshots and generated probe JSON stay local.
No original user file was edited. No release package was built.

## Findings and changes / 原因与改动

### Expanded inventory / 扩展背包

The former adapter incorrectly read vanilla FireRed SB1 pockets. An empty opening
fixture and a round-trip into those unused fields did not prove the actual game's
bag. Claims relying on that test have been withdrawn from the earlier research.

`SetMemoryForBagStorage` at `0x1D40C70` copies five pointer/count pairs from the
loaded ROM at `0x1DDDB84`. Their RAM addresses are relative to parasite RAM
`0x0203B174`. The native getter/setter `0x99DD8` / `0x99DDC` use plain u16 counts,
including with a nonzero vanilla FireRed security key.

| Pocket | RAM address | Parasite offset | Slots |
| --- | --- | --- | --- |
| General items | `0x0203BB20` | `0x9AC` | 450 |
| Key items | `0x0203C228` | `0x10B4` | 75 |
| Balls | `0x0203C354` | `0x11E0` | 50 |
| Machines | `0x0203C41C` | `0x12A8` | 128 |
| Berries | `0x0203C61C` | `0x14A8` | 75 |

The native save-tail hook `0x1D5AE08` places parasite bytes in logical section
0 at `0xF24..0xFF0`, section 4 at `0xD98..0xFF0`, and section 13 at
`0x450..0xFF0`: total `0xEC4`. Native load/save also uses physical flash sectors
30 and 31, `0xFF0` payload bytes each. The general-item pocket crosses the tail /
sector-30 boundary. PC items remain in SB1 at `0x298`, 30 slots; total inventory
entries are **808**, including **778** bag entries.

The shared save reader now configures extension storage independently of the
main-sector checksum algorithm. Ultimate retains its own tails-only storage;
BW, DP and Rocket retain their previous logical-block storage. Writes preserve
all unused bytes and auxiliary-sector footers. Mercury's auxiliary sectors are
shared by both main banks: a valid alternate main bank is not an independent
backup of this extension block. Whole-file export backups and undo retain it.

当前真实存档的普通道具栏只有 **6 个伤药**。此前显示大师球、万能粉来自错误
地址，并非这些物品真的在背包中。列表与详情同时修正“零被当成默认一”的 UI
行为：非空物品数量为零时显示零，不在选中详情时悄悄规范化或产生修改。

### Native names / 原生地点名

The former `0x45F89C` table is a legacy town-map table referenced at `0x12DB00`,
not the table used by `GetMapName`. The actual routine `0xC4D78` reads
`0xC2B000`, biased by section ID 88, and bounds IDs through 252. Invalid pointers
remain absent. It also has a special ship-name branch for section 94, dependent
on context; static labels do not simulate that override.

There are **133** independently executed static-name vectors, **31** invalid
pointer slots, and one excluded context-dependent section. The current Totodile
stores met-location **143**, independently checked with native `GetMonData(35)`.
It resolves to **若叶镇**, not 宝可梦联盟. Both map labels and met-location options
use the corrected runtime table; no extracted name catalog was added.

### Map rendering / 地图绘制

The native loaders confirm **640 primary tiles/metatiles**, 384 secondary tiles,
and 7+6 palette banks. Reducing the partition to stock Emerald's 512 would be
incorrect. `DrawMetatile 0x5A9B4` jumps to the hook at `0x1C8BDA0`. Attributes
are u32 at `Tileset+20`; bits 28..30 select these BG3/BG2/BG1 assignments:

| Type | Bottom BG3 | Middle BG2 | Top BG1 |
| --- | --- | --- | --- |
| 0 / 1 | Fixed entry `0x3014` | Entries 0–3 | Entries 4–7 |
| 2 | Entries 0–3 | Entries 4–7 | Cleared entry 0 |
| 3 | Entries 0–3 | Entries 4–7 | Entries 8–11 |
| 4 | Entries 0–3 | Cleared entry 0 | Entries 4–7 |

The base metatile stride stays **16 bytes**, including type 3; the third layer
reads the next eight bytes. Index-zero pixels are transparent to the native
filler/shared backdrop, including on the bottom layer. Types above 4 retain
previous native BG contents, so no deterministic static appearance can be
inferred; their cells use the backdrop and show an explicit UI warning.

Map **1-0**, previously mislabelled by the wrong name table, is labelled
**常磐森林** by the actual table. A disposable mGBA session directly loading its
native layout reproduced the mismatched tiles (header `0x34F214`, layout
`0x2FE46C`). The loaded map and tileset are the ROM record itself. It has **four
inbound warp records**, from 15-0 and 15-3. This is **not proof it is unused** or
that a normal story-aware entrance looks the same. Normal-entry layout/script
replacement remains unverified. The UI displays that limitation, preserving the
ROM-derived image instead of inventing replacement artwork.

1-0 的错乱在原生直接加载中同样出现，不能归咎于修改器的图层遗漏，也不能因为
画面异常就断言未被引用。扩展图层与名称修复有独立证据；这张静态图仍保留明确
提示，正常入口和剧情条件下的最终布局未确认。

## Verification / 验证

- Public core suite: **86 passed, 17 opt-in ignored**. It covers all five profiles,
  every inventory slot, nonzero encryption keys, logical/auxiliary boundaries,
  unrelated bytes, footer preservation, and party/box identity.
- Public pixel vectors: native type-3 base stride, filler/background transparency,
  flips, primary/secondary tile ownership and palette-bank ownership.
- `verify_mercury_display_storage.py`: native pocket descriptors **5**, plaintext
  count vectors **6**, save-tail vectors **3**, BG assignment vectors **5**,
  static names **133**. Original and edited emulator RAM each match **778** slots.
- Current SAV: general items **Potion ×6**, other bag pockets empty; Totodile
  met ID **143**, **若叶镇**. Opening/querying preserves input bytes.
- Disposable SAV edits: Potion ×7, Poké Ball ×5, TM01 ×1 and Cheri Berry ×1.
  Each action's undo restores the exact file, with unchanged trainer/individuals.
  Export loads in mGBA with all four values. Using the **fourth** start-menu icon
  (zero-based 3), the native Save UI saves successfully; counter advances **3→4**.
  Re-read preserves all bag entries and all Totodile fields.
- All-five exact-ROM acquisition/collection/topology regressions passed; BW/DP,
  Rocket and Ultimate retain their own layouts. Mercury current-SAV storage and
  layout opt-in tests supplement these tests.
- TypeScript check and Mercury real-SAV browser regression passed, including
  table/detail quantity consistency, the actual Origin-field label 若叶镇, mocked
  zero-count preservation, native-layout
  warning, and read-only behavior. Full five-profile browser smoke passed the
  query/reference/map/navigation workflow.
- Strict Clippy exposed pre-existing warnings in emergency cheats, trainer/forms,
  and Ultimate tests. Those unrelated warnings were not changed in this correction;
  with only those known lint categories allowed, the changed code is checked.
- Original SAV SHA-256 remains
  `33e4ac873e411cdf214efb46efc881b1c2e5d176d65cfe28f417677569ee7f70`;
  original ROM MD5 is unchanged.

The old attempt to invoke `TrySavingData` in a bounded isolated call hit its
instruction limit; it is **not** counted as a successful save. The result above
comes from the normal in-game Save menu, with its ordinary frame/interrupt flow.
No table-readable / PNG-decodable result is claimed as complete map support.

## Reproduction / 复验

Use absolute local paths; no private data belongs in Git:

```sh
cargo test -p gen3-core
GEN3_ROM_MERCURY12=/private/game.gba GEN3_SAVE_MERCURY12=/private/game.sav \
  cargo test -p gen3-core local_mercury_storage_roundtrip -- --ignored
GEN3_ROM_MERCURY12=/private/game.gba GEN3_SAVE_MERCURY12=/private/game.sav \
  cargo test -p gen3-core exact_save_layout_regression -- --ignored
uv run --with unicorn python scripts/verify_mercury_display_storage.py \
  --rom /private/game.gba
```

Optional `--snapshot`, `--ram`, `--iwram` verify a dev-bridge snapshot against a
corresponding loaded emulator capture. `test_mercury_inventory_ui.py` uses the
same environment variables with Vite and the authorized development bridge;
its zero-count fixture changes only an HTTP response, never SAV bytes.
