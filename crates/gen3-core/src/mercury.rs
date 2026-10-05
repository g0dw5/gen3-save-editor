//! Exact-ROM adapter for Pokémon Mercury FC 1.2.
//!
//! ROM tables and save layouts are specific to this FireRed/CFRU build. In
//! particular, they do not borrow Emerald offsets or world pointers.
use crate::{
    adapter::{
        Capabilities, EvolutionFormat, LearnsetFormat, MoveFormat, PokemonCodec, RomFormats,
        ScriptFormat, SpeciesFormat, TrainerFormat,
    },
    forms::BattleFormRules,
    profile::{
        MachineItemRange, MapGraphics, Profile, SaveLayout, SectorChecksum, SplitText, SpriteRules,
        Table, TeachingRules,
    },
    save::{Pocket, PocketBlock},
};

// The ROM header reports SB2 0xF24, SB1 0x3D68 and storage 0x83D0. Native
// Saving chunks SB1/storage at 0xFF0 bytes, unlike Emerald's 0xF80 or Rocket's 0xFF4.
// The lengths and bytes were verified against an in-game save and live RAM.
const MERCURY_POCKETS: [Pocket; 6] = [
    Pocket {
        block: PocketBlock::Main,
        id: "pc",
        offset: 0x298,
        count: 30,
        encrypted: false,
        category: 0,
    },
    Pocket {
        block: PocketBlock::Main,
        id: "items",
        offset: 0x310,
        count: 42,
        encrypted: true,
        category: 1,
    },
    Pocket {
        block: PocketBlock::Main,
        id: "key_items",
        offset: 0x3b8,
        count: 30,
        encrypted: true,
        category: 2,
    },
    Pocket {
        block: PocketBlock::Main,
        id: "balls",
        offset: 0x430,
        count: 13,
        encrypted: true,
        category: 3,
    },
    Pocket {
        block: PocketBlock::Main,
        id: "tmhm",
        offset: 0x464,
        count: 58,
        encrypted: true,
        category: 4,
    },
    Pocket {
        block: PocketBlock::Main,
        id: "berries",
        offset: 0x54c,
        count: 43,
        encrypted: true,
        category: 5,
    },
];
const MERCURY_SAVE: SaveLayout = SaveLayout {
    sector_checksum: SectorChecksum::Sum,
    pokemon_codec: PokemonCodec::Cfru,
    pockets: &MERCURY_POCKETS,
    dex: None,
    sizes: [
        0xf24, 0xff0, 0xff0, 0xff0, 0xd98, 0xff0, 0xff0, 0xff0, 0xff0, 0xff0, 0xff0, 0xff0, 0xff0,
        0x450,
    ],
    party_count: 0x34,
    party: 0x38,
    boxes: 14,
    slots: 30,
    key: 0xf20,
    money: 0x290,
    coins: 0x294,
    registered_item: 0x296,
    created_origin_game: 4,
    rtc_trailer_bytes: 16,
    skip_unoccupied_box_records: true,
};

pub const PROFILE: Profile = Profile {
    native_trainers: Some(crate::native_trainer::NativeTrainerRules {
        constructor: 0x09d0b150,
        enemy_party: 0x0202402c,
        battle_flags: 0x02022b4c,
        save_pointers: [0x03005008, 0x0300500c],
        rng: 0x03005000,
        instruction_limit: 1_000_000,
    }),
    clock: Some(crate::clock::ClockRules {
        starts: [4, 8, 17, 20],
        native_predicates: [0x1d20de0, 0x1d20df8, 0x1d20814],
        forced_night_flag: Some(0x1041),
    }),
    event_state: None,
    id: "mercury-fc-1.2",
    label: "宝可梦水银 FC · 1.2",
    md5: "f323df1792ac68462a34b42fe8571533",
    size: 33_554_432,
    formats: RomFormats {
        trainers: TrainerFormat::FireRed,
        scripts: ScriptFormat::FireRed,
        object_graphics_offset: 1,
        species: SpeciesFormat::Cfru28,
        moves: MoveFormat::Gen3,
        learnsets: LearnsetFormat::Move16Level8,
        evolutions: EvolutionFormat::Cfru,
    },
    capabilities: Capabilities {
        save_edit: true,
        world: true,
        dex: false,
        complete_learnsets: true,
        individual_sprites: true,
        battle_forms: true,
    },
    ability_count: 300,
    ability_description_count: 255,
    species_abilities: None,
    trainer_exclusions: &[],
    max_level: 100,
    experience_table: Some(Table {
        offset: 0x1e052c8,
        count: 6,
        stride: 1024,
    }),
    battle_forms: Some(BattleFormRules::CfruEvolutionMethods),
    storage_forms: None,
    form_families: None,
    nature_names: 0x1ff2888,
    nature_effects: 0x252b48,
    nature_product_u16: false,
    type_names: Table {
        offset: 0x1db5da4,
        count: 24,
        stride: 7,
    },
    move_names: SplitText {
        first: 0x1d93f34,
        second: 0,
        split: 1015,
        stride: 13,
    },
    move_descriptions: 0x1cc8ebc,
    ability_names: SplitText {
        first: 0x1d8b430,
        second: 0,
        split: 300,
        stride: 13,
    },
    ability_descriptions: SplitText {
        first: 0x1c93eb8,
        second: 0,
        split: 300,
        stride: 4,
    },
    mail_items: [0, 0],
    national_dex: 0x17e7a78,
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
        offset: 0x1dfd2df,
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
    eggs: 0x1781b64,
    teaching: TeachingRules {
        tm_first_item: 0,
        machine_item_ranges: &[
            MachineItemRange {
                item_first: 289,
                item_count: 50,
                move_first: 0,
            },
            MachineItemRange {
                item_first: 376,
                item_count: 70,
                move_first: 50,
            },
            MachineItemRange {
                item_first: 339,
                item_count: 8,
                move_first: 120,
            },
        ],
        tm_count: 128,
        tm_stride: 16,
        tutor_count: 145,
        tutor_stride: 20,
        shared_lists: None,
        egg_words: 14066,
    },
    tm_moves: 0x17e87be,
    tm_bits: 0x1400494,
    tutor_moves: 0x17e869a,
    tutor_bits: 0x14065b4,
    sprites: 0x17bb0a4,
    palettes: 0x17d64e0,
    shiny_palettes: 0x17e49e8,
    sprite_rules: SpriteRules {
        unown_species: 201,
        unown_b_sprite: 413,
        spinda_species: 308,
        spinda_spots: 0x25265c,
        second_frame_species: u16::MAX,
        female: None,
    },
    maps: 0xb3b100,
    map_palette_banks: [7, 6],
    map_graphics: MapGraphics {
        primary_tiles: 640,
        primary_metatiles: 640,
        layers: 2,
    },
    regions: 0x45f89c,
    region_first: 88,
    region_stride: 4,
    region_count: 256,
    wild: 0xcf915c,
    wild_selection: None,
    wild_time_tables: Some([0x1e4779c, 0x1e3ffb0, 0x1e458a0, 0x1e4380c]),
    feebas: None,
    fishing_rods: [262, 263, 264],
    trainers: Table {
        offset: 0x23eac8,
        count: 743,
        stride: 40,
    },
    trainer_classes: Table {
        offset: 0x23e558,
        count: 107,
        stride: 13,
    },
    trainer_sprites: Table {
        offset: 0x23957c,
        count: 148,
        stride: 8,
    },
    trainer_palettes: 0x239a1c,
    object_graphics: &[Table {
        offset: 0x961600,
        count: 240,
        stride: 4,
    }],
    object_palettes: Table {
        offset: 0x1e0e080,
        count: 341,
        stride: 8,
    },
    object_palette_supplements: &[Table {
        offset: 0x3a5158,
        count: 18,
        stride: 8,
    }],
    script_actors: &[],
    map_groups: &[],
    map_counts: &[
        5, 127, 60, 116, 4, 6, 8, 10, 6, 8, 20, 10, 8, 2, 10, 4, 2, 2, 2, 1, 1, 2, 2, 3, 2, 3, 2,
        1, 1, 1, 1, 7, 5, 5, 8, 8, 5, 5, 1, 1, 1, 2, 5, 10, 17, 9, 10, 8, 8, 23, 51, 11, 42, 19,
        21, 61, 82, 16,
    ],
    save: MERCURY_SAVE,
};

#[cfg(test)]
mod tests {
    use super::PROFILE;
    use crate::{
        cheats::CheatRom,
        pokemon::{self, PokemonPatch, Policy},
        rom::Rom,
        save::{Location, Save, TrainerPatch},
        session::Session,
    };

    #[test]
    #[ignore = "requires the removed 1.1 ROM via GEN3_ROM_MERCURY11"]
    fn removed_11_rom_is_rejected() {
        let bytes = std::fs::read(std::env::var("GEN3_ROM_MERCURY11").unwrap()).unwrap();
        assert_eq!(Rom::open(bytes).err().unwrap().code, "unsupported_rom");
    }

    #[test]
    #[ignore = "requires exact ROM and in-game Mercury save"]
    fn exact_save_layout_regression() {
        let rom = Rom::open(std::fs::read(std::env::var("GEN3_ROM_MERCURY12").unwrap()).unwrap())
            .unwrap();
        let original = std::fs::read(std::env::var("GEN3_SAVE_MERCURY12").unwrap()).unwrap();
        let save = Save::open(original.clone(), rom.profile.save).unwrap();
        assert_eq!(save.active_slot, 1);
        assert_eq!(save.party_count(), 0);
        assert_eq!(save.trainer(&rom).unwrap().tid, 33272);
        assert_eq!(save.bag().unwrap().len(), 216);
        let boxes = save.boxes(&rom).unwrap();
        assert_eq!(boxes.len(), 14);
        assert_eq!(boxes.iter().map(|b| b.count).sum::<usize>(), 0);
        assert_eq!(boxes[0].name, "盒子1");
        assert_eq!(boxes[13].name, "盒子14");
        assert!(save.all(&rom).unwrap().is_empty());
        assert_eq!(save.data, original);
        let mut session = Session::new(rom.clone());
        session.load(original.clone(), None).unwrap();
        assert_eq!(session.snapshot().unwrap().trainer.tid, 33272);
        let raw = Save::open(original[..0x20000].to_vec(), rom.profile.save).unwrap();
        assert_eq!(raw.trainer(&rom).unwrap().tid, 33272);
    }

    #[test]
    #[ignore = "requires exact ROM and user-supplied Mercury 1.2 save"]
    fn exact_save_12_edit_probe() {
        check_save_edit(
            "GEN3_ROM_MERCURY12",
            "GEN3_SAVE_MERCURY12",
            "GEN3_SAVE_MERCURY12_PROBE",
        );
    }

    fn check_save_edit(rom_key: &str, save_key: &str, output_key: &str) {
        let rom = Rom::open(std::fs::read(std::env::var(rom_key).unwrap()).unwrap()).unwrap();
        let original = std::fs::read(std::env::var(save_key).unwrap()).unwrap();
        let mut save = Save::open(original.clone(), rom.profile.save).unwrap();
        let trainer = save.trainer(&rom).unwrap();
        let ot = (trainer.sid as u32) << 16 | trainer.tid as u32;
        let mon = pokemon::create(&rom, 1, ot, &trainer.name, 5, 0x1234_5678).unwrap();
        save.insert(Location::Party { slot: 0 }, &mon, &rom)
            .unwrap();
        save.insert(
            Location::Box {
                box_index: 0,
                slot: 0,
            },
            &mon,
            &rom,
        )
        .unwrap();
        save.edit_trainer(
            &TrainerPatch {
                money: Some(1000),
                ..TrainerPatch::default()
            },
            &rom,
        )
        .unwrap();
        save.edit_bag("balls", 0, 4, 5, &rom, Policy::Standard)
            .unwrap();
        save.edit_box(0, "验证", 3, &rom).unwrap();
        save.validate(&rom).unwrap();
        let reopened = Save::open(save.data.clone(), rom.profile.save).unwrap();
        reopened.validate(&rom).unwrap();
        assert_eq!(reopened.party_count(), 1);
        assert_eq!(reopened.all(&rom).unwrap().len(), 2);
        assert_eq!(reopened.trainer(&rom).unwrap().money, 1000);
        assert_eq!(
            reopened
                .bag()
                .unwrap()
                .iter()
                .find(|e| e.pocket == "balls" && e.slot == 0)
                .unwrap()
                .quantity,
            5
        );
        assert_eq!(reopened.boxes(&rom).unwrap()[0].name, "验证");
        assert_eq!(&reopened.data[0x20000..], &original[0x20000..]);
        if let Ok(path) = std::env::var(output_key) {
            std::fs::write(path, &reopened.data).unwrap();
        }
    }

    #[test]
    #[ignore = "requires native in-game resave of the Mercury 1.2 edit probe"]
    fn exact_native_resave_12_regression() {
        check_native_resave("GEN3_ROM_MERCURY12", "GEN3_SAVE_MERCURY12_NATIVE");
    }

    fn check_native_resave(rom_key: &str, save_key: &str) {
        let rom = Rom::open(std::fs::read(std::env::var(rom_key).unwrap()).unwrap()).unwrap();
        let bytes = std::fs::read(std::env::var(save_key).unwrap()).unwrap();
        let save = Save::open(bytes, rom.profile.save).unwrap();
        save.validate(&rom).unwrap();
        assert_eq!(save.party_count(), 1);
        assert_eq!(save.all(&rom).unwrap().len(), 2);
        let party = save
            .pokemon(Location::Party { slot: 0 }, &rom)
            .unwrap()
            .unwrap();
        assert_eq!(
            (party.species, party.pid, party.ball, party.origin_game),
            (1, 0x1234_5678, 4, 4)
        );
        let boxed = save
            .pokemon(
                Location::Box {
                    box_index: 0,
                    slot: 0,
                },
                &rom,
            )
            .unwrap()
            .unwrap();
        assert_eq!((boxed.species, boxed.pid, boxed.ball), (1, 0x1234_5678, 4));
        assert_eq!(save.trainer(&rom).unwrap().money, 1000);
        assert_eq!(
            save.bag()
                .unwrap()
                .iter()
                .find(|e| e.pocket == "balls" && e.slot == 0)
                .unwrap()
                .quantity,
            5
        );
        assert_eq!(save.boxes(&rom).unwrap()[0].name, "验证");
        assert!(save.backup_valid);
        let mut session = Session::new(rom);
        session.load(save.data.clone(), None).unwrap();
        assert_eq!(session.snapshot().unwrap().trainer.money, 1000);
    }

    #[test]
    #[ignore = "requires exact local ROM via GEN3_ROM_MERCURY12"]
    fn exact_rom_12_regression() {
        check_rom(PROFILE, "GEN3_ROM_MERCURY12", 739);
    }

    fn check_rom(profile: crate::profile::Profile, key: &str, trainer_locations: usize) {
        let path = std::env::var(key).unwrap();
        let bytes = std::fs::read(path).unwrap();
        let rom = Rom::open(bytes.clone()).unwrap();
        assert_eq!(rom.profile.md5, profile.md5);
        let catalog = rom.catalog().unwrap();
        assert_eq!(catalog.species.len(), 1553);
        assert_eq!(catalog.moves.len(), 1015);
        assert_eq!(catalog.items.len(), 750);
        assert_eq!(catalog.abilities.len(), 300);
        assert_eq!(catalog.battle_forms.len(), 233);
        assert!(!catalog.editor_rules.pokemon_checksum);
        assert!(catalog.editor_rules.ball_options.len() >= 12);
        for ball in &catalog.editor_rules.ball_options {
            let item = rom.item(ball.item).unwrap();
            assert_eq!(ball.value, u16::from(item.item_type));
            assert_eq!(item.pocket, 3);
        }
        assert_eq!(rom.species(1).unwrap().name, "妙蛙种子");
        assert_eq!(rom.species(1553).unwrap().name, "桃歹郎");
        assert_eq!(rom.item(1).unwrap().name, "大师球");
        assert_eq!(rom.move_info(1).unwrap().name, "拍击");
        assert!(!rom.move_info(1).unwrap().description.is_empty());
        assert!(!rom.move_info(1014).unwrap().description.is_empty());
        assert_eq!(rom.item(289).unwrap().tm_move, Some(264));
        assert_eq!(rom.item(376).unwrap().tm_move, Some(395));
        assert_eq!(rom.item(339).unwrap().tm_move, Some(15));
        assert!(rom
            .learnset(1)
            .unwrap()
            .iter()
            .any(|entry| entry.source == "egg"));
        assert!(rom
            .learnset(1)
            .unwrap()
            .iter()
            .any(|entry| entry.source == "tm"));
        assert!(rom
            .learnset(1)
            .unwrap()
            .iter()
            .any(|entry| entry.source == "tutor"));
        assert_eq!(
            crate::pokemon::rom_experience(&rom, 0, 100).unwrap(),
            1_000_000
        );
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
        let world = rom.world().unwrap();
        assert_eq!(world.maps.len(), 871);
        assert_eq!(world.trainers.len(), 742);
        assert_eq!(world.encounters.len(), 10080);
        assert_eq!(world.trainer_locations.locations.len(), trainer_locations);
        for period in ["morning", "day", "dusk", "night"] {
            assert!(world.encounters.iter().any(|e| e.periods.contains(&period)));
        }
        for map_id in ["3-0", "3-1", "43-0", "55-0"] {
            assert!(rom.map_image(map_id).unwrap().starts_with(b"\x89PNG"));
        }
        for id in [0, 1, 52, 118, 147] {
            assert!(rom.trainer_sprite(id).unwrap().starts_with(b"\x89PNG"));
        }
        let all_graphics: std::collections::BTreeSet<_> = world
            .maps
            .iter()
            .flat_map(|m| m.objects.iter())
            .map(|o| o.graphics_id)
            .filter(|id| *id < 240)
            .collect();
        for id in all_graphics {
            // Two map records use graphics IDs with no static sprite descriptor.
            if matches!(id, 196 | 197) {
                continue;
            }
            assert!(rom.object_sprite(id).unwrap().starts_with(b"\x89PNG"));
        }
        let portraits: std::collections::BTreeSet<_> =
            world.trainers.iter().map(|t| t.portrait).collect();
        for id in portraits {
            assert!(rom
                .trainer_sprite(id as u16)
                .unwrap()
                .starts_with(b"\x89PNG"));
        }
        for id in [0, 1, 5, 37, 100, 151, 200, 239] {
            assert!(rom.object_sprite(id).unwrap().starts_with(b"\x89PNG"));
        }
        for id in [1, 6, 25, 150, 1001, 1553] {
            let normal = rom.sprite(id, false).unwrap();
            let shiny = rom.sprite(id, true).unwrap();
            assert!(normal.starts_with(b"\x89PNG"));
            assert!(shiny.starts_with(b"\x89PNG"));
            assert_ne!(normal, shiny, "species {id} shiny palette");
        }
        assert_ne!(
            rom.pokemon_sprite(201, false, 0).unwrap(),
            rom.pokemon_sprite(201, false, 1).unwrap()
        );
        assert_ne!(
            rom.pokemon_sprite(308, false, 0).unwrap(),
            rom.pokemon_sprite(308, false, 0x1234_5678).unwrap()
        );
        for id in 1..profile.species.count as u16 {
            if rom.valid_species(id).is_err() {
                continue;
            }
            assert!(rom.sprite(id, false).unwrap().starts_with(b"\x89PNG"));
            assert!(rom.sprite(id, true).unwrap().starts_with(b"\x89PNG"));
        }
        assert_eq!(rom.sprite(1300, false).unwrap_err().code, "species_id");
        if let Ok(directory) = std::env::var("GEN3_MERCURY_PROBES") {
            let mut probes = Vec::new();
            for species in [1, 25, 130, 201, 308, 1553] {
                for pid in 0..24 {
                    let raw = pokemon::to_party(
                        &pokemon::create(&rom, species, 0x1234_5678, "TEST", 50, pid).unwrap(),
                        &rom,
                    )
                    .unwrap();
                    for patch in [
                        PokemonPatch::default(),
                        PokemonPatch {
                            ball: Some(12),
                            ..Default::default()
                        },
                        PokemonPatch {
                            friendship: Some(201),
                            ..Default::default()
                        },
                        PokemonPatch {
                            experience: Some(234567),
                            ..Default::default()
                        },
                        PokemonPatch {
                            pp_ups: Some([3, 2, 1, 0]),
                            ..Default::default()
                        },
                        PokemonPatch {
                            ability_slot: Some(2),
                            ..Default::default()
                        },
                    ] {
                        if patch.ability_slot == Some(2)
                            && rom.species(species).unwrap().abilities[2] == 0
                        {
                            continue;
                        }
                        let after = pokemon::edit(&raw, &patch, &rom, Policy::Free).unwrap().0;
                        probes.push(serde_json::json!({
                            "before": raw, "after": after,
                            "pokemon": pokemon::decode(&after, &rom).unwrap(), "patch": patch,
                        }));
                    }
                }
            }
            let directory = std::path::Path::new(&directory);
            std::fs::create_dir_all(directory).unwrap();
            std::fs::write(
                directory.join(format!("{}.json", profile.id)),
                serde_json::to_vec(&probes).unwrap(),
            )
            .unwrap();
        }
        assert_eq!(
            Session::new(rom)
                .load(vec![0; 0x20000], None)
                .unwrap_err()
                .code,
            "save_no_valid_slot"
        );
        assert!(CheatRom::open(&bytes).unwrap().catalog().entries.is_empty());
        let mut changed = bytes;
        changed[0x141b351] ^= 1;
        assert_eq!(Rom::open(changed).err().unwrap().code, "unsupported_rom");
    }
}
