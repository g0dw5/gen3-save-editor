# Ordinary item-ball receipts / 普通道具球领取

This read-only increment extends acquisition and collection overlays to native-
verified ordinary item-ball scripts in all five exact fingerprints. The ROM
remains the source of script bytes, item IDs, quantities, coordinates and flags.
It does not add an item catalog or write any ROM/SAV data.

本轮把五份精确 ROM 的普通道具球接入存档领取状态，并据此从收集建议中排除已经
置位的领取记录。仅匹配完整、无额外操作的普通道具球脚本；复杂事件、NPC 奖励、
未知脚本继续单独保留缺口。对象消失标记本身不构成领取证明。

## Protocol and runtime qualification / 协议与运行时判定

The verified ordinary script assigns constant item and quantity to temporary
variables `0x8000` and `0x8001`, calls standard script 1, then ends. Native standard
script 1 checks bag space and preserves that result, branches accordingly, and
only its successful branch removes `LAST_TALKED` (`0x800F`). Native object removal
looks up the corresponding template and calls `FlagSet` with its flag. The native
flag getter observes the resulting bit. A full-bag branch returns without setting
this receipt flag. The removal occurs **before** the subsequent native `additem`
command; the verifier does not claim that it occurs after bag insertion.

运行时还检查整个脚本形状：只允许两个常量赋值、调用标准脚本 1、结束。修改
LAST_TALKED、原生调用前置、额外效果、动态数量、零数量、缺少领取标记，以及
普通赠送脚本都不继承这个领取协议。每个地图标记分别输出 `receipt_flag`，与
原始 `flag` 分开；查询不会再直接把所有拾取对象的可见性标记当成领取标记。

| Exact profile | Command table | Native FlagSet | Parsed pickup markers | Qualified receipts | Native branch cases |
|---|---:|---:|---:|---:|---:|
| 漆黑 BW | `0x1DB67C` | `0x9D740` | 122 | 122 | 244 |
| 漆黑 DP | `0x1DB67C` | `0x9D740` | 122 | 122 | 244 |
| 西班牙火箭队 | `0x22B218` | `0xD3A6C` | 804 | 804 | 1,608 |
| 究极绿宝石 | `0x1DB67C` | `0x9D740` | 429 | 402 | 804 |
| 水银 1.2 | `0x15F9B4` | `0x6E680` | 513 | 506 | 1,012 |

Addresses omit ROM base `0x08000000`. Counts describe parsed records, not proof
of current map access, referenced-story coverage or unique obtainable rewards.
Ultimate's callstd handler redirects through `0x14A332C`; native execution follows
this hook instead of assuming an unmodified Emerald handler. Mercury's standard
script 1 is selected through the current ROM table at `0x160450`, entry
`0x1CCEBA0`; object template lookup also follows the custom hook at `0x1D2F6F4`.
Runtime metadata is not replaced with extracted script snapshots.

## Verification / 验证

`scripts/verify_pickup_receipts.py` obtains map records from the developer CLI and
executes exact-ROM native handlers in isolated synthetic RAM. It covers **1,956**
qualified records and **3,912** branch cases, with independently compared native
FlagGet/FlagSet and item/quantity temporary variables. RAM holding event state is
intentionally changed by native FlagSet; emulated ROM memory and input files are
verified unchanged. No SAV is opened. Private JSON vectors are not shipped.

The harness injects bag-space outcomes. It bypasses text, sound, waits, presentation
and non-receipt special bookkeeping. Mercury's pre-receipt special `0x9A` and
text-color command `0xC7` additionally execute natively, rather than being skipped.
Successful execution stops immediately after native object removal, before bag
insertion and later bookkeeping. Thus the
result proves the bounded standard-script receipt branch, **not** actual inventory
capacity, complete quest-log playback, every native special or a full emulator
field interaction. Full-bag execution returns through the real control handlers.

`local_pickup_receipts_match_native_protocol` compares all qualified Rust map
receipt IDs with the independent native vectors for all five ROMs. Public tests
also reject compound scripts, changed NPC identity, unverified adapters, dynamic
quantities and missing flags. Query tests ensure set visibility without receipt
proof stays unknown, out-of-range receipt IDs stay unknown, no-SAV state stays
unknown, completed receipt sources are excluded from collection plans and all
SAVE bytes are preserved.

核心测试 **91 通过、20 项按需测试默认忽略**；原生领取向量与五 ROM 查询／地图
拓扑检查另行执行并通过。新增的收集过滤断言也单独通过。TypeScript 与采用已记录
历史 lint 豁免的 Clippy 通过；不宣称严格 Clippy 零告警。前端生产构建通过，保留
既有超过 500 kB 的包体提示。地图图层／取物说明及查询—地图—入口—返回—收集
HTML 的浏览器流程通过；新增的未验证领取说明另做中英文切换检查。
中英文切换检查也已通过，查询、地图定位、返回和 HTML 导出过程中没有发出存档
修改／导出请求。地图证据中可展开查看独立的领取标记。

Reproduce with all five private `GEN3_ROM_*` paths and the current CLI build:

```sh
cargo build -p gen3-cli
uv run --with unicorn python scripts/verify_pickup_receipts.py > /private/pickup-vectors.json
GEN3_PICKUP_PROBES=/private/pickup-vectors.json cargo test -p gen3-core local_pickup_receipts -- --ignored
cargo test -p gen3-core local_query_ -- --ignored
```

The upstream [Emerald script commands](https://github.com/pret/pokeemerald/blob/master/src/scrcmd.c)
and [FireRed script commands](https://github.com/pret/pokefirered/blob/master/src/scrcmd.c)
help locate candidate mechanisms; all enabled adapter behavior is checked against
these exact local ROMs, including their hooks.

## Boundaries / 边界

A set receipt flag is interpreted within its proven item-ball protocol; it does
not certify a whole task or story chapter. Retained/unreferenced maps, shared
flags, story initialization, alternate layout replacement and scripts that clear
flags still need broader reference/dependency analysis. An unset flag means the
verified receipt is currently clear, not that the player can reach the tile or
has enough bag space. UI uses “Parsed conditions met” for that narrower meaning.
Reset/refresh mechanisms are not exhaustively indexed, so ordinary item-ball
sources no longer promise single-time-only receipt. Unknown receipt rules show a
human-readable explanation in both languages; technical evidence remains expandable.

领取位的设置只证明该协议下当前的记录状态，不据此宣布整段剧情完成；旧地图、
共享标记、剧情初始化、动态布局与标记重置仍需继续核对。查询和收集 HTML 保留
访问／条件覆盖提示。NPC 对话领奖不能用消失标记或背包有无来替代。此轮不改变
存档编辑、导出、安全备份，不新增模拟器写入回读结论，不打包、不发布、不改版本号。
