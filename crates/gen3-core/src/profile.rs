use crate::{binary::hash, err, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Table {
    pub offset: usize,
    pub count: usize,
    pub stride: usize,
}
/// Hardware tile ownership and metatile layout are independent of save codecs.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct MapGraphics {
    pub primary_tiles: usize,
    pub primary_metatiles: usize,
    pub layers: usize,
    /// FireRed-style u32 metatile attributes at Tileset+20, bits 28..30.
    /// Extended type 3 reads a third layer without changing the 16-byte stride.
    pub extended_layer_types: bool,
    /// Static layouts independently reproduced as mismatched in the native engine.
    /// These are validation annotations, not replacement artwork or map content.
    pub native_mismatch_headers: &'static [usize],
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct SplitText {
    pub first: usize,
    pub second: usize,
    pub split: usize,
    pub stride: usize,
}
impl SplitText {
    pub fn at(&self, id: usize) -> usize {
        if id < self.split {
            self.first + id * self.stride
        } else {
            self.second + (id - self.split) * self.stride
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct EventStateLayout {
    pub effects: Option<crate::event_dependencies::Rules>,
    pub flags: &'static [crate::event_state::EventRange],
    pub variables: &'static [crate::event_state::EventRange],
    /// Native-verified ordinary item-ball protocol; individual scripts must also qualify.
    pub pickup_receipt: bool,
    /// Standard gift and additem return a native boolean in VAR_RESULT.
    pub gift_result: bool,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct HiddenItemRules {
    pub packed: bool,
    pub flag_base: u16,
    /// Runtime ROM list of map sections, terminated by 0xFF, with a separate flag base.
    pub region_override: Option<(usize, u16)>,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Profile {
    pub breeding: Option<crate::breeding::BreedingRules>,
    pub wild_items: Option<crate::wild_items::WildItemRules>,
    pub resource_checks: Option<crate::script_resources::ResourceCheckRules>,
    pub tutor_scripts: Option<crate::script_teaching::TutorScriptRules>,
    pub script_pokemon: crate::script_pokemon::PokemonScriptRules,
    pub native_trainers: Option<crate::native_trainer::NativeTrainerRules>,
    pub clock: Option<crate::clock::ClockRules>,
    /// Native-verified persistent flag/variable ranges; never inferred from item ownership.
    pub event_state: Option<EventStateLayout>,
    pub hidden_items: Option<HiddenItemRules>,
    pub formats: crate::adapter::RomFormats,
    pub capabilities: crate::adapter::Capabilities,
    pub ability_count: u16,
    /// A ROM may have more names than verified description pointers.
    pub ability_description_count: u16,
    pub species_abilities: Option<Table>,
    /// Excluded overwritten slots are not trainer records.
    pub trainer_exclusions: &'static [(u16, u16)],
    pub max_level: u8,
    pub experience_table: Option<Table>,
    pub battle_forms: Option<crate::forms::BattleFormRules>,
    pub storage_forms: Option<usize>,
    pub form_families: Option<usize>,
    pub id: &'static str,
    pub label: &'static str,
    pub md5: &'static str,
    pub size: usize,
    /// Native display strings and signed stat changes, indexed by nature ID.
    pub nature_names: usize,
    pub nature_effects: usize,
    /// The Emerald hook truncates the product before division; Rocket does not.
    pub nature_product_u16: bool,
    pub type_names: Table,
    pub move_names: SplitText,
    pub move_descriptions: usize,
    pub ability_names: SplitText,
    pub ability_descriptions: SplitText,
    pub mail_items: [u16; 2],
    pub national_dex: usize,
    pub species: Table,
    pub base_stats: Table,
    pub moves: Table,
    pub move_category_offset: usize,
    pub hidden_power: Option<HiddenPowerRules>,
    pub contest: Option<crate::contest::ContestRules>,
    pub items: Table,
    pub evolutions: Table,
    pub evolution_overrides: &'static [(u16, Table)],
    pub learnsets: usize,
    pub eggs: usize,
    pub teaching: TeachingRules,
    pub tm_moves: usize,
    pub tm_bits: usize,
    pub tutor_moves: usize,
    pub tutor_bits: usize,
    pub sprites: usize,
    pub palettes: usize,
    pub shiny_palettes: usize,
    pub sprite_rules: SpriteRules,
    pub maps: usize,
    /// Number of palette banks loaded from the primary and secondary tilesets.
    pub map_palette_banks: [usize; 2],
    pub map_graphics: MapGraphics,
    pub regions: usize,
    /// First map-section index represented by `regions` (FireRed may omit 0–87).
    pub region_first: usize,
    pub region_stride: usize,
    pub region_count: usize,
    pub wild: usize,
    pub wild_selection: Option<WildSelection>,
    /// Optional native time-of-day encounter tables, in morning/day/dusk/night order.
    pub wild_time_tables: Option<[usize; 4]>,
    pub feebas: Option<FeebasRules>,
    pub fishing_rods: [u16; 3],
    pub trainers: Table,
    pub trainer_classes: Table,
    pub trainer_sprites: Table,
    pub trainer_palettes: usize,
    pub object_graphics: &'static [Table],
    pub object_palettes: Table,
    pub object_palette_supplements: &'static [Table],
    pub script_actors: &'static [ScriptActor],
    pub map_groups: &'static [MapGroup],
    pub map_counts: &'static [usize],
    pub save: SaveLayout,
}
/// First matching map header, with an optional script-variable variant range.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct WildSelection {
    pub variable_map: &'static str,
    pub variable: u16,
    pub max_variant: u16,
}
/// Compatibility representation is separate from table addresses.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct TeachingRules {
    pub tm_first_item: u16,
    /// Item-ID ranges when machines are split across item tables.
    pub machine_item_ranges: &'static [MachineItemRange],
    pub tm_count: usize,
    pub tm_stride: usize,
    pub tutor_count: usize,
    pub tutor_stride: usize,
    /// A zero-terminated move list per species, shared by machines and tutors.
    pub shared_lists: Option<usize>,
    pub egg_words: usize,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct MachineItemRange {
    pub item_first: u16,
    pub item_count: u16,
    pub move_first: u16,
}
/// Verified battle-engine behavior, not the move table's placeholder type/power.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct HiddenPowerRules {
    pub move_id: u16,
    pub formula: HiddenPowerFormula,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HiddenPowerFormula {
    Gen3To5,
    Gen6Fixed60,
}
/// Emerald's save-seeded fishing rule, separate from ordinary encounter tables.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct FeebasRules {
    pub map_id: &'static str,
    pub seed_offset: usize,
    pub wild_record: usize,
    pub water_sections: usize,
    pub behavior_flags: usize,
    pub spot_count: u16,
}
/// Engine-specific appearance rules; graphics and spot masks remain in the ROM.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct SpriteRules {
    pub unown_species: u16,
    pub unown_b_sprite: u16,
    pub spinda_species: u16,
    pub spinda_spots: usize,
    pub second_frame_species: u16,
    pub female: Option<FemaleSprites>,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct FemaleSprites {
    pub flags: usize,
    pub sprites: usize,
    pub palettes: usize,
    pub shiny_palettes: usize,
}
/// Reviewed scene actors for battles initiated outside an object's own script.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ScriptActor {
    pub battle_offset: usize,
    pub local_ids: &'static [u8],
}
/// Verified map purposes supplement ROM region names; never imply story order.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct MapGroup {
    pub kind: &'static str,
    pub map_ids: &'static [&'static str],
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct SaveLayout {
    /// Native extension RAM is stored in logical sector tails, then these flash sectors.
    /// None disables extensions; Some(&[]) uses only the tails.
    pub extension_sectors: Option<&'static [usize]>,
    pub sector_checksum: SectorChecksum,
    pub pokemon_codec: crate::adapter::PokemonCodec,
    pub pockets: &'static [crate::save::Pocket],
    pub dex: Option<DexLayout>,
    pub sizes: [usize; 14],
    pub party_count: usize,
    pub party: usize,
    pub boxes: usize,
    pub slots: usize,
    pub key: usize,
    pub money: usize,
    pub coins: usize,
    pub registered_item: usize,
    /// Native game-version field for a newly created individual.
    pub created_origin_game: u8,
    /// Emulator RTC data may follow the 128 KiB flash image; preserve it verbatim.
    pub rtc_trailer_bytes: usize,
    /// Some ROMs preserve nonzero bytes in box slots whose native occupancy bit is clear.
    pub skip_unoccupied_box_records: bool,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum SectorChecksum {
    Sum,
    NativeConstantOne,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct DexLayout {
    pub bit_bias: u8,
    /// False: SaveBlock2; true: logical SaveBlock1 (sections 1–4).
    pub main_block: bool,
    pub count: u16,
    pub owned: usize,
    pub seen: usize,
    pub seen_mirrors: &'static [usize],
}
pub const EMERALD: SaveLayout = SaveLayout {
    extension_sectors: None,
    sector_checksum: SectorChecksum::Sum,
    pokemon_codec: crate::adapter::PokemonCodec::Gen3,
    pockets: &crate::save::POCKETS,
    dex: Some(DexLayout {
        bit_bias: 0,
        main_block: false,
        count: 416,
        owned: 0x28,
        seen: 0x5c,
        seen_mirrors: &[0x988, 0x3b24],
    }),
    sizes: [
        3884, 3968, 3968, 3968, 3848, 3968, 3968, 3968, 3968, 3968, 3968, 3968, 3968, 2000,
    ],
    party_count: 0x234,
    party: 0x238,
    boxes: 14,
    slots: 30,
    key: 0xac,
    money: 0x490,
    coins: 0x494,
    registered_item: 0x496,
    created_origin_game: 3,
    rtc_trailer_bytes: 0,
    skip_unoccupied_box_records: false,
};
pub const BW: Profile = Profile {
    breeding: Some(crate::breeding::EMERALD),
    wild_items: Some(crate::wild_items::EMERALD),
    resource_checks: Some(crate::script_resources::EMERALD),
    tutor_scripts: Some(crate::script_teaching::TutorScriptRules {
        specials: 0x1dba64,
        special: 0x1dd,
        code: 0x1b892c,
        variable: 0x8005,
        parameter: crate::script_teaching::TutorParameter::Index {
            count: 32,
            getter: 0x081b2360,
        },
    }),
    script_pokemon: crate::script_pokemon::PokemonScriptRules {
        wild: crate::script_pokemon::WildCommand::Literal,
        egg_level_instruction: 0x70978,
        native_battle: Some(crate::script_pokemon::NativeBattleCommand {
            special: 0x1e2,
            species_var: 0x8004,
            level_var: 0x8005,
        }),
        trade: Some(crate::script_pokemon::TradeRules {
            specials: 0x1dba64,
            information_special: 0xff,
            information_code: 0x7e73c,
            table_pointer: 0x7e774,
            count: 4,
            alternate_flag: None,
        }),
    },
    native_trainers: None,
    clock: None,
    hidden_items: Some(HiddenItemRules {
        packed: false,
        flag_base: 0x1f4,
        region_override: None,
    }),
    event_state: Some(EventStateLayout {
        effects: Some(crate::event_dependencies::EMERALD),
        flags: &[crate::event_state::EventRange {
            first: 0,
            count: 0x4000,
            block: crate::event_state::EventBlock::Main,
            offset: 0x1270,
        }],
        variables: &[crate::event_state::EventRange {
            first: 0x4000,
            count: 0x100,
            block: crate::event_state::EventBlock::Main,
            offset: 0x139c,
        }],
        pickup_receipt: true,
        gift_result: true,
    }),
    nature_names: 0x61cb50,
    nature_effects: 0x31e818,
    nature_product_u16: true,
    type_names: Table {
        offset: 0x31ae38,
        count: 18,
        stride: 7,
    },
    formats: crate::adapter::RomFormats::GEN3,
    capabilities: crate::adapter::Capabilities::DARK_PHANTOM,
    ability_count: 151,
    ability_description_count: 151,
    species_abilities: None,
    trainer_exclusions: &[],
    max_level: 100,
    experience_table: Some(Table {
        offset: 0x31f72c,
        count: 6,
        stride: 404,
    }),
    battle_forms: None,
    storage_forms: None,
    form_families: None,
    id: "dark-phantom-5ex-bw",
    contest: Some(crate::contest::ContestRules {
        flavor_preferences: 0x5b25a0,
        npc_blender: Some(crate::contest::NpcBlender {
            berries: 0x58a670,
            berry_count: 43,
            berry_stride: 28,
            flavors_offset: 21,
            opponents: 0x339ca0,
            master: 0x339cbe,
            rpm_upper_bound: 18000,
        }),
    }),
    label: "漆黑的魅影 5.0EX+BW",
    md5: "0d9b129f7dd76895f79bb47ad7dec2fe",
    size: 33_554_188,
    move_names: SplitText {
        first: 0x31977c,
        second: 0x1903207,
        split: 355,
        stride: 13,
    },
    move_descriptions: 0x1904a00,
    ability_names: SplitText {
        first: 0x31b6db,
        second: 0x1c00000,
        split: 78,
        stride: 13,
    },
    ability_descriptions: SplitText {
        first: 0x31bad4,
        second: 0x1bffe00,
        split: 78,
        stride: 4,
    },
    mail_items: [0x79, 0x84],
    national_dex: 0x31dc82,
    species: Table {
        offset: 0x3185c8,
        count: 412,
        stride: 11,
    },
    base_stats: Table {
        offset: 0x3203cc,
        count: 412,
        stride: 28,
    },
    moves: Table {
        offset: 0x1900000,
        count: 472,
        stride: 12,
    },
    move_category_offset: 10,
    hidden_power: Some(HiddenPowerRules {
        move_id: 237,
        formula: HiddenPowerFormula::Gen3To5,
    }),
    items: Table {
        offset: 0x5839a0,
        count: 377,
        stride: 44,
    },
    evolution_overrides: &[],
    evolutions: Table {
        offset: 0x32531c,
        count: 412,
        stride: 40,
    },
    learnsets: 0x329378,
    eggs: 0x32add8,
    teaching: TeachingRules {
        tm_first_item: 0x121,
        machine_item_ranges: &[],
        tm_count: 58,
        tm_stride: 8,
        tutor_count: 32,
        tutor_stride: 4,
        shared_lists: None,
        egg_words: 4096,
    },
    tm_moves: 0x1ca0000,
    tm_bits: 0x31e898,
    tutor_moves: 0x1ca00c0,
    tutor_bits: 0x615048,
    sprites: 0x30a18c,
    palettes: 0x303678,
    shiny_palettes: 0x304438,
    sprite_rules: SpriteRules {
        unown_species: 201,
        unown_b_sprite: 413,
        spinda_species: 308,
        spinda_spots: 0x31e2f0,
        second_frame_species: 410,
        female: None,
    },
    maps: 0xe8c020,
    map_palette_banks: [6, 7],
    map_graphics: MapGraphics {
        primary_tiles: 512,
        primary_metatiles: 512,
        layers: 2,
        extended_layer_types: false,
        native_mismatch_headers: &[],
    },
    fishing_rods: [262, 263, 264],
    regions: 0x5a1480,
    region_first: 0,
    region_stride: 8,
    region_count: 213,
    wild: 0xea2d34,
    wild_selection: None,
    wild_time_tables: None,
    feebas: Some(FeebasRules {
        map_id: "0-34",
        seed_offset: 0x2e6a,
        wild_record: 0x553a78,
        water_sections: 0x553a7c,
        behavior_flags: 0x486efc,
        spot_count: 447,
    }),
    trainers: Table {
        offset: 0x121d300,
        count: 1367,
        stride: 40,
    },
    // Relocated class-name table, referenced at ROM 0x183B4 and 0x6F0AC.
    trainer_classes: Table {
        offset: 0x119a000,
        count: 83,
        stride: 13,
    },
    trainer_sprites: Table {
        offset: 0x1198000,
        count: 203,
        stride: 8,
    },
    trainer_palettes: 0x1199000,
    // The graphics hook at 0x11960E4 selects the bank using the high byte.
    object_graphics: &[
        Table {
            offset: 0x505620,
            count: 240,
            stride: 4,
        },
        Table {
            offset: 0x1197000,
            count: 256,
            stride: 4,
        },
    ],
    object_palettes: Table {
        offset: 0x50bbc8,
        count: 35,
        stride: 8,
    },
    object_palette_supplements: &[],
    script_actors: &[ScriptActor {
        battle_offset: 0x228a51,
        local_ids: &[1],
    }],
    map_groups: &[
        MapGroup {
            kind: "gym",
            map_ids: &["11-3", "3-3", "10-0", "4-1", "8-1", "12-1", "14-0", "15-0"],
        },
        MapGroup {
            kind: "league",
            map_ids: &["16-0", "16-1", "16-2", "16-3", "16-4"],
        },
    ],
    map_counts: &[
        57, 5, 5, 6, 7, 8, 9, 7, 7, 14, 8, 17, 10, 23, 13, 15, 15, 2, 2, 2, 3, 1, 1, 1, 108, 61,
        89, 2, 1, 13, 1, 1, 3, 1, 40, 50, 60, 39,
    ],
    save: EMERALD,
};
pub const DP: Profile = Profile {
    id: "dark-phantom-5ex-dp",
    label: "漆黑的魅影 5.0EX+DP",
    md5: "cb2940215f4dafb1bef133c3af379f44",
    ..BW
};
pub const ROCKET: Profile = Profile {
    breeding: Some(crate::breeding::ROCKET),
    wild_items: Some(crate::wild_items::ROCKET),
    resource_checks: Some(crate::script_resources::ROCKET),
    tutor_scripts: Some(crate::script_teaching::TutorScriptRules {
        specials: 0x22b620,
        special: 0x1dd,
        code: 0x2070d8,
        variable: 0x8005,
        parameter: crate::script_teaching::TutorParameter::MoveId,
    }),
    script_pokemon: crate::script_pokemon::PokemonScriptRules {
        wild: crate::script_pokemon::WildCommand::RocketExtended,
        egg_level_instruction: 0x9e960,
        native_battle: None,
        trade: Some(crate::script_pokemon::TradeRules {
            specials: 0x22b620,
            information_special: 0xff,
            information_code: 0xb3d1c,
            table_pointer: 0xb3d54,
            count: 4,
            alternate_flag: None,
        }),
    },
    native_trainers: None,
    clock: None,
    hidden_items: Some(HiddenItemRules {
        packed: false,
        flag_base: 0x1f4,
        region_override: None,
    }),
    event_state: Some(EventStateLayout {
        effects: Some(crate::event_dependencies::ROCKET),
        flags: &[crate::event_state::EventRange {
            first: 0,
            count: 0x4000,
            block: crate::event_state::EventBlock::Main,
            offset: 0x1ca8,
        }],
        variables: &[crate::event_state::EventRange {
            first: 0x4000,
            count: 0x100,
            block: crate::event_state::EventBlock::Main,
            offset: 0x1f6c,
        }],
        pickup_receipt: true,
        gift_result: true,
    }),
    nature_names: 0xd052ec,
    nature_effects: 0x5b335c,
    nature_product_u16: false,
    type_names: Table {
        offset: 0x5a6480,
        count: 19,
        stride: 10,
    },
    id: "rocket-21-zh",
    contest: Some(crate::contest::ContestRules {
        flavor_preferences: 0xc7c598,
        npc_blender: None,
    }),
    label: "西班牙火箭队 2.1 汉化版",
    md5: "59c658a1081f542086de1060bb65f0b3",
    size: 0x2000000,
    formats: crate::adapter::RomFormats::ROCKET21,
    capabilities: crate::adapter::Capabilities::ROCKET21,
    ability_count: 269,
    ability_description_count: 269,
    species_abilities: None,
    trainer_exclusions: &[],
    max_level: 150,
    experience_table: Some(Table {
        offset: 0x5b3484,
        count: 6,
        stride: 604,
    }),
    battle_forms: Some(crate::forms::BattleFormRules::ExpansionEvolutionMethods),
    storage_forms: Some(0x6193ac),
    form_families: Some(0x617b00),
    move_names: SplitText {
        first: 0x5a35e1,
        second: 0,
        split: 755,
        stride: 13,
    },
    move_descriptions: 0xd045ec,
    ability_names: SplitText {
        first: 0x5a815c,
        second: 0,
        split: 269,
        stride: 17,
    },
    ability_descriptions: SplitText {
        first: 0x5a933c,
        second: 0,
        split: 269,
        stride: 4,
    },
    mail_items: [200, 211],
    national_dex: 0x5b1608,
    species: Table {
        offset: 0x59f9f0,
        count: 1395,
        stride: 11,
    },
    base_stats: Table {
        offset: 0x5b4764,
        count: 1395,
        stride: 36,
    },
    // Internal Z attacks have a separate namespace and must not be learned moves.
    moves: Table {
        offset: 0x5acd5c,
        count: 755,
        stride: 20,
    },
    move_category_offset: 16,
    hidden_power: Some(HiddenPowerRules {
        move_id: 237,
        formula: HiddenPowerFormula::Gen6Fixed60,
    }),
    items: Table {
        offset: 0xc3d558,
        count: 923,
        stride: 44,
    },
    evolution_overrides: &[],
    evolutions: Table {
        offset: 0x5f96d4,
        count: 1395,
        stride: 80,
    },
    learnsets: 0x614ac4,
    eggs: 0x61dac4,
    teaching: TeachingRules {
        tm_first_item: 592,
        machine_item_ranges: &[],
        tm_count: 254,
        tm_stride: 0,
        tutor_count: 0,
        tutor_stride: 0,
        shared_lists: Some(0x616090),
        egg_words: 4060,
    },
    tm_moves: 0xcf8c54,
    tm_bits: 0,
    tutor_moves: 0,
    tutor_bits: 0,
    sprites: 0x568880,
    palettes: 0x5545cc,
    shiny_palettes: 0x558ea4,
    sprite_rules: SpriteRules {
        unown_species: 201,
        unown_b_sprite: 1098,
        spinda_species: 327,
        spinda_spots: 0x5b2294,
        second_frame_species: u16::MAX,
        female: Some(FemaleSprites {
            flags: 0x54cbe0,
            sprites: 0x56b420,
            palettes: 0x55716c,
            shiny_palettes: 0x55ba44,
        }),
    },
    maps: 0x9f4f40,
    map_palette_banks: [7, 6],
    map_graphics: MapGraphics {
        primary_tiles: 640,
        primary_metatiles: 640,
        layers: 3,
        extended_layer_types: false,
        native_mismatch_headers: &[],
    },
    fishing_rods: [866, 867, 868],
    regions: 0xc6ad68,
    region_first: 0,
    region_stride: 8,
    region_count: 252,
    wild: 0xbe8a70,
    wild_selection: Some(WildSelection {
        variable_map: "51-106",
        variable: 0x403e,
        max_variant: 8,
    }),
    wild_time_tables: None,
    feebas: None,
    trainers: Table {
        offset: 0x586a18,
        count: 2559,
        stride: 40,
    },
    trainer_classes: Table {
        offset: 0x585f20,
        count: 216,
        stride: 13,
    },
    trainer_sprites: Table {
        offset: 0x55e2f4,
        count: 245,
        stride: 8,
    },
    trainer_palettes: 0x55ea9c,
    object_graphics: &[
        Table {
            offset: 0xb64c38,
            count: 256,
            stride: 4,
        },
        Table {
            offset: 0xb65038,
            count: 256,
            stride: 4,
        },
        Table {
            offset: 0xb65438,
            count: 30,
            stride: 4,
        },
    ],
    object_palettes: Table {
        offset: 0xb654cc,
        count: 190,
        stride: 8,
    },
    object_palette_supplements: &[],
    script_actors: &[],
    map_groups: &[],
    map_counts: &[
        61, 47, 36, 43, 11, 9, 9, 27, 23, 14, 7, 7, 12, 99, 11, 10, 8, 10, 10, 42, 20, 14, 18, 21,
        2, 74, 88, 4, 4, 7, 7, 4, 5, 6, 80, 5, 5, 6, 7, 8, 10, 7, 7, 14, 10, 17, 10, 24, 14, 17,
        15, 108, 61, 89, 68, 11,
    ],
    save: SaveLayout {
        pokemon_codec: crate::adapter::PokemonCodec::Rocket21,
        sizes: [
            0xe1c, 0xff4, 0xff4, 0xff4, 0x62c, 0xff4, 0xff4, 0xff4, 0xff4, 0xff4, 0xff4, 0xff4,
            0xff4, 0x5c0,
        ],
        pockets: &crate::save::ROCKET_POCKETS,
        dex: Some(DexLayout {
            bit_bias: 0,
            main_block: true,
            count: 955,
            owned: 0x2f5c,
            seen: 0x2ee4,
            seen_mirrors: &[],
        }),
        ..EMERALD
    },
};
pub const PROFILES: [Profile; 5] = [
    BW,
    DP,
    ROCKET,
    crate::ultimate::PROFILE,
    crate::mercury::PROFILE,
];
pub fn identify(data: &[u8]) -> Result<Profile> {
    let md5 = hash(data);
    PROFILES
        .into_iter()
        .find(|p| p.size == data.len() && p.md5 == md5)
        .ok_or_else(|| {
            err(
                "unsupported_rom",
                format!("MD5={md5}, bytes={}", data.len()),
            )
        })
}
