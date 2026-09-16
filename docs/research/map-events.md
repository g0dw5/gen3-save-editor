# Map event overlays / 地图事件图层

The map reference reads every configured ROM map without consulting a save.
Independent layers show item balls, hidden items, dialogue/event rewards and
NPC/object template positions. Pins use tile centers `(x + 0.5, y + 0.5)` relative
to the map layout, without the engine's runtime border. Overlapping events share
a pin; details retain all rewards. The interface supports search, grid lines,
zoom and English/Chinese labels.

地图资料显示 ROM 的全部事件，不按存档领取状态过滤。地面精灵球、隐藏道具、
对话／事件奖励、NPC／地图对象可独立显示和隐藏。坐标从左上角 `(0,0)` 开始，
标记落在格子中心；同格事件合并但不丢失各自的奖励信息。

## Data and evidence

- Object templates: 24 bytes; signed x/y at +4/+6, elevation +8, movement +9,
  script pointer +16, visibility flag +20. Movement 0x4C starts invisible.
- Background events: 12 bytes; x/y +0/+2, elevation +4, kind +5. Kind 7 stores
  item ID at +8 and hidden-item index at +10. The collection flag is index +0x1F4.
  This union is **not** a script pointer. Secret bases are not hidden items.
- Coordinate events: 16 bytes, script +12. Their trigger variable/value is
  retained as a reward condition.
- Object visibility flags are not automatically interpreted as NPC gift receipt
  flags. The latter require inspecting the actual reward branch and save state.
- Standard script 0 gives an item; standard script 1 picks up an item ball.
  Standard script 7 gives a decoration, whose ID belongs to a different catalog.

The bounded reward walker follows calls, returns and branches with per-path
constants. It retains flag/variable guards, invalidates values after native calls,
handles variable-length trainer battles, and reports unresolved offsets. Unknown
commands and unsupported control flow stop traversal instead of scanning bytes
for apparent item opcodes. Native calls are recorded as coverage gaps even when
traversal resumes afterward. Calls are capped at 16 levels, paths at 32 guards,
and each root at 8192 steps.

NPC markers are **initial template positions**, not a simulation of map-load
relocations, movement, temporary flags, battle results or prerequisite completion.
Unplaced map-script rewards are listed separately. Alternatives in a script do
not mean that every listed reward can be obtained together. Random facility
floors, shops, native-code rewards, berries and other recurring activities do not
necessarily have a permanent “ever collected” bit.

NPC 标注的是初始位置，不能用来保证当前可见、可达或已经满足剧情条件。
地图加载后的移动、随机设施、原生程序内部发放奖励等尚未完整模拟。未解析的
来源明确保留为未知；不能用背包里没有某道具证明玩家从未拿过。

## Verification

Synthetic Rust tests cover signed/template coordinates, hidden-item flag mapping,
invalid pointers, calls that set reward variables, receipt-flag branch evidence,
decoration exclusion, unknown commands and native-call invalidation.
The optional exact-ROM regression checks both supported BW/DP fingerprints,
including all 707 maps, 122 ordinary item-ball points and 112 hidden-item points,
plus independent representative gift/pickup distinctions.

`python3 scripts/test_map_events.py` tests the UI using synthetic responses:
layer toggles, overlapping markers, tile-center alignment, item search, grid and
zoom. Start Vite first; Playwright and Chrome are required. No ROM/SAV is embedded.

References: [event layouts](https://github.com/pret/pokeemerald/blob/master/include/global.fieldmap.h),
[opcodes](https://github.com/pret/pokeemerald/blob/master/src/scrcmd.c),
[standard script table](https://github.com/pret/pokeemerald/blob/master/data/event_scripts.s),
[item scripts](https://github.com/pret/pokeemerald/blob/master/data/scripts/obtain_item.inc).
The source describes the engine format; supported-ROM regression and runtime
ROM extraction supply version-specific values. User saves, extracted world dumps
and personal acquisition reports stay in ignored local analysis directories.
