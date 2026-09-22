# Cheat verification / 金手指验证

Verified 2026-09-22. No private saves, ROMs or extracted assets are included here.

## Ultimate Emerald 5.5 / 究极绿宝石

Exact ROM MD5: `17ce9785b33319b3dbda9a5d37c57ec1`, 33,554,432 bytes.

| File offset | CPU address | Original | Replacement |
| --- | --- | --- | --- |
| `01D492CE` | `09D492CE` | `D915` | `E015` |
| `01D4EC46` | `09D4EC46` | `D820` | `E020` |

Only two bytes differ (`01D492CF`, `01D4EC47`). Both branches select the existing
clear path for bit 7 at `*(0203F014) + 100`; the other seven bits are preserved.
This is the composite input-peeking mode, including associated proactive switch
behavior. The two original random calls remain. This does not prove absence of
all information leakage elsewhere in the ROM or identical later RNG outcomes.

GameShark Advance V1/V2 (both lines; no master code in tested engines):

```text
270AABF9 EF4D3B91
05EFAF30 6F13BEC4
```

The raw operations are `64EA4967 0000E015` and `64EA7623 0000E020`. ROM halfword
encoding keeps the physical `08000000` window; omitting that address bit happens
to be masked by some decoders but breaks VBA-M's read/restore behavior.
The Rust implementation uses wrapping TEA arithmetic and has fixed-vector tests.

Evidence collected in the preceding investigation:

- mGBA 0.10.5, upstream commit `26b7884bc25a5933960f3cdcd98bac1ae14d42e2`:
  complete 32 MiB in-memory ROM diff, exactly two changed bytes; 24 toggle/reset
  checks; enabled/disabled state loading and fresh-core loading; source ROM unchanged.
- 38,912 native routine cases covering difficulty/facility/random inputs, preserving
  the lower seven mode bits. These are routine probes, not 38,912 full battles.
- Full-core controlled fixtures: Challenge singles 125 turn updates, Lunatic
  singles 130, Lunatic doubles 56; all 311 with the peeking bit clear. Untreated
  Lunatic control: 63 peeking updates out of 129. Endurance fixtures replenished
  HP/PP and increased defense; the doubles fixture fixed moves to Amnesia.
- Separate move-choice fixture, without forced AI choice: Lv.16 Snorlax versus
  Lv.20 Alolan Graveler with deliberately selected test moves. Identical input:
  first-turn Amnesia / critical Rock Slide reduced Snorlax from 82 to 33 HP.
  On turn two, Snorlax's first Protect met Curse in the untreated run; the patched
  run selected Thunder Punch, blocked by Protect. Both conditions repeated twice
  consistently. This is an illustrative controlled example, not a general claim
  that Protect always makes a patched opponent attack.
- VBA-M **2.2.3** (actual app version): GUI import and enable/disable memory readback.
  Its manual-input detection requires each line as 16 characters without a space.
  A dropdown selection alone does not guarantee its parser chose V1/V2.

Integration verification repeated using the **new CLI's generated output** and an
independent mGBA decoder in `scripts/verify_cheats_mgba.py`: both expected patch
addresses, complete ROM diff and 24 toggle/reset checks passed. This keeps protocol
verification independent of a decode function written alongside our encoder.

Not covered: Manic EMU, Delta, entire-story play, all facility tours or extended
mobile sessions. Facilities have native branch coverage only. An already chosen
turn's flag can survive loading an old mid-battle state; use an in-game save and
start a new battle after enabling. Other difficulty bonuses remain unchanged.

## Historical Sudowoodo code / 旧胡说树代码复核

The rejected historical pair was `0146DCEA 3E32A31D` plus `83007E28 00B9`.
**Original source/version applicability is unknown. These are not released recipes.**
The historical phone/emulator input cannot be reconstructed from these strings.

复核对象是 BW MD5 `0d9b129f7dd76895f79bb47ad7dec2fe`。本轮明确验证了：

1. 第一行按 GameShark V1/V2 解密为 `64033DC3 00009940`，将 CPU 地址
   `08067B86` 的半字改成 `9940`。它不是一个抽象的“遇宠开关”。
2. 第二行按 CodeBreaker 解释：每次执行向 `03007E28` 写入 16 位 `00B9`。
   两者是不同协议，不能未经核对就粘入同一种设备格式。
3. 直接读取当前 BW ROM 的物种 185，其名确实为“胡说树”，**物种编号没有错**。
4. 当前 BW 在该补丁地址的原始指令为 `1C31`，即 `adds r1, r6, #0`。
   替换的 `9940` 是 **`ldr r1, [sp, #0x100]`**，从当前调用栈加载 32 位值，
   再传给后面的宝可梦构造例程。它不是直接读取 `03007E28` 的绝对寻址指令。
5. `08067B60` 的改版跳板会经过 `09B037BC`，处理等级后返回原路径，不能仅因
   看到跳板就认为补丁地址完全失效。普通相遇生成器在 `080B5012` 取表中物种，
   `080B5016` 调用野生构造器 `080B4E68`；不同构造调用和栈深度仍需现场验证。

Independent mGBA decode + actual Thumb instruction execution:

| Controlled case / 受控条件 | Result in species argument r1 |
| --- | ---: |
| CodeBreaker alone; original instruction has r6 = 25 | 25 |
| Both; SP = `03007D28`, so SP + 100 = `03007E28` | 185 |
| Both; SP = `03007D20`, with a different value at SP + 100 | 123 |
| GameShark alone; stack location holds 77 | 77 |
| Disable GameShark patch | Original `1C31` restored |

探针还确认 CodeBreaker 的实际写宽度为 2 字节、地址和值正确，两份源 ROM 文件
未变。公开探针可复跑，但**它没有把普通走路／甜甜香气的完整相遇现场重演一遍**。

因此，目前证据否定了“物种编号错了”，并证明“混合协议 + 依赖特定栈位置”是
真实限制；在特定栈位置两段可以共同生效，**不能断言只因 ROM 版本不同而失败**。
当时模拟器格式、代码是否执行、真实相遇入口和 SP 未复原，失败根因仍未定论。
在完成这些验证之前不把旧码加入可复制目录。此结论与天然鸟坏蛋原因无关。

## Reproduction boundaries

Public unit tests cover exact identity, cross-ROM isolation, unknown formats and
parameters, encoder vectors, and requests without a loaded ROM. The opt-in local
matrix opens all four exact ROMs, verifies immutable editor context and files,
rejects a modified Ultimate Emerald image, and denies cross-ROM recipe requests.
UI fixtures cover both languages, complete copy/export, import failure cleanup,
late responses after closure, search and narrow-window behavior.

Simulation logs/screenshots stay private under the local analysis directory. This
file describes test scope without shipping the games, user history or save data.
