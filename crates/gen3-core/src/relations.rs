//! Read-only relation graphs. Name similarities never become legal ancestry.
use crate::{
    binary::{pointer, u16, u32},
    err,
    forms::BattleForm,
    rom::{Evolution, Rom},
    Result,
};
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};

#[derive(Serialize)]
pub struct EvolutionEdge {
    pub source: u16,
    #[serde(flatten)]
    pub evolution: Evolution,
}
#[derive(Serialize)]
pub struct FormFamily {
    pub species: Vec<u16>,
    pub offset: usize,
}
#[derive(Serialize)]
pub struct NameRelation {
    pub source: u16,
    pub target: u16,
}
#[derive(Serialize)]
pub struct SpeciesRelations {
    pub species: Vec<u16>,
    pub evolutions: Vec<EvolutionEdge>,
    pub battle_forms: Vec<BattleForm>,
    pub form_families: Vec<FormFamily>,
    pub name_relations: Vec<NameRelation>,
}

impl Rom {
    pub fn form_families(&self) -> Result<Vec<FormFamily>> {
        let Some(table) = self.profile.form_families else {
            return Ok(Vec::new());
        };
        let mut seen = HashSet::new();
        let mut families = Vec::new();
        for species in 1..self.profile.species.count {
            let entry = table + species * 4;
            if u32(&self.data, entry)? == 0 {
                continue;
            }
            let offset = pointer(&self.data, entry)?;
            if !seen.insert(offset) {
                continue;
            }
            let mut members = Vec::new();
            let mut terminated = false;
            for index in 0..self.profile.species.count {
                let id = u16(&self.data, offset + index * 2)?;
                if id == 0xffff {
                    terminated = true;
                    break;
                }
                self.valid_species(id)?;
                if !members.contains(&id) {
                    members.push(id);
                }
            }
            if !terminated {
                return Err(err("form_terminator", offset));
            }
            if members.len() > 1 {
                families.push(FormFamily {
                    species: members,
                    offset,
                });
            }
        }
        Ok(families)
    }

    /// Entire connected family, including incoming edges and sibling branches.
    /// Only the reference viewer consumes name links; origin validation does not.
    pub fn species_relations(&self, id: u16) -> Result<SpeciesRelations> {
        self.valid_species(id)?;
        let species = (1..self.profile.species.count as u16)
            .map(|id| self.species(id))
            .collect::<Result<Vec<_>>>()?;
        let mut evolutions = Vec::new();
        let battle_forms = self.all_battle_forms()?;
        for row in &species {
            for evolution in self.evolutions(row.id)? {
                if evolution.target != 0 && (evolution.target as usize) < self.profile.species.count
                {
                    evolutions.push(EvolutionEdge {
                        source: row.id,
                        evolution,
                    });
                }
            }
        }
        let form_families = self.form_families()?;
        let mut name_relations = Vec::new();
        for row in &species {
            let stem = row.name.trim_end_matches(|c: char| {
                c.is_ascii_uppercase() || matches!(c, '&' | '♀' | '♂' | '-' | ' ')
            });
            if stem.len() < 3 || stem == row.name {
                continue;
            }
            for base in &species {
                if base.name == stem
                    && base.id != row.id
                    && !battle_forms.iter().any(|form| form.target == base.id)
                    && !evolutions
                        .iter()
                        .any(|edge| edge.source == base.id && edge.evolution.target == row.id)
                    && !battle_forms
                        .iter()
                        .any(|form| form.source == base.id && form.target == row.id)
                {
                    name_relations.push(NameRelation {
                        source: base.id,
                        target: row.id,
                    });
                }
            }
        }
        let mut connected = BTreeSet::from([id]);
        loop {
            let count = connected.len();
            for (source, target) in evolutions
                .iter()
                .map(|e| (e.source, e.evolution.target))
                .chain(battle_forms.iter().map(|e| (e.source, e.target)))
                .chain(name_relations.iter().map(|e| (e.source, e.target)))
            {
                if connected.contains(&source) || connected.contains(&target) {
                    connected.extend([source, target]);
                }
            }
            for family in &form_families {
                if family.species.iter().any(|s| connected.contains(s)) {
                    connected.extend(&family.species);
                }
            }
            if count == connected.len() {
                break;
            }
        }
        Ok(SpeciesRelations {
            evolutions: evolutions
                .into_iter()
                .filter(|e| connected.contains(&e.source))
                .collect(),
            battle_forms: battle_forms
                .into_iter()
                .filter(|e| connected.contains(&e.source))
                .collect(),
            form_families: form_families
                .into_iter()
                .filter(|f| f.species.iter().any(|s| connected.contains(s)))
                .collect(),
            name_relations: name_relations
                .into_iter()
                .filter(|e| connected.contains(&e.source))
                .collect(),
            species: connected.into_iter().collect(),
        })
    }
}
