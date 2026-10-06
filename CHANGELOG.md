# Changelog / 更新日志

The first public release was **0.1.5**. Earlier development versions are omitted.
Entries for 0.1.6 and 0.1.7 describe repository/build changes, not a claim about
when those builds were publicly distributed. Each release package includes this
bilingual file.

首次公开发布版本为 **0.1.5**，不追记此前的开发版本。0.1.6、0.1.7 按工程提交补记，
不代表这些构建的对外发布日期。每次发布包均附带本中英文日志。

## 0.3.0 — Unreleased / 未发布

- Follow native Dex mirror and seen/caught dependencies in Dark Phantom BW/DP
  and Ultimate; retain independent Rocket flags and Mercury split banks. Explain
  invalid positive records in the editor, collection planner and standalone HTML
  without changing SAV bytes. Verified against complete native getters.
- 漆黑 BW／DP 与究极绿宝石的图鉴读取遵循本作原生镜像及已见／已捕获依赖校验，
  西班牙火箭队独立标记、水银分段读取保持不变。修改页、收集规划与独立 HTML
  提示异常记录，查询不写入 SAV；已与完整原生例程逐项核对。

- Add Mercury 1.2 historical-Dex collection planning through native split-bank
  reads. Explain lazy initialization in both languages and standalone HTML;
  preserve raw SAV bytes and keep Dex editing disabled. Five-ROM regressions retain
  legacy readers/edit boundaries; full acquisition/access coverage remains partial.
- 水银 1.2 收集规划增加按图鉴捕获记录判断，按原生两段图鉴读取并解释未初始化
  范围。中英文界面与独立 HTML 共用说明，不改存档；图鉴写入仍关闭。其他 ROM
  的原有读取与编辑边界交叉回归，完整获取方式和可达性仍部分支持。

- Preserve reward operands, entrance coordinates and conditions through verified,
  bounded native name/number buffers across five fingerprints. Unknown formatting,
  long strings and Mercury's runtime custom-berry name keep conservative stops.
- 五指纹的名称／数字填充指令经原生核对后，保留奖励数量、入口坐标与条件。
  长字符串、未知格式及水银的运行时自定义树果名称继续保留未知边界。

- Preserve verified native player-gender reads and copied-result comparisons across
  references, maps and collection HTML. SAV overlays show the saved value; ROM-only
  checks stay unknown. Five-ROM native read/copy/branch evidence and bilingual UI
  regression cover this bounded rule; other story/access gaps remain partial.
- 五指纹补齐原生玩家性别读取、复制后比较与中英文条件展示，资料／地图／收集
  HTML 共用说明。读取 SAV 后代入存档身份；仅读 ROM 保留未知。其他剧情、领取
  与可达性缺口不因此升级为完整支持。

- Correct Dark Phantom BW/DP berry/key-item ROM category IDs. Standard pocket
  edits, ordinary holdings and reference labels now select the proper category.
  Normal choices follow the loaded ROM; free/PC choices stay broad and existing
  mismatches remain visible. Verified with five-ROM native pockets and save/reload.
- 修正漆黑 BW／DP 的树果与重要道具分类编号，恢复正确的栏位编辑校验、持有量
  判断和资料标签。普通选择按当前 ROM 分类筛选，自由编辑／电脑保留完整选择；
  已有错放道具保留并提示。五指纹原生栏位与游戏内保存回读已交叉验证。

- Verify production SAV edits through full-frame mGBA Continue, in-game Save and
  fresh reboot for all five fingerprints. Native party/storage bytes and every
  inventory pocket preserve tested IV/EV/marking, batch/move/swap and quantity
  results. Mercury's empty-box fixture and RTC trailers retain explicit limits.
- 五个精确 ROM 补齐正常按键的编辑、游戏内保存与重启回读验证，独立核对同行／
  整块盒子和所有背包栏位。本轮多字段、批量、移动／交换及数量修改保留结果；
  水银无盒子个体的副本、RTC 尾部和其他未测试操作明确保留边界。

- Preserve script-assigned coordinates and item operands through verified field
  prompts, close-message and delay commands. Read standard bodies from each ROM,
  account for the expanded message gate, and retain unknown choices and access.
  Five-ROM native comparison resolves 48 referenced coordinate cases; it does
  not establish 48 distinct doors or complete field interactions.
- 修复对话、关闭消息及等待后丢失入口坐标和道具参数的问题。通用脚本按当前 ROM
  读取，究极绿宝石的额外分派单独处理；选择与可达性仍保留未知。五指纹原生
  对照后，48 条坐标引用可定位，不将引用数当作独立入口数或完整现场验证。

- Share guarded script entrances across collection sources, evolution preparation,
  prerequisite candidates and standalone HTML. Keep route alternatives separate,
  trace their persistent guards to affected goals, preserve unresolved incoming
  references, and recheck SAV state. Access and full story coverage remain partial.
- 收集来源、进化准备、前置事件和独立 HTML 共用带条件的脚本入口。各条路线分开
  展示，持久条件关联到受影响目标，保留未解析目的格位的来源，并随 SAV 重查。
  当前可达性、完整剧情和动态通道仍部分支持。

- Add referenced script passages to map sources, destination tiles, entrance
  layers and qualified exterior chains across all five fingerprints. Recheck SAV
  guards without caching eligibility; exclude destination setters and unreferenced
  bytes. Verify 918 native operand cases; full activation/access remains partial.
- 五个精确 ROM 的地图页补入已引用脚本通道、来源／目标格位、入口图层及带提示的
  外部入口链。SAV 条件实时重查，不缓存资格；设置目的地的指令及无引用数据不当作
  通道。918 个原生参数用例已对照，触发方式、可达性及完整地图切换仍部分验证。

- Connect collection goals to regional prerequisite-event alternatives in the
  live UI and HTML. Preserve state when following references, discard stale
  ROM/SAV responses, retain cycles and unknown guards, and stop tracing already-met
  conditions. Add read-only `gen3 collection-plan`; full story/access remain partial.
- 收集规划新增按区域查看前置事件、关联目标及地图入口，并贯通独立 HTML。
  跨资料返回保留分析状态，换 ROM／SAV 丢弃旧响应；保留循环与未知条件，
  已满足条件不再推荐其解锁任务。新增只读命令行规划，完整剧情与可达性仍部分支持。

- Show acquisition type, known quantity, encounter-slot probability, periods and
  repeatability in collection rows and standalone HTML, with related item/move/
  Pokémon links. Period hours use the loaded adapter's verified rules; unknown
  rules show names only. These remain suggestions, not new reachability proofs.
- 收集清单与独立 HTML 补齐获取方式、已知数量、相遇槽位概率、时段及重复获取说明，
  支持关联道具／招式／宝可梦跳转。时段小时采用当前适配器已验证规则；未知规则只
  显示名称，不套用其他版本。仍为建议路线，不新增可达性或完整任务依赖的保证。

- Qualify crown-service selection as party-only, with B cancellation and separate
  later level checks. Native selection does not exclude fainted individuals or
  eggs; the read-only editor preview deliberately accepts non-eggs only. Box
  previews now explain the required in-game withdrawal and expose its scenario
  separately from service eligibility. No PC transfer or full transaction claim.
- 王冠服务明确为同行选择，可按 B 取消，之后另有等级检查。原生选择步骤不排除
  已倒下个体或蛋；修改器只读预览仍只接受非蛋个体。盒子预览提示先在游戏内取回
  同行，并区分模拟状态与实际服务资格，不冒充完整取回或交易验证。

- Read NPC service-menu cancellation rules from the loaded ROM. Correct Mercury's
  single-stat menu description: it ignores B after an attempted payment; its
  initial menu and Ultimate's training-choice menu allow B cancellation. Native
  input/branch evidence remains separate from full window and transaction replay.
- NPC 服务菜单按当前 ROM 显示取消规则。修正水银单项选择的描述：尝试支付之后
  的菜单忽略 B；其主菜单与究极绿宝石训练项目菜单可按 B 取消。原生输入／分支
  已对照，完整窗口与支付交易仍保留验证边界。

- Preview referenced NPC crown-service effects on stored or simulated individuals.
  Execute native level-gated helpers: Mercury changes base IVs and recalculates
  party stats; Ultimate changes training flags without refreshing stored stats.
  Keep hypothetical field effects separate from eligibility and full payment.
- NPC 王冠服务新增同行／盒子／模拟个体的原生效果预览：水银修改基础 IV 并
  重算同行能力，究极绿宝石修改训练标记而不立即刷新保存的能力。字段情景与
  资格、完整支付分开展示，预览不消耗道具、不修改存档。

- Add Mercury's runtime NPC base-IV training choices, unlock/level checks and
  item/map/prerequisite links. Separate required holdings from attempted payment:
  native silver checks silver but removes gold before stat selection, without an
  immediate removal-result guard. Keep full transactions qualified and read-only.
- 水银新增实时读取的 NPC 基础个体值训练：选择、解锁／等级检查、道具来源与
  地图／前置跳转。区分持有与尝试支付：原生银色分支检查银冠却在单项菜单前扣
  金冠，没有立即检查扣除结果；完整交易保留未知，资料页只读，不新增存档写入。

- Add runtime NPC Hyper Training reference for Ultimate: actual menu choices,
  item fees, earned-credit conditions and positioned NPC/map links. Keep base IVs
  distinct from training flags and explain the delayed party-stat refresh. SAV
  overlays are read-only; other-ROM crown services remain unverified.
- 究极绿宝石新增实时读取的 NPC 极限特训资料：菜单选择、王冠费用、认证次数条件、
  NPC 格位及地图跳转。区分基础个体值与训练标记，说明同行能力延迟刷新；
  存档条件只读叠加，其他 ROM 的王冠服务不冒充已验证。

- Add Ultimate's runtime ability-item variants and native script-prefix previews.
  Distinguish seeded normal-slot selection from PID generation, retain unrelated
  header/Hyper Training bits, and show successful-but-unchanged results.
- 究极绿宝石特性道具接入运行时变体与原生脚本前缀预览，区分随机普通槽位选择
  和 PID 生成，保留无关头部／王冠位；允许使用却未改变个体时明确提示。

- Read Rocket/Mercury ability-item handlers at runtime and preview native guards,
  actual abilities, PID and stats, linked to acquisition and maps. Rocket retains
  PID; Mercury requires an explicit seed and may reroll PID. Qualify menu/access,
  consumption, live RNG and unverified services separately; previews are read-only.
- 实时读取西班牙火箭队／水银的特性道具入口，预览原生允许／拒绝判断、实际特性、
  PID 与能力值并关联获取地图。西班牙火箭队保留 PID；水银需明确随机种子情景，
  可能重选 PID。菜单资格、可达性、消耗、实时随机状态及未验证服务分别说明；
  预览只读，不新增 SAV 写入，不向其他 ROM 套用规则。

- Read Rocket mint targets from current-ROM handlers and native getters; preview
  effective nature and stats while retaining PID/identity/history. Link item
  acquisition and map navigation. Reject unchanged effective-nature scenarios;
  qualify persistent-stage evidence separately from menus/consumption/access.
- 从当前 ROM 的入口与原生取值读取西班牙火箭队薄荷目标，预览实际性格与能力，
  保留 PID、身份和历史并关联道具来源／地图。重复性格情景保持不变；持久化阶段
  与菜单资格、消耗、可达性分开说明，不新增存档写入操作。

- Add read-only EV-item reference and native field-effect previews for stored or
  simulated individuals, linked to acquisition and map navigation. Read handlers
  from the current ROM; show actual before/after values and unresolved context.
  Do not infer a gain from classification or success, consume items or edit SAVs.
- 新增努力值道具资料及同行／盒子／模拟个体的原生战斗外效果预览，可跳转获取
  与地图。处理入口实时读取当前 ROM，展示实际前后数值和未确认上下文；不凭
  用途分类或返回成功推断增加量，不消耗道具、不修改存档。

- Add a shared read-only Game time reference page. Mercury retains its saved
  virtual clock; four hardware-clock profiles expose SAV offsets/checkpoints
  and explicit RTC scenarios calculated by the loaded ROM's native routines.
  Keep current RTC, weekday and unverified refresh rules unresolved.
- ROM 资料新增统一只读游戏时间页。水银保留虚拟时钟；其余四份指纹展示存档
  时间偏移与检查点，可输入 RTC 情景按当前 ROM 原生例程计算。不推断当前
  硬件时间、星期或未验证的事件刷新规则。

- Link qualified trainer-script references to actors, static tiles, guard checks,
  event contexts and exterior entrances. Keep rematch bases and setup records
  distinct from final parties and current access. Verify command boundaries with
  native execution across five fingerprints; correct Mercury's extended type-10
  boundary and stop unsupported formats rather than consume guessed bytes.
- 训练家资料新增战斗脚本引用、NPC／格位、条件及事件上下文跳转，可沿地图查看外部
  入口。再战基础记录、准备指令与最终配队分开展示，不推断当前可达。五份指纹核对
  原生命令参数边界，修正水银类型 10 的扩展宽度；未知格式保留停止证据。

- Add ROM-text/map search for referenced event clues with guarded writes,
  SAV snapshot observations, prerequisite tracing and tile/entrance navigation.
  Preserve search/filter/selection/page on return; keep floating windows inside
  the viewport when dragged or resized. Keep visibility, observed
  values, task completion and access distinct; no bundled quest catalog.
- 新增事件线索页：按当前 ROM 对话／地图搜索引用，查看脚本条件与保存快照，
  追查前置线索并定位格位／入口。返回保留搜索、筛选、条目和分页；浮窗拖动和缩放
  保持在窗口内，防止右侧内容越界。不把 NPC
  可见或标记值一致解释成任务已完成，不内置任务目录。

- Trace persistent prerequisites to potential referenced NPC/tile/map scripts,
  current-ROM text contexts, guard chains and map entrances. Add fingerprint-bound
  lazy queries and a linked, escaped collection HTML appendix. Verify native
  command semantics across five ROMs (360 writes); correct shared copyvar/subvar
  handling. Keep full task graphs, native writes and current access unresolved.
- 前置条件可追查可能改变它的 NPC／格位／地图脚本，关联当前 ROM 文字上下文、
  其他条件及地图入口；新增按指纹隔离的只读查询和独立 HTML 线索附录。五份 ROM
  原生命令验证 360 次写入，修正共享变量复制／减法语义。完整任务树、原生剧情写入
  和当前可达性保留未知，不修改剧情标记。

- Connect collection preparation to sampled native ordinary breeding outcomes
  from exact existing party/box parents, with held-item links, receiving-service
  references, hatching and directed evolution steps. Add a one-click full native
  receipt preview and the same explanation to standalone HTML. Cache by current
  ROM and complete SAV hash; show sampling bounds and unresolved scenarios.
- 收集准备链加入现有同行／盒子亲本的原生普通孵蛋抽样结果，关联携带道具、已定位
  领蛋服务、孵化及后续进化；可一键用完整领蛋例程复核，独立 HTML 同步说明。
  缓存绑定当前 ROM 与完整 SAV 哈希，展示搜索边界及未确定情景，不生成个体。

- Read ordinary daycare snapshots from SAV using each current ROM's native
  getters: deposited individuals, saved egg availability and the next ordinary
  production-check phase. Use exact deposited records in read-only scenarios.
  Verify 576 saved states and 40 sequential phases across five fingerprints;
  Mercury availability follows its patched flag rather than the legacy field.
  Keep custom services, live RNG and complete inheritance/hatching unresolved.
- 普通寄养支持只读存档快照：按当前 ROM 原生读取亲本、待领蛋状态及下一次普通
  产蛋检查的步数，可用实际寄养记录进行情景预览。五份指纹验证 576 个保存状态和
  40 个连续步数情景；水银按补丁标记判断待领蛋。自定义服务、实时随机状态及完整
  遗传／孵化仍有缺口，不生成蛋或修改寄养数据。

- Expand collection suggestions with bounded, directed permanent-evolution
  preparation chains, actual non-egg ancestor counts, referenced ancestor
  maps/entrances and item/move links, shared with standalone HTML. Correct missing
  held-item, gender-dependent and compound evolution cross-links. Keep current
  eligibility, breeding reversals and global route optimality unproven.
- 收集建议加入有界的永久进化准备链，区分现有非蛋个体与图鉴历史记录，关联前代
  来源、地图入口及道具／招式，独立 HTML 同步展示。修复携带道具、性别条件和复合
  进化的跳转遗漏；不推断当前可进化，不反转进化边代替孵蛋验证，不宣称全局最短。

- Preview ordinary daycare production with the current ROM's native step/item
  branches, an ordinary SAV-bag projection or explicit item simulation, and
  links to required-item acquisition. Verify 270 complete steps, 30 gate cases
  and every 16-bit roll across five fingerprints. Keep live pending eggs,
  service setup/access and complete inheritance/hatching unresolved.
- 原生寄养预览加入普通产蛋检查：运行当前 ROM 的步数与道具分支，支持普通
  存档背包投影／明确的道具模拟，并跳转所需道具的获取途径。五份指纹通过
  270 次完整步数执行、30 个门控情景及全部 16 位随机抽值；实际待领蛋、
  服务初始化／可达性和完整遗传／孵化继续保留未知。

- Add read-only native daycare scenarios with stored/simulated parents, offspring
  links and located receiving NPC/map navigation. Verify 210 scenarios across five
  fingerprints with mGBA, and correct shared odd Thumb halfword loads using 120
  independent microcases. Keep complete production context, next real eggs, complete inheritance
  and Rocket/Mercury service references explicitly unresolved.
- 新增只读原生寄养情景：选择存档／模拟亲本，跳转后代资料与已定位领蛋 NPC、地图。
  五份指纹的 210 个情景对照 mGBA；以 120 个独立微测试修正共享 CPU 的奇地址
  Thumb 半字读取。完整产蛋情景、下一颗真实蛋、完整遗传及西班牙火箭队／水银服务引用仍
  明确保留未知，不生成或改动存档中的蛋。

- Connect wild held-item sources to referenced encounter maps, levels, slots and
  time selectors. Execute each current ROM's ordinary single-wild assignment
  routine for a simulated baseline and the current SAV's first party member;
  show held-item chances separately from encounter slot probabilities. Preserve
  unreferenced records and unresolved access as unknown. Queries, collection
  suggestions and bilingual HTML share the same explanation.
- 野生携带道具来源关联真实相遇引用、地图、等级、槽位和时段；执行当前 ROM 的
  普通单只野生携带例程，分别展示无修正模拟基准和存档同行首位情景。携带概率与
  相遇槽位概率分开；无相遇引用和未确认可达性仍保留未知。查询、收集建议和
  中英文 HTML 共用说明。

- Preserve native item/money holdings predicates through script result copies and
  branch comparisons, with ROM-scoped widths and bag slot rules. Link required
  items, maps, collection planning and human-readable bilingual HTML. Keep
  facility/RAM bag selection and checks after resource-changing scripts unknown;
  holdings are not payment or receipt evidence.
- 保留原生道具／金钱持有量检查与结果复制、分支关系，按 ROM 核对数量位宽和
  背包槽位规则；所需道具、地图、收集建议和中英文 HTML 相互关联。设施临时
  背包及脚本改变资源后的检查保持未知，不把持有量当成费用、已支付或领奖证明。

- Correct Dark Phantom BW/DP tutor lookup to the table used by native code (six
  changed slots), and connect parsed teaching offers to move queries, NPC tiles,
  exterior entrances and return navigation across all five fingerprints. Execute
  indexed move selection from current ROM code, including Mercury special cases;
  keep payment, eligibility, receipt and repeat limits explicitly unresolved.
- 修正漆黑 BW／DP 教招表地址（6 个槽位与旧表不同），五份指纹的已解析教学报价
  接入招式查询、NPC 格位、外部入口和返回。索引取招执行当前 ROM 原生代码，覆盖
  水银特殊取招；费用、资格、领取和次数限制继续明确保留未知。

- Index bounded NPC trade quotes from the current ROM and connect requested/
  received Pokémon, held items, NPC tiles and entrance navigation. SAV queries
  distinguish matching non-egg party/box donors and retain unknown completion
  and access. Verify 29 offer records with 174 native ordinary-generation cases
  across five fingerprints; Mercury's alternate constructor remains separate.
  Acquisition, planning and standalone HTML explain the exchange in both languages.
- 实时解析有界 NPC 交换报价，关联交出／获得的宝可梦、携带道具、NPC 格位及
  入口导航；SAV 查询区分同行／盒子的对应非蛋个体，不把报价当成已完成或当前
  可达。五份指纹的 29 条报价通过 174 个原生普通生成情景；水银的特殊构造分支
  单独保留未知。获取途径、收集建议和独立 HTML 共用中英文交换说明。

- Connect parsed scripted Pokémon gifts, gift eggs and fixed encounters to NPC
  tiles, acquisition queries, map-to-species navigation and collection HTML.
  Keep unplaced sources separate and delivery/access conditions undetermined.
  Decode Rocket's extended wild operands and Mercury's single/double wild
  operands per adapter; read gift-egg levels from the current native constructor.
- 已解析的宝可梦赠送、赠蛋及定点相遇接入 NPC 格位、获取途径、地图宝可梦跳转
  与收集 HTML；无可靠格位的来源单列，发放结果与当前可达性保持未知。分别解析
  西班牙火箭队扩展遇怪参数、水银单只／双只参数，赠蛋等级实时读取原生构造例程。

- Trace bounded NPC gift receipt protocols at runtime, with per-reward guards,
  native boolean award outcomes and success-only flag writes. Verify 80 reward
  rows / 320 native cases across all five exact ROMs. Connect receipt evidence to
  acquisition, map details, collection filtering and bilingual standalone HTML;
  preserve uncertainty for unqualified/custom rewards and complete refresh/story rules.
- 实时追踪有界 NPC 礼物领取协议，按奖励核对条件、原生发放结果及仅成功后写入的
  标记；五份精确 ROM 的 80 条记录通过 320 个原生情景。证据贯通获取途径、地图详情、
  收集过滤和中英文独立 HTML；未资格化／自定义奖励、完整刷新与剧情条件保留未知。

- Verify ordinary item-ball receipt branches across all five exact ROMs and
  overlay 1,956 qualified script records in acquisition/collection queries.
  Keep receipt evidence separate from object visibility; compound scripts,
  dynamic quantities and out-of-range flags stay undetermined. Explain missing
  receipt evidence in both languages and avoid unproven one-time-only claims.
- 五份精确 ROM 的普通道具球领取分支通过原生验证，1,956 条符合协议的脚本记录
  接入获取途径与收集建议。领取证据与对象可见性分开；复杂脚本、动态数量和未验证
  标记保持未知，补充中英文说明，不再未经证明就宣称只能领取一次。

- Read native persistent event ranges for all five exact ROMs, including Ultimate's
  segmented flags and Mercury's extensions. Overlay verified hidden-item receipt
  state on acquisition/collection queries. Decode Mercury region-dependent hidden
  flags and packed quantities from ROM; explain underfoot pickup in maps, sources,
  planning and HTML. Unknown/native reward paths stay undetermined; NPC/compound
  pickup receipt protocols and full refresh rules still need separate proof.
- 五份精确 ROM 的存档事件条件改为按原生持久化范围读取，补齐隐藏道具领取叠加。
  水银实时读取 ROM 区域列表选择标记基址，解析数量与脚下探测器取物，并贯通地图、
  获取途径、收集建议和 HTML。未解析奖励路径保持未知，不把 NPC 消失等同已领奖；
  复杂拾取／NPC 领取协议及完整刷新规则仍待验证。

- Read Mercury 1.2's native virtual clock from SAVE extensions: date, independent
  weekday, time, speed and forced-night override. Annotate acquisition queries,
  regional collection suggestions and standalone HTML with this saved snapshot;
  retain explicit simulated-hour and all-period modes. Native restoration,
  leap/calendar validation, jump-menu behavior and post-jump re-save are verified.
  Hardware RTC and complete weekday-event refresh remain unresolved.
- 水银 1.2 查询原生扩展存档中的虚拟日期、独立星期、时间、速度及强制夜晚条件；
  获取途径、区域收集建议和独立 HTML 共用保存时间依据，仍可模拟小时或查看全部
  时段。已对照原生恢复／历法／跳时菜单，并验证跳时后的游戏再次保存回读；
  硬件 RTC 与完整星期事件刷新仍待验证。查询不修改时钟或存档。

- Correct Mercury 1.2 expanded bag storage (sector tails and auxiliary sectors),
  plaintext quantities, and native map/met-location names. Preserve displayed
  zero quantities instead of silently treating them as one. Reproduce extended
  metatile layers, filler/backdrop transparency, and flag unresolved native layouts.
- 修正水银 1.2 的扩展背包位置与数量读写、地图及相遇地点名称；数量为零时
  列表和详情一致显示，不擅自变成一个。按原生规则绘制扩展图层、填充及透明
  背景；标注在原生模拟器直接加载时同样错乱或图层行为无法确定的静态布局。
  已用含物品的真实存档副本验证加载、编辑、撤销、导出及游戏再次保存回读。


- Execute Mercury 1.2 ordinary trainer construction in read-only isolated RAM,
  with explicit scenario seeds and independently verified generated values.
  Preserve unknown FireRed hidden-item quantity/receipt packing; improve alternate
  reward selection and collection goals for forms with parsed permanent sources.
- 水银 1.2 新增隔离内存中的原生普通配队预览，明确情景种子并逐项独立验证。
  火红系隐藏道具数量及领取打包方式保留未知；改善奖励分支选择及存在已解析
  永久来源的形态收集目标。完整入战设置和特殊设施仍待验证。

- Audit actual capabilities across all five exact fingerprints. Add runtime
  acquisition queries, static map entrances/connections and target-tile navigation,
  reference back navigation, SAV receipt overlays with verified rules, and regional
  collection suggestions with standalone HTML export. Unknown conditions remain
  explicit; planning and ROM references never write save or ROM data.
- 审计五份精确 ROM 的实际能力；新增实时来源查询、静态入口／连接与目标格位
  定位、资料返回、按已验证规则叠加 SAV 领取状态，以及区域收集建议和独立 HTML
  导出。未知条件明确保留，规划和 ROM 资料均不写入 ROM 或存档。


- Remove Mercury FC 1.1 support; list and accept only the verified 1.2 fingerprint.
- 移除水银 FC 1.1 支持；首页及 ROM 加载只接受已验证的 1.2 指纹。

- Add exact-ROM Mercury FC 1.2 reference: species, moves, items,
  descriptions, evolutions and battle forms, complete move sources, native
  experience thresholds, appearance, maps, timed encounters, trainers and
  ROM-rendered map/NPC artwork. Add core save editing verified against native
  routines, private-copy round trips and a complete 1.2 mGBA load/resave/reopen.
  Expanded Pokédex flags and cheats remain pending
  verification; no ROM bytes or extracted assets are bundled.
- 新增宝可梦水银 FC 1.2 的精确指纹资料：宝可梦、招式／道具及说明、进化与
  对战形态、完整招式来源、ROM 原生经验表、外观、地图、分时段相遇、训练家，
  以及从 ROM 渲染的地图和 NPC。核心存档编辑经过原生函数对照、副本读写回读及
  1.2 mGBA 加载、游戏再次保存和回读验证；扩展图鉴位与金手指暂不开放。
  不内置 ROM 字节或导出素材。
- Verify Mercury's unencrypted individual layout against both ROMs' native
  getters/setters and stat routines. Keep its ball byte separate from Gigantamax
  flags; distinguish PID-selected ordinary abilities from its hidden-ability bit.
- 逐字段对照两版水银的原生读写及能力值例程，采用不加密的个体布局；捕获球
  与超极巨化标记分开，普通特性按 PID 选择，隐藏特性按独立标记处理。

- Add exact-ROM persistent player-side Protect for all four supported ROMs.
  Four CodeBreaker lines OR the native Protect bit for both player-side battler
  slots only while a battle callback exists. mGBA single-battle comparisons
  confirmed blocked Protect-affected attacks and preservation of other turn
  flags; complete doubles, facilities and mobile emulators remain unverified.
- 四款受支持 ROM 新增精确指纹绑定的「己方持续守住」。四行 CodeBreaker 仅在战斗
  中为己方两个场上位置按位加入原生守住标记，保留其他临时状态。mGBA 单打对照
  验证了可守住攻击被拦截；双打完整流程、设施和手机模拟器仍待验证。

- Add exact-ROM emergency battle-party recovery to Dark Phantom BW/DP, Team
  Rocket 2.1 Chinese and Ultimate Emerald 5.5.
  At the player's command menu, L+R+SELECT invokes the ROM's native whole-party
  heal without an item turn, then synchronizes active battlers and health bars.
  The complete GameShark V1/V2 groups were exercised in mGBA single battles for
  all four ROMs and doubles for Ultimate, including fainted reserves, guard
  conditions and live removal.
  Authored payload and reproducible build check are included; ROM files remain
  read-only. Mobile emulators and full battle-facility runs remain unverified.
- 漆黑的魅影 BW／DP、西班牙火箭队 2.1 汉化版和究极绿宝石 5.5 均新增按完整 ROM
  指纹绑定的战斗紧急整队恢复金手指：己方指令阶段按
  L＋R＋SELECT，调用本作原生全队治疗，无需消耗吃药回合，同步场上数据与血条。
  mGBA 四款单打、究绿双打验证包含昏厥后备队员、保护条件和即时停用；附可重建的自编指令。
  ROM 文件保持只读，手机模拟器与完整设施流程仍未验证。

- Add an exact-ROM Ultimate Emerald cheat correcting the reversed Casual/Lunatic
  accuracy-bonus side. Keep the original +20% multiplier and other difficulty
  rules; verify encoded GameShark codes in mGBA and the native accuracy branches.
- 究极绿宝石新增精确 ROM 绑定的命中修正金手指，纠正养生／疯子模式加成阵营写反的
  问题，保留原有相对 +20% 倍率及其他难度规则；验证 mGBA 加密码与原生分支。
- Remove ROM writing from the app, CLI and core. ROM reference remains read-only;
  only save data can be edited and exported. Reject `.gba` export paths. Verify
  combined Speed IV/EV and ability edits survive one save transaction on all
  supported ROMs.
- 移除应用、命令行和核心中的 ROM 写入功能；ROM 资料保持只读，只修改、导出存档。
  禁止以 `.gba` 路径导出。逐一验证四款受支持 ROM 的暴鲤龙：速度个体值／努力值
  与特性可在同一存档事务中保存。
- Calculate Ultimate Emerald trainer IVs and EVs by executing the loaded ROM's
  bounded native constructor for the selected difficulty and player party.
  Use the opened save party or a manually entered battle scenario; display
  possible random outcomes and allow per-opponent level input.
- 究绿训练家按所选难度和玩家同行执行当前 ROM 的原生个体值／努力值例程，
  可使用已打开存档的队伍或手动设置对战情景，显示随机分支并可逐只指定模拟等级。
- List every supported ROM from the registered adapters on the start screen.
  Cheats now belong only to the opened ROM; their panel cannot import another
  ROM and is no longer linked from ROM reference. Add a shared four-mode
  Ultimate trainer difficulty selector with explicit source-party and generated
  stat boundaries. Label Charizard's item-702 form as Gigantamax appearance.
- 首页从适配器自动列出全部受支持 ROM；金手指仅对应当前打开的 ROM，不再在
  窗口内另选 ROM，也不从 ROM 资料跳转。究绿训练家资料增加四档通用难度选择，
  区分 ROM 基础队伍与实战生成数值；喷火龙携带 702 对应的形态标为超极巨化外观。
- Add an exact-fingerprint Ultimate Emerald 5.5 adapter to the shared ROM reference
  and save editor. Read expanded species, moves, abilities, learning sources,
  evolutions, battle transformations, trainers and maps from the supplied ROM.
  Read enhanced trainer templates without presenting difficulty-dependent EVs as
  fixed values. Unreviewed official-species mappings remain unconfirmed.
- 究极绿宝石 5.5 按完整指纹接入共用 ROM 资料和存档修改界面，实时读取扩展宝可梦、
  招式、特性、学习来源、进化、战斗变身、训练家和地图。强化队伍展示 ROM 培养模板，
  不将受难度影响的努力值误报为固定值；官方参照映射未经审核时不冒充已确认。
- Preserve Ultimate's plain Pokémon records, disabled native checksum fields,
  mint nature, hidden ability and Hyper Training bits. Support expanded inventory
  in sector extensions and its shifted Pokédex bits; retain ordinary editor
  transactions, storage transfers, preview, undo/redo and safe export.
- 适配究绿明文个体、原生停用的校验字段、薄荷性格、隐藏特性与极限训练标记，支持
  扇区扩展背包和不同的图鉴位布局，沿用编辑事务、盒子转移、预览、撤销重做与导出。
- Add portable PC, walking-encounter suppression, guaranteed wild capture, faster
  hatching, species/level encounters, shiny wild encounters and map teleport for
  Ultimate, alongside its existing all-mode no-peeking code. Encounter/shiny hooks
  are isolated from other ROMs and retain native generation constraints.
- 究绿在全模式去窥屏之外新增随身电脑、暂停走路遇敌、野生必捕、快速孵蛋、指定遇怪
  与等级、野生闪光、地图传送；遇怪与闪光使用独立适配，保留原生个体生成约束。
- Version-specific verification and known boundaries:
  [Ultimate Emerald 5.5](docs/research/ultimate-emerald-55.md).
- 本次未制作发布安装包；具体验证范围见上述适配说明。

## 0.2.2 — Development builds / 开发构建

- Add a seven-line portable Pokémon PC cheat for Dark Phantom BW/DP and Team
  Rocket: SELECT opens the native storage menu while walking. Preserve native
  facility guards and registered-item data; provide bilingual usage and full-set
  copy/export through the existing cheat window.
- 漆黑的魅影 BW／DP、西班牙火箭队新增 7 行随身电脑金手指：自由行走时按 SELECT
  打开原生宝可梦电脑，保留原生设施限制与登记道具，沿用金手指窗口的双语说明、
  整组复制及导出。

- Add species/level encounters, shiny ordinary wild encounters and map teleport
  for Dark Phantom BW/DP and Team Rocket. Read Pokémon and map choices from the
  loaded ROM; provide searchable names, a Region → Map selector, hex map codes
  and referenced entrance tiles. Validate parameters in Rust and invalidate stale
  generated codes. Preserve native Pokémon generation and test final encrypted
  code sets in mGBA; no save or ROM file is modified by the catalog.
- 漆黑的魅影 BW／DP、西班牙火箭队新增指定遇怪与等级、普通野生必闪和地图传送。
  宝可梦及地图列表实时读取 ROM，支持名称搜索、区域→地图二级选择、十六进制地点
  编码和入口格位。后端校验参数，切换目标立即作废旧代码；保留原生个体生成流程，
  使用最终加密代码进行 mGBA 验证，金手指目录不修改 ROM／存档文件。


- Add a read-only, exact-ROM cheat window with bilingual guides, scoped verification,
  copy/export and a shared Rust/CLI generator. Includes all-mode AI input-peeking
  suppression for Ultimate Emerald 5.5; walking-encounter suppression, guaranteed
  wild capture and faster hatching for Dark Phantom BW/DP and Team Rocket;
  compatible daycare egg production for Team Rocket. Exact-ROM native mGBA tests
  cover original-byte restoration, encrypted individuals and feature guard conditions.
  The cheat-only ROM context stays independent of the editor session.
- 新增只读金手指窗口，按完整 ROM 指纹匹配，提供中英文使用说明、验证范围、整组
  复制与导出，Rust 核心和 CLI 共用生成逻辑。支持究极绿宝石 5.5 全模式关闭 AI 窥屏；
  漆黑 BW/DP、西班牙火箭队暂停走路遇敌、野生投球必定捕获、加快同行蛋孵化；
  西班牙火箭队兼容寄养组合必定产蛋。逐 ROM 通过 mGBA 原生函数、个体校验、启停
  恢复与限制条件回归；金手指 ROM 与编辑器会话独立，手机模拟器尚未实测。

- Move contest condition to Stats and label Sheen as Fullness. Show feeding
  saturation and add an optional ROM-backed solo NPC bound check for Dark
  Phantom BW/DP, enforced in the core with an explicit free-edit override.
  Passing necessary bounds is not presented as proof of a reachable recipe.
- 华丽值移至“能力”，将“光泽”明确标为“饱腹度”，展示满值喂食限制；漆黑 BW/DP
  新增实时读取树果与 NPC 配方的单机喂食边界校验，后端同步校验，自由编辑保留
  提示。不会把“未被必要条件排除”误称为正常可达，西班牙火箭队不套用漆黑配方。

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
