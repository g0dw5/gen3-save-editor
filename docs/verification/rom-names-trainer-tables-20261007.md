# ROM names and compact trainer teams / ROM 原名与紧凑配队

Date: 2026-10-07. This record is developer-only and is not included in player
packages. The change is presentation-only; ROM/SAV readers and native battle
constructors are unchanged.

## Changes

- Pokémon names in reference headings, selectors, evolution links, encounters,
  storage labels and trainer teams use the currently loaded catalog's ROM name.
  No official-reference, duplicate-name or form label is appended. IDs remain
  separate, so identically named species still have independent navigation.
- Remove guessed base/numbered/duplicate-form labels and their translations.
  Verified form metadata and PID-dependent artwork remain independent of names;
  reviewed official stat comparisons do not become Pokémon display names.
- Trainer teams use one fixed-layout table with three short headers. Each member
  keeps level, type, gender, nature, ability choices, held item and moves; a
  full-width IV/EV table follows, with alternate random outcomes and EV totals.
  Unknown values remain unknown. Ability choices still link to their reference.

## Verification

All browser cases below use public synthetic API responses, not private ROM/SAV
files. Five UI contexts represent BW, DP, Rocket, Ultimate and Mercury 1.2; these
checks verify display/state handling, not new native reverse-engineering claims.

- `npm run build`: TypeScript and production frontend build pass. The existing
  Vite large-chunk advisory remains; it is not a build failure.
- Prettier check of all modified UI sources and `git diff --check`: pass.
- `node scripts/test_species_forms.mjs`: verified Deoxys family labels,
  Mega triggers, Unown PID letters, no guessed duplicate/base labels,
  fingerprint isolation and input immutability pass.
- `node scripts/test_evolution_graph.mjs`: distinct species cards and native
  evolution/form graph isolation pass.
- `scripts/test_trainer_table_ui.py`: five bilingual contexts, identical ROM
  names with different IDs, evolution/encounter navigation, six-member teams,
  all original trainer fields, deduplicated/random ability choices and unknown
  IV/EV values pass. Table headers and cells fit at 1100, 900 and 720 px without
  horizontal overflow. Chinese and English screenshots were visually inspected.
- `scripts/test_reference_scope_ui.py`: sequential five-ROM switches in the same
  page, unadorned names, all four Ultimate difficulties, automatic saved-party
  inputs, alternate IV/EV outcomes and totals, and no-SAV unknown values pass.
- `scripts/test_native_trainer_ui.py`: ordinary generated values, automatic
  sample, native effective nature, stale-response isolation and read-only
  requests pass after the table conversion.
- `scripts/test_species_forms_ui.py`: ROM names in references/storage, separate
  verified stored form labels, PID letter and distinct family/duplicate IDs pass.
- `scripts/test_hidden_power_ui.py`: known/unknown trainer Hidden Power values,
  IV-derived editor/dropdown display and original isolated save patch pass.
- `scripts/test_reference_readability.py`: bilingual references, fixed/random
  ability options, simple difficulty selector and generated values pass.

No real save was edited. No ROM bytes were written, no native app/installer was
packaged, and the development version remains unchanged. This round does not
establish new map, acquisition, story or native trainer coverage beyond the
previously recorded verification boundaries.
