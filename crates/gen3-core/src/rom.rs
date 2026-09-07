use crate::{
    binary::*,
    err,
    profile::{self, Profile},
    text::Codec,
    Result,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashSet},
    sync::Arc,
};

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
    pub exp_yield: u8,
    pub ev_yield: u16,
    pub items: [u16; 2],
    pub gender_ratio: u8,
    pub egg_cycles: u8,
    pub friendship: u8,
    pub growth: u8,
    pub egg_groups: [u8; 2],
    pub abilities: [u8; 2],
    pub offset: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct Move {
    pub id: u16,
    pub name: String,
    pub description: String,
    pub effect: u8,
    pub power: u8,
    pub move_type: u8,
    pub accuracy: u8,
    pub pp: u8,
    pub chance: u8,
    pub target: u8,
    pub priority: i8,
    pub flags: u8,
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
pub struct Evolution {
    pub method: u16,
    pub parameter: u16,
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
pub struct Catalog {
    pub profile: Profile,
    pub species: Vec<Species>,
    pub moves: Vec<Move>,
    pub items: Vec<Item>,
    pub abilities: Vec<Ability>,
    pub met_locations: Vec<NamedLocation>,
}
#[derive(Serialize)]
pub struct SpeciesDetail {
    pub species: Species,
    pub evolutions: Vec<Evolution>,
    pub learnset: Vec<LearnSource>,
    pub encounters: Vec<crate::world::Encounter>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct RomEdit {
    pub table: String,
    pub id: u16,
    pub field: String,
    pub value: u32,
}
#[derive(Serialize)]
pub struct PatchManifest {
    pub schema: u8,
    pub profile: String,
    pub base_md5: String,
    pub output_md5: String,
    pub base_sha256: String,
    pub output_sha256: String,
    pub edits: Vec<RomEdit>,
}

impl Rom {
    pub fn is_mail(&self, id: u16) -> bool {
        (self.profile.mail_items[0]..=self.profile.mail_items[1]).contains(&id)
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
            profile: profile::BW,
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
        let b = bytes(&self.data, o, 28)?;
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
            exp_yield: b[9],
            ev_yield: u16(b, 10)?,
            items: [u16(b, 12)?, u16(b, 14)?],
            gender_ratio: b[16],
            egg_cycles: b[17],
            friendship: b[18],
            growth: b[19],
            egg_groups: [b[20], b[21]],
            abilities: [b[22], b[23]],
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
        let b = bytes(&self.data, o, 12)?;
        let no = self.profile.move_names.at(id as usize);
        Ok(Move {
            id,
            name: self.text(no, 13)?,
            description: if id > 0 {
                self.ptr_text(self.profile.move_descriptions + (id as usize - 1) * 4)
            } else {
                String::new()
            },
            effect: b[0],
            power: b[1],
            move_type: b[2],
            accuracy: b[3],
            pp: b[4],
            chance: b[5],
            target: b[6],
            priority: b[7] as i8,
            flags: b[8],
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
            tm_move: if (0x121..0x121 + 58).contains(&id) {
                Some(u16(
                    &self.data,
                    self.profile.tm_moves + (id as usize - 0x121) * 2,
                )?)
            } else {
                None
            },
            offset: o,
        })
    }
    pub fn ability(&self, id: u16) -> Result<Ability> {
        if id >= 151 {
            return Err(err("ability_id", id));
        }
        let (n, d) = (
            self.profile.ability_names.at(id as usize),
            self.profile.ability_descriptions.at(id as usize),
        );
        Ok(Ability {
            id,
            name: self.text(n, 13)?,
            description: self.ptr_text(d),
        })
    }
    pub fn catalog(&self) -> Result<Catalog> {
        Ok(Catalog {
            profile: self.profile,
            met_locations: (0..self.profile.region_count)
                .filter_map(|id| {
                    let name = self.ptr_text(self.profile.regions + id * 8);
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
            abilities: (0..151).map(|id| self.ability(id)).collect::<Result<_>>()?,
        })
    }
    pub fn evolutions(&self, id: u16) -> Result<Vec<Evolution>> {
        self.species(id)?;
        let mut result = Vec::new();
        for i in 0..5 {
            let o = self.profile.evolutions.offset
                + id as usize * self.profile.evolutions.stride
                + i * 8;
            let method = u16(&self.data, o)?;
            if method != 0 {
                result.push(Evolution {
                    method,
                    parameter: u16(&self.data, o + 2)?,
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
        let p = pointer(&self.data, self.profile.learnsets + (id as usize + 1) * 4)?;
        for i in 0..128 {
            let o = p + i * 2;
            let v = u16(&self.data, o)?;
            if v == 0xffff {
                return Ok(out);
            }
            let mv = v & 511;
            let lv = (v >> 9) as u8;
            if mv == 0 || mv as usize >= self.profile.moves.count || lv > 100 {
                return Err(err("learnset", format!("species {id}, {o:#x}")));
            }
            out.push(LearnSource {
                move_id: mv,
                source: "level".into(),
                species: id,
                level: Some(lv),
                index: None,
                offset: o,
            });
        }
        Err(err("learnset_terminator", id))
    }
    pub fn learnset(&self, id: u16) -> Result<Vec<LearnSource>> {
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
        let mut out = Vec::new();
        for s in &ancestors {
            if let Ok(rows) = self.level_moves(*s) {
                out.extend(rows);
            }
        }
        let mut current = 0;
        for i in 0..4096 {
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
        for (source, table, bits, stride, count) in [
            ("tm", self.profile.tm_moves, self.profile.tm_bits, 8, 58),
            (
                "tutor",
                self.profile.tutor_moves,
                self.profile.tutor_bits,
                4,
                32,
            ),
        ] {
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
            species: self.valid_species(id)?,
            evolutions: self.evolutions(id)?,
            learnset: self.learnset(id)?,
            encounters: self
                .encounters()?
                .into_iter()
                .filter(|e| e.species == id)
                .collect(),
        })
    }
    /// Only fixed-width scalar fields are writable. No arbitrary offset escape hatch.
    pub fn patch(&self, edits: &[RomEdit]) -> Result<(Vec<u8>, PatchManifest)> {
        let mut data = (*self.data).clone();
        for edit in edits {
            let (base, field, width, max) = match edit.table.as_str() {
                "species" => {
                    let s = self.valid_species(edit.id)?;
                    let (off, max) = match edit.field.as_str() {
                        "hp" => (0, 255),
                        "attack" => (1, 255),
                        "defense" => (2, 255),
                        "speed" => (3, 255),
                        "sp_attack" => (4, 255),
                        "sp_defense" => (5, 255),
                        "type1" => (6, 17),
                        "type2" => (7, 17),
                        "catch_rate" => (8, 255),
                        "gender_ratio" => (16, 255),
                        "egg_cycles" => (17, 255),
                        "friendship" => (18, 255),
                        "growth" => (19, 5),
                        "ability1" => (22, 150),
                        "ability2" => (23, 150),
                        _ => return Err(err("rom_field", &edit.field)),
                    };
                    (s.offset, off, 1, max)
                }
                "moves" => {
                    let m = self.move_info(edit.id)?;
                    let (off, max) = match edit.field.as_str() {
                        "power" => (1, 255),
                        "type" => (2, 17),
                        "accuracy" => (3, 100),
                        "pp" => (4, 99),
                        "chance" => (5, 100),
                        _ => return Err(err("rom_field", &edit.field)),
                    };
                    (m.offset, off, 1, max)
                }
                "items" => {
                    let i = self.item(edit.id)?;
                    if edit.field != "price" {
                        return Err(err("rom_field", &edit.field));
                    }
                    (i.offset, 16, 2, 65535)
                }
                _ => return Err(err("rom_table", &edit.table)),
            };
            if edit.value > max {
                return Err(err("range", &edit.field));
            }
            if width == 1 {
                data[base + field] = edit.value as u8;
            } else {
                put16(&mut data, base + field, edit.value as u16);
            }
        }
        let manifest = PatchManifest {
            schema: 1,
            profile: self.profile.id.into(),
            base_md5: hash(&self.data),
            output_md5: hash(&data),
            base_sha256: sha256(&self.data),
            output_sha256: sha256(&data),
            edits: edits.to_vec(),
        };
        Ok((data, manifest))
    }
}
