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

## Official base-stat reference / 官方种族值参照

The independent numeric comparison dataset `ui/data/official-stats.json` is
derived from **52Poké Wiki contributors**, principally the
[Generation IX base-stat list](https://wiki.52poke.com/wiki/种族值列表（第九世代）),
with absent entries filled from the Generation VIII and VII lists.
Source revisions: IX **2561145**, VIII **2549241**, VII **2548788**.
License: **CC BY-NC-SA 3.0**, <https://creativecommons.org/licenses/by-nc-sa/3.0/>.
Changes: numeric extraction, compact JSON conversion, stat-order conversion,
and merging missing earlier-generation rows. No wiki artwork/articles are
included. This dataset is separately attributed and is not MIT-licensed;
its attribution and license apply to redistribution, including packaged builds.
See `ui/data/README.md` for provenance and the reproducible extraction script.

官方对照数值来源为神奇宝贝百科贡献者，以第九世代为先，缺项补用最近收录世代。
数据单独保留 CC BY-NC-SA 3.0 署名及许可，不把它列为 MIT 代码，也不包含神百图片、
文章或任何 ROM 内容。修改器中当前 ROM 的数值始终实时读取，不以该资料替代。
