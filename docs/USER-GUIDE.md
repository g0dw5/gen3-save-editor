# Gen III ROM Hack Editor

Open your own supported ROM to browse game information. Open a matching battery
save (`.sav` / `.srm`) to edit the party, boxes and items. ROMs and saves remain
on your computer; the program never writes ROM bytes.

## Browse and find

The ROM reference window has Pokémon, moves, items, abilities, maps and trainers.
Names, artwork and values come from the opened ROM. You can follow a source to
its map, highlight a tile and return to the previous reference.

- **Pokémon:** compare base stats with the reviewed official reference, follow
  actual evolution links, and filter learnable moves by source, category, type
  or name. Forms and battle transformations are shown separately.
- **Moves:** find machines through pickups or shops, visit identified tutors,
  and view offspring with the egg move. This list does not guarantee any pair
  of parents can pass it on; check the Pokémon’s breeding information.
- **Items:** look up identified pickups, NPC rewards, shops and held items.
  Encounter chance and held-item chance are different probabilities.
- **Maps:** toggle NPCs, item balls, hidden items and rewards independently.
  Pokémon Encounter uses compact tables by method. If the ROM has time tables,
  select a time period or saved game time. The program does not change the clock.
- **Trainers:** check the roster and identified locations. Ultimate Emerald has
  a difficulty selector; dynamic previews use your saved party automatically.
  Without a save, unresolved generated values remain unknown.

Reserved item slots and maps with mismatched layouts or no identified entrance
are hidden by default. **Show entries with uncertain use** restores these list
entries. Hidden does not mean proven inaccessible: special entrances and scripts
may remain unparsed. A direct map link can still display its target.

## Adventure guide

The separate **Adventure guide** reads dialogue, event branches and NPC rewards
from the opened ROM. Search by dialogue, reward or area and view the related map
and prerequisite tree. Map thumbnails open the full map with the target marked.

With a save, receipt markers can identify collected rewards. Known conditions
being met do not guarantee current access. Missing or unverified conditions stay
undetermined. An item missing from the bag never proves it was not collected.

The supported Spanish Rocket ROM additionally exposes its identified main-story
state and possible next actions. These numbers can move backward or count
parallel objectives; they are not completion percentages. Mercury 1.2 additionally reads native quest titles, objectives and journal pages.
A save identifies accepted/completed quests and currently visible pages; later
pages stay collapsed and map links locate identified quest scenes. Other games
do not yet have a verified overall story-state tracker. Their identified
NPC rewards and prerequisite clues remain searchable; this is not a complete
walkthrough of every side quest. Dialogue may include other branches of a scene,
and prerequisite links can be alternatives rather than a required sequence.

## Edit safely

Party and all boxes remain expanded. Dragging between occupied slots swaps the
individuals. The Pokémon editor retains its active tab when selection changes.
Search items and moves using names from the current ROM.

Apply changes to the current session, use undo/redo to review them, then export a
SAV. Keep the original backup and use normal in-game saving after loading the
export in your emulator. External file changes are checked before export.
**Free editing** permits exceptions; it does not bypass structural save checks.
ROM reference, adventure guide and cheat-code generation are read-only queries.

Cheat codes depend on the exact ROM and code format. Follow each entry’s usage
and stop conditions; a save editor does not enable codes in the emulator for you.
Mercury 1.2’s cheat coverage remains limited. Supported input fingerprints appear
on the welcome screen; Mercury 1.1 is not supported.

Switch the interface between English and Chinese with the language button.
Interface language does not translate the ROM’s own Pokémon/item/move names.
