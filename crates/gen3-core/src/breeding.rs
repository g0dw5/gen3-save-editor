//! Read-only ordinary daycare scenarios executed in the current ROM's native code.
use crate::{
    binary::*,
    err,
    native_trainer::Sandbox,
    pokemon,
    rom::Rom,
    save::{Location, Save},
    Result,
};
use armv4t_emu::Memory;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct BreedingRules {
    pub compatibility: u32,
    pub constructor: u32,
    pub daycare: usize,
    pub parent_stride: usize,
    pub pending_pid: usize,
    pub pending_width: usize,
    pub party: u32,
    pub party_count: u32,
    pub rng: u32,
    pub save_pointers: [u32; 2],
    pub specials: usize,
    pub receive_special: u16,
    pub receive_code: usize,
    pub production: Option<crate::breeding_production::Rules>,
}
pub const EMERALD: BreedingRules = BreedingRules {
    compatibility: 0x08070d4c,
    constructor: 0x080708c8,
    daycare: 0x3030,
    parent_stride: 140,
    pending_pid: 280,
    pending_width: 4,
    party: 0x020244ec,
    party_count: 0x020244e9,
    rng: 0x03005d80,
    save_pointers: [0x03005d8c, 0x03005d90],
    specials: 0x1dba64,
    receive_special: 0xbb,
    receive_code: 0x70aa8,
    production: Some(crate::breeding_production::EMERALD),
};
pub const ROCKET: BreedingRules = BreedingRules {
    compatibility: 0x0809ed3c,
    constructor: 0x0809e8b0,
    daycare: 0x297c,
    parent_stride: 140,
    pending_pid: 280,
    pending_width: 4,
    party: 0x02025170,
    party_count: 0x0202516d,
    rng: 0x03005240,
    save_pointers: [0x0300524c, 0x03005250],
    specials: 0x22b620,
    receive_special: 0xbb,
    receive_code: 0x9ea90,
    production: Some(crate::breeding_production::ROCKET),
};
pub const MERCURY: BreedingRules = BreedingRules {
    compatibility: 0x0804654c,
    constructor: 0x080460d4,
    daycare: 0x2f80,
    parent_stride: 140,
    pending_pid: 280,
    pending_width: 2,
    party: 0x02024284,
    party_count: 0x02024029,
    rng: 0x03005000,
    save_pointers: [0x03005008, 0x0300500c],
    specials: 0x15fd60,
    receive_special: 0xb8,
    receive_code: 0x462ac,
    production: Some(crate::breeding_production::MERCURY),
};
pub const ULTIMATE: BreedingRules = BreedingRules {
    production: Some(crate::breeding_production::ULTIMATE),
    ..EMERALD
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct DaycareSource {
    pub offset: usize,
    pub conditions: Vec<crate::map_events::EventCondition>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Parent {
    Stored {
        location: Location,
    },
    Simulated {
        species: u16,
        gender: Gender,
        #[serde(default)]
        held_item: u16,
        #[serde(default)]
        trainer_id: u32,
    },
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Genderless,
}
impl Gender {
    fn label(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
            Self::Genderless => "genderless",
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub parents: [Parent; 2],
    #[serde(default)]
    pub seed: u32,
    /// A query scenario, not a prediction of the game's next pending personality.
    pub offspring_pid: u32,
    /// None projects the ordinary SAV bag, or an empty bag without a SAV.
    /// An override is only a RAM query scenario, never a save edit.
    #[serde(default)]
    pub production_item: Option<bool>,
}
#[derive(Debug, Serialize)]
pub struct Preview {
    pub rom_md5: &'static str,
    pub parents: [pokemon::Pokemon; 2],
    /// Native base compatibility, not charm-adjusted egg production probability.
    pub compatibility: u8,
    pub child: Option<pokemon::Pokemon>,
    pub seed: u32,
    pub offspring_pid: u32,
    pub partial: bool,
    pub scenario: &'static str,
    pub rng_after: u32,
    pub production: Option<crate::breeding_production::Preview>,
    #[cfg(test)]
    #[serde(skip)]
    pub(crate) child_raw: Option<Vec<u8>>,
}

fn parent_raw(rom: &Rom, save: Option<&Save>, parent: &Parent) -> Result<Vec<u8>> {
    let raw = match parent {
        Parent::Stored { location } => save
            .ok_or_else(|| err("save_required", "breeding parent"))?
            .raw(*location)?,
        Parent::Simulated {
            species,
            gender,
            held_item,
            trainer_id,
        } => {
            let mon = rom.valid_species(*species)?;
            rom.item(*held_item)?;
            let pid = match gender {
                Gender::Female => 0,
                Gender::Male => 255,
                Gender::Genderless => 24,
            };
            if pokemon::gender(mon.gender_ratio, pid) != gender.label() {
                return Err(err("breeding_gender", species));
            }
            let fields = rom.profile.save.pokemon_codec.fields();
            let mut raw = vec![0; 80];
            put32(&mut raw, 0, pid);
            put32(&mut raw, 4, *trainer_id);
            raw[8..18].fill(0xff);
            raw[20..27].fill(0xff);
            fields.language.write(&mut raw, 2)?;
            fields.has_species.write(&mut raw, 1)?;
            let mut canonical = [0; 48];
            put16(&mut canonical, 0, *species);
            put16(&mut canonical, 2, *held_item);
            put32(
                &mut canonical,
                4,
                pokemon::rom_experience(rom, mon.growth, 5)?,
            );
            if let Some(f) = fields.nature_override {
                f.write(&mut canonical, 26)?;
            }
            pokemon::pack_with(&mut raw, &canonical, rom.profile.save.pokemon_codec);
            raw
        }
    };
    pokemon::checked_unpack_with(&raw, rom.profile.save.pokemon_codec)?;
    let mon = pokemon::decode(&raw, rom)?;
    if mon.species == 0 || mon.egg {
        return Err(err("breeding_parent", "a non-egg individual is required"));
    }
    Ok(raw[..80].to_vec())
}

pub fn preview(rom: &Rom, save: Option<&Save>, request: &Request) -> Result<Preview> {
    let rules = rom
        .profile
        .breeding
        .ok_or_else(|| err("breeding_unverified", rom.profile.id))?;
    if request.offspring_pid == 0
        || rules.pending_width == 2 && request.offspring_pid > u16::MAX as u32
    {
        return Err(err("breeding_pending_pid", request.offspring_pid));
    }
    if let [Parent::Stored { location: a }, Parent::Stored { location: b }] = &request.parents {
        if a == b {
            return Err(err("breeding_parent", "select two distinct individuals"));
        }
    }
    let raws = [
        parent_raw(rom, save, &request.parents[0])?,
        parent_raw(rom, save, &request.parents[1])?,
    ];
    preview_parents(rom, save, request, raws)
}

fn preview_parents(
    rom: &Rom,
    save: Option<&Save>,
    request: &Request,
    raws: [Vec<u8>; 2],
) -> Result<Preview> {
    let rules = rom
        .profile
        .breeding
        .ok_or_else(|| err("breeding_unverified", rom.profile.id))?;
    let parents = [
        pokemon::decode(&raws[0], rom)?,
        pokemon::decode(&raws[1], rom)?,
    ];
    let mut ram = Sandbox::new(&rom.data);
    const SB1: u32 = 0x02030000;
    const SB2: u32 = 0x02034000;
    for (pointer, address) in rules.save_pointers.into_iter().zip([SB1, SB2]) {
        ram.w32(pointer, address);
    }
    // Persistent blocks are read-only input. All constructor writes go to this disposable RAM.
    if let Some(save) = save {
        for (address, data) in [(SB1, save.logical(1..=4)), (SB2, save.logical(0..=0))] {
            if data.len() > 0x4000 {
                return Err(err("breeding_save_block", data.len()));
            }
            for (i, byte) in data.into_iter().enumerate() {
                ram.w8(address + i as u32, byte);
            }
        }
    }
    let daycare = SB1 + rules.daycare as u32;
    for i in 0..rules.parent_stride * 2 + 8 {
        ram.w8(daycare + i as u32, 0);
    }
    for (slot, raw) in raws.iter().enumerate() {
        for (i, byte) in raw.iter().enumerate() {
            ram.w8(daycare + (slot * rules.parent_stride + i) as u32, *byte);
        }
    }
    for i in 0..600 {
        ram.w8(rules.party + i, 0);
    }
    ram.w8(rules.party_count, 0);
    ram.w32(rules.rng, request.seed);
    let compatibility =
        u8::try_from(ram.call(rules.compatibility, [daycare, 0, 0, 0], [0; 2], 1_000_000)?)
            .map_err(|_| err("breeding_compatibility", "unexpected value"))?;
    #[cfg(test)]
    let mut child_raw = None;
    let child = if compatibility == 0 {
        None
    } else {
        if rules.pending_width == 2 {
            ram.w16(
                daycare + rules.pending_pid as u32,
                request.offspring_pid as u16,
            );
        } else {
            ram.w32(daycare + rules.pending_pid as u32, request.offspring_pid);
        }
        ram.call(rules.constructor, [daycare, 0, 0, 0], [0; 2], 1_000_000)?;
        if ram.r8(rules.party_count) != 1 {
            return Err(err("breeding_child_count", ram.r8(rules.party_count)));
        }
        let raw: Vec<_> = (0..100).map(|i| ram.r8(rules.party + i)).collect();
        pokemon::checked_unpack_with(&raw, rom.profile.save.pokemon_codec)?;
        let child = pokemon::decode(&raw, rom)?;
        rom.valid_species(child.species)?;
        if !child.egg {
            return Err(err("breeding_child", "native output is not an egg"));
        }
        #[cfg(test)]
        {
            child_raw = Some(raw);
        }
        Some(child)
    };
    for (slot, raw) in raws.iter().enumerate() {
        let after: Vec<_> = (0..80)
            .map(|i| ram.r8(daycare + (slot * rules.parent_stride + i) as u32))
            .collect();
        if after != *raw {
            return Err(err("breeding_parent_changed", slot));
        }
    }
    let production = crate::breeding_production::preview(rom, save, request, &raws)?;
    Ok(Preview {
        rom_md5: rom.profile.md5,
        parents,
        compatibility,
        child,
        seed: request.seed,
        offspring_pid: request.offspring_pid,
        partial: true,
        scenario: "ordinary_daycare_simulation",
        rng_after: ram.r32(rules.rng),
        production,
        #[cfg(test)]
        child_raw,
    })
}
impl Rom {
    pub(crate) fn script_daycare_instruction(&self, pc: usize) -> Result<Option<DaycareSource>> {
        let Some(rules) = self.profile.breeding else {
            return Ok(None);
        };
        let op = bytes(&self.data, pc, 1)?[0];
        if !matches!(op, 0x25 | 0x26) {
            return Ok(None);
        }
        let special = u16(&self.data, pc + if op == 0x26 { 3 } else { 1 })?;
        if special != rules.receive_special {
            return Ok(None);
        }
        if pointer(&self.data, rules.specials + special as usize * 4)? & !1 != rules.receive_code {
            return Err(err("breeding_dispatch", pc));
        }
        Ok(Some(DaycareSource {
            offset: pc,
            conditions: vec![],
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn requests_reject_unsafe_or_ambiguous_parent_inputs() {
        let mut rom = crate::tests::query_fixture_rom();
        let request: Request = serde_json::from_value(json!({"parents":[
            {"kind":"simulated","species":1,"gender":"female"},
            {"kind":"simulated","species":1,"gender":"male"}],"offspring_pid":24}))
        .unwrap();
        assert_eq!(
            preview(&rom, None, &request).unwrap_err().code,
            "breeding_unverified"
        );
        rom.profile.breeding = Some(MERCURY);
        let too_large = Request {
            offspring_pid: 65536,
            ..request
        };
        assert_eq!(
            preview(&rom, None, &too_large).unwrap_err().code,
            "breeding_pending_pid"
        );
        let location = Location::Party { slot: 0 };
        let duplicate = Request {
            parents: [Parent::Stored { location }, Parent::Stored { location }],
            seed: 0,
            offspring_pid: 24,
            production_item: None,
        };
        assert_eq!(
            preview(&rom, None, &duplicate).unwrap_err().code,
            "breeding_parent"
        );
        assert!(
            serde_json::from_value::<Request>(json!({"parents":[],"offspring_pid":null})).is_err()
        );
        assert!(serde_json::from_value::<Request>(json!({"parents":[{"kind":"stored","location":{"kind":"party","slot":0},"species":1},{"kind":"simulated","species":1,"gender":"male"}],"offspring_pid":24})).is_err());
    }

    #[test]
    #[ignore = "requires five exact private ROMs and GEN3_BREEDING_PROBES full native vectors"]
    fn local_breeding_matches_complete_native_routines() {
        let probes: serde_json::Value = serde_json::from_slice(
            &std::fs::read(std::env::var("GEN3_BREEDING_PROBES").unwrap()).unwrap(),
        )
        .unwrap();
        for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
            let rom = Rom::open(
                std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap(),
            )
            .unwrap();
            let original = rom.data.clone();
            assert_eq!(
                probes[key]["engine"], "mGBA ARM7",
                "exact GBA evidence is required"
            );
            for vector in probes[key]["alignment"].as_array().unwrap() {
                let mut code = (vector["instruction"].as_u64().unwrap() as u16)
                    .to_le_bytes()
                    .to_vec();
                code.extend(0x4770u16.to_le_bytes());
                let mut cpu = Sandbox::new(&code);
                for (i, v) in vector["bytes"].as_array().unwrap().iter().enumerate() {
                    cpu.w8(0x02021000 + i as u32, v.as_u64().unwrap() as u8);
                }
                assert_eq!(
                    cpu.call(
                        0x08000000,
                        [0, 0x02021000 + vector["odd"].as_u64().unwrap() as u32, 0, 0],
                        [0; 2],
                        10
                    )
                    .unwrap() as u64,
                    vector["output"].as_u64().unwrap()
                );
            }

            assert_eq!(probes[key]["md5"], rom.profile.md5);
            for row in probes[key]["rows"].as_array().unwrap() {
                let raws: [Vec<u8>; 2] = serde_json::from_value(row["parents"].clone()).unwrap();
                for raw in &raws {
                    pokemon::checked_unpack_with(raw, rom.profile.save.pokemon_codec).unwrap();
                }
                let request = Request {
                    parents: [
                        Parent::Simulated {
                            species: 25,
                            gender: Gender::Female,
                            held_item: 0,
                            trainer_id: 1,
                        },
                        Parent::Simulated {
                            species: 25,
                            gender: Gender::Male,
                            held_item: 0,
                            trainer_id: 2,
                        },
                    ],
                    seed: row["seed"].as_u64().unwrap() as u32,
                    offspring_pid: row["offspring_pid"].as_u64().unwrap() as u32,
                    production_item: None,
                };
                let result = preview_parents(&rom, None, &request, raws).unwrap();
                assert_eq!(
                    result.compatibility as u64,
                    row["compatibility"].as_u64().unwrap(),
                    "{key}"
                );
                assert_eq!(
                    serde_json::to_value(result.child_raw).unwrap(),
                    row["child_raw"],
                    "{key}"
                );
                assert_eq!(
                    result.rng_after as u64,
                    row["rng_after"].as_u64().unwrap(),
                    "{key}"
                );
                if row["simulated"] == true {
                    for i in 0..2 {
                        assert_eq!(
                            serde_json::to_value(
                                parent_raw(&rom, None, &request.parents[i]).unwrap()
                            )
                            .unwrap(),
                            row["parents"][i]
                        );
                    }
                    let simulated = preview(&rom, None, &request).unwrap();
                    assert_eq!(
                        serde_json::to_value(simulated.child_raw).unwrap(),
                        row["child_raw"],
                        "{key}: simulated request"
                    );
                }
            }
            assert_eq!(*original, *rom.data);
            let index = crate::acquisition::AcquisitionIndex::build(&rom).unwrap();
            let services = index.daycare_sources(&rom, None);
            // Native functions may be retained without a located map reference.
            // An empty service list must remain unknown rather than invent a location.
            assert!(services
                .iter()
                .all(|s| s.status == "unknown" && s.partial && s.repeatable.is_none()));
            eprintln!(
                "{key}: {} native scenarios; {} receiving sources",
                probes[key]["rows"].as_array().unwrap().len(),
                services.len()
            );
        }
    }
}
