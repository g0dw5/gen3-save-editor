//! Read-only, level-gated field effects of referenced NPC training services.
//! Holdings/unlock checks are separate from hypothetical field execution. This
//! is not a complete NPC transaction, and never spends items or edits a SAV.
use crate::{
    acquisition::ConditionCheck, binary::*, err, native_trainer::Sandbox, pokemon, rom::Rom,
    save::Save, training::Individual, training_services::ServiceRules, Result,
};
use armv4t_emu::Memory;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub expected_rom_md5: String,
    pub service_root: usize,
    pub choice_index: u8,
    pub individual: Individual,
}
#[derive(Serialize)]
pub struct Preview {
    pub rom_md5: &'static str,
    pub service_root: usize,
    pub choice_index: u8,
    pub kind: &'static str,
    pub before: pokemon::Pokemon,
    pub after: pokemon::Pokemon,
    pub party_stats_before: [u16; 6],
    pub party_stats_after: [u16; 6],
    pub native_level: u16,
    pub minimum_level: u16,
    pub level_satisfied: bool,
    /// Only the checks parsed for this reference, not all menu qualifications.
    pub known_requirements_met: Option<bool>,
    pub conditions: Vec<ConditionCheck>,
    pub changed: bool,
    pub stat_refresh: &'static str,
    pub scenario: &'static str,
    pub party_state: &'static str,
    /// A box record is hypothetical here: the native service selects party only.
    pub withdrawal_required: bool,
    pub effect_scope: &'static str,
    pub partial: bool,
    #[cfg(test)]
    #[serde(skip)]
    pub(crate) raw: Vec<u8>,
}
impl Rom {
    pub fn training_service_preview(
        &self,
        save: Option<&Save>,
        request: Request,
    ) -> Result<Preview> {
        if request.expected_rom_md5 != self.profile.md5 {
            return Err(err("rom_mismatch", "training service scenario"));
        }
        let report = self.training_services(save)?;
        let service = report
            .services
            .into_iter()
            .find(|s| match s.evidence {
                ServiceRules::Flags(r) => r.root == request.service_root,
                ServiceRules::BaseIvs(r) => r.root == request.service_root,
            })
            .ok_or_else(|| err("training_service_unverified", request.service_root))?;
        let choice = service
            .choices
            .iter()
            .find(|c| c.menu_index == request.choice_index)
            .ok_or_else(|| err("training_service_choice", request.choice_index))?;
        let (raw, scenario, party_state) =
            self.prepare_training_individual(save, request.individual)?;
        let before = pokemon::decode(&raw, self)?;
        let b = self
            .profile
            .breeding
            .ok_or_else(|| err("training_context_unverified", self.profile.id))?;
        let mut ram = Sandbox::new(&self.data);
        for (i, byte) in raw.iter().enumerate() {
            ram.w8(b.party + i as u32, *byte);
        }
        ram.w8(b.party_count, 1);
        // Only the verified individual/selection helper stage executes. Full
        // story, bag, menu and post-service callbacks are explicitly excluded.
        let native_level = match service.evidence {
            ServiceRules::Flags(rule) => {
                ram.w16(rule.selected_individual, 0);
                ram.w16(rule.selected_individual + 2, 84);
                ram.call(rule.level_reader, [0; 4], [0; 2], 100_000)?;
                ram.r16(rule.selected_individual + 2)
            }
            ServiceRules::BaseIvs(rule) => {
                ram.w16(rule.selected_individual, 0);
                ram.call(rule.level_reader, [0; 4], [0; 2], 100_000)?;
                ram.r16(rule.selected_individual + 4)
            }
        };
        let level_satisfied = native_level >= service.minimum_level;
        let mut expected = raw.clone();
        let stat_refresh = match service.evidence {
            ServiceRules::Flags(rule) => {
                if level_satisfied {
                    ram.w16(
                        rule.selected_individual + 2,
                        if choice.menu_index == 0 {
                            choice.mask as u16
                        } else {
                            1
                        },
                    );
                    if choice.menu_index != 0 {
                        ram.w8(rule.selected_choice, choice.menu_index);
                        ram.call(rule.selection_mask, [0; 4], [0; 2], 100_000)?;
                    }
                    if ram.r16(rule.selected_individual + 2) != choice.mask as u16 {
                        return Err(err("training_service_mask", "native selection differs"));
                    }
                    ram.call(rule.mark, [0; 4], [0; 2], 100_000)?;
                    expected[30] |= choice.mask;
                }
                "deferred"
            }
            ServiceRules::BaseIvs(rule) => {
                if level_satisfied {
                    ram.w16(rule.selected_individual - 2, 0);
                    for stat in 0..6 {
                        if choice.stat.is_none_or(|s| s == stat) {
                            ram.w16(rule.selected_individual + 2, stat as u16);
                            ram.w16(rule.selected_individual + 4, 31);
                            ram.call(rule.setter, [0; 4], [0; 2], 250_000)?;
                            let bits = u32(&expected, 72)?;
                            put32(
                                &mut expected,
                                72,
                                (bits & !(31 << (stat * 5))) | (31 << (stat * 5)),
                            );
                        }
                    }
                    // Native calculation may update level/HP/stats. Everything
                    // else must remain exact, including the IV word's other bits.
                    expected[84] = ram.r8(b.party + 84);
                    for i in 86..100 {
                        expected[i] = ram.r8(b.party + i as u32);
                    }
                }
                "immediate"
            }
        };
        let after_raw: Vec<u8> = (0..100).map(|i| ram.r8(b.party + i)).collect();
        if after_raw != expected {
            return Err(err(
                "training_native_unrelated",
                "unexpected service field change",
            ));
        }
        let after = pokemon::decode(&after_raw, self)?;
        let mut conditions = service.conditions;
        if let Some(credit) = &choice.credit {
            conditions.push(credit.clone());
        }
        conditions.push(choice.item_requirement.clone());
        let known_requirements_met =
            if !level_satisfied || conditions.iter().any(|c| c.satisfied == Some(false)) {
                Some(false)
            } else if conditions.iter().any(|c| c.satisfied.is_none()) {
                None
            } else {
                Some(true)
            };
        Ok(Preview {
            rom_md5: self.profile.md5,
            service_root: request.service_root,
            choice_index: request.choice_index,
            kind: service.kind,
            before,
            after,
            party_stats_before: std::array::from_fn(|i| u16(&raw, 88 + i * 2).unwrap()),
            party_stats_after: std::array::from_fn(|i| u16(&after_raw, 88 + i * 2).unwrap()),
            native_level,
            minimum_level: service.minimum_level,
            level_satisfied,
            known_requirements_met,
            conditions,
            changed: raw != after_raw,
            stat_refresh,
            scenario,
            party_state,
            withdrawal_required: party_state == "boxed_full_hp_scenario",
            effect_scope: "level_gated_individual_helpers",
            partial: true,
            #[cfg(test)]
            raw: after_raw,
        })
    }
}
