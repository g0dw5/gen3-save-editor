# Map palettes / 地图调色板

Verified on 2026-09-12 against the exact BW and DP profiles listed in the README.
Addresses are ROM file offsets. Confidence: native-code execution, in addition
to static inspection. No ROM graphics or saves are committed.

## Native loading rules

Tile graphics IDs and palette bank IDs are independent. Graphics IDs 0–511 use
primary tiles, and 512–1023 use secondary tiles. The upper four bits of each
metatile entry address a **shared** palette, regardless of the graphics source.

The map palette loader at `0x88DD4` calls the primary and secondary wrappers:

- `0x88D8C`: primary wrapper; `0x88D90` bytes `00 21 C0 22` select destination
  color 0 and size 192 bytes (banks 0–5).
- `0x88D9C`: secondary wrapper; `0x88DA0` bytes `60 21 E0 22` select destination
  color 96 and size 224 bytes (banks 6–12).
- `0x88CC4`: common loader. `0x88D1E` bytes `C0 30` advance the secondary
  source pointer by 192 bytes. The unused first six banks in the secondary
  palette must be skipped. The primary branch forces color 0 to black.
- `0xA1938`: native `LoadPalette`, intercepted by the verification harness to
  observe source, destination and size without emulating the display hardware.

This agrees with the upstream [Emerald field map loader](https://github.com/pret/pokeemerald/blob/master/src/fieldmap.c),
but the counts above were verified from the supplied hacks' machine code.
The version profile stores the primary/secondary bank counts as `[6, 7]`.
Banks 13–15 are outside this static tileset loader; the preview initializes them
to zero. Runtime weather, palette animations and other display effects are not
reproduced by this fix.

## Reported map / 红莲之窟

Map `35-24` is 40×40 metatiles, with layout `0xE63420` (decimal 15086624).
Its primary tileset is `0x3DF704`, with palette at `0xDD4E10`; its secondary
is `0x3DF794`, with palette at `0x342E28`.

The previous renderer selected the palette using the graphics tile ID. Some
secondary cave-wall tiles reference primary palette banks; their unused
secondary palette entries produced black walls. Combining the banks before
rendering restores the wall colors without modifying the ROM or save.

## Verification / 验证

`map_palette_uses_bank_ownership_and_skips_unused_source_banks` checks the split,
backdrop, unused banks, invalid counts and truncated sources with generated data.
The optional exact-ROM regression also renders map `35-24` and Safari map `26-1`.

Run `scripts/verify_map_palettes.py` with `GEN3_ROM_BW`, `GEN3_ROM_DP` and
`GEN3_DEV_BIN` set to a `gen3-dev` build (`--features dev-server`). It requires
Unicorn and Pillow. The harness executes the native palette loader for all
707 maps in each ROM and compares eight representative static map PNGs per
ROM, pixel for pixel, against independently composed tiles using those native
palettes. This checks the palette fix, not every aspect of live game rendering.

红莲之窟的黑色岩壁来自调色板选择错误。图块编号与调色板编号彼此独立，
不能按图块来自主／副图块集来选择颜色。修复采用 ROM 原生加载规则合并
调色板；BW、DP 各 707 张地图的调色板及各 8 张静态预览已完成核对。
