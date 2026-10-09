# Mercury FC 1.33 addresses / 水银 FC 1.33 关键地址

Exact input: 33,554,432 bytes, MD5 `5ffb1cbd5c28cda9b987b3b445da68e0`,
SHA-256 `9801866280284a8ac12650666b8526ff3d07aa455f8dc623c2c56c0537313ec4`.
This replaces the registered FC 1.2 adapter. No dual-version reader or ROM writer
was added. These are developer addresses, not distributable game content.

精确指纹限定 FC 1.33。当前只保留这一份水银适配；旧版 1.2 ROM 不再接受。
以下为开发地址和格式，内容在运行时从 ROM 读取，不包含提取后的目录或图片。
All addresses below are **ROM file offsets** unless marked RAM or bus address.

## Runtime tables / 运行时表

| Data / 数据 | FC 1.33 offset | Layout / 格式 |
| --- | --- | --- |
| Pokémon names / 宝可梦名称 | `0x141B350` | 1,554 × 11 bytes, includes slot 0 |
| Base stats / 种族数据 | `0x176DFBC` | 1,554 × 28 bytes |
| Experience / 经验表 | `0x1E0B258` | 6 × 1,024 bytes |
| Nature names / 性格名称 | `0x1FF38F0` | 25 text pointers |
| Nature modifiers / 性格修正 | `0x252B48` | Native modifier table |
| Move names / 招式名称 | `0x1D97378` | 1,015 × 13 bytes; header pointer at `0x148` |
| Move records / 招式数据 | `0x1E0326F` | 1,015 × 12 bytes |
| Move descriptions / 招式说明 | `0x1CC8EBC` | Pointer table |
| Ability names / 特性名称 | `0x1D8E858` | 300 × 13 bytes; header pointer at `0x1C0` |
| Ability descriptions / 特性说明 | `0x1C93EB8` | 255 verified description slots |
| Type names / 属性名称 | `0x1DBBA70` | 24 × 7 bytes |
| Items / 道具 | `0x7C7E00` | 750 × 44 bytes |
| Evolutions / 进化 | `0x1788F5A` | 1,554 × 128 bytes |
| Item gender comparison / 道具进化性别比较 | `0x1D2A006` | Native instruction operand; getter literal at `0x1D2A578` |
| Level / egg moves / 升级与遗传 | `0x17C32BC` / `0x1781B64` | Current profile formats |
| Maps / 地图根表 | `0xB3B100` | 58 banks; configured per-bank bounds |
| Map names / 地点名 | `0xC2B000` | Native `GetMapName` at `0xC4D78`, sections 88–252 |
| Encounter headers, morning/day/dusk/night | `0xCF915C`; `0x1E4E09C` / `0x1E468B0` / `0x1E4C1A0` / `0x1E4A10C` | Native time tables, not different patches of grass |
| Trainer headers / 训练家 | `0x23EAC8` | 743 × 40 bytes including slot 0 |
| Trainer constructor / 配队生成 | `0x1D0BA44` | Ordinary zero-context preview; unresolved context stays partial |
| Object palettes / NPC 调色板 | `0x1E14010` | 341 × 8 bytes, plus existing supplement |
| Quest headers / 见闻录任务 | `0xE3BC54` | 100 × 20 bytes |
| Journal books / 日志页索引 | `0x8F5644` | 100 × 8 bytes; 344 pages in this ROM |
| Journal visibility / 日志显示条件 | `0xDF7D5C` | Acceptance/completion/appearance and per-page conditions |

Some table roots stayed put while their pointer entries or payloads changed.
Stable samples alone do not certify a table; native comparisons and complete
reader regressions are recorded in the [verification](../verification/mercury-133-20261009.md).

地图名称表保留了无效／未使用槽位，不能把表范围等同于全部有效地点名。
旧浅红道馆 `1-0` 的原生布局异常记录仍按现有规则提示；静态渲染不模拟动态布局。

## Native save / 原生存档

The verified save layout remains SB2 `0xF24`, SB1 `0x3D68`, storage `0x83D0`;
sector payloads use `0xFF0`, extension sectors are 30/31, and an optional 16-byte
RTC trailer is preserved. Pokémon party records use the existing CFRU plain codec.
PC records are compact **58-byte** records across 25 boxes; ordinary moves retain
exact records rather than expand/re-encode them. Boxes span Storage, extension RAM,
SB1 and SB2 as described by `box_storage::MERCURY`.

| Routine / 例程 | FC 1.33 offset |
| --- | --- |
| Pocket descriptors / 背包描述符初始化 | `0x1D42844`, table `0x1DE3A94` |
| Extension save tail / 扩展保存 | `0x1D5CBB4`, fast pointer literal `0x1D5CBF4` |
| Compact box pointer / 盒子指针 | `0x1D5A3E0`, table `0x1DE4A7C` |
| Expand / compress / 解压与压缩 | `0x1D5A404` / `0x1D5A630` |
| Valid tera types / 太晶类型集合 | `0x1DE6920`, 19 bytes |
| Split Pokédex / 分区图鉴 | `0x88E74` → `0x1D6970C`; second bank marker `0x11DE` |
| Mon getter / setter / 个体读取与设置 | `0x3FBE8` / `0x4037C` |
| Stats / ability / 能力与特性 | `0x3E47C` / `0x40D38` |

RAM: save pointers `0x03005008` / `0x0300500C`, extension base `0x0203B174`,
player party `0x02024284`, enemy party `0x0202402C`, RNG `0x03005000`.
The tested old battery fixture loads and re-saves in 1.33; this does not prove
all old story states or every version-migration case are compatible.

## Virtual time / 虚拟时间

Clock RAM remains `0x03005EA0`. Persisted words are extension +`0x5DE`, speed
+`0x5E6`; enable flag `0x1335`, forced night `0x1041`.

| Routine | FC 1.33 offset |
| --- | --- |
| Restore / persist | `0x1D5C840` / `0x1D5C408` |
| Morning / dusk / night | `0x1D216D4` / `0x1D216EC` / `0x1D21108` |
| Month table / native days | `0x1DE4D0C` / `0x1D5C3C4` |
| Jump period / next day | `0x1D5C98C` |
| Speed / tick | `0x1D5C45C` / `0x1D5C754` |

Periods remain 04:00, 08:00, 17:00 and 20:00. Device time is not a substitute
for the saved virtual time; these semantics were re-executed in 1.33.

## Emulator cheat locations / 模拟器金手指位置

These target emulator memory only. Source ROM bytes are never written.

| Feature | FC 1.33 location |
| --- | --- |
| Disable walking encounters | `0x1D6CD7E`, routine `0x1D6CD4E` |
| Wild catch | `0x1D0F2DA`, routine `0x1D0F09C` |
| Hatch / daycare | `0x46346` / `0x4632C` |
| Portable PC | `0x1D522CE`, literal `0x1D523AC`, authored stub `0x13FD200` |
| Species / level | `0x1D6BA80`, authored stub `0x13FD280` |
| Shiny wild caller guards | Bus return addresses `0x09D6BBE7` / `0x09D6BB25` |
| Teleport | `0x553B2` |
| Battle recovery | `0x12424`, authored payload `0x13FD400` |
| Protect | RAM `0x02023E8C`, stride 16; battle callback `0x03004F84` |

Cave space is checked against the exact ROM before generating a code. Old 1.2
codes must not be reused. Native walkthroughs, toggle controls and limitations
are in the verification; player instructions stay concise in `docs/cheats.md`.
