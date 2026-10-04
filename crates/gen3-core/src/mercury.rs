//! Exact-ROM, read-only adapter for Pokémon Mercury FC 1.1.
//!
//! Only independently located tables are enabled. In particular, this does
//! not borrow Emerald save offsets or world pointers for a FireRed/CFRU ROM.
use crate::{
    adapter::{
        Capabilities, EvolutionFormat, LearnsetFormat, MoveFormat, PokemonCodec, RomFormats,
        ScriptFormat, SpeciesFormat, TrainerFormat,
    },
    forms::BattleFormRules,
    profile::{
        MapGraphics, Profile, SaveLayout, SectorChecksum, SplitText, SpriteRules, Table,
        TeachingRules,
    },
};

const NONE: Table = Table {
    offset: 0,
    count: 0,
    stride: 0,
};
const UNVERIFIED_SAVE: SaveLayout = SaveLayout {
    sector_checksum: SectorChecksum::Sum,
    pokemon_codec: PokemonCodec::Gen3,
    pockets: &[],
    dex: None,
    sizes: [0; 14],
    party_count: 0,
    party: 0,
    boxes: 0,
    slots: 0,
    key: 0,
    money: 0,
    coins: 0,
};

pub const PROFILE: Profile = Profile {
    id: "mercury-fc-1.1",
    label: "宝可梦水银 FC · 1.1",
    md5: "7e0898caf6e7d41e8c59f838e8e595f1",
    size: 33_554_432,
    formats: RomFormats {
        trainers: TrainerFormat::DarkPhantom,
        scripts: ScriptFormat::DarkPhantom,
        object_graphics_offset: 1,
        species: SpeciesFormat::Cfru28,
        moves: MoveFormat::Gen3,
        learnsets: LearnsetFormat::Move16Level8,
        evolutions: EvolutionFormat::Cfru,
    },
    capabilities: Capabilities {
        save_edit: false,
        world: false,
        dex: false,
        complete_learnsets: false,
        individual_sprites: true,
        battle_forms: true,
    },
    ability_count: 300,
    ability_description_count: 255,
    species_abilities: None,
    trainer_exclusions: &[],
    max_level: 100,
    experience_table: None,
    battle_forms: Some(BattleFormRules::CfruEvolutionMethods),
    storage_forms: None,
    form_families: None,
    nature_names: 0x1ff3ab8,
    nature_effects: 0x252b48,
    nature_product_u16: false,
    type_names: Table {
        offset: 0x1db1998,
        count: 24,
        stride: 7,
    },
    move_names: SplitText {
        first: 0x1d8fc34,
        second: 0,
        split: 1015,
        stride: 13,
    },
    move_descriptions: 0,
    ability_names: SplitText {
        first: 0x1d87140,
        second: 0,
        split: 300,
        stride: 13,
    },
    ability_descriptions: SplitText {
        first: 0x1c93dfc,
        second: 0,
        split: 300,
        stride: 4,
    },
    mail_items: [0, 0],
    national_dex: 0x17e7a64,
    species: Table {
        offset: 0x141b350,
        count: 1554,
        stride: 11,
    },
    base_stats: Table {
        offset: 0x176dfbc,
        count: 1554,
        stride: 28,
    },
    moves: Table {
        offset: 0x1df68e3,
        count: 1015,
        stride: 12,
    },
    move_category_offset: 10,
    hidden_power: None,
    contest: None,
    items: Table {
        offset: 0x7c7e00,
        count: 750,
        stride: 44,
    },
    evolutions: Table {
        offset: 0x1788f5a,
        count: 1554,
        stride: 128,
    },
    evolution_overrides: &[],
    learnsets: 0x17c32bc,
    eggs: 0,
    teaching: TeachingRules {
        tm_first_item: 0,
        tm_count: 0,
        tm_stride: 0,
        tutor_count: 0,
        tutor_stride: 0,
        shared_lists: None,
        egg_words: 0,
    },
    tm_moves: 0,
    tm_bits: 0,
    tutor_moves: 0,
    tutor_bits: 0,
    sprites: 0x17bb0a4,
    palettes: 0x17d64cc,
    shiny_palettes: 0x17e49d4,
    sprite_rules: SpriteRules {
        unown_species: 0,
        unown_b_sprite: 0,
        spinda_species: 0,
        spinda_spots: 0,
        second_frame_species: 0,
        female: None,
    },
    maps: 0,
    map_palette_banks: [0, 0],
    map_graphics: MapGraphics {
        primary_tiles: 0,
        primary_metatiles: 0,
        layers: 0,
    },
    regions: 0,
    region_count: 0,
    wild: 0,
    wild_selection: None,
    feebas: None,
    fishing_rods: [0; 3],
    trainers: NONE,
    trainer_classes: NONE,
    trainer_sprites: NONE,
    trainer_palettes: 0,
    object_graphics: &[],
    object_palettes: NONE,
    script_actors: &[],
    map_groups: &[],
    map_counts: &[],
    save: UNVERIFIED_SAVE,
};

#[cfg(test)]
mod tests {
    use super::PROFILE;
    use crate::{cheats::CheatRom, rom::Rom, session::Session};

    #[test]
    #[ignore = "requires exact local ROM via GEN3_ROM_MERCURY"]
    fn exact_rom_read_only_regression() {
        let path = std::env::var("GEN3_ROM_MERCURY").unwrap();
        let bytes = std::fs::read(path).unwrap();
        let rom = Rom::open(bytes.clone()).unwrap();
        assert_eq!(rom.profile.md5, PROFILE.md5);
        let catalog = rom.catalog().unwrap();
        assert_eq!(catalog.species.len(), 1553);
        assert_eq!(catalog.moves.len(), 1015);
        assert_eq!(catalog.items.len(), 750);
        assert_eq!(catalog.abilities.len(), 300);
        assert_eq!(catalog.battle_forms.len(), 233);
        assert_eq!(rom.species(1).unwrap().name, "妙蛙种子");
        assert_eq!(rom.species(1553).unwrap().name, "桃歹郎");
        assert_eq!(rom.item(1).unwrap().name, "大师球");
        assert_eq!(rom.move_info(1).unwrap().name, "拍击");
        assert_eq!(rom.evolutions(1).unwrap()[0].target, 2);
        assert_eq!(rom.level_moves(1).unwrap()[0].move_id, 33);
        assert!(rom.level_moves(1300).unwrap().is_empty());
        assert_eq!(
            rom.battle_forms(3)
                .unwrap()
                .iter()
                .filter(|form| form.source == 3)
                .count(),
            2
        );
        assert!(rom.ability(299).unwrap().description.is_empty());
        assert_eq!(rom.world().err().unwrap().code, "unsupported_feature");
        for id in [1, 6, 25, 150, 1001, 1553] {
            let normal = rom.sprite(id, false).unwrap();
            let shiny = rom.sprite(id, true).unwrap();
            assert!(normal.starts_with(b"\x89PNG"));
            assert!(shiny.starts_with(b"\x89PNG"));
            assert_ne!(normal, shiny, "species {id} shiny palette");
        }
        for id in 1..PROFILE.species.count as u16 {
            if rom.valid_species(id).is_err() {
                continue;
            }
            assert!(rom.sprite(id, false).unwrap().starts_with(b"\x89PNG"));
            assert!(rom.sprite(id, true).unwrap().starts_with(b"\x89PNG"));
        }
        assert_eq!(rom.sprite(1300, false).unwrap_err().code, "species_id");
        assert_eq!(
            Session::new(rom)
                .load(vec![0; 0x20000], None)
                .unwrap_err()
                .code,
            "unsupported_feature"
        );
        assert!(CheatRom::open(&bytes).unwrap().catalog().entries.is_empty());
        let mut changed = bytes;
        changed[0x141b351] ^= 1;
        assert_eq!(Rom::open(changed).err().unwrap().code, "unsupported_rom");
    }
}
