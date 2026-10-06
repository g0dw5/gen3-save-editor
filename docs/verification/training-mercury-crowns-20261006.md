# Mercury 1.2 crown service / 水银 1.2 王冠服务

Exact registered fingerprint `f323df1792ac68462a34b42fe8571533`, readonly queries.
The shared training page now links the runtime service to requirements, item
acquisition, prerequisite clues and NPC/map/entrance navigation. This increment
adds no SAV writes, generated individuals or ROM patching. Overall coverage stays
partial. Do not equate these field/command probes with a full NPC interaction.

本增量形成只读查询、条件、所需道具获取及 NPC 地图入口跳转流程，不增加存档写入。
完整服务仍为部分验证，字段和命令证据不等同于完整 NPC 操作。

## Current referenced script

NPC 20 at `(14,10)` on map `46-0` references root `0x7b05ba`. The root tests flag
`0xb15` and enters service `0x7b000a` on its set branch. It otherwise enters an
alternative story script. The old root `0x7b0000` having no direct map reference
is not evidence that the called service is unused. NPC positioning is not proof
of present access; complete unlock progression and alternate story remain partial.

The adapter contains script/function layout addresses; map/actor names, text,
menu labels, item IDs/quantities, unlock ID and threshold are current-ROM reads.
The menu injects runtime strings through special `0x25`. Shared choices preserve
all-stat versus six single-stat operations, without a bundled crown/stat catalog.

Level checks at `0x7b07b0` / `0x7b099b` both compare selected level with **50**.
The reader trampoline `0x0896a730` reaches `0x09d5be60` and reads the selected party
individual; its result is in variable `0x8006`. Cancellation sentinel is selected
slot 7. Full party selector eligibility (including eggs) is not certified here.
The SAV unlock flag lives in Mercury's extension storage, not the Emerald main
flag block; overlays use the existing segmented event reader.

当前被地图引用的 NPC 服务检查解锁标记和至少 50 级。定位、引用、持有条件与
当前可达分开。扩展领取／剧情标记按水银自己的扩展存储读取，不能套用绿宝石位置。

## Field mechanism: base IVs, not training flags

Native special `0x10` points to `0x09d5c1d8`. Party mode (`0x8003=0`) uses selected
slot `0x8004`, stat selector `0x8005` and target `0x8006`. Selectors 0–5 change one
base IV; diagnostic selector 6 changes all six. The actual gold script calls the
six single-stat setters in succession; silver dispatch selects one. Native calls
cap the written value at 31 and invoke current-ROM stat calculation immediately.
The referenced service scripts request 31. No Hyper Training header bits are used.

Independent cases preserve PID, trainer identity, nickname, moves/PP, EVs,
friendship, source/history, ribbons, status and egg/hidden-ability bits. Only base
IVs and computed party HP/stats change; other five party individuals remain exact.
These bytes are native field effects, not a claim that editor recalculation,
checksums, inheritance or complete menus are equivalent in every context.

原生效果直接改基础 IV，并重算同行能力／HP；不是究绿的训练位。PID、身份、招式、
努力值、来源等无关字段及其他同行保留。完整编辑等价性、遗传与交易并未由本次验证。

## Required item and attempted payment are different

| Path | Check | Removal command | Timing |
|---|---|---|---|
| All stats | item 640 ×1 | item 640 ×1 | After party/level checks |
| Single stat | item 639 ×1 | item 640 ×1 | After party/level checks, before stat menu |

Both item IDs resolve from current ROM data; their native ordinary pocket is 1.
`ScrCmd_checkitem` and `ScrCmd_removeitem` execute full native handlers using the
actual ROM operands. With silver present and gold absent, the silver check returns
success but removal returns failure and leaves silver intact. With gold present,
the removal subtracts gold and leaves silver intact. The next silver instructions
set up stat-menu labels, without a branch checking the removal result. The UI
therefore calls this an attempted payment and warns about the mismatch/order.
This is an exact-ROM script anomaly, not a modification by the editor.

No complete menu/gameplay transaction or cancellation/refund path is claimed.
Cancelling the later stat menu must not be assumed to refund an earlier attempt.
No new cheat/ROM patch or automatic inventory adjustment is provided.

银色分支检查银冠，尝试扣金冠：只有银冠时检查成功、扣除失败且银冠保留；两者都有
时扣掉金冠。扣除之后立即设置单项菜单，没有先依据返回值退出。资料页分别展示
所需／尝试支付和发生顺序，不把完整操作、取消返还等未验证路径包装成结论。

## Independent evidence and regression

`scripts/verify_mercury_crowns.py` uses complete mGBA calls in disposable RAM:
no hooks, no source SAV, no ROM writes. Fixtures read runtime species and experience
records and establish valid calculated levels, then vary three synthetic species,
levels 49/50/100, PID/ability scenarios and all six party slots.

- **216** level reads and **432** native script comparison/branch decisions.
- **1,512** complete base-IV/stat calls: six slots × seven selectors × 36 fixtures.
- **216** actual six-call gold sequences agree with the all-stat helper on all 600 party bytes.
- **4** invalid selected-slot cases leave all party bytes intact and return zero level.
- **32** bag check/removal cases, including absent, 1, 2 and 65535 quantities.
- Source ROM SHA-256 stays unchanged; private vectors contain no shipped catalogs.

The opt-in core regression replays independent vectors with the bounded readonly
runner and compares all 600 party bytes, level branches and bag command results.
The service reference verifies runtime choices, NPC position, distinct mechanism,
extension-flag overlays (unset/set), ordinary bag holdings (0/1/2), and unchanged ROM-only/SAV queries. Existing five-profile
service regression keeps Ultimate's 10,752 training-flag vectors and leaves
BW/DP/Rocket unconfigured rather than assuming absence.

Bilingual browser fixtures test search, required/payment distinction, warning,
NPC/map/item/back navigation, SAV refresh and clearing service state on ROM switch.
These fixtures are UI evidence, not emulator gameplay. Tight-window QA checks
scroll width; extracted ROM images are not included in the fixture.

```sh
/usr/bin/python3 scripts/verify_mercury_crowns.py \
  --gen3 /path/to/gen3 --mgba-probe /path/to/ability-probe.dylib \
  --output /private/mercury-crowns-native.json
GEN3_MERCURY_CROWN_PROBES=/private/mercury-crowns-native.json \
  cargo test -p gen3-core local_mercury_crown_services_match_native_fields_and_payment_anomaly \
  -- --ignored --nocapture
```

Validation at this increment: **122** public core tests pass (**40** opt-in tests
ignored in the public run); both exact-ROM crown regressions pass. Five-profile
bilingual UI fixtures pass, with the corrected Mercury post-SAV-reload selection
scenario checked separately. Workspace check, TypeScript/production build,
format checks and five release-workflow checks pass. Compact 720 px screenshots
show opaque readable service text without horizontal overflow. The existing Vite
large-chunk advisory remains. Version is held; no new application is packaged.

Full UI/transaction replay, all party-selector guards, unlock story/access,
alternate bag contexts, all-IV duplicate payment behavior and a newly edited SAV
emulator round trip remain unverified. Service-only reference expansion does not
remove these gaps or prove the overall project objective complete.
