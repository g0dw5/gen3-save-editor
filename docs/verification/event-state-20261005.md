# Persistent events and hidden items / 持久化事件与隐藏道具

This increment is read-only and scoped to the five registered fingerprints.
`event_state.rs` reads configured persistent ranges from logical SaveBlock1,
SaveBlock2 or Mercury extensions. No event flags are edited. Names, map records,
hidden-item quantities and Mercury's region selector are read from the loaded ROM.

本轮补齐可验证编号范围内的存档条件与隐藏道具领取叠加。查询不修改剧情或领取标记，
不因背包里没有某道具就判断未领取。所有范围均按精确 ROM 配置，临时／未验证编号
保持未知；地址证明不能替代完整任务与领取协议的证明。

## Native evidence / 原生证据

All offsets below omit the GBA ROM base `0x08000000`.

| Profile | Native getters | Persistent flags | Persistent variables |
|---|---|---|---|
| 漆黑 BW／DP | FlagGet `0x9D790`, VarGet `0x9D694` | SB1 `0x1270`, IDs 1–`0x3FFF` | SB1 `0x139C`, IDs `0x4000–0x40FF` |
| 西班牙火箭队 | `0xD3AE4`, `0xD397C` | SB1 `0x1CA8`, IDs 1–`0x3FFF` | SB1 **`0x1F6C`**, IDs `0x4000–0x40FF` |
| 究极绿宝石 | `0x9D790`, `0x9D694`; flag resolver hook `0x1F00CEC` | Ordinary SB1 `0x1270`; extended ranges below | SB1 `0x139C`, IDs `0x4000–0x40FF` |
| 水银 1.2 | `0x6E6D0`, `0x6E568`; wrappers `0x1CCD198`, `0x1CCD170` | SB1 `0xEE0`, IDs 1–`0x8FF`; extensions `0`, IDs `0x900–0x18FF` | SB1 `0x1000`, IDs `0x4000–0x40FF`; extensions `0x200`, IDs `0x5000–0x51FF` |

Ultimate extended flags map as follows: `0x4000–0x419F` → SB1 `0x988`;
`0x41A0–0x433F` → SB1 `0x3B24`; `0x4340–0x44DF` → SB2 `0x5C`;
`0x44E0–0x467F` → SB2 `0x28`. These are four disjoint native ranges, not a
flat continuation of the ordinary array. Mercury's custom helpers return sentinels
that the wrapper resolves through stock code; interpreting helper output alone
would give incorrect addresses for low IDs. Special/RAM-only ranges are not guessed.
Flag zero remains unknown in queries: object-template zero is a sentinel, and
Ultimate's raw native handling differs from the other games.

究绿扩展标记分散在两个存档块，水银同时保留原版区和扩展区。按逻辑块组装后读取，
不把闪存中相邻的物理扇区误当作连续数据。水银的自定义辅助函数返回的哨兵还会经
包装函数转入原版路径，不能只看辅助函数的返回值猜偏移。

## Hidden-item flow / 隐藏道具流程

BW/DP/Ultimate hidden-item position routine `0xFD6D4`, and Rocket `0x1359C4`,
load the 16-bit index at background event +10, add `0x1F4`, and refuse the item
when native FlagGet returns true. The configured decoder uses this same protocol.

Mercury redirects `GetHiddenItemAttr` `0xCC44C` to `0x1D3F4BC`. The packed word
contains item ID (low 16), index (next 8), quantity (next 7), underfoot (top bit).
Its flag base is **`0xD00`** when the map section appears in the current ROM's
`0x1DDDB6E` byte list (terminated by `0xFF`), otherwise **`0x3E8`**. The list is
read at runtime, not copied into an adapter catalog. Regular interaction at
`0x6D17A` refuses underfoot items. The native underfoot setup `0x13EF40` forces
quantity **one**, ignoring the packed quantity, and uses the Itemfinder flow.

水银普通隐藏道具从面前交互；脚下道具站在目标格使用探测器。脚下道具固定获得一个，
不按打包字段的数量发放。这些取法说明同时出现在地图、获取途径、收集建议与 HTML。
区域列表实时读取 ROM，因此不跨版本缓存成固定内容。

With SAV, verified hidden flags show collected/currently uncollected state in
acquisition and collection planning. A cleared flag does not prove map access.
Hidden repeatability remains unknown because refresh/flag-reset mechanisms are
not completely indexed. NPC visibility does not become a receipt flag. Ordinary
item-ball receipt is still enabled only for independently verified BW/DP standard
scripts; the other three profiles retain unknown pickup receipt until their
standard reward scripts have separate native proof. Unknown/native script stops
also prevent a definitive blocked/available claim for item paths.

## Verification / 验证

- `verify_event_state.py`: **150,774** native comparisons over every configured
  flag/variable ID with two address-sensitive fills: BW 33,278; DP 33,278;
  Rocket 33,278; Ultimate 36,606; Mercury 14,334. ROM inputs and synthetic RAM
  remain unchanged. Private output contains parity vectors only and is not shipped.
- `local_event_state_matches_native_ranges`: compares the shared Rust reader,
  after SAVE sector assembly, to private native vectors for all five fingerprints.
- `verify_hidden_items.py`: each Emerald-layout profile has 12 native position/
  receipt checks plus coordinate rejection. Mercury has **4,096** attribute checks
  (256 regions × four packed vectors × four attributes) and **four** underfoot
  consumer checks. The latter skips only text formatting; quantity is written by
  the actual consumer. No ROM or SAV is written.
- Public tests assemble checksum-valid synthetic saves per codec, cross logical
  sector boundaries, verify all configured range reads and preserve every byte.
  Receipt tests separate hidden receipt, verified pickup and NPC visibility, keep
  no-SAV queries unknown and avoid claiming blocked status from incomplete paths.
  Map tests mutate a synthetic ROM region list to prove runtime ownership.
- Full core suite: **90 passed, 19 opt-in tests ignored**. Native event parity
  and five-profile local query checks were run separately with private inputs.
  TypeScript, production frontend build, Rust formatting and UI formatting passed.
  Clippy passed with the previously documented baseline lint allowances; this is
  not a claim of a warning-free strict Clippy run.
- Browser checks passed for map layers and underfoot instructions, collection
  HTML, both interface languages' saved-clock scenarios and clock isolation when
  switching from Mercury to the other four fingerprints. No SAV mutation was sent.
  Real-ROM query/map UI smoke also passed for all five fingerprints. The first
  attempt stalled in the development bridge; restarting that local test bridge
  and rerunning the unchanged test completed successfully.
- The macOS arm64 local preview was compiled, copied from the network workspace
  to local storage without extended attributes, ad-hoc signed and verified with
  `codesign --verify --deep --strict`. This exact app launched with all five
  fingerprints on the opening screen. It contains no ROM/SAV and no ZIP/DMG.

Reproduce with all five private `GEN3_ROM_*` paths:

```sh
uv run --with unicorn python scripts/verify_event_state.py --output /private/events.json
uv run --with unicorn python scripts/verify_hidden_items.py
GEN3_EVENT_PROBES=/private/events.json cargo test -p gen3-core local_event_state -- --ignored
```

## Remaining scope / 剩余范围

This establishes selected persistent addressing and hidden-item protocols, not a
complete quest graph, all reward scripts, current map reachability, recurring-item
resets, temporary runtime variables or quest-log playback. Ordinary pickup and
NPC gift receipts still need per-ROM protocol proof as described above. Ordinary
item-ball qualification is now extended by the later
[pickup increment](pickup-receipts-20261005.md); the remaining pickup gap concerns
compound/custom scripts rather than all three newer profiles. This
increment changes no SAVE mutation/export path and claims no new emulator edit
round trip. Existing edit safety regressions remain required.
