---
name: gen3-rom-research
description: Analyze user-supplied Gen III GBA Pokémon ROM hacks, locate text and game tables, trace encounter scripts, and gather evidence for a version adapter or ROM-specific question.
---

Work on the supplied bytes, not the filename's implied version. Research and
questions do not require a known MD5. Published write adapters do.

Use [the research workflow](references/research.md) for locating data and
[save invariants](references/saves.md) before proposing a save writer.

Record each finding as a file offset, raw bytes, interpretation, cross-reference,
and confidence. Distinguish an observed table record from a reachable game event.
When a source is incomplete, say “not found in parsed sources”; do not conclude
“impossible”. Do not turn plausible pointer patterns into verified table bounds.

This skill is standalone: Python's standard library can read bytes and pointers.
It does not import editor code or need a running GUI. A decompiler/disassembler
is useful for proving which relocated table the game actually reads.

For a new adapter, separate table locations and bounds, binary layouts/codecs,
and game-rule behavior. Test a generated fixture publicly and the exact ROM
locally. Keep ROMs, extracted artwork, game text dumps, and user saves out of the
repository. Hashes and offset evidence can be published.
