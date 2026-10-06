# Service-menu cancellation / 服务菜单取消规则

> Historical research / 历史逆向记录：the dedicated four-tab feature and its
> commands/tests were removed during reference cleanup. Procedures below
> describe commit `f5b085c`, not the current application. Findings and limits
> remain evidence; use [current key addresses](../research/reference-key-addresses.md)
> for subsequent investigation. 本文旧页面／命令已撤去，不是当前产品使用说明。


Date: 2026-10-06. Exact registered Mercury 1.2 and Ultimate Emerald 5.5 inputs.
This increment corrects reference text and adds runtime, read-only menu metadata.
No SAV editing, item consumption or ROM writing is added. Overall training is
still partial; this is not complete NPC gameplay or rendered-window verification.

## Runtime source / 运行时来源

The actual script command `0x6f` reads four operands and calls the current ROM's
menu engine. Mercury masks its fourth operand to bit 0 for the ignore-B flag;
bit 1 controls a separate presentation behavior. Ultimate retains the full byte
and treats any nonzero value as ignore-B. The adapter records this verified
engine distinction instead of borrowing Mercury's flag rule for Ultimate.
Adapter configuration records verified command locations, not extracted labels
or results. The shared service response reads these flags from the loaded ROM.

| Referenced menu | Command | B behavior | Payment ordering |
|---|---|---|---|
| Mercury service choice | `0x7b003d` | Cancels | Before payment |
| Mercury single-stat choice | `0x7b01fb` | Ignored | After attempted payment |
| Ultimate training choice | `0x181279a` | Cancels | Before payment |

主菜单、同行选择、单项菜单不能混为同一个取消步骤。水银单项菜单不会因为 B
返回取消结果。之前 UI／矩阵里的“可取消单项菜单”缺少验证且不正确，已修正。
普通同行选择的取消、资格判断与完整 UI 仍未因本次证据而升级为完整验证。

## Independent native evidence / 独立原生证据

`scripts/verify_training_service_menus.py` uses the read-only mGBA ARM7 probe:

- **3** actual script-handler argument observations stop at the native menu
  entry; registers match all four runtime operands.
- **512** flag-transfer instruction-slice observations cover all byte values in
  explicit stack-local contexts (256 per engine). They verify Mercury's bit-0
  mask and Ultimate's full-byte transfer. These slices do not render a menu.
- Native task creation completes. Normal no-input callback ticks exhaust the
  startup delay; no input routine or task callback is replaced.
- **60** input decisions cover every runtime cursor option with no key, B, A,
  and A+B. The native silent-selection flag is an explicit fixture setting.
  Observation stops before window destruction/script restart. B is ignored in
  Mercury's single-stat menu; A chooses the cursor, including A+B precedence.
- Cancellation-result prefixes include native sound and write `0x7f`. Core
  comparison stops before sound/window side effects; it independently matches
  the input decision and selected-cursor register.
- **8** actual script compare/goto commands follow that result to exit dialogue:
  Mercury `0x7b05c8`, Ultimate `0x1812884`. No payment instruction lies in those
  observed routing prefixes. Exit dialogue and its cleanup are not replayed.
- Original ROM hashes remain unchanged. No battery save is opened by the probe.

The opt-in core test compares native task creation, startup ticks, handler
arguments, flag-transfer slices, input boundaries and cancel routing against those independent vectors.
It also checks the service metadata for both games and rejects cross-profile
service leakage for BW, DP and Rocket. It does not infer nonexistence from their
currently empty configured service lists.

```sh
/usr/bin/python3 scripts/verify_training_service_menus.py \
  --mgba-probe /private/ability-probe.dylib --output /private/service-menus.json
GEN3_TRAINING_SERVICE_MENU_PROBES=/private/service-menus.json \
  cargo test -p gen3-core local_training_service_menus_match_native_input_decisions \
  -- --ignored --nocapture
```

Full party selection, egg eligibility, rendered windows, complete service
payment/cancellation/refund, story access, credit acquisition and edited-SAV
emulator round trips remain separate gaps. In particular, this does not prove
free Mercury training when gold removal fails. Its existing required/payment
anomaly remains explicitly qualified.

Validation for this increment: **123 public core tests passed**, **42** local
opt-in tests ignored by default. The native menu comparison, existing crown
field/payment regressions and service-effect comparisons pass across their
registered profiles. Five-profile bilingual training/item/map/back/ROM-clear
fixtures pass; extended Ultimate/Mercury fixtures additionally verify retaining
service results through map/item/back and rejecting pending service responses
after SAV reload or ROM switch. Compact 720px screenshots remain readable with
no horizontal overflow. Workspace check, TypeScript/production build, formatting
and five release-workflow checks pass. The Vite large-chunk advisory remains.
Version stays unchanged; this increment does not rebuild the delivered app.
