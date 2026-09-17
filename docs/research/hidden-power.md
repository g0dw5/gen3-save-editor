# Hidden Power / 觉醒力量

Verified on 2026-09-17 against both MD5-pinned Dark Phantom 5.0EX+BW/DP profiles.

## ROM evidence

Move 237 has effect 135, placeholder type 0 (Normal), placeholder power 1,
and category 1 (Special) in both move tables at file offset `0x1900000`.
The effect-table entry at `0x1910000 + 135 * 4` points to `0x2D9B37`.
That script begins with command `0xC1`; the command-table entry at `0x310804`
points to Thumb function `0x08054401` (file offset `0x54400`).

The routine reads packed IVs from `gBattleMons` (`0x02024084`), at offset `0x14`
in the attacker's `0x58`-byte record. The attacker is selected by `0x0202420B`.
It writes the calculated u16 power to `0x02024400`, and the type to byte `0x13`
of the battle struct pointed to by `0x0202449C`. The high two type bits are
battle-engine flags, not part of the type ID. Both divisions call the original
ROM division routine at `0x2E7540`.

The formulas match the upstream
[Emerald command](https://github.com/pret/pokeemerald/blob/master/src/battle_script_commands.c),
but the supplied hacks were checked directly instead of inferred from that source.

## Formulas / 公式

Use IV order **HP, Attack, Defense, Speed, Sp. Atk, Sp. Def**. For IVs `v[i]`:

```text
a = sum((v[i] % 2) * 2^i), i = 0..5
typeIndex = floor(15 * a / 63)

b = sum((floor(v[i] / 2) % 2) * 2^i), i = 0..5
power = 30 + floor(40 * b / 63)
```

`typeIndex` selects from these 16 types, in order:
Fighting, Flying, Poison, Ground, Rock, Bug, Ghost, Steel,
Fire, Water, Grass, Electric, Psychic, Ice, Dragon, Dark.
These are **list indices**, not ROM type IDs: ROM ID 9 is skipped.
All 31 IVs yield Dark/70; all 30 yield Fighting/70;
`31,30,30,31,31,31` yields Ice/70. There is no Normal or Fairy result.

中文：按 **HP、攻击、防御、速度、特攻、特防** 排列六项个体值。
属性使用最低一位（奇偶），威力使用次低一位，权重依次为 1、2、4、8、16、32。
属性索引 0–15 依次对应：格斗、飞行、毒、地面、岩石、虫、幽灵、钢、火、水、
草、电、超能、冰、龙、恶。本作两版威力均为 30–70；等级、努力值、性格和 PID
不参与这两项计算。实际伤害仍受能力值、属性一致加成、相克等战斗条件影响。

## Implementation and UI

Profiles advertise `hidden_power: { move_id: 237, formula: "gen3_to5" }`.
`ui/hiddenPower.ts` is a pure derived-value calculator. Missing/unsupported rules
or invalid/unknown IVs return no result. Future engine variants must configure
their verified formula rather than relying on an offset or move name alone.
The interface retains category from the ROM; it does not use the old type-based
physical/special classification for these split-category hacks.

The stats tab shows the potential result even if the move is not learned, with
an explicit explanation. The moves tab and search picker use the current draft
IVs. Opponent records with random IVs show an unknown result. Generic references
show 30–70 and explain variable type instead of presenting Normal/1 as actual
battle values. The separate raw ROM table patch controls still show raw bytes.

No derived field is written into the save. Only explicitly edited IVs are passed
to the existing validated Pokémon patch path.

中文：能力页未学会也可预览，招式页和下拉框随个体值编辑实时更新。训练家随机
个体值不猜测结果，通用 ROM 资料不把占位值“一般／1”当实战属性和威力。
计算结果仅用于显示，不新增存档字段。

## Validation

- `node --experimental-strip-types scripts/test_hidden_power.mjs`: known vectors,
  IV order, missing rules and invalid/unknown input.
- `scripts/verify_hidden_power.py`: executes the **actual UI calculator** and each
  ROM's native battle routine for all 4^6 low-two-bit patterns, with high IV bits
  zero and set. **8,192 comparisons per ROM / 16,384 total**; rotates through all
  four attacker indices. No stubbed arithmetic and no save writes.
- `scripts/test_hidden_power_ui.py`: live edits, dropdowns, ordinary move labels,
  party/box selection, IV-only patch, invalid IVs, bilingual display, generic
  move reference and known/random opponent IVs.
- Rust profile serialization contract test ensures the frontend's formula tag
  matches the actual catalog for both variants.

```sh
GEN3_ROM_BW='/path/BW.gba' GEN3_ROM_DP='/path/DP.gba' \
  python3 scripts/verify_hidden_power.py
# Start Vite first:
python3 scripts/test_hidden_power_ui.py
```

## Unown appearance / 未知图腾外观

This is independent of Hidden Power: it uses the 32-bit PID, not IVs.
Let `b0` be the lowest PID byte and `b3` the highest:

```text
form = ((b0 & 3) + 4*(b1 & 3) + 16*(b2 & 3) + 64*(b3 & 3)) % 28
```

Forms 0–25 are A–Z; 26 is `!`, 27 is `?`. The existing renderer already applies
this formula; see [native picture-loader evidence](pokemon-appearance.md).

中文：取 PID 每个字节的最低两位，按原高低顺序拼成 8 位数，再对 28 取余。
余数 0–25 对应 A–Z，26 对应 !，27 对应 ?。修改个体值不会改变字形；修改 PID
可能改变字形。修改器调整性格、性别或闪光时可能重选 PID，因此也可能联动外观。
