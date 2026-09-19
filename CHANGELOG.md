# Changelog / 更新日志

The first public release was **0.1.5**. Earlier development versions are omitted.
Entries for 0.1.6 and 0.1.7 describe repository/build changes, not a claim about
when those builds were publicly distributed. Each release package includes this
bilingual file.

首次公开发布版本为 **0.1.5**，不追记此前的开发版本。0.1.6、0.1.7 按工程提交补记，
不代表这些构建的对外发布日期。每次发布包均附带本中英文日志。

## 0.2.0 — 2026-09-19

Updated in place without a version bump: shared editing for Dark Phantom BW/DP
and Team Rocket 2.1 Chinese. ROM reference windows remain read-only.
本次沿用 0.2.0：西班牙火箭队与漆黑 BW／DP 共用编辑器，ROM 资料保持只读浮窗。

- Rename the application to **Gen III ROM Hack Editor** (**三代改版修改器** in
  Chinese), reflecting support for multiple ROM hacks. Update window/dialog
  titles and package names while retaining the application identifier and 0.2.0.
- 更名为 **三代改版修改器 / Gen III ROM Hack Editor**，同步界面、窗口、对话框及
  安装包名称；保留应用标识和 0.2.0 版本号。
- Enable Rocket Pokémon, party/box transfers, creation, inventory, player and
  Pokédex editing, undo/redo and verified export. Remove the temporary read-only
  Pokémon component, banner and styles instead of maintaining a separate UI.
- 接通西火宝可梦、同行／盒子拖拽、创建、道具、玩家信息、图鉴、撤销重做和导出；
  删除临时只读宝可梦组件、横幅及样式，复用原有编辑页。
- Read all 1,363 maps and 2,558 trainer records, script references, NPC artwork,
  item layers, wild/static/gift encounters, and level/machine/tutor/egg sources.
  Report unresolved scripts and absent form compatibility tables explicitly.
- 补齐 1,363 张地图、2,558 位训练家、脚本位置关联、NPC 小人、道具图层、随机／
  定点／赠送相遇与升级／技能机／教学／遗传来源；明确显示未解析脚本与空兼容表。
- Handle expanded trainer EV/nature records and random gender/ability choices;
  use the native level-150 experience table, effective-nature override, packed
  ribbons, female artwork, Unown/Spinda appearances and fixed-power Hidden Power.
- 支持扩展训练家 EV／性格及随机性别／特性、150 级经验表、实际性格覆盖、位打包
  缎带、雌雄图片、未知图腾／晃晃斑外观；西火觉醒力量固定 60 威力。
- Read and write expanded inventory through logical save blocks; medicine and
  additional pockets may cross sector boundaries. Preserve unrelated bits,
  inactive banks and checksums. Add all-pocket and all-box cross-ROM regressions.
- 背包改为按逻辑存档块读写，修复扩展药品等跨扇区栏位的访问；保留无关位、
  备用存档区及校验。增加三版所有道具格与全部盒子格的交叉回归。
- Keep Mega/primal relations separate from permanent evolution. Standard editing
  rejects direct creation of these temporary species; free editing remains explicit.
  Changed held items/moves use verified persistent form transitions.
- Mega／原始回归独立于永久进化；普通编辑禁止直接创建这些临时种类，自由编辑可显式
  操作。更换携带道具／招式时执行已验证的持久形态转换。

- Show IV-derived Hidden Power type and power in the Pokémon stats/moves tabs
  and move picker, updating immediately while editing. Trainer moves show the
  result when IVs are known; generic ROM references explain the variable values.
- 能力页、招式页和招式选择框显示觉醒力量的实际属性与威力，随个体值编辑即时更新；
  训练家个体值确定时显示结果，ROM 通用资料说明其可变数值。
- Verify the calculator against 16,384 executions of the BW/DP native routine;
  add browser coverage for live edits, selection changes and unknown IVs.
- 与 BW／DP 的原生战斗函数逐一核对 16,384 组输入，新增编辑联动、切换个体及
  未知个体值的浏览器回归测试。
- Reserve the frontend mutation lock before awaiting the unsaved-edit check.
  Duplicate drop events in one event-loop turn can no longer submit two transfers.
- 在等待未保存编辑检查前锁定修改操作，防止同一轮事件中的重复放下提交两次移动／交换。
- Add byte-for-byte storage transfer coverage for all 420 box slots, sector
  rotations, party conversions, undo/redo and export, plus a drag/drop race test.
- 新增全部 420 个盒子槽位、扇区轮转、同行转换、撤销重做和导出的逐字节验证，
  以及拖拽重复事件回归测试。该竞态尚不能认定为历史 PID 损坏的原因。

## 0.1.8 — 2026-09-17

### Added / 新增

- Calculate Feebas fishing tiles from the loaded save and the ROM's actual map
  behaviors. The Route 119 map has a separate layer, coordinate navigation,
  encounter chance and level range. The species reference links to this map.
- 根据当前存档及 ROM 实际地图行为计算笨笨鱼钓点；119 号道路增加独立钓点图层、
  坐标定位、出现概率和等级范围，宝可梦资料页可直接跳转。
- Explain that daily updates, Dewford trend changes and record mixing may change
  fishing spots. Save in-game and reopen the latest exported battery save to
  refresh them; emulator progress is not synced automatically. Changing saves
  discards stale coordinates.
- 提示跨天结算、武斗镇流行语变化和混合记录可能改变钓点；需在游戏内保存后重新
  导入最新存档，修改器不会自动同步模拟器进度。切换存档时清除旧坐标并重新计算。
- Add the read-only `gen3 fishing-spots ROM [SAVE]` command and English/Chinese UI.
- 新增只读命令 `gen3 fishing-spots ROM [SAVE]` 及中英文界面文案。

### Distribution / 发布流程

- Include this changelog in desktop bundle resources and CI release artifacts.
- 桌面安装包／应用资源及 CI 发布附件自动包含本更新日志。

## 0.1.7 — Repository build / 工程构建

- Show NPCs on ROM maps using their actual overworld sprites, scaled and anchored
  to map tiles. Keep dialogue reward badges and a fallback for unresolved sprites.
- 地图 NPC 使用 ROM 中的行走小人图片，随地图缩放并按脚底对齐格位；保留奖励标记，
  无法解析的形象使用后备标记。

## 0.1.6 — Repository build / 工程构建

- Add independent map layers for item balls, hidden items, dialogue rewards and
  NPCs, with search, coordinates, grid and zoom controls.
- 地图新增地面精灵球、隐藏道具、对话奖励和 NPC 图层，支持搜索、坐标、格线及缩放。
- Parse reward-script branches with evidence and explicitly retain unresolved
  native/dynamic conditions instead of treating every reward as immediately available.
- 解析奖励脚本并保留来源证据；标明尚未解析的原生逻辑及动态条件。
- Validate Pokémon integrity before interpreting records as empty, preventing a
  damaged empty-looking record from being silently skipped or overwritten.
- 判定空槽前先检查宝可梦数据完整性，阻止将损坏记录当空槽跳过或覆盖。

## 0.1.5 — First public release / 首次公开发布

- Release as **Dark Fantasy Hacker**, with English/Chinese UI and ROM-backed
  Pokémon, move, item, encounter, trainer and map references.
- 以 **Dark Fantasy Hacker** 名称首次公开发布，提供中英文界面及实时读取 ROM 的
  宝可梦、招式、道具、相遇、训练家和地图资料。
- Edit party/PC Pokémon, bag/PC items and Pokédex data; support guarded editing,
  explicit free editing, change history, undo/redo and save export backups.
- 支持同行／电脑宝可梦、背包／电脑道具、图鉴修改，以及受约束编辑、自由编辑、
  改动记录、撤销重做和导出备份。
- Search editable fields using names from the supplied ROM. Remove the experimental
  official-name aliases. Preserve inspector tabs when selecting another Pokémon.
- 修改栏位按当前 ROM 名称搜索，移除试验性的官方译名别名；切换宝可梦保留编辑页签。
- Support the verified Dark Phantom 5.0EX+BW and EX+DP fingerprints. Users supply
  their own ROMs and saves; game assets are not shipped in the application.
- 支持经 MD5 校验的漆黑的魅影 5.0EX+BW、EX+DP；ROM 与存档由用户提供，不内置游戏素材。
