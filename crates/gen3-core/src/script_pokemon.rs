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
    pub trade: Option<TradeRules>,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct TradeRules {
    pub specials: usize,
    pub information_special: u16,
    pub information_code: usize,
    /// Literal pool used by the native information routine, read at runtime.
    pub table_pointer: usize,
    pub count: u16,
    /// The ordinary constructor is bypassed when this native flag is set.
    pub alternate_flag: Option<u16>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TradeOffer {
    pub index: u16,
    pub record_offset: usize,
    pub requested_species: u16,
    pub level_rule: &'static str,
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
    pub trade: Option<TradeOffer>,
}
impl Rom {
    pub(crate) fn trade_offer(&self, index: u16) -> Result<(u16, u16, TradeOffer)> {
        let rule = self
            .profile
            .script_pokemon
            .trade
            .ok_or_else(|| crate::err("trade_unverified", index))?;
        if index >= rule.count {
            return Err(crate::err("trade_index", index));
        }
        if pointer(
            &self.data,
            rule.specials + rule.information_special as usize * 4,
        )? & !1
            != rule.information_code
        {
            return Err(crate::err(
                "trade_dispatch",
                "native information target changed",
            ));
        }
        let record_offset = pointer(&self.data, rule.table_pointer)? + index as usize * 60;
        bytes(&self.data, record_offset, 60)?;
        let received = u16(&self.data, record_offset + 12)?;
        let requested_species = u16(&self.data, record_offset + 56)?;
        self.valid_species(received)?;
        self.valid_species(requested_species)?;
        let held = u16(&self.data, record_offset + 40)?;
        self.item(held)?;
        Ok((
            received,
            held,
            TradeOffer {
                index,
                record_offset,
                requested_species,
                level_rule: "offered_pokemon",
            },
        ))
    }
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
                        trade: None,
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
            0x25 | 0x26 => {
                let special = u16(b, pc + if op == 0x26 { 3 } else { 1 })?;
                if let Some(rule) = self.profile.script_pokemon.native_battle {
                    if op == 0x25 && special == rule.special {
                        add(
                            resolve(rule.species_var),
                            resolve(rule.level_var).and_then(|v| u8::try_from(v).ok()),
                            None,
                            "special_battle",
                            0,
                        );
                    }
                }
                if let Some(rule) = self
                    .profile
                    .script_pokemon
                    .trade
                    .filter(|r| r.information_special == special)
                {
                    if let Some(index) = resolve(0x8004) {
                        let (species, held, trade) = self.trade_offer(index)?;
                        let conditions = rule
                            .alternate_flag
                            .map(|id| EventCondition {
                                kind: "flag",
                                id,
                                value: 1,
                                comparison: 1,
                                taken: false,
                            })
                            .into_iter()
                            .collect();
                        sources.push(PokemonSource {
                            species,
                            level: None,
                            held_item: Some(held),
                            method: "npc_trade",
                            offset: pc,
                            member: 0,
                            conditions,
                            trade: Some(trade),
                        });
                    }
                }
            }
            _ => {}
        }
        Ok(sources)
    }
}
