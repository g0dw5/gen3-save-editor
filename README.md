# Gen III ROM Hack Editor

[Download releases](https://github.com/g0dw5/gen3-save-editor/releases/latest) · [简体中文](README.zh-CN.md)

Browse your own ROM, then open its SAV to edit individuals and inventory.
The desktop app runs locally without an emulator connection. Game names, values,
maps and artwork come from the loaded ROM. ROM input is always read-only.

## Browse, follow clues and edit

- **ROM reference:** Pokémon stats, official comparisons, evolutions and learnsets;
  machine/tutor sources; item acquisition; maps, NPCs, hidden items, encounters and
  trainer parties. Follow cross-links to locate targets on maps.
- **Adventure guide:** Search parsed dialogue, rewards and prerequisite clues.
  Mercury 1.2 reads its native quest journal with saved accepted/completed states;
  Rocket provides next-action clues for identified story branches.
- **Save editing:** Expanded party/boxes, drag/swap, batch edits, nature, abilities,
  IVs/EVs, moves/PP, bag/PC items, undo/redo, backups and checked exports.
  Fields absent from a game's stored format are restricted, including Mercury's
  boxed current PP, contest values and ribbons.
- **Cheats:** Ten common features across five ROMs, including portable PC, wild
  species/level, shiny wild encounters, teleport and battle recovery. Ultimate
  additionally offers no-peeking and accuracy-direction correction. Generate codes
  for the loaded ROM, then activate the matching format in your emulator.
  See [usage and stop conditions](docs/cheats.md).

No SAV is needed to browse. Loading one adds verified receipt/task states and
party-dependent scenarios. Missing bag items never establish unclaimed rewards.
Complete task dependencies, dynamic maps, facilities and unknown scripts remain
partial; a parsed record is not proof of current obtainability.

[Player guide](docs/USER-GUIDE.md) · [Changelog](CHANGELOG.md)

## Supported inputs

| ROM | Required MD5 | Bytes |
| --- | --- | ---: |
| Dark Phantom 5.0EX+BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | 33,554,188 |
| Dark Phantom 5.0EX+DP | `cb2940215f4dafb1bef133c3af379f44` | 33,554,188 |
| Team Rocket 2.1 Chinese | `59c658a1081f542086de1060bb65f0b3` | 33,554,432 |
| Ultimate Emerald 5.5 | `17ce9785b33319b3dbda9a5d37c57ec1` | 33,554,432 |
| Mercury FC 1.2 | `f323df1792ac68462a34b42fe8571533` | 33,554,432 |


Use a 128 KiB `.sav`/`.srm` battery save (Mercury also accepts its 16-byte RTC
trailer), not an emulator save state. Compatibility follows the fingerprint,
not the filename. Mercury 1.1 is not supported.

Installers target Windows 10/11 x64 and Apple Silicon Macs. The Windows installer
can download WebView2 if needed. Builds are unsigned/ad hoc signed and not notarized.
Assets include bilingual guides and the changelog, without ROMs, saves or extracted
artwork.

## Save workflow

Back up the original SAV, apply and review changes, then export a separate copy.
Load that file in your emulator and save normally in game. External source-file
changes block overwriting. Free editing allows exceptions while retaining binary
integrity checks. Species base stats are global ROM data; a SAV edit cannot change
them for one Pokémon.

Five-ROM edit/load/in-game-save/reboot checks were completed in mGBA. This does
not cover every field, mobile emulator or facility. Windows CI builds do not
establish real Windows gameplay verification.

## Development

Install stable Rust, Node.js 22+ and the official
[Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
npm ci
npm run desktop
cargo test -p gen3-core -p gen3-cli --locked
cargo fmt --all --check
cargo clippy -p gen3-core -p gen3-cli --all-targets --all-features --locked -- -D warnings
npm run format:check
npm run build
```

Use shared models, readers and SAV transactions. Keep version differences in
adapters and verified engine rules. Real ROM/SAV fixtures remain local.
See the [capability matrix](docs/capability-matrix.md), [architecture](docs/ARCHITECTURE.md)
and [native verification records](docs/verification).

Optional local ROM regression:

```sh
GEN3_ROM_BW='/path/BW.gba' GEN3_ROM_DP='/path/DP.gba' \
  cargo test -p gen3-core local_rom_regression -- --ignored --nocapture
```

For browser tests, start `npm run dev`; synthetic tests such as
`scripts/test_trainer_table_ui.py` require Playwright and Chrome. Integration tests
can use the opt-in `gen3-dev` localhost bridge with a random `GEN3_DEV_TOKEN` of at
least 32 characters. It binds only `127.0.0.1:8766` and is absent from releases.

```sh
cargo run -p gen3-cli --bin gen3 -- help
cargo run -p gen3-cli --bin gen3 -- identify /path/game.gba
cargo run -p gen3-cli --bin gen3 -- inspect /path/game.gba /path/game.sav
```

The independent [ROM research method](skills/gen3-rom-research/SKILL.md) also
covers unknown ROM research. Code is MIT-licensed; see
[third-party notices](THIRD_PARTY_NOTICES.md) for research and reference attribution.
