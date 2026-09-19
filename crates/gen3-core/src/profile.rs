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
pub struct Profile {
    pub formats: crate::adapter::RomFormats,
    pub capabilities: crate::adapter::Capabilities,
    pub ability_count: u16,
    pub max_level: u8,
    pub experience_table: Option<Table>,
    pub battle_forms: Option<crate::forms::BattleFormRules>,
    pub storage_forms: Option<usize>,
    pub form_families: Option<usize>,
    pub id: &'static str,
    pub label: &'static str,
    pub md5: &'static str,
    pub size: usize,
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
    pub items: Table,
    pub evolutions: Table,
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
    pub region_count: usize,
    pub wild: usize,
    pub wild_selection: Option<WildSelection>,
    pub feebas: Option<FeebasRules>,
    pub fishing_rods: [u16; 3],
    pub trainers: Table,
    pub trainer_classes: Table,
    pub trainer_sprites: Table,
    pub trainer_palettes: usize,
    pub object_graphics: &'static [Table],
    pub object_palettes: Table,
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
    pub tm_count: usize,
    pub tm_stride: usize,
    pub tutor_count: usize,
    pub tutor_stride: usize,
    /// A zero-terminated move list per species, shared by machines and tutors.
    pub shared_lists: Option<usize>,
    pub egg_words: usize,
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
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct DexLayout {
    /// False: SaveBlock2; true: logical SaveBlock1 (sections 1–4).
    pub main_block: bool,
    pub count: u16,
    pub owned: usize,
    pub seen: usize,
    pub seen_mirrors: &'static [usize],
}
pub const EMERALD: SaveLayout = SaveLayout {
    pokemon_codec: crate::adapter::PokemonCodec::Gen3,
    pockets: &crate::save::POCKETS,
    dex: Some(DexLayout {
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
};
pub const BW: Profile = Profile {
    formats: crate::adapter::RomFormats::GEN3,
    capabilities: crate::adapter::Capabilities::DARK_PHANTOM,
    ability_count: 151,
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
    evolutions: Table {
        offset: 0x32531c,
        count: 412,
        stride: 40,
    },
    learnsets: 0x329378,
    eggs: 0x32add8,
    teaching: TeachingRules {
        tm_first_item: 0x121,
        tm_count: 58,
        tm_stride: 8,
        tutor_count: 32,
        tutor_stride: 4,
        shared_lists: None,
        egg_words: 4096,
    },
    tm_moves: 0x1ca0000,
    tm_bits: 0x31e898,
    tutor_moves: 0x61500c,
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
    },
    fishing_rods: [262, 263, 264],
    regions: 0x5a1480,
    region_count: 213,
    wild: 0xea2d34,
    wild_selection: None,
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
    id: "rocket-21-zh",
    label: "西班牙火箭队 2.1 汉化版",
    md5: "59c658a1081f542086de1060bb65f0b3",
    size: 0x2000000,
    formats: crate::adapter::RomFormats::ROCKET21,
    capabilities: crate::adapter::Capabilities::ROCKET21,
    ability_count: 269,
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
    evolutions: Table {
        offset: 0x5f96d4,
        count: 1395,
        stride: 80,
    },
    learnsets: 0x614ac4,
    eggs: 0x61dac4,
    teaching: TeachingRules {
        tm_first_item: 592,
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
    },
    fishing_rods: [866, 867, 868],
    regions: 0xc6ad68,
    region_count: 252,
    wild: 0xbe8a70,
    wild_selection: Some(WildSelection {
        variable_map: "51-106",
        variable: 0x403e,
        max_variant: 8,
    }),
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
            main_block: true,
            count: 955,
            owned: 0x2f5c,
            seen: 0x2ee4,
            seen_mirrors: &[],
        }),
        ..EMERALD
    },
};
pub const PROFILES: [Profile; 3] = [BW, DP, ROCKET];
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
