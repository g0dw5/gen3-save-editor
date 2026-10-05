# Saved ordinary daycare / 普通寄养保存快照

Read-only scope: original deposited individuals, native saved availability and
ordinary production-check phase. All five breeding capability rows remain
partial. No SAV or ROM bytes are written by these queries.

## Native boundaries

| Exact profile | Presence getter | Availability | Service status | SAV daycare offset |
|---|---|---|---|---|
| Dark Phantom BW / DP | `0806A674` | `08070BF0` | `08070CB0` | `3030` |
| Spanish Rocket | `080977D0` | `0809EBE0` | `0809ECA0` | `297C` |
| Ultimate Emerald 5.5 | `0806A674` | `08070BF0` | `08070CB0` | `3030` |
| Mercury 1.2 | `0803FD44` | `080463FC` | `080464B4` | `2F80` |

Ordinary parent records span 140 bytes; the deposited individual occupies the
first 80 and the accumulated step counter is at `88`. Displayed level is the
stored deposit level, not the result of withdrawing the individual. Presence is
read through native field 5; decoded non-egg species and native service status
must agree before compatibility or checkpoint information is presented.

Mercury's availability hook follows `09D1BBA4` and the current ROM flag operand
at `1D1BBB0`. Its legacy 16-bit pending field can remain zero with an egg available.
The older four profiles use their pending field. Mercury's wrapper also reads
an additional record at SB1 + `3C98`, which is **not covered** here. A legacy
pending value still suppresses the ordinary check, independently of availability.

All tested ordinary checkpoints occur when the second parent's low-byte counter
reaches 255 after increment, repeating every 256 steps. The reader validates the
native gate before reporting the distance. A check is not a guarantee of an egg;
field access, custom services, live RNG, complete inheritance and hatching remain
unresolved. The saved snapshot is not an emulator-memory observation.

## Workflow and safety

Load ROM and SAV → Pokémon acquisition → expand native daycare preview → inspect
saved ordinary daycare → use both deposited parents → preview → follow offspring,
move, required-item or located service-map links → return. Original deposited
records retain PID, trainer identity and other individual bytes. Queries operate
in disposable RAM, compare copied saved blocks afterward and invalidate on SAV
or ROM changes. No daycare editing location, egg insertion or save action is added.

CLI: `gen3 daycare-state ROM SAVE` performs the same read-only snapshot query.
Missing SAV/unsupported rules return no snapshot, not a fabricated empty state.
Unreadable occupied parents retain an issue and unknown service status. Ultimate
and Mercury plaintext codecs do not enforce the old encrypted checksum: invalid
fixture records use an out-of-range species rather than inventing checksum rules.

## Evidence and reproduction

- `scripts/verify_daycare_state.py` executes unmodified native routines in an
  independent mGBA 0.10.5 RAM probe: **576 saved states** (96 each BW, DP, Rocket,
  Ultimate; 192 Mercury) and **40 sequential counter phases**. Presence, native
  service status, compatibility and availability match; saved RAM remains intact.
- Rust opt-in tests match every oracle state and phase, preserve complete
  synthetic SAV bytes, preview exact deposited-parent records and reject invalid
  occupied records before native generation. No real SAV is edited.
- **112 public core tests passed**, 30 opt-in excluded. Bilingual browser tests
  cover deposited selection, pending-flag refresh, ordinary probability,
  offspring/item/map/back links and compact layout; no save action is requested.
- Prior native compatibility/receipt and production suites are rerun for this
  local build. This does not claim a new edited-SAV emulator re-save.

```sh
python3 scripts/verify_daycare_state.py \
  --mgba-probe /private/breeding-probe.dylib --output /private/daycare.json
GEN3_DAYCARE_STATE_PROBES=/private/daycare.json cargo test -p gen3-core \
  local_saved_daycare_matches -- --ignored --nocapture
cargo test -p gen3-core
uv run --with playwright python scripts/test_breeding_ui.py
```

Supply the five exact private ROM environment variables used by the existing
breeding probes. Private vectors, ROMs, SAVs and libraries stay outside release
inputs. Keep the existing version and unreleased changelog status.

本增量验证的是已保存的普通寄养区，不覆盖全部服务或当前可达性。水银待领蛋使用
原生剧情标记，不能因旧字段为零就断言没有蛋；步数是下次检查距离，不是必定出蛋
距离。预览只读，未改动用户存档，整体孵蛋能力仍标为部分解析。
