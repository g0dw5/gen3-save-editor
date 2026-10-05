//! Runtime script operands, separate from delivery/receipt and battle outcomes.
use crate::{binary::*, map_events::EventCondition, rom::Rom, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WildCommand {
    Literal,
    RocketExtended,
    MercuryDouble,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct NativeBattleCommand {
    pub special: u16,
    pub species_var: u16,
    pub level_var: u16,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PokemonScriptRules {
    pub wild: WildCommand,
    /// Thumb MOVS r2, #level executed by this ROM's gift-egg constructor.
    pub egg_level_instruction: usize,
    pub native_battle: Option<NativeBattleCommand>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PokemonSource {
    pub species: u16,
    pub level: Option<u8>,
    pub held_item: Option<u16>,
    pub method: &'static str,
    pub offset: usize,
    /// Individual members of a multi-Pokémon wild battle remain distinct.
    pub member: u8,
    pub conditions: Vec<EventCondition>,
}
impl Rom {
    pub(crate) fn wild_command_length(&self, pc: usize) -> Result<usize> {
        Ok(match self.profile.script_pokemon.wild {
            WildCommand::Literal => 6,
            WildCommand::RocketExtended => 11,
            WildCommand::MercuryDouble if u16(&self.data, pc + 1)? == 0xffff => 18,
            WildCommand::MercuryDouble => 6,
        })
    }
    pub(crate) fn gift_egg_level(&self) -> Result<u8> {
        let instruction = u16(
            &self.data,
            self.profile.script_pokemon.egg_level_instruction,
        )?;
        if instruction & 0xff00 != 0x2200 {
            return Err(crate::err(
                "script_egg_level",
                "expected native MOVS r2, #level",
            ));
        }
        let level = instruction as u8;
        if level == 0 || level > self.profile.max_level {
            return Err(crate::err("script_egg_level", level));
        }
        Ok(level)
    }
    pub(crate) fn script_pokemon_instruction(
        &self,
        pc: usize,
        resolve: impl Fn(u16) -> Option<u16>,
    ) -> Result<Vec<PokemonSource>> {
        let b = &self.data;
        let op = bytes(b, pc, 1)?[0];
        let mut sources = Vec::new();
        let mut add =
            |species: Option<u16>, level: Option<u8>, held_item: Option<u16>, method, member| {
                if let Some(species) = species.filter(|s| self.valid_species(*s).is_ok()) {
                    sources.push(PokemonSource {
                        species,
                        level: level.filter(|l| *l > 0 && *l <= self.profile.max_level),
                        held_item: held_item.filter(|i| self.item(*i).is_ok()),
                        method,
                        offset: pc,
                        member,
                        conditions: vec![],
                    });
                }
            };
        match op {
            0x79 => add(
                resolve(u16(b, pc + 1)?),
                Some(bytes(b, pc + 3, 1)?[0]),
                resolve(u16(b, pc + 4)?),
                "gift",
                0,
            ),
            0x7a => add(
                resolve(u16(b, pc + 1)?),
                Some(self.gift_egg_level()?),
                None,
                "egg",
                0,
            ),
            0xb6 => {
                match self.profile.script_pokemon.wild {
                    WildCommand::MercuryDouble if u16(b, pc + 1)? == 0xffff => {
                        for (member, offset) in [(0, 7), (1, 13)] {
                            add(
                                resolve(u16(b, pc + offset)?),
                                Some(bytes(b, pc + offset + 2, 1)?[0]),
                                Some(u16(b, pc + offset + 3)?),
                                "static",
                                member,
                            );
                        }
                    }
                    WildCommand::MercuryDouble => add(
                        resolve(u16(b, pc + 1)?),
                        Some(bytes(b, pc + 3, 1)?[0]),
                        Some(u16(b, pc + 4)?),
                        "static",
                        0,
                    ),
                    // These native handlers read literal halfwords, not VarGet.
                    WildCommand::Literal | WildCommand::RocketExtended => add(
                        Some(u16(b, pc + 1)?),
                        Some(bytes(b, pc + 3, 1)?[0]),
                        Some(u16(b, pc + 4)?),
                        "static",
                        0,
                    ),
                }
            }
            0x25 => {
                if let Some(rule) = self.profile.script_pokemon.native_battle {
                    if u16(b, pc + 1)? == rule.special {
                        add(
                            resolve(rule.species_var),
                            resolve(rule.level_var).and_then(|v| u8::try_from(v).ok()),
                            None,
                            "special_battle",
                            0,
                        );
                    }
                }
            }
            _ => {}
        }
        Ok(sources)
    }
}
