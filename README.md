# Gen III Save Editor

[简体中文](README.zh-CN.md)

A local desktop editor for user-supplied Gen III Pokémon ROM hacks and battery
saves. Rust owns parsing and transactions; React provides a bilingual workspace;
Tauri supplies native file dialogs. ROM data and artwork are read at runtime.

**Development preview.** BW and DP are the first adapters. Core regression tests
run against generated fixtures and optionally your exact ROMs. See
[verification and remaining work](docs/IMPLEMENTATION.md) before using a build.

## Supported inputs

| ROM | Required MD5 | Bytes |
| --- | --- | ---: |
| Dark Phantom 5.0EX+BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | 33,554,188 |
| Dark Phantom 5.0EX+DP | `cb2940215f4dafb1bef133c3af379f44` | 33,554,188 |

Use a 128 KiB `.sav`/`.srm` battery save. Emulator save states are not supported.
Renaming a ROM cannot change its compatibility. Derived ROMs have different
fingerprints and do not become supported stock releases automatically.

## Workspace

- Keep the party and all 14 boxes visible; switch compact/comfortable density or
  hide the inspector. Search dims nonmatches without moving storage coordinates.
- Edit identity, nature/shiny/gender, level/experience, IVs/EVs, moves/PP, abilities,
  held items, origin, eggs, Pokérus, ribbons and contest values.
- Drag between slots to move or swap. Alt-drag copies. Buttons provide a pointer
  alternative; modifier-click selects multiple Pokémon for batch edits.
- Browse species, learning sources, items, abilities, maps and trainers in movable
  nonmodal windows. Trainers and maps link to each other through parsed battle
  scripts, with evidence offsets and unresolved-condition labels. Drag an encounter to an empty slot to create an editable draft.
- Edit player identity, money, coins, bags, box names and Pokédex flags. Inspect
  before/after changes, undo/redo, then export. Existing output is backed up.
- Free editing permits game-rule exceptions. Binary bounds, checksums and supported
  IDs remain enforced. Missing learning evidence is “unverified”, not “illegal”.
- Change Chinese/English at any time. ROM names retain their original language.

ROM scalar editing exports a separate derived ROM and fingerprinted manifest.
It currently supports selected species stats, move parameters and item prices.
It is not a map, script or executable-code editor.

## Development

Install stable Rust, Node.js 22+, and the platform prerequisites in the official
[Tauri setup guide](https://v2.tauri.app/start/prerequisites/).

```sh
npm ci
npm run desktop
```

Build an installer with `npm run desktop:build`. Windows and Linux builds are
configured in CI; local verification status is recorded separately. No ROM,
extracted sprite collection, save, updater or development HTTP server is shipped.

```sh
cargo test -p gen3-core -p gen3-cli
cargo fmt --all --check
cargo clippy -p gen3-core -p gen3-cli --all-targets --all-features -- -D warnings
npm run check
npm run format:check
npm run build
```

Optional real-ROM regression (both files remain local):

```sh
GEN3_ROM_BW='/path/BW.gba' GEN3_ROM_DP='/path/DP.gba' \
  cargo test -p gen3-core local_rom_regression -- --ignored --nocapture
```

For native encounter-selection verification (Python Unicorn required), run
`scripts/verify_encounter_selection.py` with the same ROM environment variables.
See [encounter selection and time conditions](docs/research/encounter-time-selection.md)
for the audited code paths and validation limits.

For browser tests, run `cargo run -p gen3-cli --features dev-server --bin gen3-dev`
and `npm run dev` with the same random `GEN3_DEV_TOKEN` (32+ characters). The
bridge binds only `127.0.0.1:8766` and requires that token. Generate a disposable
fixture by adding `GEN3_TEST_SAVE=/tmp/test.sav` to the real-ROM test, then run
`scripts/test_ui.py` with `GEN3_ROM_BW`, `GEN3_TEST_SAVE`, `GEN3_DEV_TOKEN` and
Playwright/Chrome installed. This bridge is opt-in and absent from release builds.

## CLI and architecture

```sh
cargo run -p gen3-cli --bin gen3 -- help
cargo run -p gen3-cli --bin gen3 -- identify /path/game.gba
cargo run -p gen3-cli --bin gen3 -- inspect /path/game.gba /path/game.sav
```

`identify` accepts unknown ROMs. Other game-aware commands require a matched
profile. `patch-save` accepts JSON actions and supports `--dry-run` and `--free`.
See [architecture and adapter boundaries](docs/ARCHITECTURE.md) and the independent
[ROM research skill](skills/gen3-rom-research/SKILL.md).

MIT license applies to this project's code. Refer to
[third-party notices](THIRD_PARTY_NOTICES.md) for format research sources.
