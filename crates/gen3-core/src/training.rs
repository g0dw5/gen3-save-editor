//! Bounded EV-item, nature and ability queries through native ROM routines.
//! Classification is not a claim about application, consumption or current access.
use crate::{
    adapter::Field,
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
    pub abilities: &'static [AbilityRules],
}
pub const EMERALD: Rules = Rules {
    classify: 0x081b7cec,
    apply: 0x0806bd04,
    medicine_handlers: &[0x080fdea1],
    reduction_handler: Some(0x080fdebd),
    context_item: 175,
    nature: None,
    abilities: &[],
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
    abilities: &[
        AbilityRules {
            handler: 0x081361f1,
            mechanism: "normal_swap",
            variants: None,
            ability_header: None,
            requires_rng_seed: false,
            execution: AbilityExecution::Slot {
                initialize: 0x08203398,
                initialized: 0x082033fa,
                task_data: 0x030052c8,
                callback: 0x0820318c,
                accepted: 0x08203238,
                rejected: 0x082031fa,
                applied: 0x0820336a,
            },
        },
        AbilityRules {
            handler: 0x0813620d,
            mechanism: "hidden_toggle",
            variants: None,
            ability_header: None,
            requires_rng_seed: false,
            execution: AbilityExecution::Slot {
                initialize: 0x08203648,
                initialized: 0x082036ac,
                task_data: 0x030052c8,
                callback: 0x08203430,
                accepted: 0x082034e8,
                rejected: 0x082034ac,
                applied: 0x0820361a,
            },
        },
    ],
};
pub const MERCURY: Rules = Rules {
    classify: 0x08126c68,
    apply: 0x08042414,
    medicine_handlers: &[0x080a16e1],
    reduction_handler: None,
    context_item: 175,
    nature: None,
    abilities: &[AbilityRules {
        handler: 0x09d58019,
        mechanism: "normal_swap",
        variants: None,
        ability_header: None,
        requires_rng_seed: true,
        execution: AbilityExecution::Pid {
            selected_item: 0x0203ad30,
            select: 0x09d561e0,
            selected: 0x09d56246,
            target_register: 4,
            apply: 0x09d55fb4,
            applied: 0x09d56006,
        },
    }],
};
pub const ULTIMATE: Rules = Rules {
    abilities: &[AbilityRules {
        handler: 0x08f7f111,
        mechanism: "normal_swap",
        variants: Some(AbilityVariants {
            getter: 0x080d7644,
            choices: &[
                AbilityVariant {
                    value: 1,
                    mechanism: "normal_swap",
                    requires_rng_seed: false,
                },
                AbilityVariant {
                    value: 2,
                    mechanism: "hidden_toggle",
                    requires_rng_seed: true,
                },
            ],
        }),
        ability_header: Some(Field::new(30, 0, 1)),
        requires_rng_seed: false,
        execution: AbilityExecution::Script {
            selected_individual: 0x020375e0,
            selected_item: 0x0203ce7c,
            entry: 0x09f00dd0,
            accepted: 0x09f00e20,
            rejected: 0x09f00e3e,
            applied: 0x09f00e2c,
            target_pointer_register: 6,
        },
    }],
    ..EMERALD
};
#[derive(Clone, Copy, Debug, Serialize)]
pub struct AbilityVariants {
    pub getter: u32,
    pub choices: &'static [AbilityVariant],
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct AbilityVariant {
    pub value: u32,
    pub mechanism: &'static str,
    pub requires_rng_seed: bool,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct AbilityRules {
    pub handler: u32,
    pub mechanism: &'static str,
    pub execution: AbilityExecution,
    pub variants: Option<AbilityVariants>,
    pub ability_header: Option<Field>,
    pub requires_rng_seed: bool,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AbilityExecution {
    /// A script-native prefix with acceptance before the setter and a separate
    /// post-setter boundary. Hidden targets 2/3 share one header flag.
    Script {
        selected_individual: u32,
        selected_item: u32,
        entry: u32,
        accepted: u32,
        rejected: u32,
        applied: u32,
        target_pointer_register: usize,
    },
    Slot {
        initialize: u32,
        initialized: u32,
        task_data: u32,
        callback: u32,
        accepted: u32,
        rejected: u32,
        applied: u32,
    },
    Pid {
        selected_item: u32,
        select: u32,
        selected: u32,
        target_register: usize,
        apply: u32,
        applied: u32,
    },
}
#[derive(Serialize)]
pub struct AbilityOffer {
    pub item: u16,
    pub handler: u32,
    pub mechanism: &'static str,
    pub random_pid: bool,
    pub requires_rng_seed: bool,
    pub partial: bool,
}
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
    pub ability_items: Vec<AbilityOffer>,
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
        #[serde(default)]
        ability_slot: Option<u8>,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub expected_rom_md5: String,
    pub item: u16,
    pub individual: Individual,
    #[serde(default)]
    pub rng_seed: Option<u32>,
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
    pub ability_target: Option<u16>,
    pub rng_seed: Option<u32>,
    pub rng_after: Option<u32>,
    pub partial: bool,
    #[cfg(test)]
    #[serde(skip)]
    pub(crate) raw: Vec<u8>,
}
/// Native guard result. `target` is a slot for Slot rules, an ability ID for Pid rules.
/// This stage deliberately stops before confirmation, mutation and item consumption.
pub(crate) struct AbilityDecision {
    pub accepted: bool,
    pub target: u32,
}
pub(crate) fn ability_decision(
    ram: &mut Sandbox<'_>,
    rule: &AbilityRules,
    item: u16,
) -> Result<AbilityDecision> {
    let (accepted, target) = match rule.execution {
        AbilityExecution::Script { accepted, .. } => {
            return script_ability_stage(ram, rule, item, accepted)
        }
        AbilityExecution::Slot {
            initialize,
            initialized,
            task_data,
            callback,
            accepted,
            rejected,
            ..
        } => {
            ram.observe(initialize, [0; 4], [0; 2], 250_000, initialized)?;
            let target = ram.r16(task_data + 4) as u32;
            let (_, boundary) =
                ram.observe_any(callback, [0; 4], [0; 2], 250_000, &[accepted, rejected])?;
            (boundary == accepted, target)
        }
        AbilityExecution::Pid {
            selected_item,
            select,
            selected,
            target_register,
            ..
        } => {
            ram.w16(selected_item, item);
            let registers = ram.observe(select, [0; 4], [0; 2], 250_000, selected)?;
            let target = *registers
                .get(target_register)
                .ok_or_else(|| err("training_ability_register", target_register))?;
            (target != 0, target)
        }
    };
    Ok(AbilityDecision { accepted, target })
}
fn script_ability_stage(
    ram: &mut Sandbox<'_>,
    rule: &AbilityRules,
    item: u16,
    stop: u32,
) -> Result<AbilityDecision> {
    let AbilityExecution::Script {
        selected_individual,
        selected_item,
        entry,
        rejected,
        target_pointer_register,
        ..
    } = rule.execution
    else {
        return Err(err("training_ability_stage", "not a script prefix"));
    };
    ram.w16(selected_individual, 0);
    ram.w16(selected_item, item);
    let (registers, boundary) =
        ram.observe_any(entry, [0; 4], [0; 2], 250_000, &[stop, rejected])?;
    let target = if boundary == stop {
        let pointer = *registers
            .get(target_pointer_register)
            .ok_or_else(|| err("training_ability_register", target_pointer_register))?;
        ram.r8(pointer) as u32
    } else {
        0
    };
    Ok(AbilityDecision {
        accepted: boundary == stop,
        target,
    })
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
        let mut ability_items = Vec::new();
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
            if let Some(ability) = rules.abilities.iter().find(|r| r.handler == handler) {
                let (mechanism, requires_rng_seed) = if let Some(variants) = ability.variants {
                    let value = ram.call(variants.getter, [item as u32, 0, 0, 0], [0; 2], 8192)?;
                    let choice = variants
                        .choices
                        .iter()
                        .find(|v| v.value == value)
                        .ok_or_else(|| err("training_ability_variant", value))?;
                    (choice.mechanism, choice.requires_rng_seed)
                } else {
                    (ability.mechanism, ability.requires_rng_seed)
                };
                ability_items.push(AbilityOffer {
                    item,
                    handler,
                    mechanism,
                    requires_rng_seed,
                    random_pid: matches!(ability.execution, AbilityExecution::Pid { .. }),
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
            ability_items,
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
        let ability_offer = catalog
            .ability_items
            .iter()
            .find(|o| o.item == request.item);
        let ability_rule =
            ability_offer.and_then(|o| rules.abilities.iter().find(|r| r.handler == o.handler));
        let random_pid =
            ability_rule.is_some_and(|r| matches!(r.execution, AbilityExecution::Pid { .. }));
        let rng_seed = if ability_offer.is_some_and(|offer| offer.requires_rng_seed) {
            Some(request.rng_seed.ok_or_else(|| {
                err(
                    "training_seed_required",
                    "explicit native random-selection scenario",
                )
            })?)
        } else {
            None
        };
        if target_nature.is_none()
            && ability_rule.is_none()
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
                ability_slot,
            } => {
                let raw = pokemon::create(self, species, 1, "", level, 42)?;
                let (raw, _) = pokemon::edit(
                    &raw,
                    &pokemon::PokemonPatch {
                        evs: Some(evs),
                        friendship: Some(friendship),
                        nature_override,
                        ability_slot,
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
        let mut ability_target = None;
        if let Some(seed) = rng_seed {
            ram.w32(b.rng, seed);
        }
        let (native_no_effect, effect_scope) = if let Some(rule) = ability_rule {
            // Script prefixes run through mutation exactly once, so a native
            // random slot selection is neither replaced nor advanced twice.
            let decision = if let AbilityExecution::Script { applied, .. } = rule.execution {
                script_ability_stage(&mut ram, rule, request.item, applied)?
            } else {
                ability_decision(&mut ram, rule, request.item)?
            };
            if decision.accepted {
                ability_target = Some(match rule.execution {
                    AbilityExecution::Script { .. } => {
                        if decision.target > 3 {
                            return Err(err("training_ability_target", decision.target));
                        }
                        let slot = if decision.target >= 2 {
                            2
                        } else {
                            decision.target
                        };
                        self.species(before.species)?
                            .abilities
                            .get(slot as usize)
                            .copied()
                            .ok_or_else(|| err("training_ability_target", decision.target))?
                    }
                    AbilityExecution::Slot { .. } => self
                        .species(before.species)?
                        .abilities
                        .get(decision.target as usize)
                        .copied()
                        .ok_or_else(|| err("training_ability_target", decision.target))?,
                    AbilityExecution::Pid { .. } => u16::try_from(decision.target)
                        .map_err(|_| err("training_ability_target", decision.target))?,
                });
                match rule.execution {
                    AbilityExecution::Script { .. } => {}
                    AbilityExecution::Slot {
                        task_data,
                        callback,
                        applied,
                        ..
                    } => {
                        ram.w16(task_data, 5);
                        ram.observe(callback, [0; 4], [0; 2], 250_000, applied)?;
                    }
                    AbilityExecution::Pid { apply, applied, .. } => {
                        ram.observe(apply, [0; 4], [0; 2], 1_000_000, applied)?;
                    }
                }
            }
            let accepted = decision.accepted;
            (!accepted, "ability_persistent_stage")
        } else if let Some(target) = target_nature {
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
        if ability_rule.is_some() && native_no_effect && after_raw != raw {
            return Err(err(
                "training_native_unrelated",
                "rejected ability operation changed the individual",
            ));
        }

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
        if ability_rule.is_some() {
            let before_canonical =
                pokemon::checked_unpack_with(&raw, self.profile.save.pokemon_codec)?;
            let field = self.profile.save.pokemon_codec.fields().ability;
            field.write(&mut after_canonical, field.read(&before_canonical)?)?;
            let mut after_header = after_raw[..32].to_vec();
            if let Some(field) = ability_rule.and_then(|rule| rule.ability_header) {
                field.write(&mut after_header, field.read(&raw)?)?;
            }
            if after_canonical != before_canonical
                || raw[4..28] != after_header[4..28]
                || raw[30..32] != after_header[30..32]
            {
                return Err(err(
                    "training_native_unrelated",
                    "unexpected ability persistent-data change",
                ));
            }
        }
        let after = pokemon::decode(&after_raw, self)?;
        if (!random_pid && before.pid != after.pid)
            || before.ot_id != after.ot_id
            || before.species != after.species
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
            ability_target,
            rng_seed,
            rng_after: rng_seed.map(|_| ram.r32(b.rng)),
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
