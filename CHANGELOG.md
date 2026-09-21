# Changelog / 更新日志

The first public release was **0.1.5**. Earlier development versions are omitted.
Entries for 0.1.6 and 0.1.7 describe repository/build changes, not a claim about
when those builds were publicly distributed. Each release package includes this
bilingual file.

首次公开发布版本为 **0.1.5**，不追记此前的开发版本。0.1.6、0.1.7 按工程提交补记，
不代表这些构建的对外发布日期。每次发布包均附带本中英文日志。

## 0.2.1 — Unreleased / 未发布

- Maintain annotated release tags, draft GitHub Releases with Windows/macOS
  artifacts, source/version checks, bilingual documentation and SHA-256 hashes.
- 增加版本标签、双平台 Release 草稿、源码版本校验、双语文档及 SHA-256；
  GitHub Release 从已有 0.2.0 安装包开始维护，开发版仍未公开。

Keep this development version until the user requests release packaging.
开发期间保持此版本号，待明确要求后再制作发布包。

- Show each internal Pokémon entry once in the evolution viewer. Preserve all
  distinct evolution conditions, fold additional forms, and avoid re-listing
  base forms already shown in evolution or battle relationships.
- 进化树按内部编号去重展示，每个条目只保留一张卡片；保留不同进化条件，其他
  形态折叠展示，避免进化、战斗形态及形态家族重复列出同一本体。

- Read nature and type names from each loaded ROM across Pokémon editing,
  trainer parties, move prefixes, Hidden Power and evolution conditions. Keep
  native names in both UI languages; remove the shared bundled name lists.
- 性格、属性名称改为实时读取当前 ROM，覆盖宝可梦编辑、训练家队伍、招式前缀、
  觉醒力量和进化条件。西火可搜索“内敛”，漆黑保留本作译名；切换界面语言不
  覆盖游戏原名，移除原有内置名称列表。
- Read nature stat changes from ROM and share them between calculation and UI
  markers. Preserve BW/DP's native 16-bit multiplication behavior separately
  from Rocket, verified against all 25 natures in each native engine.
- 性格能力修正与 ↑↓ 标识共用 ROM 表；区分漆黑性格乘法的 16 位截断和西火规则，
  修复高能力值边界的计算差异，逐版交叉验证全部 25 种性格。

## 0.2.0 — 2026-09-19

Updated in place without a version bump: shared editing for Dark Phantom BW/DP
and Team Rocket 2.1 Chinese. ROM reference windows remain read-only.
本次沿用 0.2.0：西班牙火箭队与漆黑 BW／DP 共用编辑器，ROM 资料保持只读浮窗。

- Bundle the completed manual reference mappings for Dark Phantom BW (114),
  DP (115), and Rocket (399, including 110 explicit no-counterpart decisions).
  Isolate mappings by ROM fingerprint and internal species ID; keep proposals,
  review notes and the standalone review tool outside the application bundle.
- 纳入已人工审核的官方参照映射：漆黑 BW 114 条、DP 115 条、西火 399 条（含
  110 条明确无官方对应）；按 ROM 指纹及内部编号隔离。审核建议、备注及独立
  审核工具不进入应用包，其余条目保留原有自动匹配行为。
- Display form identities in ROM references and saved Pokémon using shared
  ROM-derived relationships, verified form rules and reviewed direct mappings.
  Distinguish Mega X/Y and Deoxys forms without treating comparison-only
  references as identity or adding a save mutation.
- ROM 资料与存档宝可梦共用形态展示逻辑，结合 ROM 关联、已验证的形态规则及
  人工确认的直接映射，区分 Mega X/Y、代欧奇希斯等形态；仅作数值参照的映射
  不改变形态身份，展示功能不改写存档。
- Rename the ROM Species tab to Pokémon. All supported ROMs share a navigable
  evolution tree with incoming evolutions, sibling branches, battle forms and
  ROM form families. Name-only associations are visibly unverified and never
  become legal origins. Add Rocket's native form-family table reader.
- ROM“种族”改名“宝可梦”；三版共用可反复跳转的进化树，展示进化前后、分支、
  战斗形态及 ROM 形态家族。名称关联单独标为未确认，不扩大合法来源范围；补读
  西火原生形态家族表，区分水箭龟、Mega 水箭龟及水箭龟 Z。
- Compare six vertical ROM base stats and their total with the latest available
  official numeric reference from 52Poké Wiki. Label source generation, support
  manual reference selection, and avoid reused National Dex ID collisions.
  Include separate dataset attribution; no wiki artwork or ROM assets are bundled.
- 六围改为纵排，左列显示神百最新收录世代官方参照，右列为 ROM 数值，底部显示
  总种族值；标注来源世代，支持手选参照，避免复用图鉴编号造成误配。数值资料
  单独署名，不打包神百图片或 ROM 素材。
- Audit runtime data ownership, read BW/DP growth thresholds from ROM, remove
  the frontend's default rod IDs, and fix search popups hidden under ROM windows.
- 审查运行时数据来源：漆黑经验阈值改读 ROM 表，移除前端默认钓竿编号，修复
  搜索下拉列表被 ROM 资料窗口遮挡的问题。
- Fix Rocket map rendering with three layers, a 640-tile/metatile boundary and
  7+6 palette banks; preserve BW/DP's independent format. Add pixel-level tests.
- 修复西火地图黑洞和错图：正确读取三层、640 主图块及 7+6 调色板；漆黑保留独立
  配置，并新增跨版本像素回归。
- Replace raw evolution methods, effect IDs, pocket categories and story variables
  with readable ROM-derived labels; keep raw evidence collapsible. Fix Rocket
  rod names, Fairy labels and fixed Hidden Power reference power.
- 进化条件、招式效果、道具栏位和剧情相遇改为可读说明；原始编号折叠保留。修正
  西火钓竿名、妖精属性及觉醒力量固定威力的资料显示。
- Deduplicate opponent ability choices and distinguish fixed from randomly generated
  abilities. Verify all 13 Anya teams over 32 seeds against native ROM routines.
- Publish the unsaved-edit drag guard before the updated form becomes interactive.
- 对手特性候选去重，明确区分固定／随机；用原生 ROM 函数验证安雅的 13 份队伍，
  每份 32 组随机种子。修复未保存编辑状态传递的短暂延迟，及时阻止拖拽。
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
