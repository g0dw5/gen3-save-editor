# Pokémon appearance / 宝可梦外观

Verified against both exact Dark Phantom 5.0EX+BW / DP profiles on 2026-09-12.
All addresses below are ROM file offsets. No ROM graphics are distributed.

## Rules and evidence

The native front-picture loader at `0x34A40` takes species, PID and a front/back
flag. Its Unown branch (`0x34A56`, bytes `C9 2F`) recognizes species 201. It packs
the low two bits of each PID byte and takes the remainder modulo 28 (`0x34A7C`,
bytes `1C 21`). Letter A uses picture 201; the remaining letters use indices
413–439. The picture-table literal at `0x34AB8` is `8C A1 30 08`.
These are **picture indices**, not additional editable species IDs.
Palettes continue to use the original species index.

Spinda's front-picture routine at `0x6D664` recognizes species 308. Four spot
records, 36 bytes each, begin at `0x31E2F0`; the native literal at `0x6D72C` is
`F0 E2 31 08`. Each record contains x/y coordinates, sixteen u16 mask rows and
padding. One PID byte offsets each spot by -8 through +7 pixels in each axis.
Only body palette indices 1–3 become 5–7. Transparent pixels, outlines and
already painted spots remain unchanged. Masks and palettes are read from the
user's ROM, including for shiny individuals.

The helper at `0x34C30` recognizes species 410 (Deoxys) and copies the second
2048-byte frame over the first. A simple first-frame decoder did not reproduce
this static picture selection. Castform retains its normal storage appearance;
weather transformations require battle state and are not inferred from saves.

The profile configures species IDs, extra picture indices, masks and the special
frame. The renderer implements these verified rules. Offsets alone do not
express PID-dependent image selection or procedural spot drawing.

## UI and scope / 界面与范围

Party and box slots and the Pokémon editor pass the individual's PID together
with species and shiny state. The shared cache includes PID, so different Unown
letters and Spinda patterns cannot reuse another individual's image. Changes
refresh the picture without reopening the application.

Species references, creation drafts and trainer records without a known complete
PID use a representative PID of zero. They are not claims about a generated
trainer Pokémon's exact appearance. The trainer constructor's shiny hook may
change the full PID; nature, gender and ability summaries alone do not determine
an exact Unown letter or Spinda pattern. Temporary battle forms and animations
are outside this static save-view renderer. This change does not edit saves or
add a form selector that rewrites PID.

中文：同行、盒子和编辑页已按具体个体的 PID 显示未知图腾字形与晃晃斑斑点，
同时修正代欧奇希斯的静态帧。图鉴资料、创建草稿及没有完整 PID 的训练家记录
使用代表性图片；不据此声称已预测其实际个体外观。漂浮泡泡在存档中显示普通形态。

## Validation

- Public fixture test covers all 256 Unown selector values and irrelevant PID bits.
- `scripts/test_sprite_appearance.py`: synthetic browser test for simultaneous
  individual images, PID changes, shiny changes and cache reuse. Start Vite first;
  requires Playwright and Chrome, no ROM/save.
- `scripts/verify_pokemon_sprites.py`: executes the ROM's native Thumb picture
  loader in Unicorn, stubbing only BIOS decompression/copy helpers. Compares its
  output pixel for pixel with PNGs from the actual editor API, for both normal
  and shiny palettes. 536 cases per ROM, **1,072 exact comparisons passed**.
- The native test covers every Unown selector, spot-offset extremes and mixed
  Spinda PIDs, an ordinary species, Castform and Deoxys. It starts/stops an
  authenticated development bridge and never loads or writes a save.

```sh
cargo build -p gen3-cli --features dev-server
GEN3_DEV_BIN=target/debug/gen3-dev \
GEN3_ROM_BW='/path/BW.gba' GEN3_ROM_DP='/path/DP.gba' \
python3 scripts/verify_pokemon_sprites.py
```

Python dependencies: Unicorn and Pillow. `GEN3_SPRITE_PREVIEW` optionally selects
an output PNG contact-sheet path; keep extracted previews outside the repository.

Upstream references, checked against the supplied ROM rather than assumed:
[front-picture loader](https://github.com/pret/pokeemerald/blob/master/src/decompress.c),
[Spinda drawing](https://github.com/pret/pokeemerald/blob/master/src/pokemon.c).
