# ROM research

## Identify and scope

Record size, MD5, SHA-256 and the GBA header at 0xA0–0xBF. Header strings are
hints, not authentication. A hack may be shorter than its nominal cartridge
capacity. All pointer reads require explicit bounds; GBA ROM addresses normally
start at 0x08000000. Keep the original immutable during research.

Use a narrowly scoped question: “where can species X appear?” requires more than
the wild encounter table. Inventory grass, water, rods, rock smash, map scripts,
gifts, eggs, starters, trades, roamers and battle specials separately.

## Text and Chinese fonts

Find several known strings from screenshots or supplied official documentation.
A Chinese character may use two bytes, including a zero second byte. 0xFF is
often the terminator, 0xFE a newline; this must be verified against the print
routine. Do not decode the ROM as UTF-8 or stop on zero bytes.

Trace the text drawing function to its glyph indexing and tile blitter. Locate
font graphics separately from the byte-to-character map. Check tile depth,
character dimensions, multibyte lead ranges, control sequences and table banks.
Validate names with both Chinese and Latin characters, punctuation, short and
maximum encoded lengths. A decoded label is evidence only after comparing its
glyphs or another independent source. Never package the glyph graphics as a
substitute for runtime extraction.

## Species, moves and items

Locate multiple adjacent records with structural constraints: six species base
stats, type IDs, growth curve, abilities; move power/PP/type/target/effect; item
name, price, effect and description pointer. Prove record stride using neighboring
records and code cross-references. Names may live in multiple relocated banks.
Internal species IDs, national numbers and regional numbers are separate spaces.

Find front sprite and palette pointer tables; validate GBA LZ10 headers and
back-reference bounds. Render a few species, including shiny palettes. A valid
compressed stream does not prove the correct sprite frame or species mapping.

For learning sources, inspect level-up pointer tables, packed move/level words,
terminators, TM/HM bitsets, tutor bitsets and egg lists. Hacks can expand move IDs
or change word width. Include pre-evolution sources and annotate the contributing
species. Cross-check a documented species and a boundary species in each bank.

## Maps and encounters

Follow map-group pointer arrays to map headers, layouts, event arrays and map
script lists. Infer group bounds from references and valid structures rather than
continuing until any pointer happens to fail. Render blocks through metatiles,
tiles, flips and palette banks. Label a static layout preview accurately; it does
not simulate NPCs, movement permissions, animated tiles or event state.

Wild records usually select four encounter tables. Keep slot weights separate
from map encounter-rate bytes and gameplay probability. Rod tables have separate
subgroups. Preserve map IDs and the exact record offset.

Walk scripts from map objects, coordinate events, background events and map
script tables. Use an opcode length table verified against the target engine.
Follow calls and branches with per-path constant state. Clear values when a
callee or special may overwrite them. Bound steps and report where traversal
stops. Avoid scanning every byte for an opcode: data bytes resemble instructions.

For gifts, eggs, static battles and variable-driven specials, report prerequisite
conditions as unresolved until traced. Inspect in-game trade tables and roamer
state separately. An official guide is an independent check, not an excuse to
silently invent a missing script interpretation.

## Trainers

Find code references to the active trainer header table; relocated obsolete
copies can look entirely valid. Party count and flags select party layout.
Inspect actual load instructions: a u8 level followed by flags can look like a
nonsensical u16 level. Custom move and held-item records may have different
strides. Record party offsets and whether moves are explicit or generated from
level-up data. Distinguish unused malformed headers from reachable battles by
linking trainer IDs back to script roots.

## Promotion to a write adapter

A profile needs exact fingerprint, table bounds, codec/layout families and a
capability list. Reuse a codec only after proving its shape, not because offsets
look similar. A new battle engine may require new decoding and rule code even
if all table addresses are configurable. Prove bounds, untouched-byte retention,
rollback, and game behavior before enabling each write capability.
