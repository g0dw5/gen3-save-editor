# Cheat verification / 金手指验证

Verified 2026-09-22. No private saves, ROMs or extracted assets are included here.

## Common recipes / 常用功能

Exact MD5s and supported features are listed in [the user guide](../cheats.md).
BW and DP share these instruction locations but were executed independently;
Rocket uses its own bindings. Offsets below are **file offsets**, not RAM writes.

| Feature | BW / DP offset and change | Rocket offset and change |
| --- | --- | --- |
| Pause walking encounters | `000B52A2: D100 → 46C0` | `000EC7EC: D100 → 46C0` |
| Guaranteed wild capture | `00056566: D92F → 46C0` | `00080BB4: D948 → 46C0` |
| Faster party hatching | `00070B46: D13B → 46C0` | `0009EB36: D13B → 46C0` |
| Compatible daycare eggs | Not offered / 不开放 | `0009EB1C: 4284 → 2C00` |

`46C0` is the Thumb `mov r8,r8` no-op. The first three remove a conditional branch:
walking takes the existing disabled return; capture takes the existing success
path; hatching processes egg cycles at every eligible call rather than only the
counter boundary. Rocket's fourth patch compares adjusted compatibility with zero
instead of the random roll. The preceding RNG call and eligibility guards remain.

前三项分别选择已有的停止走路遇敌、捕捉成功、处理孵化周期分支，不覆盖背包或
宝可梦数据块。第四项仅修改西班牙火箭队的生蛋概率比较，保留步数、父母数量、
待领取蛋检查和原生遗传流程。孵化／捕捉引起的数据更新仍由游戏自身完成。

Reproduction: `scripts/verify_common_cheats_mgba.py`, against mGBA **0.10.5**
(commit `26b7884bc25a5933960f3cdcd98bac1ae14d42e2`) and the new CLI output.
The native harness loads no save and exposes no file-writing API. All branch
tests use controlled RAM; native functions and their callees run without stubs.

- **All ten ROM/recipe bindings:** mGBA independently decodes final generated
  GameShark lines. Compare all 32 MiB of in-memory ROM against exactly the expected
  halfword replacement, then test 24 enable/disable/reset cycles per binding and
  complete restoration. Also compare all supported recipes enabled together and
  restored together. Source ROM bytes remain unchanged.
- **Hatching, per ROM:** 360 cases = 24 PID permutations × cycles 0/1/2/5/255 ×
  valid egg/ordinary Pokémon/Bad Egg. Check unpatched behavior, patched behavior,
  disabling without rolling back progress, complete encrypted records and checksum.
  Three additional boundary tests keep the ROM's ability-based two-cycle bonus.
  Ability carriers are found by invoking the ROM's native bonus routine, not a
  bundled species list. Boxed data is outside this party-only function.
- **Capture, per ROM:** 90 full native ball-command calls = three actual ball IDs
  × wild/trainer/tutorial flags × five RNG seeds × patched/unpatched. Patched wild
  calls select the native success script; trainer/tutorial results remain equal.
  Check actual ball ID and encrypted record changes (only ball bits and checksum).
  The three ball IDs differ between Rocket and BW/DP. This does not exercise every
  special battle facility or story-specific restriction.
  Separate full-core UI smoke tests on disposable BW and Rocket save copies entered
  a constructed Lv.5 Mewtwo battle, selected an ordinary Poké Ball from the bag and
  reached the successful catch/dex-registration flow at full target HP. This was a
  deliberately constructed battle, not a natural encounter; source saves were not edited.
- **Walking, per ROM:** native `StandardWildEncounter` on a ROM-selected land table
  and 32 seeds: original 5 encounters, patched 0. Synthetic party/map state, not a
  claim that every map was walked. Fishing, Sweet Scent and scripted battles use
  separate entry points and are not patched.
- **Rocket breeding:** 640 native-entry calls cover 32 seeds × charm/no charm ×
  compatible/incompatible/empty parent/pending egg/non-checkpoint × on/off. Compatible
  fixtures produced 17/32 without charm and 26/32 with charm; patched 32/32 in both.
  Incompatible/empty/non-checkpoint fixtures stayed at zero. Existing pending PID
  and both parents' encrypted records remained unchanged.

验证使用完整原生函数，包含受控内存夹具，不等于长期游戏实测。手机模拟器和新增
条目的 VBA-M 兼容性尚未逐项验证；界面中明确区分这一范围。关闭后已捕获的宝可梦、
已减少的孵化周期或待领取蛋不会回退。

### Rejected shared daycare patch / 未共用的产蛋补丁

BW/DP's compatibility path hooks into file offset `00310EA0`: with item 56 present,
50 becomes 80, 70 becomes 88, and **other values gain 20, including zero**.
Rocket's helper at `0009F338` checks item 846 and maps only 20→40, 50→80, 70→88;
zero remains zero. These are ROM-specific item IDs, not shared inventory IDs.
Forcing the later comparison to `compatibility > 0` would therefore lose the
intended incompatible-parent guarantee in BW/DP. That recipe is not bound there.

漆黑的魅影护符钩子和西班牙火箭队存在实际控制流差异。不能仅凭两边函数形似就
共用“兼容组合必定产蛋”代码；本批在漆黑 BW／DP 中不展示这项功能。

Harness detail: mGBA's `mCheatDeviceClear` frees sets without restoring active ROM
patches. The runner explicitly disables/refreshes every set before clearing, and
asserts original halfwords at the start of **every** fixture. This prevents an
earlier enabled test from contaminating its supposedly unpatched control.

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
