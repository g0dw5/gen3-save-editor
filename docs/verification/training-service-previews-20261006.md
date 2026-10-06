# NPC training-service effect previews / NPC 培育服务效果预览

> Historical research / 历史逆向记录：the dedicated four-tab feature and its
> commands/tests were removed during reference cleanup. Procedures below
> describe commit `f5b085c`, not the current application. Findings and limits
> remain evidence; use [current key addresses](../research/reference-key-addresses.md)
> for subsequent investigation. 本文旧页面／命令已撤去，不是当前产品使用说明。


Date: 2026-10-06. Scope: exact registered Ultimate Emerald 5.5 and Mercury 1.2
fingerprints. Overall training coverage remains **partial** for all five ROMs.

## Read-only workflow / 只读流程

Training reference → referenced crown NPC → runtime choice → stored party/box
or simulated individual → native field preview. Item acquisition, NPC tile,
entrance and back navigation remain shared with service reference queries.

The API and CLI `gen3 training-service-preview ROM REQUEST.json [SAVE]` require
the expected fingerprint, verified service root, runtime choice index and a typed
individual scenario. Unknown roots/choices and stale fingerprints are rejected.
No ROM or SAV bytes are written; disposable RAM executes only the configured,
verified native helpers. It does not execute the complete NPC transaction.

服务根与选择必须属于当前指纹的已核实记录，不能由客户端指定任意原生地址。
预览只读：选择现有个体不会改动存档，模拟个体也不会生成到存档中。

## Meaning of results / 结果语义

- Ultimate: native minimum-level reader, selection mask and training marker.
  Base IVs and stored party stats stay unchanged. The display distinguishes
  training flags from base IVs and explains deferred stat refresh.
- Mercury: native minimum-level reader and selected-stat setter; gold uses the
  script's six sequential setters. Base IVs become 31 and native recalculation
  updates party HP/stats. Egg/hidden-ability bits and unrelated fields survive.
- Under-level individuals remain unchanged. Unlock, credit and item checks are
  shown separately. Even when these are false or unknown, a level-qualified
  hypothetical field effect may be displayed; it is not permission to use the
  service. Parsed checks never certify complete menu eligibility or access.
- Stored party inputs retain actual saved HP/stats. Box/simulated inputs use a
  full-HP party projection; this is not a native PC-transfer validation. The UI
  compares actual raw party stat fields, rather than the general decoded stat
  estimate. Simulated inputs use explicit IVs, zero EVs and friendship 70.

水银银色分支的持有／尝试支付不一致仍单独说明。字段结果不能证明扣款成功、
免费训练、取消退款或完整交易。究极绿宝石完整存取刷新、菜单、解锁、费用与
可达性也没有因本预览而升级为完整验证。

## Evidence / 验证

- Independent complete mGBA vectors from the existing crown probes were reused:
  [Ultimate](training-crowns-20261006.md),
  [Mercury](training-mercury-crowns-20261006.md).
- Core API matches **252 Mercury** field cases against the independent vectors,
  gated by runtime level, and **56 Ultimate** level/header/choice cases. Original
  saves and ROM hashes remain unchanged. Other three profiles reject unsupported
  services instead of borrowing another game's rules.
- **123 public core tests passed**, 41 local opt-in tests ignored by default.
  Service tests and five-profile EV/mint/ability native regressions passed after
  extracting shared individual preparation.
- Five-profile bilingual browser fixtures passed, including runtime choice,
  simulated/stored selection, under-level results, stale input response rejection,
  item/map/back navigation and ROM clearing. Both new service panels were checked
  at 720px width after table styling. Browser response fixtures verify UI behavior;
  native numeric evidence comes from the separate core/mGBA comparisons.
- CLI ROM-only previews passed for both services. TypeScript and production
  frontend build passed. No new edited-SAV emulator round trip is claimed.

Private ROMs, saves, screenshots and vectors remain in ignored local analysis;
none are release inputs. Full NPC menus, payment/cancellation and story access
remain unverified. BW/DP/Rocket empty service lists mean unresolved coverage,
not evidence that crown services are absent.
