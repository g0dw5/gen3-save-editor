//! Bounded EV-item and mint persistent-stage queries through native ROM routines.
//! Classification is not a claim about application, consumption or current access.
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
pub struct Rules {
    pub classify: u32,
    pub apply: u32,
    pub medicine_handlers: &'static [u32],
    pub reduction_handler: Option<u32>,
    pub context_item: u16,
    pub nature: Option<NatureRules>,
}
pub const EMERALD: Rules = Rules {
    classify: 0x081b7cec,
    apply: 0x0806bd04,
    medicine_handlers: &[0x080fdea1],
    reduction_handler: Some(0x080fdebd),
    context_item: 175,
    nature: None,
};
pub const ROCKET: Rules = Rules {
    classify: 0x082064ac,
    apply: 0x08098eb0,
    medicine_handlers: &[0x081361d5],
    reduction_handler: None,
    context_item: 303,
    nature: Some(NatureRules {
        handler: 0x08136ddd,
        item_nature: 0x0810fa0c,
        set_mon_data: 0x08097e2c,
        field: 89,
        calculate_stats: 0x080967b4,
    }),
};
pub const MERCURY: Rules = Rules {
    classify: 0x08126c68,
    apply: 0x08042414,
    medicine_handlers: &[0x080a16e1],
    reduction_handler: None,
    context_item: 175,
    nature: None,
};
/// Verified persistent stage of a field mint callback; menu/consumption excluded.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct NatureRules {
    pub handler: u32,
    pub item_nature: u32,
    pub set_mon_data: u32,
    pub field: u32,
    pub calculate_stats: u32,
}
#[derive(Serialize)]
pub struct NatureOffer {
    pub item: u16,
    pub nature: u8,
    pub handler: u32,
    pub partial: bool,
}
#[derive(Serialize)]
pub struct Offer {
    pub item: u16,
    /// Native category decoded from verified switch cases, not item-name inference.
    pub stat: usize,
    pub direction: &'static str,
    pub handler: u32,
    pub native_category: u32,
    pub partial: bool,
}
#[derive(Serialize)]
pub struct Catalog {
    pub rom_md5: &'static str,
    pub offers: Vec<Offer>,
    pub nature_items: Vec<NatureOffer>,
    pub partial: bool,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Individual {
    Stored {
        location: Location,
    },
    Simulated {
        species: u16,
        level: u8,
        evs: [u8; 6],
        friendship: u8,
        #[serde(default)]
        nature_override: Option<u8>,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub expected_rom_md5: String,
    pub item: u16,
    pub individual: Individual,
}
#[derive(Debug, Serialize)]
pub struct Preview {
    pub rom_md5: &'static str,
    pub item: u16,
    pub before: pokemon::Pokemon,
    pub after: pokemon::Pokemon,
    /// Actual native party stat storage, not decode-time recalculated estimates.
    pub party_stats_before: [u16; 6],
    pub party_stats_after: [u16; 6],
    pub native_no_effect: bool,
    pub changed: bool,
    pub scenario: &'static str,
    pub context: &'static str,
    pub party_state: &'static str,
    pub effect_scope: &'static str,
    pub partial: bool,
    #[cfg(test)]
    #[serde(skip)]
    pub(crate) raw: Vec<u8>,
}
fn category_stat(category: u32) -> Option<usize> {
    match category {
        13 => Some(0),
        12 => Some(1),
        17 => Some(2),
        16 => Some(3),
        14 => Some(4),
        15 => Some(5),
        _ => None,
    }
}
impl Rom {
    pub fn training_catalog(&self) -> Result<Catalog> {
        let rules = self
            .profile
            .training
            .ok_or_else(|| err("training_unverified", self.profile.id))?;
        let mut ram = Sandbox::new(&self.data);
        let mut offers = Vec::new();
        let mut nature_items = Vec::new();
        for item in 1..self.profile.items.count as u16 {
            // Enigma uses save-dependent effect bytes; never classify it in zero RAM.
            if item == rules.context_item {
                continue;
            }
            let offset = self.profile.items.offset + item as usize * self.profile.items.stride;
            let handler = u32(&self.data, offset + 28)?;
            if let Some(nature) = rules.nature.filter(|n| n.handler == handler) {
                let target = ram.call(nature.item_nature, [item as u32, 0, 0, 0], [0; 2], 8192)?;
                if target >= 25 {
                    return Err(err("training_nature_target", target));
                }
                nature_items.push(NatureOffer {
                    item,
                    nature: target as u8,
                    handler,
                    partial: true,
                });
                continue;
            }
            let reduction = rules.reduction_handler == Some(handler);
            if !reduction && !rules.medicine_handlers.contains(&handler) {
                continue;
            }
            let category = ram.call(rules.classify, [item as u32, 0, 0, 0], [0; 2], 8192)?;
            if let Some(stat) = category_stat(category) {
                offers.push(Offer {
                    item,
                    stat,
                    direction: if reduction { "decrease" } else { "increase" },
                    handler,
                    native_category: category,
                    partial: true,
                });
            }
        }
        Ok(Catalog {
            rom_md5: self.profile.md5,
            offers,
            nature_items,
            partial: true,
        })
    }
    pub fn training_preview(&self, save: Option<&Save>, request: Request) -> Result<Preview> {
        if request.expected_rom_md5 != self.profile.md5 {
            return Err(err("rom_mismatch", "training scenario"));
        }
        if let Some(save) = save {
            if save.layout.pokemon_codec != self.profile.save.pokemon_codec
                || save.layout.sizes != self.profile.save.sizes
            {
                return Err(err("training_save_layout", "SAVE and ROM layouts differ"));
            }
        }
        let rules = self
            .profile
            .training
            .ok_or_else(|| err("training_unverified", self.profile.id))?;
        let catalog = self.training_catalog()?;
        let target_nature = catalog
            .nature_items
            .iter()
            .find(|offer| offer.item == request.item)
            .map(|offer| offer.nature);
        if target_nature.is_none()
            && !catalog
                .offers
                .iter()
                .any(|offer| offer.item == request.item)
        {
            return Err(err("training_item_unverified", request.item));
        }
        let (raw, scenario, party_state) = match request.individual {
            Individual::Stored { location } => {
                let save = save.ok_or_else(|| err("save_required", "training individual"))?;
                if save.layout.pokemon_codec != self.profile.save.pokemon_codec
                    || save.layout.sizes != self.profile.save.sizes
                {
                    return Err(err("training_save_layout", "SAVE and ROM layouts differ"));
                }
                let raw = save.raw(location)?;
                let state = if raw.len() == 100 {
                    "stored_party"
                } else {
                    "boxed_full_hp_scenario"
                };
                let raw = if raw.len() == 100 {
                    raw
                } else {
                    pokemon::to_party(&raw, self)?
                };
                (raw, "stored_individual", state)
            }
            Individual::Simulated {
                species,
                level,
                evs,
                friendship,
                nature_override,
            } => {
                let raw = pokemon::create(self, species, 1, "", level, 42)?;
                let (raw, _) = pokemon::edit(
                    &raw,
                    &pokemon::PokemonPatch {
                        evs: Some(evs),
                        friendship: Some(friendship),
                        nature_override,
                        ..Default::default()
                    },
                    self,
                    pokemon::Policy::Free,
                )?;
                (
                    pokemon::to_party(&raw, self)?,
                    "simulated_individual",
                    "simulated_full_hp",
                )
            }
        };
        pokemon::checked_unpack_with(&raw, self.profile.save.pokemon_codec)?;
        let before = pokemon::decode(&raw, self)?;
        if before.species == 0 || before.egg {
            return Err(err("training_individual", "requires a non-egg individual"));
        }
        let b = self
            .profile
            .breeding
            .ok_or_else(|| err("training_context_unverified", self.profile.id))?;
        let mut ram = Sandbox::new(&self.data);
        for (pointer, address, section) in [
            (b.save_pointers[0], 0x02030000, 1..=4),
            (b.save_pointers[1], 0x02034000, 0..=0),
        ] {
            ram.w32(pointer, address);
            if let Some(save) = save {
                let bytes = save.logical(section);
                if bytes.len() > 0x4000 {
                    return Err(err("training_save_block", bytes.len()));
                }
                for (i, byte) in bytes.into_iter().enumerate() {
                    ram.w8(address + i as u32, byte);
                }
            }
        }
        for (i, byte) in raw.iter().enumerate() {
            ram.w8(b.party + i as u32, *byte);
        }
        ram.w8(b.party_count, 1);
        // Ordinary field use, party index zero, move index zero. Native item effect
        // only: bag consumption, menu eligibility and live battle/facility state excluded.
        let (native_no_effect, effect_scope) = if let Some(target) = target_nature {
            let nature = rules
                .nature
                .ok_or_else(|| err("training_nature_unverified", request.item))?;
            let no_effect = before.effective_nature == target;
            // The field callback rejects an unchanged effective nature. Reproduce
            // its confirmed persistent stage only, never its menu or bag calls.
            if !no_effect {
                ram.w8(0x02038000, target);
                ram.call(
                    nature.set_mon_data,
                    [b.party, nature.field, 0x02038000, 0],
                    [0; 2],
                    250_000,
                )?;
                ram.call(nature.calculate_stats, [b.party, 0, 0, 0], [0; 2], 250_000)?;
            }
            (no_effect, "nature_persistent_stage")
        } else {
            (
                ram.call(
                    rules.apply,
                    [b.party, request.item as u32, 0, 0],
                    [0; 2],
                    250_000,
                )? != 0,
                "field_effect",
            )
        };
        let after_raw: Vec<u8> = (0..100).map(|i| ram.r8(b.party + i)).collect();
        let mut after_canonical =
            pokemon::checked_unpack_with(&after_raw, self.profile.save.pokemon_codec)?;
        if target_nature.is_some() {
            let before_canonical =
                pokemon::checked_unpack_with(&raw, self.profile.save.pokemon_codec)?;
            let field = self
                .profile
                .save
                .pokemon_codec
                .fields()
                .nature_override
                .ok_or_else(|| err("training_nature_layout", "missing override field"))?;
            field.write(&mut after_canonical, field.read(&before_canonical)?)?;
            if after_canonical != before_canonical
                || raw[..28] != after_raw[..28]
                || raw[30..32] != after_raw[30..32]
            {
                return Err(err(
                    "training_native_unrelated",
                    "unexpected mint persistent-data change",
                ));
            }
        }
        let after = pokemon::decode(&after_raw, self)?;
        if before.pid != after.pid || before.ot_id != after.ot_id || before.species != after.species
        {
            return Err(err(
                "training_native_identity",
                "unexpected native identity change",
            ));
        }
        let mut party_stats_before = [0; 6];
        let mut party_stats_after = [0; 6];
        for i in 0..6 {
            party_stats_before[i] = u16(&raw, 88 + i * 2)?;
            party_stats_after[i] = u16(&after_raw, 88 + i * 2)?;
        }
        Ok(Preview {
            rom_md5: self.profile.md5,
            party_stats_before,
            party_stats_after,
            item: request.item,
            before,
            after,
            native_no_effect,
            effect_scope,
            changed: raw != after_raw,
            scenario,
            context: if save.is_some() {
                "save_blocks"
            } else {
                "zero_save_blocks"
            },
            party_state,
            partial: true,
            #[cfg(test)]
            raw: after_raw,
        })
    }
}
