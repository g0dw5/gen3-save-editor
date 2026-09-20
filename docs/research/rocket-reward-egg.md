# Apollo's reward egg / 阿波罗奖励蛋

2026-09-20. Exact supported Rocket ROM only; no private save or generated record
is included in the repository.

## Script and delivery

Map `34-24` contains the Apollo report/reward scene beginning at `0x3C8F18`.
The script awards money, displays the egg message, then executes
`giveegg 228` at `0x3C9073`. Species 228 is the ROM's Houndour (戴鲁比), evolving
at level 24 to Houndoom (黑鲁加). The scene ends by setting variable `0x40F7` to 6
at `0x3C90CB`.

Command table `0x22B218`, entry `0x7A`, points to handler `0xD12E4`. It calls the
native gift-egg wrapper `0x1313FC`, which calls `CreateEgg` at `0x9E93C` and then
`GiveMonToPlayer` at `0x98604`.

- A free party slot receives the egg and delivery returns 0.
- A full party calls the PC-storage routine at `0x98680`. It scans boxes and
  inserts into a free slot, returning 1. This was executed using an isolated RAM
  copy of a six-member party and a PC with available slots.
- Only if both party and all PC slots are full does the routine return 2.
  The Apollo scene does not branch on this result, so that condition can lose
  the reward. A full party alone does **not** establish that the egg was lost.

## Reproducing a real gift egg

For a specifically authorized restoration, execute the ROM's own wrapper using
an isolated copy of the recipient's save blocks, party and PC. Do not substitute
an ordinary created Pokémon with only its egg flag toggled: the native routine
also sets the egg nickname/language, incubation counter, level-5 starting
experience/moves, gift origin, ball and recipient identity.

Runtime pointers in this ROM:

- `0x0300524C`: SaveBlock1; `0x03005250`: SaveBlock2.
- `0x03005254`: PC storage; `0x02025170`: party.
- `0x03005240`: RNG state, advanced by `0x9D40C`.
- Variables: SaveBlock1 + `0x1F6C + 2*(id - 0x4000)`, verified by `0xD397C`.

Selecting a normal generated PID with the requested nature preserves native
record construction. Apply explicitly requested IV optimization through the
native field setter, retaining egg state and other bytes. For the requested
special-attacking Houndoom, Timid and IVs 31/0/31/31/31/31 (HP, Attack, Defense,
Speed, Sp. Attack, Sp. Defense) favor Speed and minimize unnecessary physical
Attack. This is deliberate optimization, not a claim about the original lost
individual's random stats. The ROM's Houndoom has Sheer Force; do not assume the
unmodified official game's ability or stat data.

Verify the native getters as well as the editor checksum: egg flag = 1, actual
species = 228, display species = 1395 (the egg), native effective nature = 10.
Import the resulting 80-byte record into a confirmed empty slot through the
transactional save writer. Compare all prior records and logical save fields,
verify changed byte ranges, retain an original backup and re-open the final save.
Do not modify storyline variables merely to make the restoration look claimed.

中文：阿波罗发的是戴鲁比蛋。满队会自动送入电脑，只有连电脑也满了才会失败。
原生赠蛋函数生成的记录包括真实的孵化周期、初始招式、语言及来源字段；补蛋须
在独立内存中生成并验证，不能仅给普通宝可梦勾“蛋”。个体值优化属于用户明确
要求的修改，不应声称还原了遗失个体本身。剧情标记应保持原样。
