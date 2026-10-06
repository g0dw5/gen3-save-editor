# Crown-service party selection / 王冠服务同行选择

> Historical research / 历史逆向记录：the dedicated four-tab feature and its
> commands/tests were removed during reference cleanup. Procedures below
> describe commit `f5b085c`, not the current application. Findings and limits
> remain evidence; use [current key addresses](../research/reference-key-addresses.md)
> for subsequent investigation. 本文旧页面／命令已撤去，不是当前产品使用说明。


Date: 2026-10-06. Scope: exact registered Mercury 1.2 and Ultimate Emerald 5.5
fingerprints. Overall NPC training remains **partial**; BW/DP/Rocket services
remain unresolved, not proven absent.

## Usable workflow / 使用流程

Training reference → runtime crown choice → individual preview → item source or
NPC tile/entrance → return. Both referenced native services select from the
party. A boxed individual must first be withdrawn in-game. The query can still
preview its hypothetical full-HP party record, with a visible withdrawal step
and `withdrawal_required: true`; this is neither a PC-transfer replay nor proof
that the NPC is currently reachable or its prerequisites satisfied.

两作原生同行选择均可按 B 取消，随后才检查等级。选择步骤不排除已倒下个体，也
没有执行通用选择器供其他用途使用的蛋检查。合成蛋状态仅用于验证这一局部事实，
不能推断正常蛋可接受训练；修改器预览仍主动拒绝蛋、空记录和损坏个体。

## Runtime evidence / 当前 ROM 证据

Adapters contain verified script addresses, special-table layouts and engine
semantics, no extracted content. The service reader validates opcode `0x25`,
its runtime special ID/native pointer and the selected-slot cancel comparison.
The actual script cancel value is read from ROM. Native probes show:

- Mercury gold/silver scripts call special `0x9f` at `0x7b00b7/0x7b00e0`.
  Table `0x15fd60` resolves to `0x080bf8fc`; its native launcher reaches
  `InitPartyMenu 0x0811ea44` with type 3, layout 0, action 11, false. Action 11
  writes the chosen party slot into variable `0x8004`. B cancellation writes 7.
  Runtime comparisons at `0x7b07a0/0x7b098b` branch on 7 before the level checks.
- Ultimate script `0x1812776` calls special `0xa2`. Table `0x1dba64` resolves to
  `0x081b94b0`; its launcher reaches `InitPartyMenu 0x081b0038` with the same
  arguments. Chosen slot is returned by callback `0x081b9390`: 0–5 stay intact,
  values above 5 become 255. The cancel handler's temporary 7 is thus normalized
  before the script comparison at `0x181277a`. Its level check follows selection.

Normal action 11 accepted slots 0–5 in the explicit test contexts independently
of egg bit, HP and selected status. That is a selection-stage statement, not a
claim about level validity, complete eligibility or every possible status.

## Verification / 验证

[`scripts/verify_training_party_selection.py`](../../scripts/verify_training_party_selection.py)
runs exact native instructions in independent mGBA disposable RAM without hooks:

- Three actual script dispatch/launcher observations, including native task
  creation, explicit completed-fade context, callbacks and Init arguments.
- **288 input/state cases**: two ROMs × six populated slots × normal/egg-bit
  fixture × HP 0, HP 20, HP 20/status 8 × no input, A, B, A+B. No/A+B wait, A
  chooses, B cancels in these contexts. All 600 party bytes stay unchanged.
- **256 Ultimate return prefixes**, covering every byte-valued menu slot.
- **21 native script routes**, covering six selected slots and cancellation for
  all three referenced script branches. Source ROM SHA-256 remains unchanged;
  no actual SAV is opened or edited.

Core opt-in test
`local_training_party_selection_matches_native_and_keeps_individuals_unchanged`
compares all five profiles, runtime service metadata, script dispatch, Init
argument prefixes, all input/state cases, normalization and routes against the
independent output. Native input execution uses explicit valid save-block
pointers and zeroed synthetic option fields. The core runner does **not** replay
full fade/task setup or overworld cleanup: its strict address checks rejected
uninitialized UI-engine accesses, so only bounded parameter prefixes are used
there. Full launch observations come from mGBA. No guards were bypassed and the
core emulator's memory restrictions were not weakened.

Existing service previews also test boxed withdrawal flags and retain read-only
SAV/ROM checks. Shared bilingual browser fixtures exercise simulated/party/box
selection, the withdrawal step before querying, ROM-only references, item/map/back
navigation, stale responses, SAV reloads and ROM switching. Browser fixtures
verify UI/state semantics; native values come from the separate ROM probes.

Final checks: **123 public core tests passed**, 43 opt-in tests ignored by
default. Both crown-service tests, service previews, menu input and the new
party-selection regression passed with all five exact ROMs configured. All five
bilingual UI fixtures passed, including the new box/party switch and withdrawal
wording; 720px service screenshots were visually inspected. Workspace checking,
TypeScript/production build, Rust/UI formatting, manifest validation and five
release-workflow tests passed. The existing Vite chunk-size advisory remains.

## Remaining limits / 保留边界

Rendered windows, directional cursor traversal, empty-slot reachability,
complete PC withdrawal, full menu cleanup, payment/refunds and story access
are not certified. The existing Mercury silver/gold payment mismatch remains
qualified. No new edited-SAV emulator save/reload round trip is claimed.

Private ROMs, fixture saves, native outputs and screenshots are ignored local
analysis only and are excluded from release inputs. This increment does not
turn the entire training category or other ROM services into verified support.
