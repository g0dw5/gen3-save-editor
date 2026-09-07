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
codec or rule implementations. The current save codec is Emerald-family; the
current world parser targets the Dark Phantom engine. It is not claimed to parse
FireRed or arbitrary Emerald expansion hacks by changing an offset.

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
