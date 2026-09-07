# Dark Phantom implementation and verification

This records observed behavior. ROMs and saves remain local. Public tests use
generated binary fixtures. This is a development preview, not a completed
compatibility certification.

## Implemented

- [x] Exact MD5 and length registry for BW and DP; isolated ROM sessions.
- [x] ROM text, species, moves, abilities, items, learnsets, evolution and sprites.
- [x] Map headers and static layout rendering; wild tables and bounded script
  encounter extraction; trainer headers and parties with raw-data diagnostics.
- [x] Complete save-bank validation, counter rollover, lossless unchanged reads.
- [x] Pokémon identity, origin, moves/PP, stats, EV/IV, status, eggs and ribbons.
- [x] Create, move, swap, copy, delete, sort, batch and fingerprinted import/export.
- [x] Trainer identity, money, coins, pockets, box names and mapped Pokédex flags.
- [x] Standard/free policy, undo/redo, actual field diffs, atomic output and conflict
  detection. Linked mail operations remain blocked.
- [x] All boxes and party, movable nonmodal references, actual drag/drop and drafts.
- [x] Chinese/English UI, file workflow and non-drag transfer controls.
- [x] Bounded ROM scalar patches with base/output fingerprints and manifests.
- [x] CLI, standalone research skill, bilingual README and contribution guidelines.
- [x] macOS desktop preview build; Windows/Linux CI and artifact workflow authored.

## Local evidence

Validation performed on macOS Apple Silicon, September 2026:

- **27 Rust tests passed**, including the opt-in regression covering both exact
  ROMs. Includes all 24 Pokémon permutations, a known checksum vector, all growth
  curves, byte retention, mixed/corrupt banks, counter wrap, cross-sector storage,
  the last box slot, batch rollback, free-mode boundaries, PP Ups, fixed text
  capacity, before/after values, backup/conflict export, bounded reproducible ROM patches and mail attachment guards.
- Both ROMs: 411 species records, 472 moves, 707 maps, 2,659 encounters extracted
  from supported sources, 1,365 nonempty trainer headers. Every valid species'
  normal/shiny sprite and learnset is parsed. Six representative maps are rendered.
- Both ROMs map 636 trainer IDs to 680 map references from battle scripts.
  References retain exact command offsets; shared roots and cycles are deduplicated.
  Trainer-to-map and map-to-trainer UI navigation passes the browser regression.
- 96 trainer headers have raw level parameters outside 1–100. All have references:
  93 through map scripts and three through an active rematch table. The ROM
  converts out-of-range parameters to the highest player party slot level; 92
  headers use 101. Four raw-244 headers still have party-layout concerns.
  See the [reference audit](research/dark-phantom-trainer-audit.md) for evidence
  and reachability limits. Dynamic levels are displayed as a rule, previewed from
  the loaded party when available; raw parameters remain in evidence details.
- Semantic Pokémon controls show nature modifiers, current/max PP, origin names,
  marking symbols, Pokérus state/strain/days and structured ribbon awards. Read-only
  and reserved data are disabled; applying changes preserves the active tab.
- Trainer battle portraits and verified map characters read from ROM at runtime,
  including extended graphics banks and multi-actor trainers. See the bilingual
  [field and image audit](research/editor-fields-and-trainer-art.md).
- Trainer tags and composable role/location/format/level filters distinguish
  same-name teams. Roles read the relocated ROM class table; verified map-purpose
  metadata identifies eight gyms and the League rooms without trainer ID lists.
  Search includes tags, maps, species and IDs. Story conditions remain unresolved.
- Gender, nature, ability, six IVs and six creation-time EVs are derived from the
  target engine. Isolated Unicorn execution of both ROMs' original constructors
  matches 20 trainer parties / 106 Pokémon, including first League, Miltank and
  dynamic/custom-personality opponents. This is not a full in-game battle replay.
  A regression specifically checks that the level's adjacent byte is not consumed.
- Browser workflow passed: 426 storage slots, all 14 boxes, both UI languages,
  level edit, undo/redo, box-to-party drag, reference-to-box draft drag, creation,
  role/location/battle/level filters, tag clicks, zero-result recovery,
  player money, download and small-window layout. Downloaded output is 128 KiB and
  independently passes CLI validation.
- Rust formatting and Clippy with warnings denied; TypeScript and Vite build.
- Tauri macOS `.app` built and launched successfully at `tauri://localhost`;
  native file selector opened. Native file selection/complete edit loop is not
  marked verified because the computer-use session timed out/lost state.
- A supplied BW gameplay save with one valid bank was parsed successfully.
  A disposable copy was edited and loaded by VisualBoyAdvance-M 2.2.3. The emulator
  reported reading its battery file. Game-internal inspection and re-save are
  still outstanding; this is not counted as an emulator round trip.
- Research skill passes its structural validator.

## Still required before a stable release

- [ ] Complete native dialog/edit/export/close-confirmation workflow on each platform.
- [ ] In-game inspection of edited values, save again, re-read, on both BW and DP.
- [ ] Run the configured Windows/Linux CI and installers; signing/notarization.
- [ ] Complete script coverage, trades, roamers, special rules and current-save trainer
  reachability, checked against version-specific official documentation.
- [ ] Index rematches, investigate inconsistent party layouts and later script
  party overrides, and distinguish unreachable rows from parser gaps.
- [ ] Linked mail content and attachment editing.

Story flags, arbitrary teleportation, scripted quest state, ROM expansion,
code injection and full map geometry editing require separate research and are
not enabled by generic offsets. A static map preview is not a live game scene.

Git remote is configured as `git@github.com:g0dw5/gen3-save-editor.git`.
Local commits work. Remote push is currently blocked by SSH authentication
(`Permission denied (publickey)`); GitHub CLI is also not authenticated.
