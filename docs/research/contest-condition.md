# Contest condition and fullness / 华丽值与饱腹度

The editor's former **Sheen / 光泽** field is the same stored value commonly
called fullness. It was already the sixth byte of `Pokemon.condition`; this
change exposes it clearly as **Fullness (Sheen) / 饱腹度（光泽）** in the Stats tab.
It does not add a new field to a Pokémon's binary record.

原先“高级”里的“光泽”就是饱腹度，现与五项华丽值一起移至“能力”。五项分别是
帅气、美丽、可爱、聪明、强壮，均为 0–255；饱腹度也是独立的 0–255 字节。
五项没有类似努力值 510 的总和限制，也不能以华丽值总和推导饱腹度。

## Exact-ROM evidence / 精确版本证据

All addresses below are file offsets, unless identified as RAM. The exact
fingerprints remain in `profile.rs`; ROMs and saves are private inputs.

| Data / native routine | Dark Phantom BW and DP | Rocket 2.1 Chinese |
| --- | --- | --- |
| Add Pokéblock to conditions | `0x167054` | `0x1b31dc` |
| Calculate flavor effects | `0x167184` | `0x1b330c` |
| Nature/flavor preferences, 25 × 5 signed bytes | `0x5b25a0` | `0xc7c598` |
| Ordinary berry records, 43 × 28 bytes | `0x58a670`, flavors/feel at +21 | NPC model not enabled |
| NPC berry selector | `0x80674` | NPC model not enabled |
| NPC berry sets / Blender Master choices | `0x339ca0` / `0x339cbe` | NPC model not enabled |
| Blender output calculation | `0x81be0` | NPC model not enabled |
| Update speed from input | `0x81370` | NPC model not enabled |

Observed rules:

- Refuse another block if the existing fullness equals 255.
- Otherwise apply the block's flavor gains, independently saturating every
  condition at 255; add feel to fullness, saturating at 255 afterward.
- The final block takes full effect even when its feel exceeds the remaining
  fullness. The game does not scale that block down to the remaining capacity.
- Nature effects use the ROM flavor preference table and the sign of the block's
  aggregate gain. Only a flavor with the same preference sign receives the
  rounded 10% adjustment. If aggregate gain is zero, neither side is adjusted.

中文：满饱腹度不禁止编辑器直接改数值，只禁止游戏继续喂食。校验时必须允许最后
一块跨过饱腹度上限。性格与口味的关系读 ROM，并非简单给每块同时乘 1.1 和 0.9。

## Validation contract / 校验范围

Byte ranges are always enforced in UI and deserialization, including free
editing. Party and box records share the same six canonical bytes (30–35).

BW/DP additionally offer an explicitly scoped **ordinary solo NPC feeding
bound**. It assumes all five conditions began at zero; includes all ordinary
NPC groups and the Blender Master; and excludes link blending, imported
individuals, and event-replaced Enigma Berry data. This is optional because
those excluded origins cannot be inferred from the six saved bytes.

The core reads berry flavors, feel, NPC choices and flavor preferences from the
loaded ROM. It creates optimistic upper envelopes for those recipes, including
all black-block outcomes. The bound grants every recipe 180.00 RPM, always
boosts liked flavors and never penalizes disliked flavors. These are deliberately
more generous than play; the result must not be shown as a usable recipe.

For positive integer weights `w`, let `R` be the greatest weighted condition
gain per feel of any envelope block, and `L` the greatest weighted gain of one
block. From zero, any ordinary feeding history must satisfy:

```text
fullness < 255:   weighted conditions <= fullness × R
fullness = 255:   weighted conditions <= 254 × R + L
```

The second inequality preserves the final-block exception. The implementation
uses rational integer comparisons, testing small weight vectors. A separating
vector proves the proposed values exceed this scope's bounds. Failure to find
one is **not** a certificate of attainability or an exact minimum. The UI says
“not ruled out”, and does not silently increase fullness or normalize other
fields. Core enforcement repeats on apply when `contest_scope: "npc"` is set;
free editing retains a finding. Unrelated edits never revalidate an assumed
feeding history.

中文：支持“明确排除”，不冒充完整合法性鉴定。校验通过只表示必要条件尚未排除，
不能据此声称有可达配方。用户可关闭不符合实际来源的单机范围，或开启自由编辑；
两种操作都不解除字节范围与校验和约束。西班牙火箭队已验证六项字段和原生喂食
规则，但尚未核对它的 NPC 配方，所以只显示字段范围与喂食状态，不套用漆黑配方。

## Speed ceiling used by the bound / 转速上界

BW/DP's input routine increases speed by `384 / players` below speed 1500 and
`128 / players` above it for Best, or `256 / players` below 1500 for Good. The
progress updates charge `speed / 55` and `speed / 70` respectively. Progress
ends at 1000. Starting speed is 128; normal slowdown/misses cannot increase it.

A shortest-path relaxation allows arbitrary perfect timing and free slowdown,
then grants up to a full frame of additional player inputs after reaching the
progress threshold. Its two-, three- and four-player RPM ceilings are 170.23,
144.74 and 131.67. Thus the 180.00 ceiling used for the ordinary NPC upper
envelope is conservative. This is a proof bound, not an attainable speed record.

For the narrower two-player pure-Beauty calculation, a second relaxation keeps
the engine's one-speed-unit slowdown every six frames. Positive hits cannot
occupy consecutive frames: the player's button is edge-triggered, the NPC
latches a visit, and their opposite hit windows cannot overlap across the NPC's
one-frame delay at these speeds. Even with all other timing restrictions removed,
the ceiling is 166.33 RPM, below the 166.50 needed to turn a 19-flavor base into
a 29-point block.

## User-requested Beauty-only result / 本次只提升美丽值

For PID nature 9, dry flavor is neutral. Native NPC selection and blending at
140.00 RPM give the following ordinary two-player recipes (names shown here
describe the researched BW input, and are not embedded in the application):

| Player berry item ID | NPC berry item ID | Beauty gain | Feel |
| --- | --- | --- | --- |
| 144 | 134 | 27 | 23 |
| 140 | 134 | 13 | 20 |

Nine of the first block plus one of the second produce 256 raw Beauty gain,
capped at 255, and `9 × 23 + 20 = 227` fullness. All four other conditions remain
zero. The native blending and feeding routines reproduce this sequence.
No PID, nature, IV, move or other individual data needs to change.

For a lower bound, enumerate all pure-dry NPC recipes. The best can supply at
most 28 Beauty for 23 feel; generously grant all other pure-dry recipes 14
Beauty for only 20 feel. Even this optimistic integer combination cannot reach
255 below 227 feel. The cap cannot help a result whose final fullness is below
255. Therefore 227 is the minimum within the stated ordinary solo scope while
keeping the other four conditions at zero. Recipes requiring additional flavors
cannot be substituted because this engine's feeding does not reduce them later.

中文：本次最终目标是“仅美丽 255，另外四项不喂”，不是最初的五项全满。按现有
性格和上述单机范围，设置为 `[0,255,0,0,0,227]`。这是华丽值和饱腹度的等效编辑，
没有扣除树果、模拟混合小游戏耗时或生成宝可方块物品。计算中包含混合名人作为
可选配方，并不承诺其当前已在地图出现；实际选中的两种配方不需要混合名人。

For comparison, all five values at 255 are disproved even at fullness 255 by
the conservative NPC envelope for this nature. There is no ordinary solo
minimum for that original target. This conclusion is not generalized to link
mixing, event berries, other starting conditions or other hacks.

## Verification / 验证

- `scripts/verify_contest.py`: 225 native feeding vectors per ROM (675 total),
  all 25 PID natures, zero/full/near-full states and unowned-byte preservation.
- BW and DP separately: 172 native NPC selections and 860 native blends per
  ROM; normal/Black blocks, rounding and feel, speed transition probes, relaxed
  speed bounds, and the complete Beauty-only feeding sequence.
- Rust public fixtures cover saturation, gain direction, all 24 encryption
  permutations across the three profiles, field independence and malformed
  numeric input. The ignored exact-ROM test compares the native vectors and
  covers NPC rejection, free editing, unrelated edits and runtime table changes.
- Browser fixtures cover both languages, location in Stats, numeric limits,
  fullness status, scoped patch submission and Rocket's independent capability.
- The supplied save was edited through the transactional core and independently
  decoded before/after; only two condition bytes and checksum bytes changed.
  Private save hashes and backups remain outside the repository.

These are native routine probes and editor tests, not a claim that the full
blending minigame or an edited save was played through in every emulator.

```sh
export GEN3_ROM_BW='/private/BW.gba'
export GEN3_ROM_DP='/private/DP.gba'
export GEN3_ROM_ROCKET='/private/Rocket.gba'
export GEN3_CONTEST_PROBES='/private/contest-probes.json'
python3 scripts/verify_contest.py
cargo test -p gen3-core contest_matches_native_feeding_and_npc_bounds -- --ignored
python3 scripts/test_contest_ui.py  # requires Vite and Playwright/Chrome
gen3 contest-check "$GEN3_ROM_BW" 9 /private/condition.json
```

Primary structural references:
[use_pokeblock.c](https://github.com/pret/pokeemerald/blob/master/src/use_pokeblock.c),
[berry_blender.c](https://github.com/pret/pokeemerald/blob/master/src/berry_blender.c).
The supplied ROM routines, not those upstream files alone, establish this adapter.
