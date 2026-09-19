# Team Rocket 2.1 Chinese ROM compatibility / 西班牙火箭队兼容性评估

Assessment date: 2026-09-19. This is research, **not a supported write adapter**.
No ROM, game text dump, graphics, or user save is distributed with this document.

ROM fingerprint: MD5 `59c658a1081f542086de1060bb65f0b3`, 33,554,432 bytes.
GBA title/code: `POKEMON EMER` / `BPEE`. Findings apply to these exact bytes,
not every release carrying the Team Rocket 2.1 name.

## Conclusion / 结论

The application architecture can be retained, but this is not an offset-only
profile. It requires a second Pokémon field codec, expanded ROM record readers,
profile-owned bag/dex definitions and rule/capability configuration. The largest
remaining research scope is complete encounter/script/trainer coverage.

中文：可沿用现有应用框架。对底层适配层属于中等偏大的扩展，对 UI 属于局部调整；
不需要复制整套修改器，但不能只新增偏移量、表数量和 MD5 就开放写入。
现有 BW/DP 两个 profile 共享全部布局，还没有真正验证跨引擎的字段复用。

## Directly verified / 已验证

### Save container and record encryption

The supplied battery save is 131,072 bytes, with two complete 14-sector banks,
the standard `0x08012025` signature, and consecutive save counters. The native
section-layout table is at ROM file offset `0xC9BDFC`; code at `0x19EB30`
references it via the literal at `0x19EB98`.

Logical section lengths are:

```text
0: E1C
1–3: FF4
4: 62C
5–12: FF4
13: 5C0
```

Both banks pass checksums using these lengths. Zero padding makes other checksum
lengths appear valid for some sectors; the native layout table resolves that
ambiguity. Do not infer logical lengths by checksum matches alone.

A separate read-only Rust harness directly called the current `gen3-core`:

- The production ROM fingerprint gate rejects this ROM as `unsupported_rom`.
- `Save::open` with `EMERALD` lengths rejects both banks at section 1.
- Supplying the verified lengths to the existing `SaveLayout` opens both banks
  and selects the newer bank correctly. Party offsets `0x234`/`0x238` still fit.
- All **6 party and 3 boxed records** pass the existing Pokémon checksum and
  byte-for-byte `unpack` → `pack` round trip. The in-memory save remains identical.
- The PC still has 14 × 30 conventional 80-byte slots, names and wallpapers in
  the expected prefix. The longer `0x8560` storage block has additional bytes
  after that prefix; their purpose is not established and they must be preserved.

This verifies container/crypto reuse, **not field semantics or safe editing**.
No edited save was exported or loaded into the game during this assessment.

### Pokémon field semantics: a required new codec

The ROM's actual GetMonData-style routine at `0x976D0` was executed in Unicorn,
using the supplied party records. Its input record was checked unchanged after
each call. Synthetic copies were decrypted, had one bit flipped, were reencrypted
with a fresh checksum, and were queried again to identify the controlling bits.
These mutations existed only in host/emulated memory, never in the user's save.

Offsets below refer to the 48 decrypted bytes in canonical G/A/E/M order:

| Field | Current Dark Phantom codec | This ROM's native getter |
|---|---|---|
| Experience | u32 at +4 | bits 0–22 at +4 |
| Packed PP Ups | byte +8 | byte +7 |
| Friendship | byte +9 | byte +8 |
| Capture ball | origin word +38, bits 11–14 | byte +9, bits 0–4 |
| Ability slot | IV word +40, bit 31 | byte +47, bits 0–1 |
| Current PP | bytes +20…23 | same positions in tested getters |
| IVs | six 5-bit fields at +40 | same positions in tested getters |

Concrete mismatch in the first supplied party record:

| Value | Current field interpretation | Native getter |
|---|---:|---:|
| Friendship | 65 | 120 |
| Packed PP Ups | 120 | 0 |
| Capture ball | 0 | 1 |
| Ability slot | 0 | 1 |

Consequences are not cosmetic: the current friendship writer would overwrite
bits used by the new capture-ball field; a whole-u32 experience write would
overwrite the new PP Ups byte. Existing ribbon editing also owns bits now used
by other fields. Passing the checksum cannot detect such semantic mistakes.

中文：外层仍是三代加密格式，内部字段却重新分配了位。必须按新布局读写并保留
同一字节里的其他字段，不能将“解密成功”理解为“可以直接套用旧编辑器”。
上述字段定位不等于已穷尽全部新字段、形态、缎带及联动规则。

### ROM tables and text

The embedded metadata header provides useful initial pointers. Multiple adjacent
records and native references were inspected; table boundaries still need a
complete adapter audit.

| Table / behavior | Observed evidence | Framework impact |
|---|---|---|
| Species names | `0x59F9F0`, 11-byte records; current Chinese map decodes the party names and additional forms | Existing codec is promising; control codes/full coverage remain unverified |
| Species stats | `0x5B4764`, 36-byte records; native indexing at `0x9B2B8` multiplies by 36 | Current reader assumes 28-byte field layout, not just stride |
| Abilities | Three u16 slots at stats +24/+26/+28; IDs above 255 occur | Current `[u8; 2]` species abilities, u8 individual ability ID and fixed 151-entry catalog must change |
| Move names | `0x5A35E1`, 13-byte records; move 608 is present in the supplied party | Cannot use a 9-bit level-up move decoder |
| Battle moves | `0x5ACD5C`, 20-byte records; u16 effect/power, category +16 | Current 12-byte reader and several u8 fields must change |
| Level-up moves | Pointer table candidate `0x614AC4`; entries 0/1 point to `0x5C0B90`, holding u16 move/u16 level pairs | Add a separate list encoding; do not decode as `(level << 9) \| move` |
| Items | `0xC3D558`, sampled 44-byte records | Name/description reading largely reusable; pocket/ID semantics are different |
| Ball item IDs | IDs 1/2/3/4 decode as Poké/Great/Ultra/Master Ball | Remove assumptions such as Master Ball = 1 |
| Hidden Power | Move 237 record declares power 60, whereas Dark Phantom declares placeholder 1 | Do not enable the existing 30–70 formula by inheritance; native battle calculation remains to verify |
| Types | Move 608 uses type 18 | Add Fairy and make type support/profile rules explicit |

Do not interpret the number of contiguous name records as the number of distinct
obtainable Pokémon. Base species, alternate forms and dex numbers need separate
mapping. Likewise, unusually high learnset level values are not proof of a raised
playable level cap without tracing their consumers.

### Bag, dex and graphics

The header advertises bag capacities, in the order items/key items/balls/TMs/
berries/PC: **255/64/16/254/68/10**. Sample bag contents match these locations:

```text
PC:       0498, 10 unencrypted entries
Items:    04C0, 255 entries
Key:      08BC, 64 entries
Balls:    09BC, 16 entries
TMs:      09FC, 254 entries
Berries:  0DF4, 68 entries
```

The sampled encrypted quantities and money decode plausibly with the current
key offset `0xAC`. The existing global `POCKETS` constant must become profile data;
item pocket-category semantics also need verification, not just offsets.

Current dex code hardcodes 416 flags and mirrored Emerald offsets including
`0x3B24`; the new SaveBlock1 is only `0x3608` bytes. It cannot be reused as-is.

Front/normal/shiny graphics tables at `0x568880`/`0x5545CC`/`0x558EA4` successfully
decode sampled species using the existing LZ10 decompressor: 4,096-byte picture
data and 32-byte palettes. This supports compression/tile renderer reuse, not
complete alternate-form or animation selection. Dark Phantom's extra Unown
picture indices and other appearance rules must not be inherited blindly.

### Maps, encounters and trainers: preliminary only

The map-group pointer table candidate at `0x9F4F40` points into the contiguous
map-pointer area beginning `0x9F39F4`. Sample headers use the familiar 28-byte
structure with layouts, events and scripts. The first sampled layout is at
`0x7C10D0`; its tilesets use familiar compressed tile/palette/metatile pointers.

Full bounds, tile attributes/palette ownership, wild selection, scripted gifts,
hidden items, NPC graphics and trainer party formats have **not** been verified.
Do not advertise current map/script parsing as fully compatible. The existing
trainer personality/IV generation code explicitly models Dark Phantom behavior;
it requires a separate rule before displaying deterministic opponent details.

## Proposed adaptation boundary / 建议改造边界

1. Keep the React/Tauri application, shared view models, search, party/box UI,
   drag/drop, transactions, undo/redo, atomic export and backup infrastructure.
2. Make ROM records select explicit layout families: species, moves, learnsets,
   evolutions and trainer parties. Widen ability IDs and represent three slots.
3. Split Pokémon envelope crypto from field decode/patch/create. Keep the current
   codec intact and add a verified codec for this ROM; conversion and stat
   recalculation must dispatch through it.
4. Move bag/dex layouts, item mappings, valid types, limits and feature support
   into the adapter. Use capabilities to avoid presenting unverified features.
5. Release write support only against the exact ROM hash, after native getter/
   setter comparisons, preservation tests, all storage boundaries, failed-batch
   rollback and disposable in-game round trips.

中文：先开放只读资料和存档预览，再开放经验证的基础编辑，最后补齐地图剧情、
相遇条件、训练家及高级字段。采用“通用算法 + 布局类型 + 版本配置 + 特殊规则”，
不要复制完整引擎，也不要把所有差异硬塞进 offset 字典或 scattered ROM-name checks。

The native evidence indicates an expanded Emerald family with local changes.
Upstream expansion headers help identify candidates but are not an exact schema
for this ROM: its measured PP/friendship/ball layout differs from the example
upstream version below.

## References

- [Emerald metadata header](https://github.com/pret/pokeemerald/blob/master/src/rom_header_gf.c)
- [Expansion 1.5 Pokémon/ROM structures](https://github.com/rh-hideout/pokeemerald-expansion/blob/expansion/1.5.0/include/pokemon.h)
- [Author's Emerald project thread](https://www.pokecommunity.com/threads/pok%C3%A9mon-team-rocket-edition-dragonsden-version-kanto-sevii-johto-next-release-dlc-season-4.527368/)
