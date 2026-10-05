# Architecture / 架构

The three layers can be used independently:

1. `skills/gen3-rom-research`: methods and evidence practices, no editor imports.
2. `crates/gen3-core`: ROM profiles, bounded binary reads, codecs, save transactions.
3. `crates/gen3-cli` and `apps/desktop` + `ui`: consumers of the same core.

A `Rom` owns immutable bytes and one verified profile. A `Session` owns its save,
undo/redo and source fingerprint. Switching profiles creates a new session.
React sends typed actions; it never computes save checksums or writes offsets.

## What configuration can reuse

Table address/count/stride, split text banks, map-group counts and save payload
sizes are data. BW and DP share these descriptors and codecs while retaining
separate exact MD5 identities. Names, stats and assets still come from each ROM.

Configuration is not a universal reverse-engineering shortcut. Packed 9-bit
learnsets versus wider entries, expanded species IDs, custom battle categories,
font encodings, save encryption, extra flags and event semantics require explicit
codec or rule implementations. Explicit codecs cover stock encrypted Gen III, Rocket, Ultimate and CFRU
individual layouts. Script/trainer formats are selected separately from the
individual codec. The five registered fingerprints share readers where their
layouts agree; registration does not establish complete script semantics or
playthrough reachability. See the [capability matrix](capability-matrix.md).

新增版本时，先确定差异属于哪一类：地址与表长 → 配置；结构与编码 → codec；
游戏行为和合法性 → 规则；证据不足 → 先关闭该能力。不要复制整套存档代码，也
不要把所有分歧藏进越来越长的版本条件分支。

## Transaction contract

`Session::apply` clones the active save, applies the complete action, validates,
then commits one undo step. Failed batches do not change bytes. Direct low-level
`Save` mutators are implementation building blocks; external callers should use
`Session` for transactional guarantees. Export stages in the target directory,
verifies staged bytes, backs up an existing destination, then renames. The active
bank is edited; the other bank and unowned bytes remain intact.

A legal-game judgment is separate from binary validity. Unknown source findings
remain warnings. Free editing cannot authorize an unsupported fingerprint, an
out-of-bounds write, invalid checksum or unrepresentable field.

## UI and localization

All party and PC slots retain stable location keys. Dragging an existing record
moves/swaps; a ROM encounter creates a draft at an empty destination. Floating
reference windows are nonmodal and never edit the save implicitly. Before/after
changes are derived from actual parsed state, not just the requested patch.

UI strings live in `ui/i18n.tsx`, with Chinese required to contain every English
key. ROM names remain in their ROM language. New translations must preserve
technical IDs in developer details, while normal labels explain their meaning.

Resource prerequisites use `script_resources.rs` native-dispatch/width rules and
`map_events.rs` symbolic RESULT predicates. Save overlays use the exact item
pocket and native duplicate-slot strategy. Unknown alternate-bag context and
post-mutation checks are explicit; enough holdings do not certify access or
payment. Required-item links and the shared bilingual `ConditionDetails` formatter
connect query, maps, planning and safe HTML without a bundled quest catalog.

## Known research limits

Static map rendering does not simulate events, sprites, animation or collision.
Script analysis is bounded and stops at unknown opcodes; `map_report` returns
stop offsets. Script conditions, trades, roamers and special event engines need
additional coverage. Trainer tables contain unused or inconsistent rows; the
reader reports anomalies rather than silently repairing flags. No full legality
oracle or complete playthrough reachability is claimed.

Mail attachments are linked save structures. Assigning/removing held mail or
transferring/importing/deleting a Pokémon carrying mail is blocked until linked
mail editing is implemented. Ordinary item pockets and existing unrelated mail
bytes remain preserved.

Verified format reference for trainer level width:
[pret/pokeemerald include/data.h](https://github.com/pret/pokeemerald/blob/master/include/data.h).
Dark Phantom table offsets and anomalies are established against the supplied
bytes; upstream struct definitions alone are not proof that every hacked field
has identical runtime semantics.

## Native wild held-item queries

`wild_items.rs` runs the current ROM's ordinary single-wild held-item assignment
routine in isolated ARM RAM, including native RNG, getters and setter. Adapter
metadata contains addresses/layouts, not content or percentages. Acquisition
indexing reads current held columns and native exceptional tables, joins actual
random encounter references and preserves selectors. Candidate rows are not
obtainability proof. Distribution caching binds to the ROM data Arc and includes
normalized layout plus the complete leading party record; no cache is bundled.
A uniform 16-bit native output supplies conditional counts, separate from encounter
slot probabilities. Unqualified contexts remain unknown. Bilingual UI and HTML
share `WildHeldDetails.tsx`; queries never modify a ROM or SAV.

## Native daycare scenarios

`daycare_state.rs` copies logical SAV blocks into disposable RAM and executes the
current ROM's configured presence, availability and service-status getters. It
checks that copied blocks remain unchanged. Structurally invalid parents remain
unknown and never enter native compatibility/receipt routines. Deposited-parent
scenarios use the original 80-byte records; they are not save-edit locations.
The snapshot is separate from simulated production, acquisition caches and live
emulator RNG. Mercury's native flag hook is executed without inferring availability
from its legacy pending field. Custom additional service records remain unknown.

`breeding.rs` shares the bounded read-only `Sandbox` and codec readers. Adapter
configuration holds entry points/RAM layouts/dispatch identities; current ROM
code supplies compatibility, offspring selection and individual construction.
Stored parents are read as original boxed records; simulated parents are named
minimal scenarios using ROM experience thresholds. Neither path writes a save.
`map_events.rs` indexes qualified receiving specials and guards without treating
native calls as receipt proof. `AcquisitionIndex.daycare_sources` joins those
references to NPC coordinates, and `BreedingPanel.tsx` supplies the common
bilingual offspring/map/back flow with stale-request guards. No extracted baby
list, incense catalog or percentage fallback is bundled.

The shared CPU corrects only odd Thumb halfword loads whose upstream interpreter
loses the GBA address/rotation behavior. This is instruction semantics, not an
adapter-specific function hook. mGBA full-record and halfword tests establish the
bounded correction; ARM halfword conformance is not implied. Full daycare live
state, complete service setup/access and full collection dependencies remain separate gaps.

`collection.rs` derives bounded directed preparation chains from the same runtime
permanent-evolution index used by acquisition queries. Actual healthy non-egg
individual counts are separate from historical Dex goal coverage. The search is
cycle-safe (eight edges, 512 examined/queued paths), retains compound condition
records and never reverses evolution into breeding. Referenced ancestor sources
share clock/resource overlays, map topology and HTML output. Possession changes
the preparation suggestion, not the acquisition eligibility status. Evolution
item/move/species cross-links are decoded once in the core and shared by both UIs;
names stay in the currently loaded ROM catalog. Complete eligibility and a joint
minimum-individual/route solver remain gaps.

`breeding_production.rs` observes the complete ordinary step before its native
comparison, without replacing functions. It projects each ordinary bag's ordered
slots into disposable descriptors with the correct security key, and reads the
modifier operand and roll parameters from ROM. The verified instruction sequence
defines the theoretical count across all 16-bit draws; unknown patterns fail.
Native pending-availability differences stay in adapter rules, not ROM-name checks.
Production state is an explicit simulation, separate from live daycare state and
the receipt scenario. UI item overrides never call save editing APIs.
