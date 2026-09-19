# Team Rocket battle forms and persistence / 战斗形态与存档边界

Assessment: 2026-09-19. Exact ROM MD5:
`59c658a1081f542086de1060bb65f0b3`. Read-only research, not a supported writer.
All ROM addresses are file offsets; RAM addresses retain their GBA prefixes.

## Conclusion / 结论

The application, transaction system and encryption primitives can be shared with
Dark Phantom. The Pokémon codec and form-transition rules cannot be shared
unchanged. An offset-only profile would corrupt meaningful fields even when the
resulting record checksum remains correct.

中文：无需另做一套应用，但应新增完整的宝可梦字段编解码器及版本规则。不能把
Mega、Z 招式、地区形态、持有物品形态全部做成同一个可写的“形态编号”字段。
当前漆黑的魅影读写路径保持原状；本研究不开放火箭队存档写入。

## Mechanism matrix / 机制结论

| Mechanism | Exact-ROM evidence | Persistence and editor treatment |
|---|---|---|
| Mega | 55 item associations plus one move association; eligibility, transition and restoration code present | Party species can change during battle. Original species and usage flags live in battle RAM. Ordinary saved-party/box editing should retain the storage form and show Mega as a derived preview. |
| Primal-style transformations | Three associations; shared item lookup and separate restoration flag | Treat separately from permanent evolution. The common restoration function uses the form-family base species. |
| Z-Moves | Native eligibility and queue functions work for 38 signature associations and all 18 type crystals | The selected Z move, underlying move and usage state are battle RAM. Eligibility/queue probes leave the complete party record unchanged. Do not offer internal Z attacks as ordinary learned moves. |
| Dynamax/Gigantamax | Related resource names and Eternamax species data exist; no working standard activation path established | The three anti-Dynamax moves' effect has no target-Dynamax branch in the native base-power function. Do not advertise an enabled mechanic or expose invented save fields. |
| Terastallization | No activation path or persistent Tera field identified in the inspected interfaces | Unsupported/unverified capability. A five-bit field could superficially look like a type, but the field examined here is an effective-nature override. |

中文：Mega 和 Z 已验证到执行代码；极巨化有资源残留，但没有建立可用机制的证据，
且关键伤害分支没有实现。太晶化也未确认。这里没有把“尚未找到”写成“绝对不可能”，
也没有把 ROM 中出现道具或高种族值形态当作机制已开放的证明。

## Mega lifecycle / Mega 生命周期

- Item target lookup `0x67614`: the literals at `0x67644/48/4C` are
  `D4 96 5F 08`, `FF FF 00 00`, `FD FF 00 00`. This exact ROM accepts both
  Mega and primal methods through this lookup. Move lookup is `0x67660`.
- Eligibility `0x676BC` checks item 860 (超级环) through `0x10EAE0`, reads
  party species/item/moves through the native getter, checks battle usage and
  pending-Z state, and recognizes held effects `0x8C` and `0x91`. The presence
  of a target association alone does not satisfy all battle conditions.
- The transition command around `0x77492..0x77660` records the original species,
  selects the target, updates the battler species and emits the party species
  update. `0x775C4` calls stat recalculation; `0x775DC` onward records usage.
- Battle-state pointer: RAM `0x020250EC`. The inspected layout contains the
  player evolved-party bitset at `+0x2B1`, original player species at `+0x2C0`,
  and primal-party bitset at `+0x2C2`.
- Restoration `0x678E0` calls species setter `0x97E2C` and stat recalculation
  `0x967B4`. The post-battle loop at `0x53942..0x5395E` calls it for all six
  party slots, followed by other form cleanup. A second caller is `0x4FCA8`.
- Form-family lookup `0x9D19C` uses pointer table `0x617B00`; the table's first
  member supplies the base species for the primal restoration branch.

Controlled tests cover all 59 associations across all 24 PID permutations.
Species returns to the expected source/base, checksums remain valid, and the
rest of the encrypted canonical fields remain unchanged. Party cached stats
are recalculated. A paired negative test with no battle bookkeeping leaves
each special species untouched: the game does not infer restoration history
from the species number alone.

中文：把沙漠蜻蜓的种类直接写成 Mega 沙漠蜻蜓，并不等价于在游戏中正常 Mega。
没有战斗内存里的原种类及标志，恢复函数不会自动修复这种硬写结果。即使校验和
正常，也可能留下游戏规则不一致的记录。正常模式应编辑基础个体和持有物，展示
可用的战斗形态预览；高级模式也不能承诺硬写临时形态能够正常恢复。

## Z-Move lifecycle / Z 招式生命周期

Native `IsZMove` equivalent `0x688B8` recognizes IDs 755–789 (inclusive).
Signature lookup `0x68F58` reads 38 eight-byte associations at `0x5AA704`.
These are species/item/base-move/derived-move tuples, not 38 distinct Z attacks.
Type conversion `0x68F94` skips the reserved type ID 9.

Eligibility `0x68950` checks held-item effect `0x9F`, signature requirements or
move type, used flags, Mega state, and excluded battle modes. This inspected
routine does **not** perform a Z-ring inventory check; do not import that
requirement from another expansion release merely because a Z-ring item exists.
The move-selection wrapper at `0x68B0A` calls it, as do two AI paths.

Queue function `0x688D4` writes the selected Z move to battle state `+0x2E2`,
the underlying move to `+0x2EA`, and category metadata to `+0x2F2` (arrays indexed
by battler). The chosen move is at `+0x2DA`; used flags start at `+0x2DD`.
Native base-power code at `0x63572..0x6359A` switches to the underlying move's
Z-power data while the Z-active bit is set.

All 38 signature associations passed lookup, eligibility and queue probes with
byte-for-byte preservation of the complete synthetic party record. All 18 type
crystals passed a damaging-move eligibility probe. Negative tests cover already
used, prior Mega, wrong item, wrong type and a restricted battle mode.

This does not simulate the full attack, animation, PP-consumption controller,
or every status Z effect. Normal PP consumption can persist; a selected Z attack
is not a fifth learned move or a permanent replacement for the original move.
Battle-use limits should not be presented as editable battery-save counters.

## Negative evidence for Dynamax and Tera / 极巨与太晶的证据边界

The native base-power dispatcher is at `0x6354C`. Its jump table at `0x635CC`
is indexed by effect minus seven. Effect 356 (the anti-Dynamax effect used by
three moves) points directly to `0x08064280`, the common exit, with no target
Dynamax test. All three native executions return their ordinary base power.
This is stronger evidence than a text search, although it is not proof that no
custom event anywhere could implement a separate scripted transformation.

Items 56 and 133 have Dynamax-related names, but share the generic unusable
field handler `0x08136DBD` and have no battle-use handler. Species 1299 has the
Eternamax data and a BST of 1,125. Neither establishes a standard player
Dynamax/Gigantamax system. No standard Max-move bank was identified in the
verified move table. No Tera activation path, selector, or saved Tera type was
identified in the inspected battle and Pokémon accessors.

Future profiles may need optional persistent attributes such as a Dynamax level,
Gigantamax eligibility or Tera type, with their own verified codecs and rules.
Do not reserve or overwrite unknown bytes in this ROM to manufacture them.

## Other persistent and temporary state / 其他战斗外状态

The fixed cleanup table at `0x5AA6A4` has 16 rows. Native cleanup `0x679BC`
distinguishes switching from full cleanup: for example, Aegislash's blade form
reverts on switching, while the listed Mimikyu/Greninja/Meloetta forms do not
use the switching branch. Tests cover all rows, both sides, and both modes.
This table is not a complete catalogue of every possible form transition.

The separate form-rule pointer table is `0x6193AC`, consumed by `0x9D224` and
its party wrapper `0x9D210`. Its method meanings do not exactly match the
reference expansion tag. For example, the inspected method 5 also checks an
ability, and method 6 checks time. Copying upstream enum labels would be wrong.

A concrete persistent-form probe: native held-item setter alone changes
Giratina's item to 413 while leaving species 487 unchanged. A subsequent form
rule query returns species 1143. The editor must perform the appropriate
dependent transition, not only modify the held-item word. Item-use and move-based
forms similarly need explicit rules and their applicable game triggers.

### Effective nature and header packing / 实际性格与头部字段

Native getter field 89 uses five bits spanning canonical byte 9 bits 5–7 and
byte 10 bits 0–1. `0x9A334` selects between this override and PID-derived nature;
26 means use the PID nature. Values 0, 3, 10, 24 and sentinel 26 were executed
with both selector modes. The item-use path at `0x20830E` sets field 89 and
recalculates stats. This is persistent effective-nature state, **not Tera type**.

The same bytes also contain the five-bit ball ID. Further native setter probes
verify experience, PP Ups, friendship, ball, nature and ability-slot masks across
all 24 permutations while retaining every other canonical bit.

The outer header also differs from Dark Phantom:

| Field | Dark Phantom codec | Rocket native accessors |
|---|---|---|
| Language | whole raw byte 18 | raw byte 18, bits 0–2 |
| Bad Egg / has species / egg flags | raw byte 19 | raw byte 18, bits 3 / 4 / 5 |
| OT name | raw bytes 20–26 | raw bytes 19–25 |
| Markings | raw byte 27 | raw byte 26, low four bits |

Evidence: getter dispatch `0x97864`, handlers `0x97A8E..0x97ACE`, and species
setter behavior on the synthetic fixtures. The current `checked_unpack` check
of `raw[19] & 1` would inspect an OT-name byte in Rocket. The crypto transform
can be shared; header validation and mutation belong to the selected codec.

## Shared architecture proposal / 共用框架的边界

This is a proposed adaptation boundary, not code implemented in this change.

| Layer | Reuse | Required isolation |
|---|---|---|
| File/transaction layer | Backups, atomic export, undo, conflict detection, sector assembly | Profile-specific section lengths; validation calls the chosen codec |
| Encryption | PID/OT XOR, 24 permutations, checksum arithmetic | Header flags, known-field ownership, unknown-bit retention |
| Pokémon model | Species, items, moves, IVs/EVs and slot operations | Separate stored nature/effective nature; three ability slots and wider ability IDs |
| ROM catalogue | Search, detail panels, runtime assets | New record formats; separate ordinary moves and derived battle attacks |
| Form model | Shared reference and preview UI | Storage form, battle form, appearance-only variation, derived item/move form, unresolved form |
| Rules | Common validation/reporting interface | Transition triggers, restoration semantics, held-item dependencies, supported mechanics |
| Transfer | Transactional move/swap/copy orchestration | Party-stat rebuilding and form rules through the codec/rules; no cross-ROM raw copying |

Recommended model separation:

- `StoredPokemon`: the persistent record, retaining unknown bytes and provenance.
- `FormDefinition`: stable identity, base species, trigger and verified persistence
  category; unknown forms stay unknown instead of being silently normalized.
- `BattlePreview`: derived species/stats/types/ability and possible Z attacks.
  Generating a preview has no write effect on the stored Pokémon.
- Profile-owned capabilities: verified, absent in inspected paths, or unresolved.
  The UI only exposes supported persistent fields and verified transitions.

The current species reader's two u8 abilities, move reader's u8 power/effect,
PID-only nature, sprite selection rules and all-u32 experience writer are not
valid universal interfaces. The existing import path already carries and checks
the ROM MD5; preserve that gate. Sharing the UI does not imply sharing raw
Pokémon records between the two games.

中文：基础字段和编辑操作仍然共用；“怎么解码、哪些位能写、改物品后要不要变形、
能力值按哪种性格算、哪些形态允许长期存放”由版本适配器负责。正常模式和自由编辑
可以共用这套架构，但自由编辑也不能绕过加密、校验、字段边界和未知位保留。
读取异常形态时应先保留并说明，不在打开存档时擅自改回基础形态。

## Validation and remaining work / 验证与待验证

Run the committed harness with a user-supplied ROM:

```sh
python3 scripts/verify_rocket_battle_forms.py --rom '/path/to/rocket.gba'
```

Dependency: Unicorn. No source save is opened; all test Pokémon are synthetic.
The harness checks the exact MD5, runs unmodified native functions without
eligibility/stat-calculation stubs, and verifies the ROM file remains unchanged.
The 3,960 controlled cases include 1,416 restoration cases, 1,416 missing-state
controls, 64 other-form resets, 38 signature-Z cases, 18 type-Z cases, five Z
negative controls, 850 Z-range checks, three anti-Dynamax move checks, one
held-item dependency check, five effective-nature checks and 144 field-mask checks.

These are native-function tests, not 3,960 complete gameplay battles. Before
enabling save writes, additionally verify disposable in-game saves across battle
entry/exit, fainting, switching, capture, deposit/withdrawal, item removal and
evolution. Full PP consumption, HP retention and every custom story form require
their own coverage. Battery `.sav` files and emulator save states are distinct:
the latter can include the very battle RAM needed to resume a transformation.
This editor's battery-save support must not imply save-state support.

Reference source used only to locate candidate routines:
[expansion 1.0 battle utilities](https://github.com/rh-hideout/pokeemerald-expansion/blob/expansion/1.0.0/src/battle_util.c),
[Z-Move code](https://github.com/rh-hideout/pokeemerald-expansion/blob/expansion/1.0.0/src/battle_z_move.c),
[Pokémon code](https://github.com/rh-hideout/pokeemerald-expansion/blob/expansion/1.0.0/src/pokemon.c).
The binary differs from those sources; findings above follow the native bytes.
