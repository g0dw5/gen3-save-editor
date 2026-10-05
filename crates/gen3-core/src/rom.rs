use crate::{
    binary::*,
    err,
    profile::{self, Profile},
    text::Codec,
    Result,
};
use serde::Serialize;
use std::{
    collections::{BTreeSet, HashSet},
    sync::Arc,
};

pub(crate) fn evolution_condition(method: u16, expanded: bool) -> &'static str {
    match method {
        1 => "friendship",
        2 => "friendship_day",
        3 => "friendship_night",
        4 => "level",
        5 => "trade",
        6 => "trade_item",
        7 => "item",
        8 => "attack_higher",
        9 => "attack_equal",
        10 => "defense_higher",
        11 | 12 => "personality",
        13 => "level",
        14 => "shedinja",
        15 => "beauty",
        16 if expanded => "female_level",
        17 if expanded => "male_level",
        18 if expanded => "night_level",
        19 if expanded => "day_level",
        20 if expanded => "dusk_level",
        21 if expanded => "held_day",
        22 if expanded => "held_night",
        23 if expanded => "move",
        24 if expanded => "move_type",
        25 if expanded => "region",
        26 if expanded => "male_item",
        27 if expanded => "female_item",
        28 if expanded => "rain_level",
        29 if expanded => "party_species",
        30 if expanded => "party_dark",
        31 if expanded => "trade_species",
        32 if expanded => "map",
        _ => "unknown",
    }
}

pub(crate) fn cfru_evolution_condition(method: u16) -> &'static str {
    match method {
        1 => "friendship",
        2 => "friendship_day",
        3 => "friendship_night",
        4 => "level",
        5 => "trade",
        6 => "trade_item",
        7 => "item",
        8 => "attack_higher",
        9 => "attack_equal",
        10 => "defense_higher",
        11 | 12 => "personality",
        13 => "level",
        14 => "shedinja",
        15 => "beauty",
        16 => "rain_or_fog",
        17 => "move_type",
        18 => "party_type",
        19 => "map",
        20 => "male_level",
        21 => "female_level",
        22 => "night_level",
        23 => "day_level",
        24 => "held_night",
        25 => "held_day",
        26 => "move",
        27 => "party_species",
        28 => "time_range",
        29 => "flag",
        30 => "critical_hits",
        31 => "nature_high",
        32 => "nature_low",
        33 => "damage_location",
        34 => "item_location",
        35 => "level_hold_item",
        36 => "item_hold_item",
        37 => "move_male",
        38 => "move_female",
        39 => "item_night",
        _ => "unknown",
    }
}

#[derive(Clone)]
pub struct Rom {
    pub data: Arc<Vec<u8>>,
    pub profile: Profile,
    pub codec: Codec,
}
#[derive(Clone, Debug, Serialize)]
pub struct Species {
    pub dex_number: u16,
    pub id: u16,
    pub name: String,
    pub stats: [u8; 6],
    pub types: [u8; 2],
    pub catch_rate: u8,
    pub exp_yield: u16,
    pub ev_yield: u16,
    pub items: [u16; 2],
    pub gender_ratio: u8,
    pub egg_cycles: u8,
    pub friendship: u8,
    pub growth: u8,
    pub egg_groups: [u8; 2],
    pub abilities: Vec<u16>,
    pub offset: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct Move {
    pub id: u16,
    pub name: String,
    pub description: String,
    pub effect: u16,
    pub power: u16,
    pub move_type: u8,
    /// Raw split category: 0 physical, 1 special, 2 status, 3 Curse in these profiles.
    pub category: u8,
    pub accuracy: u8,
    pub pp: u8,
    pub chance: u8,
    pub target: u16,
    pub priority: i8,
    pub flags: u32,
    pub offset: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct Item {
    pub id: u16,
    pub name: String,
    pub description: String,
    pub price: u16,
    pub hold_effect: u8,
    pub hold_param: u8,
    pub pocket: u8,
    pub item_type: u8,
    pub tm_move: Option<u16>,
    pub offset: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct Ability {
    pub id: u16,
    pub name: String,
    pub description: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct EvolutionRequirement {
    pub kind: &'static str,
    pub value: u16,
}
#[derive(Clone, Debug, Serialize)]
pub struct Evolution {
    pub method: u16,
    pub condition: &'static str,
    pub requirements: Vec<EvolutionRequirement>,
    pub parameter: u16,
    /// CFRU uses the last halfword for compound requirements (item, type, time).
    pub auxiliary: u16,
    pub target: u16,
    pub offset: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct LearnSource {
    pub move_id: u16,
    pub source: String,
    pub species: u16,
    pub level: Option<u8>,
    pub index: Option<u16>,
    pub offset: usize,
}
#[derive(Serialize)]
pub struct NamedLocation {
    pub id: u8,
    pub name: String,
}
#[derive(Serialize)]
pub struct BallOption {
    pub value: u16,
    pub item: u16,
}
#[derive(Serialize)]
pub struct EditorRules {
    pub balls: Vec<u16>,
    pub ball_options: Vec<BallOption>,
    pub hyper_training: bool,
    pub pokemon_checksum: bool,
    pub origin_game_max: u8,
    pub met_level_max: u8,
    pub nature_override: bool,
    pub contest_ranks: [u8; 5],
}
#[derive(Serialize)]
pub struct Nature {
    pub id: u8,
    pub name: String,
    /// Attack, Defense, Speed, Sp. Attack, Sp. Defense; -1/0/+1.
    pub stat_changes: [i8; 5],
}
#[derive(Serialize)]
pub struct Catalog {
    pub natures: Vec<Nature>,
    pub type_names: Vec<String>,
    pub form_families: Vec<crate::relations::FormFamily>,
    pub battle_forms: Vec<crate::forms::BattleForm>,
    pub editor_rules: EditorRules,
    pub profile: Profile,
    pub species: Vec<Species>,
    pub moves: Vec<Move>,
    pub items: Vec<Item>,
    pub abilities: Vec<Ability>,
    pub met_locations: Vec<NamedLocation>,
}
#[derive(Serialize)]
pub struct SpeciesDetail {
    pub relations: crate::relations::SpeciesRelations,
    pub teaching_list_present: bool,
    pub battle_forms: Vec<crate::forms::BattleForm>,
    pub encounters_verified: bool,
    pub species: Species,
    pub evolutions: Vec<Evolution>,
    pub learnset: Vec<LearnSource>,
    pub encounters: Vec<crate::world::Encounter>,
    pub origins: OriginOptions,
}
#[derive(Serialize)]
pub struct OriginOptions {
    pub ancestors: Vec<u16>,
    pub encounters: Vec<crate::world::Encounter>,
    pub can_hatch: bool,
    pub hatch_regions: Vec<u8>,
}
impl Rom {
    pub fn is_mail(&self, id: u16) -> bool {
        id != 0 && (self.profile.mail_items[0]..=self.profile.mail_items[1]).contains(&id)
    }
    pub fn open(data: Vec<u8>) -> Result<Self> {
        let profile = profile::identify(&data)?;
        let r = Self {
            data: Arc::new(data),
            profile,
            codec: Codec::new(),
        };
        r.catalog()?;
        Ok(r)
    }
    #[cfg(test)]
    pub(crate) fn fixture(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(data),
            profile: profile::Profile {
                wild_items: None,
                ..profile::BW
            },
            codec: Codec::new(),
        }
    }
    pub fn text(&self, o: usize, n: usize) -> Result<String> {
        Ok(self.codec.decode(bytes(&self.data, o, n)?))
    }
    pub fn cstring(&self, o: usize) -> String {
        let end = (o + 1024).min(self.data.len());
        self.data
            .get(o..end)
            .map(|b| self.codec.decode(b))
            .unwrap_or_default()
    }
    pub fn ptr_text(&self, o: usize) -> String {
        pointer(&self.data, o)
            .map(|p| self.cstring(p))
            .unwrap_or_default()
    }
    pub fn species(&self, id: u16) -> Result<Species> {
        if id as usize >= self.profile.species.count {
            return Err(err("species_id", id));
        }
        let o = self.profile.base_stats.offset + id as usize * self.profile.base_stats.stride;
        let expanded = self.profile.formats.species == crate::adapter::SpeciesFormat::Expanded36;
        let b = bytes(&self.data, o, if expanded { 36 } else { 28 })?;
        let tail = if expanded { 2 } else { 0 };
        Ok(Species {
            dex_number: if id == 0 {
                0
            } else {
                u16(
                    &self.data,
                    self.profile.national_dex + (id as usize - 1) * 2,
                )?
            },
            id,
            name: self.text(
                self.profile.species.offset + id as usize * self.profile.species.stride,
                11,
            )?,
            stats: b[..6].try_into().unwrap(),
            types: [b[6], b[7]],
            catch_rate: b[8],
            exp_yield: if expanded { u16(b, 10)? } else { b[9] as u16 },
            ev_yield: u16(b, 10 + tail)?,
            items: [u16(b, 12 + tail)?, u16(b, 14 + tail)?],
            gender_ratio: b[16 + tail],
            egg_cycles: b[17 + tail],
            friendship: b[18 + tail],
            growth: b[19 + tail],
            egg_groups: [b[20 + tail], b[21 + tail]],
            abilities: if let Some(table) = self.profile.species_abilities {
                (0..table.stride / 2)
                    .map(|slot| {
                        u16(
                            &self.data,
                            table.offset + id as usize * table.stride + slot * 2,
                        )
                    })
                    .collect::<Result<_>>()?
            } else if expanded {
                vec![u16(b, 24)?, u16(b, 26)?, u16(b, 28)?]
            } else if self.profile.formats.species == crate::adapter::SpeciesFormat::Cfru28 {
                vec![b[22] as u16, b[23] as u16, b[26] as u16]
            } else {
                vec![b[22] as u16, b[23] as u16]
            },
            offset: o,
        })
    }
    pub fn valid_species(&self, id: u16) -> Result<Species> {
        let s = self.species(id)?;
        if id == 0 || s.stats[0] == 0 {
            return Err(err("species_id", id));
        }
        Ok(s)
    }
    pub fn move_info(&self, id: u16) -> Result<Move> {
        if id as usize >= self.profile.moves.count {
            return Err(err("move_id", id));
        }
        let o = self.profile.moves.offset + id as usize * self.profile.moves.stride;
        let expanded = self.profile.formats.moves == crate::adapter::MoveFormat::Expanded20;
        let b = bytes(&self.data, o, if expanded { 20 } else { 12 })?;
        let no = self.profile.move_names.at(id as usize);
        Ok(Move {
            id,
            name: self.text(no, 13)?,
            description: if id > 0 && self.profile.move_descriptions != 0 {
                self.ptr_text(self.profile.move_descriptions + (id as usize - 1) * 4)
            } else {
                String::new()
            },
            effect: if expanded { u16(b, 0)? } else { b[0] as u16 },
            power: if expanded { u16(b, 2)? } else { b[1] as u16 },
            move_type: if expanded { b[4] } else { b[2] },
            category: bytes(&self.data, o + self.profile.move_category_offset, 1)?[0],
            accuracy: if expanded { b[5] } else { b[3] },
            pp: if expanded { b[6] } else { b[4] },
            chance: if expanded { b[7] } else { b[5] },
            target: if expanded { u16(b, 8)? } else { b[6] as u16 },
            priority: if expanded { b[10] as i8 } else { b[7] as i8 },
            flags: if expanded { u32(b, 12)? } else { b[8] as u32 },
            offset: o,
        })
    }
    pub fn item(&self, id: u16) -> Result<Item> {
        if id as usize >= self.profile.items.count {
            return Err(err("item_id", id));
        }
        let o = self.profile.items.offset + id as usize * self.profile.items.stride;
        let b = bytes(&self.data, o, 44)?;
        Ok(Item {
            id,
            name: self.text(o, 14)?,
            description: self.ptr_text(o + 20),
            price: u16(b, 16)?,
            hold_effect: b[18],
            hold_param: b[19],
            pocket: b[26],
            item_type: b[27],
            tm_move: if self.profile.capabilities.complete_learnsets {
                let teaching = self.profile.teaching;
                let machine = if teaching.machine_item_ranges.is_empty() {
                    (teaching.tm_first_item..teaching.tm_first_item + teaching.tm_count as u16)
                        .contains(&id)
                        .then_some(id.saturating_sub(teaching.tm_first_item))
                } else {
                    teaching.machine_item_ranges.iter().find_map(|range| {
                        (range.item_first..range.item_first + range.item_count)
                            .contains(&id)
                            .then(|| range.move_first + id - range.item_first)
                    })
                };
                machine
                    .map(|index| u16(&self.data, self.profile.tm_moves + index as usize * 2))
                    .transpose()?
            } else {
                None
            },
            offset: o,
        })
    }
    pub fn ability(&self, id: u16) -> Result<Ability> {
        if id >= self.profile.ability_count {
            return Err(err("ability_id", id));
        }
        let (n, d) = (
            self.profile.ability_names.at(id as usize),
            self.profile.ability_descriptions.at(id as usize),
        );
        Ok(Ability {
            id,
            name: self.text(n, self.profile.ability_names.stride)?,
            description: if id < self.profile.ability_description_count {
                self.ptr_text(d)
            } else {
                String::new()
            },
        })
    }
    pub fn nature_changes(&self, id: u8) -> Result<[i8; 5]> {
        if id >= 25 {
            return Err(err("nature_id", id));
        }
        let row = bytes(&self.data, self.profile.nature_effects + id as usize * 5, 5)?;
        let changes = std::array::from_fn(|i| row[i] as i8);
        if changes.iter().any(|v| !(-1..=1).contains(v)) {
            return Err(err("nature_effect", id));
        }
        Ok(changes)
    }
    pub fn nature(&self, id: u8) -> Result<Nature> {
        let stat_changes = self.nature_changes(id)?;
        let address = pointer(&self.data, self.profile.nature_names + id as usize * 4)?;
        Ok(Nature {
            id,
            name: self.cstring(address),
            stat_changes,
        })
    }
    pub(crate) fn region_name(&self, id: usize) -> String {
        if id < self.profile.region_first || id >= self.profile.region_count {
            return String::new();
        }
        self.ptr_text(
            self.profile.regions + (id - self.profile.region_first) * self.profile.region_stride,
        )
    }
    pub fn catalog(&self) -> Result<Catalog> {
        let ultimate = self.profile.save.pokemon_codec == crate::adapter::PokemonCodec::Ultimate55;
        let cfru = self.profile.save.pokemon_codec == crate::adapter::PokemonCodec::Cfru;
        let mut ball_options = Vec::new();
        let candidates = if cfru {
            1..self.profile.items.count as u16
        } else {
            0..self.profile.save.pokemon_codec.fields().ball.max() as u16 + 1
        };
        for candidate in candidates {
            let item_id = if ultimate {
                crate::ultimate::ball_item(self, candidate)?
            } else {
                candidate
            };
            let Ok(item) = self.item(item_id) else {
                continue;
            };
            if !self
                .profile
                .save
                .pockets
                .iter()
                .any(|p| p.id == "balls" && p.category == item.pocket)
            {
                continue;
            }
            let value = if cfru {
                item.item_type as u16
            } else {
                candidate
            };
            if !ball_options.iter().any(|b: &BallOption| b.value == value) {
                ball_options.push(BallOption {
                    value,
                    item: item_id,
                });
            }
        }
        Ok(Catalog {
            natures: (0..25).map(|id| self.nature(id)).collect::<Result<_>>()?,
            type_names: (0..self.profile.type_names.count)
                .map(|id| {
                    self.text(
                        self.profile.type_names.offset + id * self.profile.type_names.stride,
                        self.profile.type_names.stride,
                    )
                })
                .collect::<Result<_>>()?,
            form_families: self.form_families()?,
            battle_forms: self.all_battle_forms()?,
            profile: self.profile,
            editor_rules: EditorRules {
                balls: ball_options.iter().map(|b| b.value).collect(),
                ball_options,
                hyper_training: ultimate,
                pokemon_checksum: !self.profile.save.pokemon_codec.plain_substructures(),
                origin_game_max: self.profile.save.pokemon_codec.fields().origin_game.max() as u8,
                met_level_max: self.profile.save.pokemon_codec.fields().met_level.max() as u8,
                nature_override: ultimate
                    || self
                        .profile
                        .save
                        .pokemon_codec
                        .fields()
                        .nature_override
                        .is_some(),
                contest_ranks: if self.profile.save.pokemon_codec
                    == crate::adapter::PokemonCodec::Rocket21
                {
                    [1, 1, 4, 4, 4]
                } else {
                    [4; 5]
                },
            },
            met_locations: (0..self.profile.region_count)
                .filter_map(|id| {
                    let name = self.region_name(id);
                    (!name.trim().is_empty()).then_some(NamedLocation { id: id as u8, name })
                })
                .collect(),
            species: (1..self.profile.species.count as u16)
                .map(|id| self.species(id))
                .collect::<Result<_>>()?,
            moves: (0..self.profile.moves.count as u16)
                .map(|id| self.move_info(id))
                .collect::<Result<_>>()?,
            items: (0..self.profile.items.count as u16)
                .map(|id| self.item(id))
                .collect::<Result<_>>()?,
            abilities: (0..self.profile.ability_count)
                .map(|id| self.ability(id))
                .collect::<Result<_>>()?,
        })
    }
    pub fn evolutions(&self, id: u16) -> Result<Vec<Evolution>> {
        self.species(id)?;
        let mut result = Vec::new();
        let table = self
            .profile
            .evolution_overrides
            .iter()
            .find(|(species, _)| *species == id)
            .map(|(_, table)| *table)
            .unwrap_or(crate::profile::Table {
                offset: self.profile.evolutions.offset
                    + id as usize * self.profile.evolutions.stride,
                count: self.profile.evolutions.stride / 8,
                stride: 8,
            });
        for i in 0..table.count {
            let o = table.offset + i * table.stride;
            let method =
                if self.profile.formats.evolutions == crate::adapter::EvolutionFormat::Ultimate55 {
                    self.data[o] as u16
                } else {
                    u16(&self.data, o)?
                };
            if method != 0 && !crate::forms::is_battle_method(self.profile.battle_forms, method) {
                result.push(Evolution {
                    method,
                    requirements: if self.profile.formats.evolutions
                        == crate::adapter::EvolutionFormat::Ultimate55
                    {
                        crate::ultimate::evolution_requirements(&self.data, o)?
                    } else {
                        Vec::new()
                    },
                    condition: if self.profile.formats.evolutions
                        == crate::adapter::EvolutionFormat::Ultimate55
                    {
                        crate::ultimate::evolution_condition(method)
                    } else if self.profile.formats.evolutions
                        == crate::adapter::EvolutionFormat::Cfru
                    {
                        cfru_evolution_condition(method)
                    } else {
                        evolution_condition(
                            method,
                            self.profile.formats.evolutions
                                == crate::adapter::EvolutionFormat::Expanded,
                        )
                    },
                    parameter: u16(&self.data, o + 2)?,
                    auxiliary: if self.profile.formats.evolutions
                        == crate::adapter::EvolutionFormat::Cfru
                    {
                        u16(&self.data, o + 6)?
                    } else {
                        0
                    },
                    target: u16(&self.data, o + 4)?,
                    offset: o,
                });
            }
        }
        Ok(result)
    }
    pub fn level_moves(&self, id: u16) -> Result<Vec<LearnSource>> {
        self.species(id)?;
        let mut out = Vec::new();
        use crate::adapter::LearnsetFormat;
        let format = self.profile.formats.learnsets;
        let (stride, bias) = match format {
            LearnsetFormat::Packed9Bit => (2, 1),
            LearnsetFormat::MoveLevel16 => (4, 0),
            LearnsetFormat::Move16Level8 => (3, 0),
        };
        let entry = self.profile.learnsets + (id as usize + bias) * 4;
        if format == LearnsetFormat::Move16Level8 && u32(&self.data, entry)? == 0 {
            return Ok(out);
        }
        let p = pointer(&self.data, entry)?;
        for i in 0..128 {
            let o = p + i * stride;
            let v = u16(&self.data, o)?;
            let (mv, lv) = match format {
                LearnsetFormat::Packed9Bit => (v & 511, v >> 9),
                LearnsetFormat::MoveLevel16 => (v, u16(&self.data, o + 2)?),
                LearnsetFormat::Move16Level8 => (v, bytes(&self.data, o + 2, 1)?[0] as u16),
            };
            if v == 0xffff || (format == LearnsetFormat::Move16Level8 && mv == 0 && lv == 255) {
                return Ok(out);
            }
            if mv == 0
                || mv as usize >= self.profile.moves.count
                || lv
                    > if format == LearnsetFormat::Packed9Bit {
                        100
                    } else {
                        255
                    }
            {
                return Err(err("learnset", format!("species {id}, {o:#x}")));
            }
            out.push(LearnSource {
                move_id: mv,
                source: "level".into(),
                species: id,
                level: Some(lv as u8),
                index: None,
                offset: o,
            });
        }
        Err(err("learnset_terminator", id))
    }
    pub fn ancestors(&self, id: u16) -> Result<BTreeSet<u16>> {
        self.valid_species(id)?;
        let mut ancestors = BTreeSet::from([id]);
        loop {
            let before = ancestors.len();
            for s in 1..self.profile.species.count as u16 {
                if self
                    .evolutions(s)?
                    .iter()
                    .any(|e| ancestors.contains(&e.target))
                {
                    ancestors.insert(s);
                }
            }
            if before == ancestors.len() {
                break;
            }
        }
        Ok(ancestors)
    }
    pub fn learnset(&self, id: u16) -> Result<Vec<LearnSource>> {
        let ancestors = self.ancestors(id)?;
        let mut out = Vec::new();
        for s in &ancestors {
            if let Ok(rows) = self.level_moves(*s) {
                out.extend(rows);
            }
        }
        if !self.profile.capabilities.complete_learnsets {
            return Ok(out);
        }
        let mut current = 0;
        for i in 0..self.profile.teaching.egg_words {
            let o = self.profile.eggs + i * 2;
            let v = u16(&self.data, o)?;
            if v == 0xffff {
                break;
            }
            if v > 20000 && v < 20000 + self.profile.species.count as u16 {
                current = v - 20000;
            } else if v > 0 && v < (self.profile.moves.count as u16) {
                if ancestors.contains(&current) {
                    out.push(LearnSource {
                        move_id: v,
                        source: "egg".into(),
                        species: current,
                        level: None,
                        index: None,
                        offset: o,
                    });
                }
            } else {
                break;
            }
        }
        let teaching = self.profile.teaching;
        if let Some(table) = teaching.shared_lists {
            let entry = table + id as usize * 4;
            // Some alternate species have no compatibility list in the ROM.
            if u32(&self.data, entry)? == 0 {
                return Ok(out);
            }
            let list = pointer(&self.data, entry)?;
            for index in 0..256 {
                let offset = list + index * 2;
                let move_id = u16(&self.data, offset)?;
                if move_id == 0 {
                    return Ok(out);
                }
                self.move_info(move_id)?;
                let mut machine = false;
                for tm in 0..teaching.tm_count {
                    if u16(&self.data, self.profile.tm_moves + tm * 2)? == move_id {
                        machine = true;
                        out.push(LearnSource {
                            move_id,
                            source: "tm".into(),
                            species: id,
                            level: None,
                            index: Some(tm as u16 + 1),
                            offset,
                        });
                    }
                }
                if !machine {
                    out.push(LearnSource {
                        move_id,
                        source: "tutor".into(),
                        species: id,
                        level: None,
                        index: None,
                        offset,
                    });
                }
            }
            return Err(err("learnset_terminator", id));
        }
        for (source, table, bits, stride, count) in [
            (
                "tm",
                self.profile.tm_moves,
                self.profile.tm_bits,
                teaching.tm_stride,
                teaching.tm_count,
            ),
            (
                "tutor",
                self.profile.tutor_moves,
                self.profile.tutor_bits,
                teaching.tutor_stride,
                teaching.tutor_count,
            ),
        ] {
            // Ultimate's native teaching predicate excludes its egg species.
            if self.profile.formats.evolutions == crate::adapter::EvolutionFormat::Ultimate55
                && id == 412
            {
                continue;
            }
            for i in 0..count {
                let o = table + i * 2;
                let v = u16(&self.data, o)?;
                if v == 0 || v as usize >= self.profile.moves.count {
                    break;
                }
                let bit = bytes(&self.data, bits + id as usize * stride + i / 8, 1)?[0];
                if bit & (1 << (i % 8)) != 0 {
                    out.push(LearnSource {
                        move_id: v,
                        source: source.into(),
                        species: id,
                        level: None,
                        index: Some(i as u16 + 1),
                        offset: o,
                    });
                }
            }
        }
        Ok(out)
    }
    pub fn known_moves(&self, id: u16, level: u8) -> Result<HashSet<u16>> {
        Ok(self
            .learnset(id)?
            .iter()
            .filter(|r| r.level.is_none_or(|l| l <= level))
            .map(|r| r.move_id)
            .collect())
    }
    pub fn detail(&self, id: u16) -> Result<SpeciesDetail> {
        Ok(SpeciesDetail {
            relations: self.species_relations(id)?,
            teaching_list_present: match self.profile.teaching.shared_lists {
                Some(table) => u32(&self.data, table + id as usize * 4)? != 0,
                None => true,
            },
            battle_forms: self.battle_forms(id)?,
            encounters_verified: false,
            species: self.valid_species(id)?,
            evolutions: self.evolutions(id)?,
            learnset: self.learnset(id)?,
            encounters: if self.profile.capabilities.world {
                self.encounters()?
            } else {
                Vec::new()
            }
            .into_iter()
            .filter(|e| e.species == id)
            .collect(),
            origins: self.origin_options(id)?,
        })
    }
    /// Candidate origins, not a proof of story reachability or full legality.
    pub fn origin_options(&self, id: u16) -> Result<OriginOptions> {
        if !self.profile.capabilities.world {
            return Ok(OriginOptions {
                ancestors: self.ancestors(id)?.into_iter().collect(),
                encounters: Vec::new(),
                can_hatch: false,
                hatch_regions: Vec::new(),
            });
        }
        let ancestors = self.ancestors(id)?;
        // Babies may belong to the Undiscovered group while their evolved
        // parents can breed. Expand only for breeding eligibility, never for
        // wild encounter locations (which must exclude sibling branches).
        let mut family = ancestors.clone();
        loop {
            let previous = family.len();
            for s in family.clone() {
                for e in self.evolutions(s)? {
                    self.valid_species(e.target)?;
                    family.insert(e.target);
                }
            }
            if previous == family.len() {
                break;
            }
        }
        let can_hatch = family.into_iter().any(|s| {
            self.species(s)
                .is_ok_and(|s| s.egg_groups.iter().all(|g| *g != 0 && *g != 13 && *g != 15))
        });
        Ok(OriginOptions {
            encounters: self
                .encounters()?
                .into_iter()
                .filter(|e| ancestors.contains(&e.species))
                .collect(),
            ancestors: ancestors.into_iter().collect(),
            can_hatch,
            hatch_regions: if can_hatch {
                self.maps()?
                    .into_iter()
                    .map(|m| m.region)
                    .filter(|r| *r < 253)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            },
        })
    }
}
