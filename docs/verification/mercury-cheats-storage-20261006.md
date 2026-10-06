# Mercury 1.2 common cheats and compact storage correction

Developer evidence; excluded from player documentation/release inputs.
Exact input: MD5 `f323df1792ac68462a34b42fe8571533`. Source ROM/SAV files stay
read-only. All generated saves, vectors and screenshots are local test artifacts.

## Ten common recipes across five exact ROMs

BW, DP, Rocket, Ultimate and Mercury now expose the same ten common recipe IDs.
Ultimate additionally retains its two exclusive battle fixes. Similar engine
functions do not establish shared addresses or identical native constraints.

| Mercury feature | Verified native binding (ROM file offset) |
|---|---|
| Pause walking encounters | `1D69E72 D1F3 → E7F3`, routine `1D69E42` |
| Eligible wild capture | `1D0E9E6 D92E → 46C0`, command `1D0E7A8` |
| Faster hatching | `46346 D12F → 46C0`, check `463B8` |
| Compatible daycare eggs | `4632C 4284 → 2C00`; ready flag `266`, compatibility `4654C` |
| Portable PC | Live SELECT dispatch `1D50658`, compare `1D506C6`, script literal `1D507A4`; native PC special `3C` |
| Wild species/level | Constructor hook `1D68B74`; continuation `1D68B7C`, cave `13FD280` |
| Shiny ordinary wild | Nature constructor `3DDBC`; gender/letter and ordinary branches `3DE44` / `3DEAA`; caves `13FD000`, `13FD080`, `13FD100` |
| Teleport | Native SetWarp arguments `553B2`; setter/apply `5538C` / `55378` |
| Emergency party recovery | Callback literal `12424`, authored payload cave `13FD400`, native heal `A0058` |
| Persistent Protect | Guarded CodeBreaker OR; callback `03004F84`, player flags `02023E8C` with stride `16` |

Caves are exact-fingerprint padding, independently checked before generation;
these codes affect emulator memory only. No ROM patching/export API is added.
Mercury shiny generation needs all three native constructor paths; testing only
the letter-filter branch missed ordinary gender-constrained wild generation.
Capture records the ball's native `item_type`, not its inventory ID. Native
capture guards are not assumed to equal another ROM's trainer-flag branch.
Mercury egg readiness is a flag: the pending halfword is not an immediately
written offspring PID. Tests observe the native readiness flag.

BW/DP compatible-daycare codes additionally suppress the native charm's flat
`+20` adjustment at `310EBC`. Otherwise zero compatibility would become positive.
The positive-compatibility check at `70B2C` alone was deliberately rejected in
older work; the two-site solution now retains incompatible pairs as zero.
Ultimate's different positive-compatibility path has its own binding.

Bounded tests performed with final CLI codes in mGBA:

- Four existing ROMs: 640 native daycare cases each, including charm, incompatible,
  empty, pending and pre-checkpoint controls. BW/DP's unpatched charm produced
  readiness in 8/32 incompatible cases; enabled groups produced 0/32.
- Mercury: 360 native hatch boundary/bad-egg cases; 640 native daycare cases;
  768 species/level/slot/shiny cases; 32 exact non-wild constructor controls;
  256 lead-ability checks; five native SetWarp/Apply transitions; 90 native ball
  command cases including two native blocking flag branches.
- Mercury walking: a native initialized map/header, three tile attributes,
  64 seeds and off/on groups. Patched encounters are zero; controls encounter.
  This is a native routine comparison, not a claim of every full-frame terrain.
- All five: full single-battle off/on Protect comparisons, neighboring flags and
  overworld guard; emergency recovery of party/reserves/active records with
  opponent bytes unchanged and no-key/link/Safari/multi/partner/HP/disabled guards.
- Mercury PC: actual SELECT → Organize → deposit → native slot move → game Save
  → fresh boot/Continue → withdrawal → reopen/cancel → exit/walk, exact native
  individual bytes and registered item preserved.
- Encrypted GameShark decoding, allowed byte ranges and eight toggle/reset cycles
  per Mercury ROM-memory recipe; four-line Protect is checked separately in battle.
  Authored emergency payloads reproduce from source for all three engine layouts.

No mobile emulator, complete facility walkthrough, long multi-code combination
or every story capture/teleport/PC restriction is newly certified.

## Correction: 25 compressed boxes, not fourteen 80-byte records

The earlier Mercury storage claim based on a header-sized contiguous storage
block was insufficient. Empty-box fixtures and comparing that block's bytes did
not exercise native deposit/withdrawal. Older claims of box record semantics and
created-box round trips using an 80-byte layout are withdrawn. No automatic
migration of potentially miswritten older saves is attempted.

Native converter `1D58854` compresses to **58 bytes**; `1D58628` expands to 80.
`GetCompressedMonPtr 1D58604` reads **25** pointers at `1DDEB68`.

| Boxes (one-based) | SAV logical block / offset | Native RAM |
|---|---|---|
| 1–19 | Storage + `4`, stride `1740` per box | `02029318` onward |
| 20–22 | Sector extensions + `19D0` | `0203CB44` onward |
| 23–24 | Main + `1F08` | `02027434` onward |
| 25 | Trainer + `B0` | `02024638` |

Native names use `1DDEBCC`; wallpapers use `1DDEC30`. Names for the first 14
start at storage `8344`, later names run backward in 9-byte slots; extra
wallpapers start at `8128`. No names or wallpapers are extracted into code.

Compact fields preserve header identity, growth, packed four 10-bit move IDs,
EVs, Pokérus, origin and IVs. Header byte 19's upper five bits encode the native
tera override, **not nature**. The valid-type table is read at `1DE0A18`;
code 31 is retained. Default tera selection uses native species types/PID parity.
Native PC storage omits current PP, contest condition and ribbons; withdrawal
rebuilds maximum PP from the ROM and PP Ups. The editor restricts unrepresentable
box edits instead of silently discarding them. Ordinary box moves/copies/swaps
and sorting keep the exact 58-byte record, including reserved tera bits.

- **320 independent native compress/expand vectors** pass byte-for-byte Rust
  comparisons, spanning all 32 tera codes, five species (including dual types), both PID parities,
  PP Ups, boundary move IDs and deliberately nonzero unstored fields.
- Public synthetic tests move an exact compact record through all **750** slots
  and back, check metadata for all 25 boxes, preserve unrelated reserved bits and
  reject unsupported edits without changing SAV bytes.
- A disposable fixture created through native converters populates boxes 1, 20,
  23 and 25. Editor IV/EV/marking, batch friendship, occupied swap and empty-slot
  move survive normal-key game Save and fresh Continue. The full-frame verifier
  compares **every native box pointer**, including SB1/SB2/extension regions, plus
  party and inventory. No user save is overwritten.

## Reproduction

Build `gen3-cli` and `scripts/native/storage_cheat_probe.c` against mGBA 0.10.5.
Supply exact private ROM and, for frame tests, disposable matching battery/state
fixtures through environment/CLI options. Run:

```sh
python3 scripts/verify_mercury_cheats_mgba.py --probe /private/probe.dylib
python3 scripts/verify_mercury_cheats_mgba.py --probe /private/probe.dylib \
  --only compact --compact-vectors /private/compact.json
GEN3_COMPACT_VECTORS=/private/compact.json cargo test -p gen3-core \
  compact_storage_matches_native_mercury_converters -- --ignored
python3 scripts/verify_mercury_cheats_mgba.py --probe /private/probe.dylib \
  --only pc --pc-state /private/overworld.state --pc-save /private/test.sav \
  --output /private/pc-results
```

`verify_save_roundtrip.py` accepts a separate private scenario to certify editor
edits through normal game save/reboot without cheat sets, savestates or RAM writes.
The adapter layout is separate from the Pokémon codec and can support another
verified compact-storage engine without hard-coded ROM-name checks.

## Battle form correction

CFRU `FD/FE` table rows with parameter zero are **return-to-base** records, read
by native reversal routines `1D4559C` and `1D21484`. They are excluded from
forward battle transformations; variants identify Mega/move-triggered, Primal
and Ultra transformations separately. Mercury now has 127 forward references;
that is not a count of accessible transformations. Base Pikachu/Seadra are no
longer incorrectly excluded as battle-only wild species. Other adapters keep
separate transformation rules. See [Wiki audit](mercury-wiki-journal-20261006.md).


Final checks: 129 public core tests passed (45 opt-in skipped); selected five-ROM
read-only reference/guide, exact cheat catalogs, Mercury ROM/table and compact
converter opt-in checks passed. Bilingual cheat copy/export/context isolation,
player-reference/guide flows and compact-box/party field boundaries passed in
Chrome. TypeScript/Vite build, workspace compile, formatting, payload reproduction
and release-input tests passed. No application package was built or released.
