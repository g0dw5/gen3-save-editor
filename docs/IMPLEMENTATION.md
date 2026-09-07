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

- **21 Rust tests passed**, including the opt-in regression covering both exact
  ROMs. Includes all 24 Pokémon permutations, a known checksum vector, all growth
  curves, byte retention, mixed/corrupt banks, counter wrap, cross-sector storage,
  the last box slot, batch rollback, free-mode boundaries, PP Ups, fixed text
  capacity, before/after values, backup/conflict export, bounded reproducible ROM patches and mail attachment guards.
- Both ROMs: 411 species records, 472 moves, 707 maps, 2,653 encounters extracted
  from supported sources, 1,365 nonempty trainer headers. Every valid species'
  normal/shiny sprite and learnset is parsed. Six representative maps are rendered.
- Both ROMs map 587 trainer IDs to 620 map references from battle scripts.
  References retain exact command offsets; shared roots and cycles are deduplicated.
  Trainer-to-map and map-to-trainer UI navigation passes the browser regression.
- 96 trainer headers have out-of-range raw level parameters. Most use 101, which
  may be a runtime level rule; its semantics are not asserted here. A few old
  records have inconsistent flags. Raw values and diagnostics remain visible.
  A regression specifically checks that the level's adjacent byte is not consumed.
- Browser workflow passed: 426 storage slots, all 14 boxes, both UI languages,
  level edit, undo/redo, box-to-party drag, reference-to-box draft drag, creation,
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
- [ ] Decode runtime trainer level parameters and distinguish unused malformed rows.
- [ ] Linked mail content and attachment editing.

Story flags, arbitrary teleportation, scripted quest state, ROM expansion,
code injection and full map geometry editing require separate research and are
not enabled by generic offsets. A static map preview is not a live game scene.

Git remote is configured as `git@github.com:g0dw5/gen3-save-editor.git`.
Local commits work. Remote push is currently blocked by SSH authentication
(`Permission denied (publickey)`); GitHub CLI is also not authenticated.
