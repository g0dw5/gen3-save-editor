use crate::{binary::hash, err, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Table {
    pub offset: usize,
    pub count: usize,
    pub stride: usize,
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
    pub items: Table,
    pub evolutions: Table,
    pub learnsets: usize,
    pub eggs: usize,
    pub tm_moves: usize,
    pub tm_bits: usize,
    pub tutor_moves: usize,
    pub tutor_bits: usize,
    pub sprites: usize,
    pub palettes: usize,
    pub shiny_palettes: usize,
    pub maps: usize,
    pub regions: usize,
    pub region_count: usize,
    pub wild: usize,
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
    pub sizes: [usize; 14],
    pub party_count: usize,
    pub party: usize,
    pub boxes: usize,
    pub slots: usize,
    pub key: usize,
    pub money: usize,
    pub coins: usize,
}
pub const EMERALD: SaveLayout = SaveLayout {
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
    tm_moves: 0x1ca0000,
    tm_bits: 0x31e898,
    tutor_moves: 0x61500c,
    tutor_bits: 0x615048,
    sprites: 0x30a18c,
    palettes: 0x303678,
    shiny_palettes: 0x304438,
    maps: 0xe8c020,
    regions: 0x5a1480,
    region_count: 213,
    wild: 0xea2d34,
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
pub const PROFILES: [Profile; 2] = [BW, DP];
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
