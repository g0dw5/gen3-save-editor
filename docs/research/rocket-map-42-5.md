# Rocket map 42-5: scrambled static layout / 西火 42-5 静态布局错乱

2026-09-19. Exact ROM MD5: `59c658a1081f542086de1060bb65f0b3`.
The user observed this in the editor's map list; no normal in-game rendering of
the corresponding room was supplied. ROMs and extracted images remain private.

## Finding

The scrambled preview is reproducible, but no discrepancy was found between
the editor's static rendering and the exact ROM's native layout/grid/background
selection and palette-loading routines. The layout appears incompatible with
its assigned graphics. This does **not** prove the room is unreachable, that it
is safe to delete, or that every possible in-game entry path displays it this way.
No rendering-rule or ROM-data change is justified by this case alone.

| Evidence | File offset / value |
| --- | --- |
| Header | `0x9F07A4` |
| Header layout ID | 62 (`0x003E`) |
| Layout pointer | `0x80808C`, 14 × 10 cells |
| Map cell data | `0x807F74`, 140 `u16` entries |
| Primary / secondary tilesets | `0x7C0360` / `0x7BFDA0` |
| Compressed tiles | `0x6C6C44` / `0x681638`; decode to 640 / 384 tiles |
| Metatile tables | `0x7680D2` / `0x7253CE` |
| Palette tables | `0x6C6AA4` / `0x6826BC` |
| Layout table | `0x9E90C4`; entry 61 also points to `0x80808C` |

Native verification with synthetic RAM:

- `0xB9FD0` selects layout 62 from the native table, returning `0x0880808C`.
  Thus the header pointer is not merely an obsolete alternate pointer bypassed
  by ordinary layout-ID lookup.
- `0xBD894` copies the ROM cells into the map grid, using BIOS CpuSet semantics
  at the copy boundary. `0xBDC6C` returns all 140 expected metatile IDs.
- `0xBFB90` and `0xBFBFC` select/write all three layers: **1,680 tile entries**
  match the reader. Layer-type results are 120 cells of type 0 and 20 of type 2.
- `0xBE918` / `0xBE808` load the palette through `0xD8090`: primary banks 0–6,
  secondary banks 7–12, with a black backdrop. The weather-backup helper is
  skipped; no day/night/weather transform is claimed.
- An independent compositor using the native tile entries/palette and decoded
  ROM tile bytes matches **all 35,840 RGBA pixels** of the editor's PNG.

The inspected map script table is at `0x260ADD`. On-load branches include calls
to door-tile scripts at `0x2E4FEE`, `0x2E5014` and `0x2E503A`, changing pairs of
cells at x=5 or x=9, y=2/3. These do not replace the complete 14 × 10 layout.
This is a bounded inspection of those branches, not execution of every event or
special in every save state.

Run the focused regression with a private ROM and an editor-produced PNG:

```sh
python3 scripts/verify_rocket_map_42_5.py \
  --rom /path/to/exact-rocket.gba \
  --rendered /path/to/editor-map-42-5.png \
  --output /private/output/native-static.png
```

Requires Unicorn and Pillow. No save is opened or written. PNGs are extracted
game assets and must stay outside tracked files and releases. The public test
contains only offset evidence and comparison logic, not a stored correct image.

中文结论：错乱确实存在，但这次原生布局读取、格位复制、三层图块写入、调色板加载
以及独立像素合成都与修改器一致。证据指向原始布局和它所引用的图块资源不配套，
而非已确认的修改器静态读取错误。普通布局编号查询没有切换到另一份正常布局；
检查到的进图分支只替换少量门块。尚未完整运行游戏中的进入流程，因此不宣称该
地图不可达，也不宣称游戏内必然同样错乱。本次保留数据原貌，不猜测替换资源。
