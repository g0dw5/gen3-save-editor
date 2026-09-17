# Dark Fantasy Hacker

[English](README.md)

面向用户自备三代改版 ROM 的本地桌面修改器。Rust 负责解析和事务读写，React
提供中英双语界面，Tauri 提供跨平台桌面外壳。名称、图片和地图由 ROM 实时读取。

**当前为开发预览。** 优先实现漆黑的魅影 BW / DP。测试与尚未完成的项目见
[验证记录](docs/IMPLEMENTATION.md)，不会把尚未验证的能力写成完整支持。

## 支持版本

| ROM | 必须匹配的 MD5 | 字节数 |
| --- | --- | ---: |
| 漆黑的魅影 5.0EX+BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | 33,554,188 |
| 漆黑的魅影 5.0EX+DP | `cb2940215f4dafb1bef133c3af379f44` | 33,554,188 |

存档需为 128 KiB 的 `.sav` / `.srm` 电池存档，不支持模拟器即时存档。
ROM 文件名不参与兼容性判断。编辑生成的派生 ROM 不会自动成为正式支持版本。

## 操作方式

- 同行、盒子和编辑页根据 PID 显示未知图腾字形、晃晃斑斑点，普通与闪光配色均从 ROM 读取。详见[外观验证](docs/research/pokemon-appearance.md)。
- 同行和全部 14 个盒子同时展开，可切换紧凑／舒适密度或收起右侧编辑栏。
  搜索只降低不匹配格的亮度，保持真实位置不变。
- 支持种族、昵称、性格、性别、闪光、等级、经验、个体值、努力值、特性、
  技能及 PP、携带道具、来源、蛋、病毒、缎带和华丽度字段。
- 拖拽可移动或交换；按住 Alt 拖拽可复制。也提供“移动到／复制到”按钮。
  按住 Ctrl、Command 或 Shift 点选多个个体，可以批量应用相同字段修改。
- 修改栏位支持输入 当前 ROM 的名称或编号搜索；普通编辑的
  相遇地点按本体及进化前形态筛选，并提供独立的孵蛋来源。规则与验证见
  [搜索和来源说明](docs/research/search-and-origins.md)。
- ROM 资料在可移动的非模态浮窗展示，包含种族、技能来源、道具、特性、地图、
  相遇位置和训练家队伍。训练家与地图可按战斗脚本关联双向跳转，并保留证据
  偏移及条件待核实提示。将相遇条目拖入空格，会先生成草稿，再创建和编辑。
- 支持玩家信息、金钱、代币、背包、盒子名称及图鉴标记；可查看前后差异、
  撤销与重做。导出前校验，覆盖已有输出时自动备份，并检测源存档的外部变更。
- “自由编辑”允许突破部分游戏规则，仍保留二进制结构和有效 ID 检查。
  例如选择斗笠菇、打开自由编辑、选择“点到为止”，再应用与导出。
  未找到学习来源只表示资料待核实，不直接判为非法。
- 界面中英文即时切换；ROM 自带名称和存档文字不会被翻译或改写。

ROM 修改目前支持部分种族、技能和道具的定长数值字段，导出独立的派生 ROM
及补丁清单；地图结构、脚本、剧情状态和代码注入不在当前写入范围。

## 开发与构建

安装稳定版 Rust、Node.js 22+ 及 [Tauri 平台依赖](https://v2.tauri.app/start/prerequisites/)。

```sh
npm ci
npm run desktop
npm run desktop:build
```

测试命令和真实 ROM 回归环境变量详见 [English README](README.md#development)。
公开测试使用合成二进制数据；本地 ROM 回归不会上传 ROM。发布包不包含 ROM、
存档、完整精灵图片集或开发 HTTP 服务。

相遇选表的原生代码验证可使用相同的 ROM 环境变量运行
`python3 scripts/verify_encounter_selection.py`（需要 Python Unicorn）。
已核对的代码路径、昼夜条件结论和验证边界见
[相遇表与时间条件](docs/research/encounter-time-selection.md)。

资料页切换的浏览器回归可先启动 `npm run dev`，再运行
`python3 scripts/test_reference_navigation.py`。需要 Playwright 和 Chrome，
使用合成接口数据，无需 ROM、存档或开发桥接服务。

静态地图的调色板规则及原生 ROM 验证方法见
[地图调色板](docs/research/map-palettes.md)。

宝可梦编辑栏在切换个体时保留当前标签，顶部标签和底部操作区固定，仅中间
内容滚动。启动 Vite 后运行 `python3 scripts/test_editor_navigation.py` 可验证
标签保留、草稿隔离及桌面／最小窗口／手机尺寸下的滚动行为；使用合成数据，
需要 Playwright 和 Chrome。

## 三层结构

- [方法层](skills/gen3-rom-research/SKILL.md)：可独立供 agent 使用，不依赖修改器。
  未知 ROM 研究不要求已登记 MD5。
- [代码层](docs/ARCHITECTURE.md)：统一事务和数据模型；地址、表长、文本分段等
  放入版本配置。格式或引擎变化需要新增解析器／规则，不能只改偏移。
- 应用层：桌面与 CLI 使用同一个 Rust 核心；界面仅维护草稿、展示和用户意图。

代码采用 MIT 许可；格式研究来源见 [第三方说明](THIRD_PARTY_NOTICES.md)。

## Windows 安装包

Windows x64 包面向 Windows 10/11，提供中英文安装向导，默认安装到当前用户。
缺少 WebView2 时由安装程序联网安装。交叉编译不等于 Windows 实机验证，具体命令、
依赖和验证范围见 [Windows 构建说明](docs/windows-build.md)。

地图资料支持地面精灵球、隐藏道具、对话／事件奖励和 NPC 独立图层，以及格位、
道具搜索、格线和缩放。显示所有 ROM 事件，不按存档领取状态过滤；
[解析范围与限制](docs/research/map-events.md)说明了静态位置和未知脚本的处理。

发布记录见 [更新日志](CHANGELOG.md)，从首次公开发布版 0.1.5 起记录。119 号道路钓点依当前存档计算，可在地图图层查看或从宝可梦资料页跳转。
