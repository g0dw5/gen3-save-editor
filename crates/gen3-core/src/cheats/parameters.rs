//! Parameterized recipes. Names and landing records are read from the loaded ROM.
use super::*;
use crate::{binary::*, rom::Rom};
use std::collections::BTreeSet;

pub const ENCOUNTER: &str = "specified-wild-encounter";
pub const SHINY: &str = "shiny-wild-encounters";
pub const TELEPORT: &str = "teleport-to-map";

pub(super) fn supported(md5: &str) -> bool {
    [
        crate::profile::BW.md5,
        crate::profile::DP.md5,
        crate::profile::ROCKET.md5,
        ULTIMATE_MD5,
        crate::mercury::PROFILE.md5,
    ]
    .contains(&md5)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Parameters {
    Encounter { species: u16, level: u8 },
    Teleport { map_id: String, warp_id: u8 },
}
#[derive(Clone, Default, Serialize)]
pub struct Options {
    pub species: Vec<SpeciesChoice>,
    pub maps: Vec<MapChoice>,
}
#[derive(Clone, Serialize)]
pub struct SpeciesChoice {
    pub id: u16,
    pub name: String,
}
#[derive(Clone, Serialize)]
pub struct Landing {
    pub id: u8,
    pub x: u16,
    pub y: u16,
}
#[derive(Clone, Serialize)]
pub struct MapChoice {
    pub id: String,
    pub group: u8,
    pub number: u8,
    pub name: String,
    pub region: u8,
    pub code: String,
    pub landings: Vec<Landing>,
}
impl Options {
    pub fn read(rom: &Rom) -> Result<Self> {
        let mut species = Vec::new();
        for id in 1..rom.profile.species.count as u16 {
            if let Ok(s) = rom.valid_species(id) {
                if !rom.is_battle_species(id)? {
                    species.push(SpeciesChoice { id, name: s.name });
                }
            }
        }
        let maps = rom.maps()?;
        let mut incoming = BTreeSet::new();
        for m in &maps {
            if let Some(e) = m.events {
                let count = bytes(&rom.data, e + 1, 1)?[0] as usize;
                if count == 0 {
                    continue;
                }
                let table = pointer(&rom.data, e + 8)?;
                for i in 0..count {
                    let w = bytes(&rom.data, table + i * 8, 8)?;
                    incoming.insert((w[7], w[6], w[5]));
                }
            }
        }
        let mut choices = Vec::new();
        for m in maps {
            let mut landings = Vec::new();
            if m.group < 128 && m.number < 128 {
                if let Some(e) = m.events {
                    let count = bytes(&rom.data, e + 1, 1)?[0] as usize;
                    if count > 0 {
                        let table = pointer(&rom.data, e + 8)?;
                        for i in 0..count.min(128) {
                            let o = table + i * 8;
                            let x = u16(&rom.data, o)?;
                            let y = u16(&rom.data, o + 2)?;
                            if incoming.contains(&(m.group, m.number, i as u8))
                                && u32::from(x) < m.width
                                && u32::from(y) < m.height
                            {
                                landings.push(Landing { id: i as u8, x, y });
                            }
                        }
                    }
                }
            }
            choices.push(MapChoice {
                id: m.id,
                group: m.group,
                number: m.number,
                name: rom.region_name(usize::from(m.region)),
                region: m.region,
                code: format!("{:02X} {:02X}", m.group, m.number),
                landings,
            });
        }
        Ok(Self {
            species,
            maps: choices,
        })
    }
}

pub(super) fn encounter(rocket: bool, species: u16, level: u8) -> Vec<RomHalfword> {
    let start = if rocket { 0xec2d4 } else { 0xb4e6c };
    [0x0400, 0x0c06, 0x0609, 0x0e0f]
        .into_iter()
        .zip([
            0x2600 | species >> 8,
            0x0236,
            0x3600 | species & 255,
            0x2700 | u16::from(level),
        ])
        .enumerate()
        .map(|(i, (before, after))| RomHalfword {
            offset: start + i as u32 * 2,
            before,
            after,
        })
        .collect()
}
pub(super) fn teleport(rocket: bool, group: u8, number: u8, warp: u8) -> Vec<RomHalfword> {
    let start = if rocket { 0xba4be } else { 0x84c12 };
    [0x1c21, 0x1c2a, 0x1c33]
        .into_iter()
        .zip([
            0x2100 | u16::from(group),
            0x2200 | u16::from(number),
            0x2300 | u16::from(warp),
        ])
        .enumerate()
        .map(|(i, (before, after))| RomHalfword {
            offset: start + i as u32 * 2,
            before,
            after,
        })
        .collect()
}
pub(super) fn teleport_for(md5: &str, group: u8, number: u8, warp: u8) -> Vec<RomHalfword> {
    let mut patches = teleport(md5 == crate::profile::ROCKET.md5, group, number, warp);
    if md5 == crate::mercury::PROFILE.md5 {
        for (i, patch) in patches.iter_mut().enumerate() {
            patch.offset = 0x553b2 + i as u32 * 2;
        }
    }
    patches
}

// Source: docs/research/shiny-wild-hook.s. Preserve native nature/gender loops;
// substitute only PID's high half, and only when the caller is CreateWildMon.
const SHINY_STUB: &[u16] = &[
    0xb50e, 0x0004, 0x4b0c, 0xf000, 0xf816, 0x990e, 0x4a0b, 0x4291, 0xd108, 0x2107, 0x4008, 0x490a,
    0x6809, 0x894a, 0x4050, 0x898a, 0x4050, 0x4060, 0x0424, 0x0c24, 0x0400, 0x4304, 0xbc0e, 0xbc01,
    0x4686, 0x4804, 0x4700, 0x4718,
];
pub(super) fn shiny(rocket: bool) -> Vec<RomHalfword> {
    let base = if rocket { 0x1f00000 } else { 0x311200 };
    let sites = if rocket {
        [
            (0x95a90, 0xec3a9, 40, 0xfcbb),
            (0x95b7e, 0xec389, 52, 0xfc44),
        ]
    } else {
        [
            (0x67eb4, 0xb4f41, 40, 0xfb89),
            (0x67fa2, 0xb4f21, 52, 0xfb12),
        ]
    };
    let mut result = Vec::new();
    for (i, (site, caller, stack, random_bl)) in sites.into_iter().enumerate() {
        let start = base + i as u32 * 128;
        let mut stub = SHINY_STUB.to_vec();
        stub[5] = 0x9900 | ((stack + 16) / 4) as u16;
        for value in [
            if rocket { 0x809d40d } else { 0x806f5cd },
            0x8000000 + caller,
            if rocket { 0x3005250 } else { 0x3005d90 },
            0x8000000 + site + 15,
        ] {
            stub.extend([value as u16, (value >> 16) as u16]);
        }
        result.extend(stub.into_iter().enumerate().map(|(j, after)| RomHalfword {
            offset: start + j as u32 * 2,
            before: 0xffff,
            after,
        }));
        let mut jump = if site % 4 == 0 {
            vec![0x4b00, 0x4718]
        } else {
            vec![0x4b01, 0x4718, 0x46c0]
        };
        let address = 0x8000000 + start + 1;
        jump.extend([address as u16, (address >> 16) as u16]);
        jump.resize(7, 0x46c0);
        let original = [0x1c04, 0xf007, random_bl, 0x0424, 0x0c24, 0x0400, 0x4304];
        result.extend(
            original
                .into_iter()
                .zip(jump)
                .enumerate()
                .map(|(j, (before, after))| RomHalfword {
                    offset: site + j as u32 * 2,
                    before,
                    after,
                }),
        );
    }
    result
}
// Mercury's ordinary wild routine calls the native nature constructor through
// an extra wrapper frame. Match its saved caller, never a generic constructor.
fn mercury_shiny() -> Vec<RomHalfword> {
    let mut result = Vec::new();
    for (i, (site, caller, stack, random_bl)) in [
        (0x3ddbc, 0x1d68cdb, 96, 0xf883),
        (0x3de44, 0x1d68c19, 52, 0xf83f),
        (0x3deaa, 0x1d68c19, 52, 0xf80c),
    ]
    .into_iter()
    .enumerate()
    {
        let start = 0x13fd000 + i as u32 * 128;
        let mut stub = SHINY_STUB.to_vec();
        stub[5] = 0x9900 | ((stack + 16) / 4) as u16;
        for value in [
            0x08044ec9,
            0x08000000 + caller,
            0x0300500c,
            0x08000000 + site + 15,
        ] {
            stub.extend([value as u16, (value >> 16) as u16]);
        }
        result.extend(stub.into_iter().enumerate().map(|(j, after)| RomHalfword {
            offset: start + j as u32 * 2,
            before: 0xffff,
            after,
        }));
        let address = 0x08000000 + start + 1;
        let mut jump = if site % 4 == 0 {
            vec![0x4b00, 0x4718]
        } else {
            vec![0x4b01, 0x4718, 0x46c0]
        };
        jump.extend([address as u16, (address >> 16) as u16]);
        jump.resize(7, 0x46c0);
        for (j, (before, after)) in [0x1c04, 0xf007, random_bl, 0x0424, 0x0c24, 0x0400, 0x4304]
            .into_iter()
            .zip(jump)
            .enumerate()
        {
            result.push(RomHalfword {
                offset: site + j as u32 * 2,
                before,
                after,
            });
        }
    }
    result
}

pub(super) fn generate(rom: &CheatRom, request: &GenerateRequest) -> Result<Vec<RomHalfword>> {
    if !supported(&rom.md5) {
        return Err(err(
            "unsupported_feature",
            "no parameterized cheats for this ROM",
        ));
    }
    let options = &rom.options;
    match (request.cheat_id.as_str(), &request.parameters) {
        (ENCOUNTER, Some(Parameters::Encounter { species, level }))
            if (1..=100).contains(level) && options.species.iter().any(|s| s.id == *species) =>
        {
            Ok(encounter_for(&rom.md5, *species, *level))
        }
        (TELEPORT, Some(Parameters::Teleport { map_id, warp_id })) => {
            let m = options
                .maps
                .iter()
                .find(|m| m.id == *map_id && m.landings.iter().any(|w| w.id == *warp_id))
                .ok_or_else(|| err("cheat_parameters", "select a referenced ROM landing"))?;
            Ok(teleport_for(&rom.md5, m.group, m.number, *warp_id))
        }
        (SHINY, None) => Ok(shiny_for(&rom.md5)),
        _ => Err(err(
            "cheat_parameters",
            "missing, invalid or unexpected cheat parameters",
        )),
    }
}

/// Encode a native Thumb BL. The free-space routines stay inside its ±4 MiB range.
fn ultimate_hook(site: u32, before: [u16; 2], target: u32, code: &[u16]) -> Vec<RomHalfword> {
    let delta = target as i32 - site as i32 - 4;
    assert!(delta % 2 == 0 && (-0x400000..0x400000).contains(&delta));
    let branch = [
        0xf000 | ((delta >> 12) as u16 & 0x7ff),
        0xf800 | ((delta >> 1) as u16 & 0x7ff),
    ];
    let mut result: Vec<_> = code
        .iter()
        .enumerate()
        .map(|(i, after)| RomHalfword {
            offset: target + i as u32 * 2,
            before: 0xffff,
            after: *after,
        })
        .collect();
    result.extend((0..2).map(|i| RomHalfword {
        offset: site + i as u32 * 2,
        before: before[i],
        after: branch[i],
    }));
    result
}
pub(super) fn encounter_for(md5: &str, species: u16, level: u8) -> Vec<RomHalfword> {
    if md5 == crate::mercury::PROFILE.md5 {
        let target = 0x093fd281u32;
        let return_to = 0x09d68b7du32;
        let stub = [
            0x2500 | species >> 8,
            0x022d,
            0x3500 | species & 255,
            0x2100 | u16::from(level),
            0x9106,
            0x9205,
            0x2b00,
            0x4c01,
            0x4720,
            0x46c0,
            return_to as u16,
            (return_to >> 16) as u16,
        ];
        let mut result: Vec<_> = stub
            .into_iter()
            .enumerate()
            .map(|(i, after)| RomHalfword {
                offset: 0x13fd280 + i as u32 * 2,
                before: 0xffff,
                after,
            })
            .collect();
        result.extend(
            [0x0005, 0x9106, 0x9205, 0x2b00]
                .into_iter()
                .zip([0x4800, 0x4700, target as u16, (target >> 16) as u16])
                .enumerate()
                .map(|(i, (before, after))| RomHalfword {
                    offset: 0x1d68b74 + i as u32 * 2,
                    before,
                    after,
                }),
        );
        return result;
    }
    if md5 != ULTIMATE_MD5 {
        return encounter(md5 == crate::profile::ROCKET.md5, species, level);
    }
    ultimate_hook(
        0x1f06100,
        [0x2296, 0x0004],
        0x1fff080,
        &[
            0x2400 | species >> 8,
            0x0224,
            0x3400 | species & 255,
            0x2100 | level as u16,
            0x2296,
            0x4770,
        ],
    )
}
pub(super) fn shiny_for(md5: &str) -> Vec<RomHalfword> {
    if md5 == crate::mercury::PROFILE.md5 {
        return mercury_shiny();
    }
    if md5 != ULTIMATE_MD5 {
        return shiny(md5 == crate::profile::ROCKET.md5);
    }
    // Assembled from docs/research/ultimate-shiny-wild-hook.s. Only the ordinary
    // wild constructor's call frame matches; breeding and other callers pass through.
    ultimate_hook(
        0x1f00742,
        [0xf000, 0xfb20],
        0x1fff000,
        &[
            0xb50e, 0x4b0a, 0xf000, 0xf810, 0x990d, 0x4a09, 0x4291, 0xd108, 0x2107, 0x4008, 0x4659,
            0x0c0a, 0x4050, 0x0409, 0x0c09, 0x4048, 0x4060, 0xbc0e, 0xbc08, 0x4718, 0x4718, 0x46c0,
            0xf5cd, 0x0806, 0x6155, 0x09f0,
        ],
    )
}
