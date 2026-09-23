# Cheat codes / 金手指

The window is a read-only, locally generated catalog of project-verified recipes,
not a cheat table extracted from the ROM. ROM import checks the entire MD5 and
size. No emulator connection, save or ROM writes are performed.

金手指窗口是项目逐项验证后的代码目录，不是声称从 ROM 内找到一张金手指表。
必须提供匹配完整 MD5 与大小的 ROM，不要求存档，不修改文件或替模拟器开启代码。

## Current coverage / 当前范围

| ROM | MD5 | Recipes / 条目 |
| --- | --- | --- |
| Ultimate Emerald / 究极绿宝石 5.5 | `17ce9785b33319b3dbda9a5d37c57ec1` | Portable PC, walking suppression, capture, hatching, species/level, shiny, teleport, all-mode no peeking / 随身电脑、暂停走路遇敌、必捕、孵蛋、指定遇怪、闪光、传送、全模式去窥屏 |
| Dark Phantom / 漆黑的魅影 5.0EX+BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | Portable PC; pause walking encounters; guaranteed wild capture; faster hatching; species/level; shiny; map teleport / 随身电脑、暂停走路遇敌、必定捕获、加快孵蛋、指定遇怪、闪光、地图传送 |
| Dark Phantom / 漆黑的魅影 5.0EX+DP | `cb2940215f4dafb1bef133c3af379f44` | Same seven features, independently tested / 同上七项，独立验证 |
| Team Rocket / 西班牙火箭队 2.1 中文 | `59c658a1081f542086de1060bb65f0b3` | Those seven plus compatible daycare eggs / 上述七项及兼容寄养组合必定产蛋 |

Ultimate Emerald is supported in both the editor and cheat window. Opening another
ROM in this window does not replace the editor session. With an editor ROM already
open, the cheat window initially displays that ROM's availability. Changing the
editor ROM resets the displayed cheat context. Reopening the window starts from
the editor ROM (or the no-ROM prompt), not a hidden last-selected cheat ROM.

究极绿宝石可在主编辑器打开，也可单独在金手指窗口选择。
窗口初始展示编辑器 ROM 的支持状态，允许另外选择仅用于金手指的 ROM。编辑器
切换 ROM 后，金手指展示同步重置；关闭再打开窗口不隐式恢复另选的 ROM。

## Usage / 使用

1. Open **Cheats** from the toolbar or ROM reference; choose the matching ROM.
2. Search/select a recipe and read its scope, protocol and enable/disable steps.
3. Copy the complete set or export a text guide. Enable every line shown for the selected recipe.
4. Apply the set in your emulator, not in this editor.

1. 顶栏或 ROM 资料中打开“金手指”，选择对应 ROM。
2. 搜索／选择功能，阅读影响范围、代码格式与启停步骤。
3. 复制整组或导出代码与说明；当前去窥屏条目必须同时启用两条。
4. 到模拟器中添加；修改器不会自动连接模拟器。

All current recipes use **GameShark Advance V1/V2**. The default 8+8 layout and the
VBA-M 16-character layout encode the **same** protocol; removing spaces is not
conversion to CodeBreaker or Action Replay V3. VBA-M 2.2.3's input detection needs
the compact layout. Back up and save outside battle, enable the full set, restart,
and load the in-game save. Disable the full set and restart to stop. The new common
fixed recipes have one line each; the Ultimate Emerald recipe requires two together.
Portable PC uses seven lines and teleport three. Ultimate species/level uses eight
lines and shiny 28; other ROMs use four and 86 respectively. Copy the entire set.
For teleport, follow the recipe-specific live enable/disable steps below.

当前条目为 **GameShark Advance V1/V2**。8+8 与 VBA-M 不带空格的 16 位形式只是
同一协议的排版，不能当作转换协议。战斗外正常保存并备份后启用整组，重启并从
游戏内存档继续；停用整组后重启。固定常用功能各一条，究极绿宝石去窥屏必须两条一起。
随身电脑 7 行、传送 3 行；究绿指定遇怪 8 行、闪光 28 行，其他版本分别为 4 行和 86 行。
必须完整复制；传送按下方专门的即时启停步骤操作。

### Portable Pokémon PC / 随身电脑

Enable all seven lines as one GameShark Advance V1/V2 set. While freely walking,
press **SELECT** to open the ROM's original Pokémon PC menu. It offers deposit,
withdrawal, organization and held-item management. Exit normally with B or the
exit option; disable the complete set only after leaving the PC, then restart.
SELECT returns to the previously registered item, which is never cleared.

将 7 行作为同一组 GameShark Advance V1/V2 启用。在可以自由行走时按 **SELECT**，
原地打开本作的宝可梦电脑菜单，可存取、整理宝可梦及整理携带道具。用 B 或退出选项
正常退出电脑，再停用整组并重启；SELECT 恢复原登记道具，登记内容不会清空。

Native overworld input and Union Room / certain battle-facility restrictions
remain. This does not force-open a PC during battle, dialogue or other menus.
Access away from a PC is a gameplay convenience change; transfers and saving
still use native routines. Disabling does not undo saved transfers. Exact ROM
fingerprints are required; mobile emulators and every story location are untested.

保留原生地图输入处理以及联机房、部分对战设施的限制，不强行从战斗、对话或其他
菜单打开电脑。这会改变远离电脑时可换队的规则；实际存取和保存仍由游戏原生流程
处理，停用不会撤销已经保存的整理结果。仅支持所列完整 ROM 指纹，尚未逐剧情地点
或手机模拟器测试。

### Common recipes / 常用功能

- **Pause walking encounters:** use the native walking encounter routine's disabled
  path. Grass, cave and surfing checks through that routine stop; fishing, Sweet
  Scent, scripted encounters and trainers are not disabled. This is not walk-through-walls.
- **Guaranteed wild capture:** remove the random failure branch from the native
  ball command. Preserve the actual ball, normal consumption, capture records,
  trainer blocking and tutorial branches. Does not unlock bags or scripted restrictions.
- **Faster party hatching:** decrement egg cycles on every eligible step check;
  retain native ability bonuses, Bad Egg checks, native record handling and hatch animation.
  Boxed eggs do not change. A zero-cycle egg hatches at the next eligible check.
- **Compatible daycare eggs (Rocket only):** at the normal step checkpoint,
  positive compatibility always succeeds. Two parents and no pending egg are still
  required; inheritance is unchanged. Dark Phantom's Oval Charm hook has different
  semantics, so the same comparison patch is intentionally not offered there.

暂停走路遇敌不影响钓鱼、甜甜香气、脚本定点或训练家，也不提供穿墙。必定捕获
仍需正常投球，消耗并记录实际使用的球；不解锁剧情捕捉限制。快速孵蛋仍使用原生
特性加速、坏蛋检查、原生个体处理和孵化动画，只加速同行蛋的周期扣减。西班牙火箭队
必定产蛋仍需两只兼容父母、无待领取蛋，并走到游戏原有检查点；不改变遗传。

Stopping a code restores instructions, **not completed gameplay changes**: captures,
hatching progress and pending eggs can persist after normal saving. These codes do
not give maximum IVs or inject inventory slots.

关闭代码恢复的是指令，不会撤销已经捕获的宝可梦、已减少的孵化周期和已产生的蛋。
不提供强行满个体或直接覆盖背包槽位的代码。

### Species, shiny and teleport / 指定遇怪、闪光与传送

Select a ROM Pokémon (searchable by native name/ID) and level 1–100. The
species/level recipe changes the ordinary wild
constructor's inputs; IVs, moves, encryption and subsequent encounter handling
remain native. It does not start a battle or replace separate static/gift/egg/
trainer/roamer constructors. Battle-only species are excluded. Disable the old
set before choosing another target. Pause-walking-encounter codes can prevent
walking encounters from starting even when the species recipe is enabled.

指定遇怪读取当前 ROM 的名称和编号，可搜索并选择 1–100 级。后续个体值、招式、
加密仍由原生流程生成；不主动发动战斗，不替换独立的定点、礼物、蛋、训练家或
游走生成器，排除临时战斗形态。换目标前关闭旧组，走路遇敌暂停功能也要先停用。

The shiny recipe may be combined with species/level. It constrains the newly
created PID only in the ordinary wild call chain, leaving native nature and
gender acceptance loops (including Synchronize and Cute Charm) intact. Other
callers retain their original random-PID construction. No existing Pokémon's
PID or Pokémon payloads are edited. Mobile emulator support for the longer
ROM hooks is untested; the complete sets were imported in mGBA.

闪光与指定遇怪可以叠加，仅约束新生成的普通野生 PID，保留原生性格／性别筛选、
同步和迷人之躯。不会改已有宝可梦的 PID 或记录。整组已在 mGBA 导入与启停
验证，手机模拟器对长代码组的支持尚未实测，不能只启用前几行。

Teleport uses **Region → Map**, then offers referenced entrance landing tiles.
Labels and codes are read from the ROM at runtime. `GG NN` displays hexadecimal
map group and map number separately; decimal `group-number` remains visible.
This is not a packed integer or a met-location code. Maps without an eligible
incoming warp reference or in-bounds landing remain visible but disabled.
A static entrance reference is not proof of story reachability, a usable exit,
or complete gameplay testing of that destination.

传送使用“区域 → 具体地图”二级选择，可再选入口格位。地图名称及编码直接从 ROM
读取；`GG NN` 分别是十六进制地图组、地图号，旁边保留十进制 `组-图`，不是相遇
地点编号，也不是把两项拼成整数。没有入口引用或有效格位的地图仅显示编码，不
生成传送代码。入口引用不证明当前剧情可达、出口可用或已逐图实机验证。

Back up, stand outside an ordinary door, enable the complete teleport set, then
enter. Disable **immediately after arrival**, before using another exit. Verify
movement and a normal exit before saving; arrival scripts may run story events.
Continuous road connections and some dynamic return warps bypass the patched
setter. If the emulator cannot restore ROM instructions live, retain the original
save and restart to verify instead of relying on a still-active warp override.

先备份，在普通门外启用整组，进入门后切换到目的地。**抵达后立即停用**，再测试
行走和正常出入，确认后保存。进图脚本仍会执行，可能触发剧情；连续道路连接与
部分动态返回入口不经过这个设置函数。如果模拟器不能即时恢复指令，保留原存档
并重启验证，不要带着仍生效的传送替换继续过门。

CLI: `gen3 cheat-code ROM CHEAT_ID gameshark_v1_v2 [PARAMETERS.json]`.
Examples of the parameter file (IDs must exist in that exact ROM's catalog):

```json
{"kind":"encounter","species":185,"level":17}
```

```json
{"kind":"teleport","map_id":"0-3","warp_id":0}
```

参数文件必须对应同一个 ROM 的可选条目，后端会拒绝空值、越界等级、无效地图、
入口、临时形态和多余字段；无需参数的条目不接受附带参数。

### Ultimate Emerald / 究极绿宝石

This selects the native non-peeking branch, including the combined flag's related
switching behavior. Existing non-peeking prediction/scoring/switching remain.
Difficulty bonuses (EVs, accuracy, 1 HP survival, PP) are unchanged. The native
RNG calls remain, without a promise of identical later random outcomes.
Old mid-battle states can restore a flag already chosen for that turn.

这走游戏自己的不窥屏分支，也关闭组合开关附带的换人行为；保留不窥屏分支原有
的预测／评分／换人，不改超模努力值、命中、1 HP 与 PP 等难度规则。原位置随机
数调用保留，但后续随机结果未必相同。旧战斗中即时存档可能恢复已选好的本轮标记。

mGBA 0.10.5 has decoder/full-core test coverage. For the Ultimate Emerald recipe,
VBA-M 2.2.3 has GUI import and memory readback coverage; the new common recipes have
not each been tested there. Mobile emulators are **untested**. Facility coverage is
native branch testing, not full facility tours. See [evidence](research/cheat-verification.md).

## Developer interface / 开发接口

```sh
cargo run -p gen3-cli --bin gen3 -- cheats /path/to/ROM.gba
cargo run -p gen3-cli --bin gen3 -- cheat-code /path/to/ROM.gba disable-input-peeking gameshark_v1_v2
```

`cheats::CheatRom` holds a validated identity, separate from the editor session.
`Recipe` supplies bilingual product descriptions and evidence, a
ROM-specific binding supplies expected original instructions and replacements,
and the encoder owns protocol arithmetic. The UI never invents addresses or
encodes instructions. Parameterized recipes use typed core validation and
runtime names/rules from that ROM; unknown request fields are rejected.

- `open_cheat_rom {path | bytes}` validates a ROM without changing `App::session`.
- `cheats {expected_rom_md5}` reads the catalog of a loaded matching identity.
- `cheat_code {expected_rom_md5, cheat_id, format}` checks exact identity/recipe/format.
- Unknown fingerprints, stale identities, unsupported recipes and protocols fail.
- Generation is deterministic and does not enter edit history or serialize a save.

UI content is discarded on ROM changes, including copy state; obsolete async
responses cannot populate a closed/replaced window. Failed imports clear visible
old codes. Static documentation/configuration is not an extracted ROM catalog;
no ROM, save, game artwork or private simulation dump is shipped.

## Tests / 测试

```sh
cargo test -p gen3-core -p gen3-cli
npm run check
# Against Vite; uses only synthetic fixtures:
python3 scripts/test_cheats_ui.py
# Requires your four exact ROM paths in GEN3_ROM_ULTIMATE/BW/DP/ROCKET:
cargo test -p gen3-core local_cheat_catalog_cross_rom_regression -- --ignored
# Optional independent mGBA decoder/execution check (see script's environment help):
python3 scripts/verify_cheats_mgba.py
python3 scripts/verify_common_cheats_mgba.py
python3 scripts/verify_parameter_cheats_mgba.py
python3 scripts/verify_storage_cheats_mgba.py
```

The mGBA probe expects the 0.10.5 non-minimal library built with GB/GBA and debugger
support; headers and library must have matching build flags. It compiles in a
temporary directory, loads no save, and compares source file bytes afterward.
The old Sudowoodo probe is an **instruction-level test**, not an overworld encounter
replay; do not promote the rejected recipe based on that test alone.
