//! Native ordinary-wild held-item selection in isolated RAM, never save editing.
use crate::{binary::*, err, native_trainer::Sandbox, pokemon, rom::Rom, Result};
use armv4t_emu::Memory;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct WildItemRules {
    pub routine: u32,
    pub rng: u32,
    pub rng_parameters: [usize; 2],
    pub enemy: u32,
    pub player: u32,
    pub getter: u32,
    pub map_header: Option<u32>,
    /// Native immediate used for the exceptional map-layout comparison.
    pub special_layout_instruction: Option<usize>,
    pub special_table_pointer: Option<usize>,
    pub special_rare_pointer: Option<usize>,
    pub special_count: usize,
    pub save_pointers: [u32; 2],
}
pub const EMERALD: WildItemRules = WildItemRules {
    routine: 0x0806ea68,
    rng: 0x03005d80,
    rng_parameters: [0x6f5ec, 0x6f5f0],
    enemy: 0x02024744,
    player: 0x020244ec,
    getter: 0x0806a518,
    map_header: Some(0x02037318),
    special_layout_instruction: Some(0x6eaca),
    special_table_pointer: Some(0x6ea54),
    special_rare_pointer: Some(0x6eb28),
    special_count: 9,
    save_pointers: [0x03005d8c, 0x03005d90],
};
pub const ROCKET: WildItemRules = WildItemRules {
    routine: 0x0809c610,
    rng: 0x03005240,
    rng_parameters: [0x9d42c, 0x9d430],
    enemy: 0x020253c8,
    player: 0x02025170,
    getter: 0x080976d0,
    map_header: Some(0x02036de0),
    special_layout_instruction: Some(0x9c6b2),
    special_table_pointer: Some(0x9c5fc),
    special_rare_pointer: Some(0x9c718),
    special_count: 9,
    save_pointers: [0x0300524c, 0x03005250],
};
pub const MERCURY: WildItemRules = WildItemRules {
    routine: 0x080443f4,
    rng: 0x03005000,
    rng_parameters: [0x44ee0, 0x44ee4],
    enemy: 0x0202402c,
    player: 0x02024284,
    getter: 0x0803fbe8,
    map_header: None,
    special_layout_instruction: None,
    special_table_pointer: None,
    special_rare_pointer: None,
    special_count: 0,
    save_pointers: [0x03005008, 0x0300500c],
};
#[derive(Clone, Debug, Serialize)]
pub struct HeldOutcome {
    pub item: u16,
    pub count: u32,
}
#[derive(Clone, Debug, Serialize)]
pub struct HeldDistribution {
    /// Counts under a uniform 16-bit native random output, not encounter probability.
    pub outcomes: Vec<HeldOutcome>,
    pub denominator: u32,
    pub lead_species: Option<u16>,
    pub lead_ability: Option<u16>,
    pub lead_egg: bool,
}
impl HeldDistribution {
    pub fn count(&self, item: u16) -> u32 {
        self.outcomes
            .iter()
            .find(|o| o.item == item)
            .map_or(0, |o| o.count)
    }
    pub fn percent(&self, item: u16) -> f64 {
        self.count(item) as f64 * 100.0 / self.denominator as f64
    }
}
#[derive(Clone, Serialize)]
pub struct HeldContext {
    pub species: u16,
    pub layout: u16,
    pub routine: u32,
    pub baseline: HeldDistribution,
    pub current_party: Option<HeldDistribution>,
}
/// Minimal synthetic record for native read-only queries. It is never inserted into a SAV.
fn fixture(rom: &Rom, species: u16) -> Result<Vec<u8>> {
    let codec = rom.profile.save.pokemon_codec;
    let fields = codec.fields();
    let mut raw = vec![0; 100];
    put32(&mut raw, 0, 24);
    put32(&mut raw, 4, 0x12345678);
    fields.language.write(&mut raw, 2)?;
    fields.has_species.write(&mut raw, 1)?;
    let mut c = [0; 48];
    put16(&mut c, 0, species);
    if let Some(f) = fields.nature_override {
        f.write(&mut c, 26)?;
    }
    pokemon::pack_with(&mut raw, &c, codec);
    Ok(raw)
}
fn inverse_odd(a: u32) -> Result<u32> {
    if a & 1 == 0 {
        return Err(err("wild_item_rng", "native multiplier is not invertible"));
    }
    let mut x = a;
    for _ in 0..5 {
        x = x.wrapping_mul(2u32.wrapping_sub(a.wrapping_mul(x)));
    }
    Ok(x)
}
impl Rom {
    pub(crate) fn held_layout(&self, layout: u16) -> Result<u16> {
        let rules = self
            .profile
            .wild_items
            .ok_or_else(|| err("wild_item_unverified", self.profile.id))?;
        let Some(pc) = rules.special_layout_instruction else {
            return Ok(0);
        };
        // MOVS r0,#imm; LSLS r0,r0,#1. Read the exact ROM's comparison value.
        let instruction = bytes(&self.data, pc, 4)?;
        if instruction[1] != 0x20 || instruction[2..] != [0x40, 0] {
            return Err(err("wild_item_layout", pc));
        }
        let special = instruction[0] as u16 * 2;
        Ok(if layout == special { layout } else { 0 })
    }
    pub(crate) fn special_held_species(&self, item: u16) -> Result<Vec<u16>> {
        let rules = self
            .profile
            .wild_items
            .ok_or_else(|| err("wild_item_unverified", self.profile.id))?;
        let Some(p) = rules.special_table_pointer else {
            return Ok(vec![]);
        };
        let table = pointer(&self.data, p)?;
        bytes(&self.data, table, rules.special_count * 4)?;
        // Index zero explicitly means normal-table fallback in the native helper.
        let mut species: Vec<u16> = (1..rules.special_count)
            .filter_map(|i| {
                let row = table + i * 4;
                match (u16(&self.data, row), u16(&self.data, row + 2)) {
                    (Ok(s), Ok(v)) if v == item => Some(Ok(s)),
                    (Ok(_), Ok(_)) => None,
                    (Err(e), _) | (_, Err(e)) => Some(Err(e)),
                }
            })
            .collect::<Result<_>>()?;
        // The exceptional-layout fallback can use a different column pointer
        // from the ordinary species table (notably Ultimate's unpatched pool).
        // Candidate indexing follows that actual pointer; execution still decides
        // whether a referenced encounter can select the requested item.
        if let Some(word) = rules.special_rare_pointer {
            let rare = pointer(&self.data, word)?;
            for id in 1..self.profile.species.count as u16 {
                if self.valid_species(id).is_ok()
                    && u16(
                        &self.data,
                        rare + id as usize * self.profile.base_stats.stride,
                    )? == item
                {
                    species.push(id);
                }
            }
        }
        species.sort_unstable();
        species.dedup();
        Ok(species)
    }

    pub fn wild_item_distribution(
        &self,
        species: u16,
        layout: u16,
        lead: Option<&[u8]>,
    ) -> Result<HeldDistribution> {
        self.valid_species(species)?;
        let rules = self
            .profile
            .wild_items
            .ok_or_else(|| err("wild_item_unverified", self.profile.id))?;
        let layout = self.held_layout(layout)?;
        let lead_info = lead
            .map(|raw| {
                if raw.len() != 100 {
                    return Err(err("wild_item_lead", raw.len()));
                }
                pokemon::decode(raw, self)
            })
            .transpose()?;
        let enemy = fixture(self, species)?;
        let baseline = fixture(self, 0)?;
        let mut ram = Sandbox::new(&self.data);
        for (address, block) in [
            (rules.save_pointers[0], 0x02030000),
            (rules.save_pointers[1], 0x02034000),
        ] {
            ram.w32(address, block);
        }
        if let Some(header) = rules.map_header {
            ram.w16(header + 18, layout);
        }
        for (i, byte) in lead.unwrap_or(&baseline).iter().enumerate() {
            ram.w8(rules.player + i as u32, *byte);
        }
        let a = u32(&self.data, rules.rng_parameters[0])?;
        let c = u32(&self.data, rules.rng_parameters[1])?;
        let inverse = inverse_odd(a)?;
        let mut counts = BTreeMap::new();
        // Verified routines use one native u16 Random() % 100 in a single ordinary
        // battle. Enumerate every residue with its exact number of u16 preimages.
        // The original RNG, branch selection, getters and setter all execute.
        for residue in 0u32..100 {
            for (i, byte) in enemy.iter().enumerate() {
                ram.w8(rules.enemy + i as u32, *byte);
            }
            let next = residue << 16;
            ram.w32(rules.rng, next.wrapping_sub(c).wrapping_mul(inverse));
            ram.call(rules.routine, [0; 4], [0; 2], 100_000)?;
            if ram.r32(rules.rng) != next {
                return Err(err("wild_item_rng", "unexpected native RNG consumption"));
            }
            let item = ram.call(rules.getter, [rules.enemy, 12, 0, 0], [0; 2], 100_000)?;
            if item > u16::MAX as u32 {
                return Err(err("wild_item_value", item));
            }
            *counts.entry(item as u16).or_insert(0) += if residue < 36 { 656 } else { 655 };
        }
        Ok(HeldDistribution {
            outcomes: counts
                .into_iter()
                .map(|(item, count)| HeldOutcome { item, count })
                .collect(),
            denominator: 65536,
            lead_species: lead_info.as_ref().map(|p| p.species),
            lead_ability: lead_info.as_ref().map(|p| p.ability_id),
            lead_egg: lead_info.is_some_and(|p| p.egg),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rng_preimages_are_exact_and_do_not_depend_on_game_content() {
        for a in [1u32, 3, 0x41c64e6d, u32::MAX] {
            let inverse = inverse_odd(a).unwrap();
            assert_eq!(a.wrapping_mul(inverse), 1);
            for next in [0u32, 1, 0x12340000, u32::MAX] {
                assert_eq!(
                    next.wrapping_sub(0x6073)
                        .wrapping_mul(inverse)
                        .wrapping_mul(a)
                        .wrapping_add(0x6073),
                    next
                );
            }
        }
        assert!(inverse_odd(2).is_err());
        assert_eq!(
            (0..100)
                .map(|v| if v < 36 { 656 } else { 655 })
                .sum::<u32>(),
            65536
        );
    }
    #[test]
    #[ignore = "requires all five exact local ROMs"]
    fn local_native_wild_item_baseline() {
        for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
            let r = Rom::open(
                std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap(),
            )
            .unwrap();
            let species = (1..r.profile.species.count as u16)
                .find(|s| {
                    r.valid_species(*s)
                        .is_ok_and(|s| s.items[1] != 0 && s.items[0] == 0)
                })
                .unwrap();
            let distribution = r.wild_item_distribution(species, 0, None).unwrap();
            assert_eq!(
                distribution.outcomes.iter().map(|o| o.count).sum::<u32>(),
                65536
            );
            assert_eq!(
                distribution.count(r.species(species).unwrap().items[1]),
                3275
            );
            println!("{key} native wild item baseline passed");
        }
    }
}

#[cfg(test)]
mod parity {
    use super::*;
    #[test]
    #[ignore = "requires five exact ROMs and GEN3_WILD_ITEM_PROBES independent CPU vectors"]
    fn local_native_wild_item_distribution_parity() {
        let vectors: serde_json::Value = serde_json::from_slice(
            &std::fs::read(std::env::var("GEN3_WILD_ITEM_PROBES").unwrap()).unwrap(),
        )
        .unwrap();
        for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
            let rom = Rom::open(
                std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap(),
            )
            .unwrap();
            let data = rom.data.clone();
            let report = &vectors[key];
            assert_eq!(report["md5"], rom.profile.md5);
            for row in report["rows"].as_array().unwrap() {
                let raw = row["lead"]["raw"].as_array().map(|values| {
                    values
                        .iter()
                        .map(|v| v.as_u64().unwrap() as u8)
                        .collect::<Vec<_>>()
                });
                let before = raw.clone();
                let distribution = rom
                    .wild_item_distribution(
                        row["species"].as_u64().unwrap() as u16,
                        row["layout"].as_u64().unwrap() as u16,
                        raw.as_deref(),
                    )
                    .unwrap();
                let expected: BTreeMap<u16, u32> = row["counts"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(id, n)| (id.parse().unwrap(), n.as_u64().unwrap() as u32))
                    .collect();
                let actual: BTreeMap<_, _> = distribution
                    .outcomes
                    .iter()
                    .map(|o| (o.item, o.count))
                    .collect();
                assert_eq!(
                    actual, expected,
                    "{key} species {} layout {} lead {} egg {}",
                    row["species"], row["layout"], row["lead"]["ability"], row["lead"]["egg"]
                );
                assert_eq!(distribution.denominator, 65536);
                assert_eq!(
                    distribution.lead_ability,
                    row["lead"]["ability"].as_u64().map(|v| v as u16)
                );
                assert_eq!(distribution.lead_egg, row["lead"]["egg"].as_bool().unwrap());
                assert_eq!(raw, before);
            }
            assert!(std::sync::Arc::ptr_eq(&data, &rom.data));
            println!(
                "{key}: {} native distribution contexts matched",
                report["rows"].as_array().unwrap().len()
            );
        }
    }
}
