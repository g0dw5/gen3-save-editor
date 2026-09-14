# Sources and attribution

The binary format research uses the public `pret/pokeemerald` decompilation as a
reference, supplemented by inspection of the exact supported ROM files. The Rust
implementation is written for this project. ROM-derived text, sprites, maps and
save files are not shipped.

- https://github.com/pret/pokeemerald (structure and algorithm reference)
- https://github.com/Wokann/Pokemon_GBA_Font_Patch (Chinese encoding mapping)
- Existing user-owned `pokemon-hack` research (Dark Phantom addresses and evidence)

`crates/gen3-core/data/charmap.txt` records byte-to-character encoding facts used by
the Chinese font patch. It contains no font glyphs. The original mapping was
identified as `pokeE/PMRSEFRLG_charmap.txt`; the single-byte `71` narrow-space case
is handled separately. Encoding mappings are not game-name translations.

Dependency licenses remain with their authors. The project MIT license applies
to project code and does not grant rights to ROM contents or third-party works.
