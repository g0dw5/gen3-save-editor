# NPC Hyper Training / NPC 极限特训资料

> Historical research / 历史逆向记录：the dedicated four-tab feature and its
> commands/tests were removed during reference cleanup. Procedures below
> describe commit `f5b085c`, not the current application. Findings and limits
> remain evidence; use [current key addresses](../research/reference-key-addresses.md)
> for subsequent investigation. 本文旧页面／命令已撤去，不是当前产品使用说明。


Scope: exact registered ROMs, read-only reference queries. Overall training stays
partial. This increment does not change SAV editing, create individuals, spend
items/credits or certify full menus/current access.

范围：精确指纹的只读资料查询。整体培育仍为部分支持。本增量不改变存档编辑，
不生成个体、不支付王冠／认证，不声称完整菜单或当前可达已经验证。

## Runtime service identity / 服务身份

Ultimate's referenced script root `0x1812758` belongs to NPC 19 at tile `(3,39)`
on map `35-28`. Map names, positions, menu strings, item IDs/quantities, minimum
level and condition IDs are read from the supplied ROM. Map references do not
prove reachability; the shared map screen retains incoming/exterior approaches.
The adapter contains verified layout/function addresses, not extracted content.

The runtime seven-choice menu is selected by script opcode `0x6f`; its table
at `0x1700000` and item branches establish one all-stat operation and six selected
stat operations. The native mask helper determines each selected-stat mask.
Both branches require the crown **and** an additional persistent earned-credit
counter to be nonzero. Successful changing operations subtract one counter unit
and remove one crown. Holding an item alone cannot establish eligibility.

究绿引用的脚本位于地图 `35-28`、NPC 19、格位 `(3,39)`。地图名称、NPC 坐标、
菜单文字、费用道具与数量、等级阈值、条件编号均来自当前 ROM。原生菜单有一项
全部能力和六项单能力选择。两个分支都检查王冠和额外的持久化认证计数，成功后
各消耗一次。背包有王冠不能单独证明可以接受特训；认证的完整获取途径尚待验证。

## Native semantics / 原生字段逻辑

| Stage | Native entry / verified script |
|---|---|
| Selected individual level-byte reader | `0x098126d8`; script compares against runtime threshold 100 |
| Selected-stat mask | `0x09812720`; native menu index 1–6 selects raw-header bits 1–6 |
| Training marker update | `0x098126fc`; ORs the mask into individual header byte 30 |
| Gold prerequisite/payment | variable `0x40fb`, runtime item 687; both consumed on changing success |
| Silver prerequisite/payment | variable `0x40fc`, runtime item 688; both consumed on changing success |

The marker helper preserves header bits 0 and 7, canonical substructures, base
IVs, PID, trainer identity, party status/HP/stats and the other five individuals.
It records before/after marker bytes for the script's duplicate-operation check.
When the mask adds no new flags, the script skips payment and takes its rejection
dialogue. This is a flag-change test, not a comparison with the base IV value.
The persistent helper does **not** calculate party stats; the ROM's completion
dialogue asks the player to deposit the individual in a PC. The reference keeps
this distinction visible instead of promising an immediate six-stat refresh.

字段步骤仅给头部第 30 字节加训练位，保留隐藏特性位和保留位，不改变基础 IV、
PID、身份、招式、来源、努力值、状态、当前 HP、已保存六围或其他同行。没有新增
训练位时，脚本不支付费用而进入重复训练提示；不是判断基础 IV 是否已等于 31。
本步骤不计算同行能力，完成对话要求将个体放入电脑，不能把标记更新等同于六围
已经立刻刷新。完整寄存／取回工作流不由本增量认证。

## Independent verification / 独立验证

`scripts/verify_training_crowns.py` executes complete native helper calls in
disposable mGBA RAM, with no hooks, ROM writes or SAV input. Private vectors are
never committed/bundled. The opt-in core regression replays them with the shared
readonly runner and compares all 600 party bytes:

- **48** byte-reader cases: all six slots, including level boundaries and
  deliberately non-gameplay byte values to verify raw-byte semantics.
- **10,752** mutation cases: six slots × 256 header values × seven menu choices;
  **6,120** change the marker and **4,632** leave it unchanged.
- Mask, pre/post marker bytes, all unrelated individual bytes and other slots
  agree with independent mGBA results; source ROM SHA-256 stays unchanged.
- Five exact-ROM service queries isolate the adapter. Ultimate alone has this
  verified flag-based service reference; Mercury now has a distinct base-IV service. Its seven choice records, NPC tile, ROM dialogue,
  and SAV credit values 0/1/65535 are checked without changing the SAV.
- Public API regression rejects stale fingerprints and retains the empty,
  explicitly partial result for an unconfigured profile.

The bilingual browser regression extends the existing five-profile training
fixtures with seven searchable service choices, item-acquisition/map/back links,
preserved training state, reloading credit overlays and clearing the service on
a ROM switch. A compact selected-choice card replaces repeated full rows. These are synthetic
browser fixtures, not gameplay evidence. SAV overlays use the shared query
revision and stale-response cancellation.

Reproduction requires the private exact inputs plus:

```sh
/usr/bin/python3 scripts/verify_training_crowns.py \
  --gen3 /path/to/gen3 --mgba-probe /path/to/breeding-probe.dylib \
  --output /private/crowns-native.json
GEN3_TRAINING_CROWN_PROBES=/private/crowns-native.json \
  cargo test -p gen3-core local_crown_services_match_native_and_isolate_save_progress \
  -- --ignored --nocapture
```

## Explicit gaps / 明确缺口

Full NPC menu execution, native credit acquisition, facility bag context,
visibility activation, present access and deposit/retrieve/stat refresh are not
fully replayed. The existing free editor training checkbox is separate from this
reference and does not prove a legitimate full NPC transaction. No new edited-SAV
emulator round trip is claimed here.

Mercury 1.2 now has a separately verified base-IV service reference; see
[its distinct mechanism/payment evidence](training-mercury-crowns-20261006.md).
Empty BW/DP/Rocket service results remain **unverified**, not absence. Full
transactions are still partial for both configured service adapters. The adjacent
Ultimate trampoline investigated earlier handles forms/fusion, not crown training.

水银已另行接入基础个体值训练服务，不能沿用究极绿宝石的训练位规则。
漆黑 BW／DP、西班牙火箭队空服务结果仍表示尚未验证，不代表没有机制。
两个已配置服务的完整交易仍有缺口，见水银独立证据说明。
