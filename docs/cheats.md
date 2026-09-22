# Cheat codes / 金手指

The window is a read-only, locally generated catalog of project-verified recipes,
not a cheat table extracted from the ROM. ROM import checks the entire MD5 and
size. No emulator connection, save or ROM writes are performed.

金手指窗口是项目逐项验证后的代码目录，不是声称从 ROM 内找到一张金手指表。
必须提供匹配完整 MD5 与大小的 ROM，不要求存档，不修改文件或替模拟器开启代码。

## Current coverage / 当前范围

| ROM | MD5 | Recipes / 条目 |
| --- | --- | --- |
| Ultimate Emerald / 究极绿宝石 5.5 | `17ce9785b33319b3dbda9a5d37c57ec1` | All modes: disable AI input peeking / 全模式关闭窥屏 |
| Dark Phantom / 漆黑的魅影 5.0EX+BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | Pause walking encounters; guaranteed wild capture; faster hatching / 暂停走路遇敌、必定捕获、加快孵蛋 |
| Dark Phantom / 漆黑的魅影 5.0EX+DP | `cb2940215f4dafb1bef133c3af379f44` | Same three features, independently tested / 同上三项，独立验证 |
| Team Rocket / 西班牙火箭队 2.1 中文 | `59c658a1081f542086de1060bb65f0b3` | Those three plus compatible daycare eggs / 上述三项及兼容寄养组合必定产蛋 |

Ultimate Emerald is supported **only in the cheat window**. Open it there using
“Choose cheat ROM”; the ordinary editor importer still rejects it. Opening another
ROM in this window does not replace the editor session. With an editor ROM already
open, the cheat window initially displays that ROM's availability. Changing the
editor ROM resets the displayed cheat context. Reopening the window starts from
the editor ROM (or the no-ROM prompt), not a hidden last-selected cheat ROM.

究极绿宝石请在金手指窗口内选择；主编辑器的打开 ROM 不会把它当作完整适配版本。
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
recipes have one line each; the Ultimate Emerald recipe requires two together.

当前条目为 **GameShark Advance V1/V2**。8+8 与 VBA-M 不带空格的 16 位形式只是
同一协议的排版，不能当作转换协议。战斗外正常保存并备份后启用整组，重启并从
游戏内存档继续；停用整组后重启。新增常用功能各一条，究极绿宝石去窥屏必须两条一起。

### Common recipes / 常用功能

- **Pause walking encounters:** use the native walking encounter routine's disabled
  path. Grass, cave and surfing checks through that routine stop; fishing, Sweet
  Scent, scripted encounters and trainers are not disabled. This is not walk-through-walls.
- **Guaranteed wild capture:** remove the random failure branch from the native
  ball command. Preserve the actual ball, normal consumption, capture records,
  trainer blocking and tutorial branches. Does not unlock bags or scripted restrictions.
- **Faster party hatching:** decrement egg cycles on every eligible step check;
  retain native ability bonuses, Bad Egg checks, checksum updates and hatch animation.
  Boxed eggs do not change. A zero-cycle egg hatches at the next eligible check.
- **Compatible daycare eggs (Rocket only):** at the normal step checkpoint,
  positive compatibility always succeeds. Two parents and no pending egg are still
  required; inheritance is unchanged. Dark Phantom's Oval Charm hook has different
  semantics, so the same comparison patch is intentionally not offered there.

暂停走路遇敌不影响钓鱼、甜甜香气、脚本定点或训练家，也不提供穿墙。必定捕获
仍需正常投球，消耗并记录实际使用的球；不解锁剧情捕捉限制。快速孵蛋仍使用原生
特性加速、坏蛋检查、个体校验和孵化动画，只加速同行蛋的周期扣减。西班牙火箭队
必定产蛋仍需两只兼容父母、无待领取蛋，并走到游戏原有检查点；不改变遗传。

Stopping a code restores instructions, **not completed gameplay changes**: captures,
hatching progress and pending eggs can persist after normal saving. These codes do
not give maximum IVs, force a species or inject inventory slots.

关闭代码恢复的是指令，不会撤销已经捕获的宝可梦、已减少的孵化周期和已产生的蛋。
本批不提供指定遇宠、强行满个体或直接覆盖背包槽位的代码。

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

`cheats::CheatRom` holds a validated identity, separate from the editor's full
`Profile`. `Recipe` supplies bilingual product descriptions and evidence, a
ROM-specific binding supplies expected original instructions and replacements,
and the encoder owns protocol arithmetic. The UI never invents addresses or
encodes instructions. Full editor profiles are not synthesized for cheat-only
ROMs. Adding future species/level parameters requires typed core validation and
runtime names/rules from that ROM; the current recipes intentionally accept no
parameters and rejects unknown request fields.

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
```

The mGBA probe expects the 0.10.5 non-minimal library built with GB/GBA and debugger
support; headers and library must have matching build flags. It compiles in a
temporary directory, loads no save, and compares source file bytes afterward.
The old Sudowoodo probe is an **instruction-level test**, not an overworld encounter
replay; do not promote the rejected recipe based on that test alone.
