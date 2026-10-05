//! Ultimate Emerald 5.5 adapter. Addresses and engine rules only; game content
//! stays in the user-supplied, fingerprint-verified ROM.
use crate::{
    adapter::*,
    profile::*,
    save::{Pocket, PocketBlock},
};

pub const POCKETS: [Pocket; 6] = [
    Pocket {
        block: PocketBlock::Main,
        id: "pc",
        offset: 0x498,
        count: 50,
        encrypted: false,
        category: 0,
    },
    Pocket {
        block: PocketBlock::SectorExtensions,
        id: "items",
        offset: 0xcc,
        count: 100,
        encrypted: true,
        category: 1,
    },
    Pocket {
        block: PocketBlock::SectorExtensions,
        id: "key_items",
        offset: 0x25c,
        count: 50,
        encrypted: true,
        category: 5,
    },
    Pocket {
        block: PocketBlock::SectorExtensions,
        id: "balls",
        offset: 0x324,
        count: 32,
        encrypted: true,
        category: 2,
    },
    Pocket {
        block: PocketBlock::SectorExtensions,
        id: "tmhm",
        offset: 0x3a4,
        count: 130,
        encrypted: true,
        category: 3,
    },
    Pocket {
        block: PocketBlock::SectorExtensions,
        id: "berries",
        offset: 0x5ac,
        count: 50,
        encrypted: true,
        category: 4,
    },
];

pub const PROFILE: Profile = Profile {
    script_pokemon: crate::script_pokemon::PokemonScriptRules {
        wild: crate::script_pokemon::WildCommand::Literal,
        egg_level_instruction: 0x70978,
        native_battle: None,
    },
    native_trainers: None,
    clock: None,
    hidden_items: Some(crate::profile::HiddenItemRules {
        packed: false,
        flag_base: 0x1f4,
        region_override: None,
    }),
    event_state: Some(crate::profile::EventStateLayout {
        flags: &[
            crate::event_state::EventRange {
                first: 0,
                count: 0x4000,
                block: crate::event_state::EventBlock::Main,
                offset: 0x1270,
            },
            crate::event_state::EventRange {
                first: 0x4000,
                count: 0x1a0,
                block: crate::event_state::EventBlock::Main,
                offset: 0x988,
            },
            crate::event_state::EventRange {
                first: 0x41a0,
                count: 0x1a0,
                block: crate::event_state::EventBlock::Main,
                offset: 0x3b24,
            },
            crate::event_state::EventRange {
                first: 0x4340,
                count: 0x1a0,
                block: crate::event_state::EventBlock::Trainer,
                offset: 0x5c,
            },
            crate::event_state::EventRange {
                first: 0x44e0,
                count: 0x1a0,
                block: crate::event_state::EventBlock::Trainer,
                offset: 0x28,
            },
        ],
        variables: &[crate::event_state::EventRange {
            first: 0x4000,
            count: 0x100,
            block: crate::event_state::EventBlock::Main,
            offset: 0x139c,
        }],
        pickup_receipt: true,
        gift_result: true,
    }),
    id: "ultimate-emerald-55",
    label: "究极绿宝石 5.5 · 失落之古遗",
    md5: "17ce9785b33319b3dbda9a5d37c57ec1",
    size: 0x2000000,
    formats: RomFormats {
        learnsets: LearnsetFormat::Move16Level8,
        evolutions: EvolutionFormat::Ultimate55,
        trainers: TrainerFormat::Ultimate55,
        ..RomFormats::GEN3
    },
    capabilities: Capabilities {
        battle_forms: true,
        ..Capabilities::DARK_PHANTOM
    },
    ability_count: 342,
    ability_description_count: 342,
    species_abilities: Some(Table {
        offset: 0x17a0000,
        count: 1200,
        stride: 6,
    }),
    trainer_exclusions: &[(902, 918)],
    nature_product_u16: false,
    type_names: Table {
        offset: 0x1d382b8,
        count: 24,
        stride: 7,
    },
    species: Table {
        offset: 0xf2b790,
        count: 1200,
        stride: 11,
    },
    base_stats: Table {
        offset: 0xf186e0,
        count: 1200,
        stride: 28,
    },
    national_dex: 0xf50370,
    move_names: SplitText {
        first: 0x1d3022c,
        second: 0,
        split: 938,
        stride: 13,
    },
    moves: Table {
        offset: 0x1d863b9,
        count: 938,
        stride: 12,
    },
    move_descriptions: 0x1d2ad00,
    ability_names: SplitText {
        first: 0x1d03068,
        second: 0,
        split: 342,
        stride: 13,
    },
    ability_descriptions: SplitText {
        first: 0x1d00218,
        second: 0,
        split: 342,
        stride: 4,
    },
    items: Table {
        offset: 0xfc2c7c,
        count: 800,
        stride: 44,
    },
    evolution_overrides: &[(
        133,
        Table {
            offset: 0x1f0b760,
            count: 10,
            stride: 8,
        },
    )],
    evolutions: Table {
        offset: 0xf387c0,
        count: 1200,
        stride: 40,
    },
    learnsets: 0x1d89518,
    eggs: 0x1d78128,
    teaching: TeachingRules {
        machine_item_ranges: &[],
        tm_first_item: 378,
        tm_count: 128,
        tm_stride: 16,
        tutor_count: 127,
        tutor_stride: 16,
        shared_lists: None,
        egg_words: 2990,
    },
    tm_moves: 0x1e0fe80,
    tm_bits: 0xfcd8f4,
    tutor_moves: 0xfd80b8,
    tutor_bits: 0x1750000,
    sprites: 0xf20a20,
    palettes: 0xf25520,
    shiny_palettes: 0xf27aa0,
    sprite_rules: SpriteRules {
        unown_b_sprite: 413,
        second_frame_species: u16::MAX,
        ..BW.sprite_rules
    },
    maps: 0xa54698,
    map_counts: &[
        57, 5, 5, 6, 7, 8, 9, 7, 7, 14, 8, 17, 10, 23, 13, 15, 15, 2, 2, 2, 3, 1, 1, 1, 108, 61,
        89, 2, 1, 13, 1, 1, 3, 1, 96, 89, 97, 122,
    ],
    region_count: 213,
    wild: 0xe17d50,
    feebas: BW.feebas,
    trainers: Table {
        offset: 0x10019f8,
        count: 1355,
        stride: 40,
    },
    trainer_classes: Table {
        offset: 0x30fcd4,
        count: 66,
        stride: 13,
    },
    trainer_sprites: Table {
        offset: 0x101be90,
        count: 250,
        stride: 8,
    },
    trainer_palettes: 0x101c660,
    object_graphics: &[Table {
        offset: 0x505620,
        count: 240,
        stride: 4,
    }],
    script_actors: &[],
    map_groups: &[],
    contest: Some(crate::contest::ContestRules {
        flavor_preferences: 0x5b25a0,
        npc_blender: None,
    }),
    hidden_power: Some(HiddenPowerRules {
        move_id: 237,
        formula: HiddenPowerFormula::Gen6Fixed60,
    }),
    battle_forms: Some(crate::forms::BattleFormRules::UltimateEvolutionMethods),
    save: SaveLayout {
        extension_sectors: Some(&[]),
        sector_checksum: SectorChecksum::NativeConstantOne,
        pokemon_codec: PokemonCodec::Ultimate55,
        pockets: &POCKETS,
        dex: Some(DexLayout {
            bit_bias: 1,
            main_block: true,
            count: 905,
            owned: 0x5d8,
            seen: 0x560,
            seen_mirrors: &[],
        }),
        ..EMERALD
    },
    ..BW
};

pub(crate) fn evolution_condition(method: u16) -> &'static str {
    match method {
        16 => "move",
        17 => "region",
        18 => "day_level",
        19 => "night_level",
        20 => "held_day",
        21 => "held_night",
        22 => "male_level",
        23 => "female_level",
        24 => "rain_level",
        25 => "party_species",
        26 => "party_dark",
        27 => "male_item",
        28 => "female_item",
        29 => "move_type",
        30 => "dusk_level",
        31..=34 => "level",
        _ => crate::rom::evolution_condition(method, false),
    }
}

/// The native ball decoder (0x1F05BB0) dispatches the first five indices through
/// ROM instructions and maps the remaining indices arithmetically.
pub(crate) fn ball_item(rom: &crate::rom::Rom, index: u16) -> crate::Result<u16> {
    use crate::binary::{pointer, u16};
    Ok(match index {
        0..=4 => {
            let code = pointer(&rom.data, 0x1f08268 + index as usize * 4)? & !1;
            let instruction = u16(&rom.data, code)?;
            if instruction & 0xff00 != 0x2300 {
                return Err(crate::err("ball_decoder", index));
            }
            instruction & 255
        }
        5..=11 => index + 1,
        12..=31 => index + 277,
        _ => return Err(crate::err("ball_id", index)),
    })
}

pub(crate) fn evolution_requirements(
    data: &[u8],
    offset: usize,
) -> crate::Result<Vec<crate::rom::EvolutionRequirement>> {
    use crate::{
        binary::{bytes, u16},
        rom::EvolutionRequirement as R,
    };
    let row = bytes(data, offset, 8)?;
    Ok(match row[0] {
        31 => vec![R {
            kind: "region",
            value: row[1] as u16,
        }],
        32 => vec![R {
            kind: "weather",
            value: row[1] as u16,
        }],
        33 => vec![R {
            kind: "map",
            value: (row[1] as u16) << 8 | u16(row, 6)?,
        }],
        34 => vec![
            R {
                kind: "region",
                value: row[1] as u16,
            },
            R {
                kind: "hour_boundary",
                value: 0,
            },
        ],
        _ => match row[1] {
            255 => vec![R {
                kind: "outside_region",
                value: 6,
            }],
            254 => vec![R {
                kind: "outside_region",
                value: 104,
            }],
            _ => Vec::new(),
        },
    })
}

/// Read the native enhanced-team template. EVs are computed later from mode,
/// player party and battle context; the per-stat byte here is not six fixed EVs.
pub(crate) fn trainer_template(
    rom: &crate::rom::Rom,
    species: u16,
    index: u8,
) -> crate::Result<crate::world::TrainerMonGeneration> {
    let row = crate::binary::bytes(&rom.data, 0x1f0af60 + index as usize * 8, 8)?;
    let species = rom.species(species)?;
    let ability = *species
        .abilities
        .get(row[2] as usize)
        .ok_or_else(|| crate::err("trainer_ability", index))?;
    Ok(crate::world::TrainerMonGeneration {
        context: "ultimate_template",
        ev_increment: Some(row[0]),
        gender: match species.gender_ratio {
            0 => "male",
            254 => "female",
            255 => "genderless",
            _ => "random",
        },
        nature: row[3],
        ability_id: ability,
        ability_options: vec![ability],
        ivs: (row[1] <= 31).then_some([row[1]; 6]),
        evs: None,
        personality_parameter: index,
    })
}
