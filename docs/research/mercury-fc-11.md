# Pokémon Mercury FC 1.1 / 1.2 / 宝可梦水银 FC

The original 1.1 profile recognizes a user-supplied 32 MiB ROM: MD5
`7e0898caf6e7d41e8c59f838e8e595f1`, SHA-256
`131b009df7ab252deff0d6a0518ab82f88e82c940ee68d50d31033a899f7e3dd`.
Its GBA header identifies `POKEMON FIRE` / `BPRE`. It is based on FireRed with
CFRU-style expanded structures; Emerald map/save offsets are **not** evidence
for this ROM. All data below is read from the supplied ROM at runtime. ROM and
extracted text/images are not bundled.

Version 1.2 is a separate exact profile: MD5
`f323df1792ac68462a34b42fe8571533`, SHA-256
`b98d9701f4b567810c70221564c348f4482791c614c3f1bb282e96678b7a0896`.
Both builds are 32 MiB. The original table inventory below describes 1.1;
the relocated 1.2 entries are recorded in the final section.

水银按精确 MD5 开放 ROM 资料和存档编辑。已在 mGBA 中从新游戏正常保存，
并验证扇区、角色资料、空队伍、背包、箱名及 RTC 尾部。随后在副本中创建测试
宝可梦、修改金钱与精灵球及箱名，修改器完成副本回读与校验；1.2 的副本在
mGBA 中正常加载，同行图标及运行内存字段与编辑结果一致；游戏再次保存后
完成回读，同行、箱子、捕获球、PID、金钱、球数量及箱名保持正确。真实进度
中扩展图鉴位等结构仍需另行验证。

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
| Move descriptions | `0x1CC8E08`, 1,014 pointers for moves 1–1014 | `34 28 48 08` | Referenced from two display routines; move 0 has no description; high |
| Item data | `0x07C7E00`, 750 × 44 | `AC AC AC AC AC AC AC AC` | Coherent Master Ball and other item records with valid description pointers; no direct base pointer found, so medium |
| Ability names | `0x1D87140`, 300 × 13 | `AE AE AE AE AE AE AE FF` | Pointer at `0x1C0`; high |
| Ability descriptions | `0x1C93DFC`, 255 verified × 4 | `C4 F3 24 08 D8 F3 24 08` | Pointer at `0x1C4`; entries 255–299 are **not** treated as text; high for verified range |
| Evolution records | `0x1788F5A`, 1,554 × 16 × 8 | `00 00 00 00 00 00 00 00` | Five direct pointer references; methods `0xFD` and `0xFE` are battle forms, not permanent evolution; high |
| Level-up lists | `0x17C32BC`, 1,554 × pointer | `C8 64 7D 09 89 64 7D 09` | Six references; packed `{u16 move,u8 level}`, `00 00 FF` terminator; high |
| Experience thresholds | `0x1DFE8CC`, six growth rows × 256 × 4 | `00 00 00 00 01 00 00 00` | 26 code references into medium-fast row; other five rows and level 100 thresholds agree with native table structure; high |
| Egg moves | `0x1781B64`, 14,065 words + terminator | `21 4E 82 00` | Three direct references; species markers use 20000 + ID; high |
| TM/HM moves | `0x17E87AA`, 120 TM + eight HM halfwords | `08 01 51 01 60 01` | Native lookup references at `0x125A8C` and `0x125AAC`; high |
| Tutor moves | `0x17E8686`, 145 halfwords | `—` | Native lookup at `0x1D3DC00` through pointer at `0x120BE4`; high |
| Machine compatibility | `0x1400494`, 1,554 × 16-byte bitsets | `00…` | Native predicate at `0x1D3DA2C` uses species × 16 and 128 bits; high |
| Tutor compatibility | `0x14065B4`, 1,554 × 20-byte bitsets | `00…` | Native predicate at `0x1D3DABC` uses species × 20 and 145 bits; high |
| Front sprites | `0x17BB0A4`, 1,554 × 8 | `C0 85 54 09 00 08 00 00` | All 1,435 nonempty/stat-valid species render normally and shiny; no direct table-base pointer found, so medium-high |
| Back sprites | `0x176AF2C`, 1,554 × 8 | `18 F6 41 09 00 08 00 00` | ROM header pointer `0x12C`; high |
| Normal palettes | `0x17D64CC`, 1,554 × 8 | `04 88 54 09 00 00 00 00` | ROM header pointer `0x130`; high |
| Shiny palettes | `0x17E49D4`, 1,554 × 8 | `5C F8 41 09 12 06 00 00` | ROM header pointer `0x134`; tags start at 1554; high |
| Map groups | `0xB3B100`, 58 group pointers, 871 map headers | `04 20 35 08 98 F7 CA 08` | Native reference at `0x5524C`; group counts bounded and four rendered maps inspected; high for listed maps |
| Map-section names | `0x45F89C`, pointer names for section IDs 88–255 | `45 0C 1B 08` | Native reference at `0x12DB00`; index 88 is 真新镇; high |
| Trainer roster | `0x23EAC8`, 743 × 40-byte records (742 populated) | `—` | 21 direct references; party records and classes decode; high for raw roster |
| Trainer classes | `0x23E558`, 107 × 13-byte names | `—` | Two direct references; high |
| Trainer art | `0x23957C` sprites, `0x239A1C` palettes | `—` | Every portrait used by the parsed roster renders as PNG; high |
| NPC art | `0x961600` graphics pointers, `0x1E07684` expanded palettes plus stock palette bank `0x3A5158` | `—` | All static graphics IDs used by parsed map objects render except unresolved IDs 196 and 197; medium-high |
| Wild encounters | base `0xCF915C`; day `0x1E395B4`; night `0x1E3CE10`; morning `0x1E40DA0`; dusk `0x1E3EEA4` | 20-byte FireRed records | Native time selector at `0x1D65828`; periods are 04–07, 08–16, 17–19, 20–03; high for source tables |

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

The map renderer uses FireRed's 640 primary tiles and 640 primary metatiles,
with seven primary and six secondary palette banks. The ROM yields 871 maps;
Pallet Town, the adjacent route, and two added groups were rendered as PNGs.
Script traversal locates 737 trainer-to-map references. These are references,
not proof that every map or encounter is reachable in the current story state.
Time-specific wild tables are presented separately from the base table because
the native selector switches by clock period; grouping equal physical slots
does not imply separate grass patches. Two NPC graphics IDs, 196 and 197,
do not point to static descriptors and remain unavailable in the UI.

## Enabled and pending

Enabled now: exact-ROM identification; species/stats, abilities, moves and
descriptions, items, natures/types, permanent evolution and battle-form links;
complete level-up, egg, TM/HM and tutor move sources; ROM experience thresholds;
normal/shiny front pictures and PID-dependent Unown/Spinda appearance; maps,
events, wild encounters, trainer rosters, portraits and static NPC art.
The split TM item ranges are configuration only; compatibility values come
from this ROM's bitsets. Evolution conditions use CFRU method semantics.

Pending verification: female-specific picture rules; exact CFRU trainer
generation (nature, gender, ability, IV and EV rather than just raw roster
quality); deeper story reachability; expanded Pokédex layout and special
individual fields in a progressed save; and runtime cheat hooks. Save loading
and writing are enabled for the verified core fields, while Pokédex flag edits
and cheats remain disabled. The adapter does not reuse another ROM's save
offsets or cheats.

## Save structure verified with an in-game baseline

The GFRomHeader at `0x100` supplies FireRed SaveBlock2 size `0xF24`,
SaveBlock1 size `0x3D68`, party count at `+0x34`, party records at `+0x38`,
and bag capacities. In contrast to Emerald's `0xF80` and Rocket's `0xFF4` sector chunks, the
game writes SaveBlock1 and Pokémon Storage in `0xFF0`-byte chunks. The last
chunks are `0xD98` and `0x450`; those exact lengths reproduce all fourteen
native checksums. Live mGBA RAM (`gSaveBlock1Ptr` at `0x03005008`,
`gPokemonStoragePtr` at `0x03005010`) matches the assembled bytes. Using
`0xFF4` would shift the last box and misinterpret the default box names as
Pokémon even though sector checksum verification could pass.

The stock FireRed offsets found in the ROM header and baseline are: money
`0x290`, coins `0x294`, registered item `0x296`, PC items `0x298` (30), items
`0x310` (42), key items `0x3B8` (30), balls `0x430` (13), TM/HM `0x464`
(58), berries `0x54C` (43), and encryption key in SaveBlock2 at `0xF20`.
Storage holds fourteen 30-slot boxes, with names beginning at logical
storage `0x8344`. An empty storage record can contain nonzero unrelated
bytes; its native `hasSpecies` bit is checked before Pokémon checksum
validation. The mGBA `.sav` has a 16-byte RTC trailer, retained verbatim;
raw 128 KiB saves are also accepted.

Both exact ROMs disable Pokémon encryption, block shuffling and checksum
checks (`0x3F906`, `0x3F90C`, `0x3F92A`, `0x3F930`, `0x3F94C`,
`0x3FD94`, `0x4051E`, `0x40AE6`). Individuals contain ordered plaintext
substructures. The ball uses Growth byte 10; Misc origin bit 11 is the
Gigantamax flag and is preserved. Bit 31 of the Misc IV word marks a hidden ability; ordinary
abilities use PID parity with the native no-second-ability fallback. The origin
game of a newly created individual is 4 (FireRed). Ball choices read each ball
item's native `item_type`, rather than assuming the item ID equals its stored
ball type. These semantics have separate adapter configuration/codec behavior.

`scripts/verify_mercury_native.py` calls each ROM's own `GetMonData`,
`SetMonData`, `GetMonAbility` and stat routine. For each build, 840 synthetic
records across six species and 24 PIDs passed 18,480 field comparisons,
576 setter byte comparisons and 840 stat comparisons. ROM/save input files
are never written by this verifier.

A local probe applied a created test Pokémon in the party and a box, money,
Poké Balls, and a box rename to a copy, then reopened it and verified all
sector checksums and fields. mGBA loaded the 1.2 edited copy with the expected
party count, species, ball type and PID in native RAM, and displayed its party
sprite and edited money on the trainer card. The 1.2 game then saved the edited
copy through its native menu. The editor reopened that save, validated both
save banks and confirmed party/box species, PID, ball, origin, money, ball
quantity and box name. mGBA added a supported 16-byte RTC trailer. This covers
the tested core fields, not every later-game extension. No test ROM/save or
extracted assets are committed.

## Regression procedure

`cargo test -p gen3-core` exercises public fixtures. The ignored exact-ROM
test `mercury::tests::exact_rom_regression` runs with
`GEN3_ROM_MERCURY` pointed to the user's ROM. It verifies fingerprint,
table sizes and sample contents, complete machine/egg/tutor sources,
experience thresholds, maps/encounters/trainers, representative map images,
all roster portraits and static NPC graphics, invalid-save rejection, the
changed-byte fingerprint rejection, and PNG rendering of both
normal and shiny front sprites for every stat-valid species. Existing exact-ROM
regressions for BW, DP, Rocket and Ultimate Emerald are run separately with
their local ROM files. No private ROM, save, or extracted asset is committed.

The corresponding 1.2 test uses `GEN3_ROM_MERCURY12`. Supply an absolute
`GEN3_MERCURY_PROBES` directory to produce synthetic individual probe JSON;
pass the matching file to `verify_mercury_native.py --rom … --probes …`.
Save probes use `GEN3_SAVE_MERCURY` / `GEN3_SAVE_MERCURY12` and write a test
copy only when `GEN3_SAVE_MERCURY_PROBE` / `GEN3_SAVE_MERCURY12_PROBE` is set.
The public save matrix covers both profiles, individual field ownership,
party/storage transfers, undo/export validation and inventory sector boundaries.
Additional CFRU checks preserve Gigantamax and hidden-ability flags, including
native hidden-ability fallback, and avoid forcing even PIDs for species with
only one ordinary ability.
The 1.2 native resave regression uses `GEN3_SAVE_MERCURY12_NATIVE`. The custom
start menu's third icon is Save; the fourth icon changes time. Frame-held
inputs need spacing through menu transitions in mGBA. Native UI input can miss
very short key presses; check input routing and the input method before
interpreting a missed press as an unavailable game feature.

## Version 1.2 relocations

4,669,666 bytes differ from 1.1. Species names, machine compatibility,
egg moves and evolution records match at their original addresses. Base stats
and trainer contents have changes even though their table addresses remain
the same; they are read directly from the loaded version. The map and trainer
counts remain 871 and 742; script traversal finds 739 trainer map references,
compared with 737 in 1.1.

| Table | 1.2 offset | Evidence |
| --- | --- | --- |
| Move names / data | `0x1D93F34` / `0x1DFD2DF` | GFRomHeader pointers |
| Ability names / descriptions | `0x1D8B430` / `0x1C93EB8` | GFRomHeader pointers |
| Move descriptions | `0x1CC8EBC` | Relocated pointer table and decoded text |
| Type names | `0x1DB5DA4` | Full label block comparison |
| Nature names | `0x1FF2888` | 25 relocated text pointers |
| Experience thresholds | `0x1E052C8` | Relocated table, native stat comparisons |
| National dex / tutor moves / TM moves | `0x17E7A78` / `0x17E869A` / `0x17E87BE` | Data and native lookup pointers |
| Normal / shiny palettes | `0x17D64E0` / `0x17E49E8` | GFRomHeader pointers |
| NPC palettes | `0x1E0E080` | Pointer table and complete used-NPC render pass |
| Morning / day / dusk / night encounters | `0x1E4779C` / `0x1E3FFB0` / `0x1E458A0` / `0x1E4380C` | Native time-table pointer literals |

The user-supplied 1.2 battery save is 128 KiB. It contains an opening-stage
trainer and no party/storage Pokémon or bag items. It loads losslessly through
the shared verified sector layout; this does not establish later-game Pokédex
or story-state semantics.
