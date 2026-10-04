# Pokémon Mercury FC 1.1 / 宝可梦水银 FC 1.1

This adapter recognizes one user-supplied 32 MiB ROM: MD5
`7e0898caf6e7d41e8c59f838e8e595f1`, SHA-256
`131b009df7ab252deff0d6a0518ab82f88e82c940ee68d50d31033a899f7e3dd`.
Its GBA header identifies `POKEMON FIRE` / `BPRE`. It is based on FireRed with
CFRU-style expanded structures; Emerald map/save offsets are **not** evidence
for this ROM. All data below is read from the supplied ROM at runtime. ROM and
extracted text/images are not bundled.

水银目前按精确 MD5 开放 ROM 只读资料。尚无该版本的真实 `.sav` 用于验证，
因此存档加载、编辑和导出保持关闭；这不是声称它不能修改，而是避免把火红存档
误按绿宝石布局写坏。

## Located tables

File offsets below exclude the GBA `0x08000000` bus base. Raw bytes show
the first record or pointer and are provided so a later patch can be checked
independently. A direct pointer reference supports location, while bounds are
cross-checked against adjacent records and decoding.

| Table | Offset / layout | First bytes | Evidence / confidence |
| --- | --- | --- | --- |
| Species names | `0x141B350`, 1,554 × 11 | `AC AC AC AC AC AC FF FF` | ROM header pointer `0x144`; species 1 妙蛙种子 and 1553 桃歹郎 decode; high |
| Base stats | `0x176DFBC`, 1,554 × 28 | `00 00 00 00 00 00` | 56 pointer references; byte 26 is hidden ability; high |
| Move names | `0x1D8FC34`, 1,015 × 13 | `AE FF FF FF` | ROM header pointer `0x148`; high |
| Move data | `0x1DF68E3`, 1,015 × 12 | `00 28 09 64 23 00` | 434 pointer references; category at byte 10; high |
| Item data | `0x07C7E00`, 750 × 44 | `AC AC AC AC AC AC AC AC` | Coherent Master Ball and other item records with valid description pointers; no direct base pointer found, so medium |
| Ability names | `0x1D87140`, 300 × 13 | `AE AE AE AE AE AE AE FF` | Pointer at `0x1C0`; high |
| Ability descriptions | `0x1C93DFC`, 255 verified × 4 | `C4 F3 24 08 D8 F3 24 08` | Pointer at `0x1C4`; entries 255–299 are **not** treated as text; high for verified range |
| Evolution records | `0x1788F5A`, 1,554 × 16 × 8 | `00 00 00 00 00 00 00 00` | Five direct pointer references; methods `0xFD` and `0xFE` are battle forms, not permanent evolution; high |
| Level-up lists | `0x17C32BC`, 1,554 × pointer | `C8 64 7D 09 89 64 7D 09` | Six references; packed `{u16 move,u8 level}`, `00 00 FF` terminator; high |
| Front sprites | `0x17BB0A4`, 1,554 × 8 | `C0 85 54 09 00 08 00 00` | All 1,435 nonempty/stat-valid species render normally and shiny; no direct table-base pointer found, so medium-high |
| Back sprites | `0x176AF2C`, 1,554 × 8 | `18 F6 41 09 00 08 00 00` | ROM header pointer `0x12C`; high |
| Normal palettes | `0x17D64CC`, 1,554 × 8 | `04 88 54 09 00 00 00 00` | ROM header pointer `0x130`; high |
| Shiny palettes | `0x17E49D4`, 1,554 × 8 | `5C F8 41 09 12 06 00 00` | ROM header pointer `0x134`; tags start at 1554; high |

The species and picture tables contain 116 aligned empty slots in the expanded
range; IDs 252 and 412 also have zero HP despite picture data. They stay visible
as ROM slots in the catalog, but `valid_species` rejects them for detail/edit
operations. Nature and type labels likewise come from ROM pointer/name tables.
The battle-form pass finds 144 Mega and 89 Gigantamax edges; their display is
a table interpretation, not a claim that every transformation is reachable.

The structure interpretations were cross-checked against the upstream
[CFRU `include/pokemon.h`](https://github.com/Skeli789/Complete-Fire-Red-Upgrade/blob/master/include/pokemon.h),
especially `BaseStats`, packed `LevelUpMove`, `EvolutionMethods`,
`EVO_GIGANTAMAX = 0xFD`, and `EVO_MEGA = 0xFE`. The exact water-Mercury ROM is
still the authority for addresses, names, counts and values.

### Map research in progress

A likely FireRed-style map-group pointer table starts at `0xB3B100`
(`04 20 35 08 98 F7 CA 08`) and has a code reference at `0x5524C`.
It contains 58 consecutive candidate group pointers. Groups 0–42 track the
stock FireRed group order from
[pret/pokefirered's map groups](https://github.com/pret/pokefirered/blob/master/data/maps/map_groups.json),
with some enlarged groups; additional groups 43–57 are plausible but are not
yet bounded by native lookup logic. One candidate map-name pointer run for
section IDs 88 and above starts at `0x45F89C` (`45 0C 1B 08 51 0C 1B 08`),
with a reference at `0x12DB00`; index 88 decodes to 真新镇. These facts are
recorded for the next stage, **not** activated as a complete map adapter:
group counts, tileset partitions, event commands, and map rendering still
need verification against the running ROM.

## Enabled and pending

Enabled now: exact-ROM identification, species and stat reference, abilities,
moves, items, natures/types, permanent evolution and battle-form links,
level-up moves, normal and shiny front pictures. Evolution conditions are
translated using CFRU method semantics; unresolved tile/map/event identities
are described generically rather than inventing a place.

Pending verification: move descriptions, complete TM/tutor/egg learnsets,
species-specific picture rules (Spinda/Unown/female), map layouts and graphics,
encounters, NPCs, trainer rosters, story scripts, save sector/individual layout,
and cheat hooks. The UI hides map/trainer tabs and disables save loading for
this profile. The read-only adapter does not reuse another ROM's world pointers
or cheats.

## Regression procedure

`cargo test -p gen3-core` exercises public fixtures. The ignored exact-ROM
test `mercury::tests::exact_rom_read_only_regression` runs with
`GEN3_ROM_MERCURY` pointed to the user's ROM. It verifies fingerprint,
table sizes and sample contents, level-up and form edges, explicit unsupported
operations, the changed-byte fingerprint rejection, and PNG rendering of both
normal and shiny front sprites for every stat-valid species. Existing exact-ROM
regressions for BW, DP, Rocket and Ultimate Emerald are run separately with
their local ROM files. No private ROM, save, or extracted asset is committed.
