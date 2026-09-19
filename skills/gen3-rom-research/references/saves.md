# Save research and writing

Distinguish battery save from emulator state. Find flash sector signatures,
section IDs, counters, checksums and payload lengths. A complete bank requires
all expected IDs exactly once with one consistent counter. Compare counters with
wraparound semantics. Do not assemble one bank from unrelated generations.

Emerald-style saves have rotating physical sectors. Concatenate logical payloads
before reading PC storage; an 80-byte record can cross a sector boundary. Preserve
padding, unowned bytes and the other bank. Learn field offsets through controlled
game saves: change one property, save, compare logical sections, repeat.

Gen III boxed Pokémon commonly have an 80-byte structure, with a 100-byte party
variant. Four 12-byte substructures are shuffled by PID modulo 24 and encrypted
with PID XOR OT ID. The checksum covers decrypted canonical words. Validate all
24 permutations against an independent vector. PID changes require decrypting
with the old key, updating, reordering, encrypting with the new key and recomputing
the checksum. Preserve unknown fields and synchronize mirrored egg/species flags.

Changing level, nature, EVs or IVs changes party cached stats. Decide how to retain
HP damage and fainted status explicitly. Party-to-box conversion discards only
party-only fields. Compaction and count changes must be transactional. Last-party
and mail attachment behavior need explicit rules.

Separate storage constraints from game rules: IV bit widths and record checksums
always hold, while free editing can exceed EV totals or learned-move assumptions.
Unknown learning sources are inconclusive. Never silently “legalize” other fields.

Write only after an in-memory transaction validates. Keep undo snapshots, show
actual before/after values, detect external source changes, make a backup, stage
and verify in the destination directory, then rename. Test re-opened output,
corrupt banks, last PC slot, cross-sector records and failed batch rollback.
A synthetic save tests serialization; it does not establish in-game progression
or emulator compatibility. Verify a disposable real gameplay save separately.

## Emulator input preflight

Before automating a desktop emulator, verify the loaded ROM/save paths, switch
to an English input source, focus the game window, and read the actual joypad
mapping. Confirm one harmless input visibly changes the game before proceeding.
If it does not, inspect pause/background-input settings and input interception;
do not repeatedly send keys or conclude that the save failed to load. A menu
virtual-button/auto-hold control can isolate keyboard delivery from game input;
release every held button immediately after the intended action.

2026-09-19, VBA-M 2.2.3 on macOS: simulated Return/L keys did not affect the
loaded Rocket test save, while menu Auto hold Start/A/B did. The user reported
a possible Chinese input-source conflict and manually loaded the save. The
input source is a troubleshooting prerequisite, not a confirmed root cause;
focus, background input and event delivery remain possible contributors.
