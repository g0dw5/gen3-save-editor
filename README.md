# Gen III ROM Hack Editor

Downloads: [GitHub Releases](https://github.com/g0dw5/gen3-save-editor/releases/latest). Choose Windows x64 or macOS Apple Silicon. Release assets contain the application and documentation, not ROMs or saves.

[简体中文](README.zh-CN.md)

A local desktop editor for user-supplied Gen III Pokémon ROM hacks and battery
saves. Rust owns parsing and transactions; React provides a bilingual workspace;
Tauri supplies native file dialogs. ROM data and artwork are read at runtime.

**Development preview.** BW, DP, Team Rocket 2.1 Chinese and Ultimate Emerald 5.5 share the editing
workspace. Core regression tests
run against generated fixtures and optionally your exact ROMs. See
[verification and remaining work](docs/IMPLEMENTATION.md) before using a build.
Mercury FC 1.2 ROM reference and save editing are available on the development
branch. Its expanded Pokédex flags and cheat catalog are still under study.

## Queries and collection planning

ROM reference windows link Pokémon, item and move sources to map tiles,
connected maps and exterior entrance chains. Read-only **Collection planning**
uses an opened SAV: choose Pokédex ownership or existing individuals, optionally
group permanent evolution families, and export a standalone HTML suggestion.
Receipt status is independent of inventory; unknown rewards are opt-in.
Missing species also show a bounded, directed permanent-evolution preparation
chain when a current non-egg ancestor or referenced ancestor source is found.
Open each required item/move and the origin map/entrance; the same explanation is
included in standalone HTML. Historical Dex records do not supply usable parents,
and evolution edges are never reversed into assumed breeding outcomes. These are
preparation suggestions, not verification of current evolution eligibility.
Coverage is partial: script prerequisites, dynamic access and special sources
may remain undetermined. Mercury currently supports individual-based planning,
not unverified expanded Pokédex flags. See the [exact-ROM capability matrix](docs/capability-matrix.md).

Mercury 1.2 acquisition queries and collection HTML use the native virtual-clock
snapshot from SAV, including saved weekday, speed and forced-night state. Choose
an explicit simulated hour or all periods as needed. Hardware RTC mode remains
unresolved; device time is never substituted. See [clock verification](docs/verification/mercury-clock-20261005.md).
The read-only CLI supports `gen3 game-clock ROM [SAVE]`.

Verified persistent event ranges now support hidden-item and qualified ordinary
item-ball receipt overlays for all five exact profiles. Qualified NPC gifts also
trace the unset guard, native success/failure result and successful flag write.
The flag is separate from NPC visibility; collected gifts leave the regional
suggestion while ROM reference retains every event. Unqualified rewards stay
undetermined, with the same explanation in acquisition, planning and HTML.
Mercury's region-dependent hidden flags and underfoot pickup
are read from native ROM rules. Ordinary NPC receipts and complete quest/refresh
coverage remain partial. See [event verification](docs/verification/event-state-20261005.md)
and [ordinary pickup verification](docs/verification/pickup-receipts-20261005.md),
plus [bounded NPC receipt verification](docs/verification/npc-receipts-20261006.md).

Parsed scripted Pokémon gifts, eggs and fixed encounters also link to NPC tiles,
species references and collection HTML. Unplaced records remain separate;
delivery and current access remain undetermined. See [scope and verification](docs/verification/script-pokemon-20261006.md).
Bounded NPC trade quotes link to the exact requested Pokémon and held-item
sources. With SAV, party and box donors are distinguished; no Pokémon is created
or exchanged automatically. See [native trade verification and limits](docs/verification/npc-trades-20261006.md).

## Cheat codes

Open an exact-match ROM first, then use **Cheats** in the toolbar; no save is
required. The cheat window uses the currently open ROM. One verified recipe
disables AI input peeking in all modes of
Ultimate Emerald 5.5, exact MD5 `17ce9785b33319b3dbda9a5d37c57ec1`.
Ultimate Emerald also supports ROM reference and save editing; see its
[adapter notes and verification boundaries](docs/research/ultimate-emerald-55.md).
All four ROMs include a portable Pokémon PC, walking-encounter suppression, guaranteed
wild capture, faster egg hatching, species/level selection, shiny wild encounters
and map teleport with a Region → Map selector and ROM-derived landing tiles.
Team Rocket also has guaranteed daycare eggs
for compatible parents at the normal checkpoint. Availability is fingerprint-scoped;
all added recipes have native mGBA regression tests.
All four ROMs have exact-fingerprint command-menu emergency party recovery codes:
press L+R+SELECT to heal and revive without spending an item turn. Each complete
code group requires an emulator that applies and removes patches live.
They also support a four-line CodeBreaker recipe for persistent player-side
Protect, preserving other battle flags and the ROM's native Protect exceptions.
See [usage, formats, limitations and developer tests](docs/cheats.md).

## Supported inputs

| ROM | Required MD5 | Bytes |
| --- | --- | ---: |
| Dark Phantom 5.0EX+BW | `0d9b129f7dd76895f79bb47ad7dec2fe` | 33,554,188 |
| Dark Phantom 5.0EX+DP | `cb2940215f4dafb1bef133c3af379f44` | 33,554,188 |
| Team Rocket 2.1 Chinese | `59c658a1081f542086de1060bb65f0b3` | 33,554,432 |
| Ultimate Emerald 5.5 | `17ce9785b33319b3dbda9a5d37c57ec1` | 33,554,432 |
| Mercury FC 1.2 (development branch) | `f323df1792ac68462a34b42fe8571533` | 33,554,432 |

Use a 128 KiB `.sav`/`.srm` battery save (Mercury also accepts its 16-byte RTC
trailer). Emulator save states are not supported.
Renaming a ROM cannot change its compatibility. The ROM is read-only; edits
and exports apply only to save files.

## Workspace

The same editor registers five exact ROMs across four games. Rocket adds its packed nature,
third ability, nine inventory pockets, level cap, graphics and expanded ROM tables
through reusable adapter components. ROM reference windows remain read-only.
See [coverage, verification and extension rules](docs/multi-rom-adapters.md).

- Individual sprites use PID-derived Unown letters and Spinda spots, with normal
  and shiny palettes read from the ROM. See [appearance verification](docs/research/pokemon-appearance.md).
- Keep the party and all 14 boxes visible; switch compact/comfortable density or
  hide the inspector. Search dims nonmatches without moving storage coordinates.
- Edit identity, nature/shiny/gender, level/experience, IVs/EVs, moves/PP, abilities,
  held items, origin, eggs, Pokérus, ribbons and contest values/fullness.
  See [feeding rules and validation scope](docs/research/contest-condition.md).
- Search editing fields by names/IDs from the loaded ROM.
  Standard origin choices include the species and its pre-evolutions, with a
  separate hatch source. See [search and origin rules](docs/research/search-and-origins.md).
- Drag between slots to move or swap. Alt-drag copies. Buttons provide a pointer
  alternative; modifier-click selects multiple Pokémon for batch edits.
- Browse Pokémon, learning sources, items, abilities, maps and trainers in movable
  nonmodal windows. Trainers and maps link to each other through parsed battle
  scripts, with evidence offsets and unresolved-condition labels. Drag an encounter to an empty slot to create an editable draft.
- Follow the evolution tree in both directions, including sibling branches and
  separately labeled battle/form families. Compare vertical ROM base stats and
  totals with the latest available official values from 52Poké Wiki. This small,
  separately attributed numeric reference is the only bundled Pokémon catalog;
  ambiguous or custom species require manual reference selection. See the
  [runtime-data audit](docs/research/runtime-data-and-evolution-tree.md).
- Edit player identity, money, coins, bags, box names and Pokédex flags. Inspect
  before/after changes, undo/redo, then export. Existing output is backed up.
- Free editing permits game-rule exceptions. Binary bounds, native checksum rules and supported
  IDs remain enforced. Missing learning evidence is “unverified”, not “illegal”.
- Change Chinese/English at any time. ROM names retain their original language.

ROMs are read-only. Pokémon IVs, EVs and ability selection are edited in the SAV;
species base stats are global ROM data and cannot be changed for one Pokémon in a SAV.

## Development

Install stable Rust, Node.js 22+, and the platform prerequisites in the official
[Tauri setup guide](https://v2.tauri.app/start/prerequisites/).

```sh
npm ci
npm run desktop
```

Build an installer with `npm run desktop:build`. Windows x64 targets Windows 10/11;
the bilingual per-user EXE installer installs WebView2 online when missing. See
[Windows builds and validation limits](docs/windows-build.md). Windows and Linux builds are
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

For the synthetic reference-navigation regression, start `npm run dev` and run
`python3 scripts/test_reference_navigation.py`. This test requires Playwright and
Chrome but no ROM, save, or development bridge; API responses are generated fixtures.

Static map palette rules and the native-ROM verification harness are documented
in [map palettes](docs/research/map-palettes.md).

The editor preserves its active tab across Pokémon selections and keeps its
header and action footer outside the scrolling fields. Run
`python3 scripts/test_editor_navigation.py` against Vite to verify tab retention,
draft isolation and scrolling at desktop, minimum-window and mobile sizes.
The test uses synthetic data and requires Playwright and Chrome. Run
`python3 scripts/test_search_origins.py` for searchable fields and origin choices.

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

Map references include independently switchable item-ball, hidden-item, dialogue-reward
and NPC layers, with tile coordinates, search, grid and zoom. See
[map event evidence and limitations](docs/research/map-events.md).

Release history: [Changelog](CHANGELOG.md), starting with the first public release 0.1.5. Route 119 fishing spots are calculated from the loaded save; use the map layer or the species reference shortcut.

Mercury trainer references execute the loaded ROM's ordinary-party constructor
in isolated RAM. These are explicitly seeded, zero-context scenarios; script
replacements, special facilities and full live battle-entry state remain unknown.
[Verification and current gaps](docs/verification/query-collection-20261005.md).

Mercury 1.2 uses its native expanded inventory and section-name table. Static maps
with independently reproduced native layout mismatches or unresolved layer types
show warnings; a parsed map entry is not proof of a normally accessible area.
Validation scope: [Mercury display/storage](docs/verification/mercury-display-storage-20261005.md).

Parsed tutor offers now link move queries to NPC tiles, exterior entrances and back navigation. Dark Phantom BW/DP uses the corrected native tutor table; payment, one-time limits and special eligibility remain unresolved. See [teaching evidence](docs/verification/tutor-sources-20261006.md).

Parsed resource checks show item names and holdings, with prerequisite-item links
in sources/maps/planning and standalone HTML. Money checks are not payment
proof; alternate facility bags and post-mutation checks remain unknown.
See [native rules and evidence](docs/verification/resource-conditions-20261006.md).

Wild held-item queries link referenced Pokémon to encounter maps and preserve
slot/time selectors. The current ROM's native ordinary single-wild routine
provides a simulated no-modifier baseline and, with a SAV, the first party
member's context. Held chances are conditional on that Pokémon being encountered;
encounter slot probabilities remain separate. Unreferenced records do not prove
obtainability. See [native evidence and limits](docs/verification/wild-held-20261006.md).

Pokémon acquisition includes a read-only native daycare preview. Choose saved
parents or explicit simulated parents, follow the resulting offspring and located
receiving NPC/map links. A separate ordinary production check projects the SAV bag
or an explicit item scenario, with current-ROM probabilities and a required-item
link. Neither predicts the next real egg; Rocket/Mercury service locations and full breeding coverage
remain unresolved. See [native daycare evidence](docs/verification/breeding-20261006.md).
Production assumptions and exhaustive native evidence are [documented separately](docs/verification/breeding-production-20261006.md).

With a SAV loaded, the preview also shows the saved ordinary daycare's deposited
individuals, native egg-availability marker and distance to the next ordinary
production check. Deposited parents can be used directly in a read-only scenario;
their original records are preserved. This is a saved snapshot, not live emulator
state or a forecast of the next egg. Mercury's additional custom service record
remains unresolved. See [saved-state evidence](docs/verification/daycare-state-20261006.md).

Collection preparation also samples the current ROM's ordinary offspring selection
using exact existing party/box parents and their held items. A suggestion can lead
through receiving-service references, hatching and directed evolution; its pairing
can be checked with the full native receipt preview. Standalone HTML includes parent
locations and the same limitations. Sampling is bounded, is not a complete offspring
catalog and does not include already deposited daycare parents. Missing suggestions
do not prove breeding impossible. See [planning evidence](docs/verification/breeding-planning-20261006.md).

Prerequisite checks now offer **Find prerequisite clues**: potential map-referenced
script writers, ROM text context, further guards and tile/entrance/back navigation.
Standalone collection HTML includes a linked, bounded appendix from the same
current ROM/SAV snapshot. These are partial clues, not a complete quest graph or
proof of access/completion. See [evidence and limits](docs/verification/event-dependencies-20261006.md).

The read-only **Event clues** reference page searches dialogue contexts and map
references directly from the loaded ROM, then links guards to prerequisite traces,
static tiles and exterior entrances. Saved observations are separate from quest
completion and access. This remains partial story coverage; see
[event-clue evidence](docs/verification/event-clues-20261006.md).

Trainer references now link qualified battle-record operands to guarded script
contexts, static actors/tiles and map entrances. Rematch bases and setup records
are identified separately; a referenced record does not establish an available
battle or its final party. See [native boundary evidence and limits](docs/verification/trainer-locations-20261006.md).

The ROM reference **Game time** tab distinguishes Mercury’s saved virtual clock from explicit hardware RTC scenarios. Other registered profiles show SAV offsets/checkpoints and use native ROM routines for simulated input; current RTC and unverified weekday/refresh rules remain unknown. See [verification](docs/verification/hardware-clock-20261006.md).

The read-only **Training reference** tab links verified EV-item handlers to acquisition and map queries, with native before/after previews for stored or simulated individuals. Custom mechanisms/menu eligibility/consumption remain partial. [Evidence](docs/verification/training-items-20261006.md). CLI: `gen3 training-items ROM` and `gen3 training-preview ROM REQUEST.json [SAVE]`.

Rocket mint targets are also read at runtime in Training reference, with stored/simulated effective-nature and stat previews. In verified changing scenarios, the existing nature-override SAV patch matches native persistent effects while preserving PID/identity/history. Menus, consumption and other-ROM mint services remain unverified. [Evidence](docs/verification/training-natures-20261006.md).

Training reference also reads Rocket and Mercury ability-item handlers at runtime,
executes native acceptance and persistent effects, and links item acquisition/maps.
Rocket retains PID; Mercury requires an explicit random-seed scenario and may
reroll PID. Actual ability/PID/nature/stats are shown; these read-only previews do
not consume items or certify all menus, services or PID-linked appearances.
[Evidence and limits](docs/verification/training-abilities-20261006.md).

Ultimate ability items now join the shared native training previews: runtime variants, seeded hidden-to-normal selection without PID changes, and explicit acceptance-without-change warnings. Menu/consumption/access remain qualified. [Evidence](docs/verification/training-ultimate-abilities-20261006.md).

The Training reference now includes Ultimate's referenced NPC Hyper Training
service: runtime menu choices, crown fees, separate earned-credit requirements,
SAV condition overlays and NPC-to-map/item-acquisition navigation. Native helpers
keep base IVs and unrelated party data unchanged; existing stored party stats are
not refreshed by this script stage. Full menus, credit acquisition and access
remain partial; Mercury crown item names do not establish a verified service.
Read-only CLI: `gen3 training-services ROM [SAVE]`.
[Evidence and scope](docs/verification/training-crowns-20261006.md).
