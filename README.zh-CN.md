# 三代改版修改器 · Gen III ROM Hack Editor

[下载正式版](https://github.com/g0dw5/gen3-save-editor/releases/latest) · [English](README.md)

打开自己提供的 ROM 查资料，再打开对应的 SAV 改存档。程序离线运行，不需要连接
模拟器；文件留在本机。游戏名称、数值、地图和图片读取当前 ROM，ROM 始终只读。

## 查询、攻略与修改

- **ROM 资料：**宝可梦种族值、官方参照、进化与可学招式；招式机／教学来源；
  道具获取；地图、NPC、隐藏道具、相遇表与训练家完整配队。关联条目可跳转地图定位。
- **冒险攻略：**搜索已解析的对白、奖励与前置线索。水银 1.33 读取见闻录任务及
  存档接取／完成状态；西班牙火箭队提供已识别主线分支的下一步线索。
- **存档编辑：**同行和全部盒子展开，支持拖拽、交换、批量编辑、性格、特性、
  IV／EV、招式／PP、背包及电脑道具；带撤销／重做、备份和导出检查。
  各游戏没有保存的字段限制编辑，例如水银的盒内当前 PP、华丽值和缎带。
- **金手指：**五份 ROM 提供 10 项通用功能，包括随身电脑、指定遇怪、闪光、
  传送和战斗救急。究极绿宝石另有去窥屏与命中加成方向修正。代码随当前 ROM
  生成，按对应格式自行在模拟器启用，见[使用方法](docs/cheats.md)。

查询无需 SAV。加载 SAV 后才叠加已验证的领取、任务和同行情景；不凭背包没有
某件道具判断没领过。完整任务依赖、动态地图、特殊设施和未知脚本仍有缺口。
程序明确显示未知，不将表中存在的记录等同于当前可以获得。

[详细使用说明](docs/USER-GUIDE.zh-CN.md) · [更新日志](CHANGELOG.md)

## 支持版本

| ROM | 必须匹配的 MD5 | 字节数 |
| --- | --- | ---: |
| 漆黑的魅影 5.0EX+BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | 33,554,188 |
| 漆黑的魅影 5.0EX+DP | `cb2940215f4dafb1bef133c3af379f44` | 33,554,188 |
| 西班牙火箭队 2.1 汉化版 | `59c658a1081f542086de1060bb65f0b3` | 33,554,432 |
| 究极绿宝石 5.5 · 失落之古遗 | `17ce9785b33319b3dbda9a5d37c57ec1` | 33,554,432 |
| 宝可梦水银 FC 1.33 | `5ffb1cbd5c28cda9b987b3b445da68e0` | 33,554,432 |


使用 128 KiB 的 `.sav`／`.srm` 游戏内电池存档（水银也接受 16 字节 RTC 尾部），
不支持模拟器即时存档。版本按指纹判断，改文件名不能改变兼容性；不支持水银 1.1。

Windows 安装包面向 Windows 10／11 64 位，缺少 WebView2 时联网安装运行环境。
Mac 安装包面向 Apple Silicon（M 系列）。程序未做商业代码签名／公证。
发行包不含 ROM、存档或提取素材，附中英文使用说明与更新日志。

## 修改前后

先备份原始 SAV，应用修改并核对差异，再导出副本。将导出的 SAV 导入模拟器后
正常游戏内保存。源文件有外部更新时，程序会阻止继续覆盖。
自由编辑允许特殊修改，仍进行结构校验。宝可梦的种族值是 ROM 全局资料，不能
通过修改某一个体的 SAV 改变。

五份精确 ROM 的本轮编辑已做 mGBA 加载、游戏内保存和重启回读；这不等于所有
字段、手机模拟器、特殊设施都已验证。Windows CI 构建也不等于 Windows 游戏实测。

## 开发

安装稳定版 Rust、Node.js 22+ 及 [Tauri 平台依赖](https://v2.tauri.app/start/prerequisites/)。

```sh
npm ci
npm run desktop
cargo test -p gen3-core -p gen3-cli --locked
cargo fmt --all --check
cargo clippy -p gen3-core -p gen3-cli --all-targets --all-features --locked -- -D warnings
npm run format:check
npm run build
```

架构采用共享数据模型和查询／存档事务，版本差异放在适配器和已验证规则中。
真实 ROM／SAV 回归只在本机运行，不提交测试文件。

[架构](docs/ARCHITECTURE.md) · [实际能力矩阵](docs/capability-matrix.md) ·
[开发测试方法](README.md#development) · [独立 ROM 研究方法](skills/gen3-rom-research/SKILL.md)

代码采用 MIT 许可，参照数据与格式来源见[第三方说明](THIRD_PARTY_NOTICES.md)。
