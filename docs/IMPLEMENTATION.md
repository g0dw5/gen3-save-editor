# Dark Phantom implementation

This checklist records verified behavior, not intended compatibility. ROMs and saves
remain local and are never committed. Public tests use generated binary fixtures.

## Milestones

- [ ] Profile registry: exact MD5 and length for BW and DP; isolated ROM sessions.
- [ ] ROM: text, species, moves, abilities, items, learnsets, evolution, sprites.
- [ ] ROM: active maps, rendered maps, encounters, scripts and trainer parties.
- [ ] Save: complete slot validation, rollover selection, lossless round trips.
- [ ] Pokemon: identity, origin, moves/PP, stats, EV/IV, status, egg, ribbons.
- [ ] Storage: create, move, swap, clone, delete, sort, batch, import/export.
- [ ] Player: trainer identity, money, coins, pockets, box names, Pokedex.
- [ ] Editing: standard/free policy, undo/redo, atomic output, conflict detection.
- [ ] UI: all boxes, nonmodal reference windows, drag/drop, draft editing.
- [ ] UI: Chinese/English, file workflow, error messages and keyboard alternatives.
- [ ] ROM edits: bounded field patches, base/output fingerprints, derived ROM output.
- [ ] CLI and standalone agent research methodology.
- [ ] Integration tests with both local ROMs; emulator round trip with a sample save.
- [ ] Desktop build, bilingual documentation, CI and release workflow.

## Architecture

`gen3-core` owns binary parsing, version profiles, game rules and transactions.
The CLI and Tauri commands consume the same core. React only handles presentation,
drafts and user intent. No global active-ROM variable is allowed in the core.

Research accepts arbitrary ROMs for identification and evidence gathering. Editing
requires an exact published profile. An edited ROM is a separate derived artifact;
its fingerprint must never be treated as a stock supported release.

## Deferred until individually researched

Story flags, arbitrary teleportation, scripted quest state, arbitrary ROM expansion,
code injection and full map geometry editing are not enabled by generic offsets.
They require explicit layouts, ownership rules and game-level verification.
