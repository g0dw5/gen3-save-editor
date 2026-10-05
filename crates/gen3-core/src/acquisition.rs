//! Runtime-only acquisition index. All content comes from the loaded ROM.
use crate::{
    err,
    event_state::EventSnapshot,
    map_events::EventCondition,
    rom::{Evolution, LearnSource, Rom, Species},
    save::Save,
    world::World,
    Result,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Species,
    Item,
    Move,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub kind: TargetKind,
    pub id: u16,
}
/// Cross-links from already decoded evolution semantics, never a name-based guess.
pub(crate) fn evolution_targets(e: &Evolution) -> Vec<Target> {
    let mut targets = vec![];
    let mut add = |kind, id| {
        if !targets
            .iter()
            .any(|t: &Target| t.kind == kind && t.id == id)
        {
            targets.push(Target { kind, id });
        }
    };
    if matches!(
        e.condition,
        "item"
            | "trade_item"
            | "held_day"
            | "held_night"
            | "male_item"
            | "female_item"
            | "item_night"
            | "item_location"
            | "item_hold_item"
    ) {
        add(TargetKind::Item, e.parameter);
    }
    if matches!(e.condition, "level_hold_item" | "item_hold_item") {
        add(TargetKind::Item, e.auxiliary);
    }
    if matches!(e.condition, "move" | "move_male" | "move_female") {
        add(TargetKind::Move, e.parameter);
    }
    if matches!(e.condition, "party_species" | "trade_species") {
        add(TargetKind::Species, e.parameter);
    }
    for r in &e.requirements {
        match r.kind {
            "item" | "held_item" => add(TargetKind::Item, r.value),
            "move" => add(TargetKind::Move, r.value),
            _ => {}
        }
    }
    targets
}
#[derive(Clone, Serialize)]
pub struct ConditionCheck {
    pub condition: EventCondition,
    pub satisfied: Option<bool>,
    pub actual: Option<u32>,
    pub unresolved: Option<&'static str>,
}
#[derive(Clone, Serialize)]
pub struct TradeContext {
    pub party_levels: Vec<u8>,
    pub box_levels: Vec<u8>,
}
impl TradeContext {
    fn read(save: &Save, rom: &Rom, requested: u16) -> Result<Self> {
        let mut context = Self {
            party_levels: vec![],
            box_levels: vec![],
        };
        for stored in save
            .all(rom)?
            .into_iter()
            .filter(|p| p.pokemon.species == requested && !p.pokemon.egg)
        {
            match stored.location {
                crate::save::Location::Party { .. } => {
                    context.party_levels.push(stored.pokemon.level)
                }
                crate::save::Location::Box { .. } => context.box_levels.push(stored.pokemon.level),
            }
        }
        context.party_levels.sort_unstable();
        context.box_levels.sort_unstable();
        Ok(context)
    }
}
#[derive(Clone, Serialize)]
pub struct AcquisitionSource {
    pub kind: String,
    pub map_id: Option<String>,
    pub region: Option<u8>,
    pub x: Option<i16>,
    pub y: Option<i16>,
    pub underfoot: Option<bool>,
    pub related: Vec<Target>,
    pub quantity: Option<u16>,
    pub min_level: Option<u8>,
    pub max_level: Option<u8>,
    /// Slot selection conditional on this encounter method, not held-item chance.
    pub encounter_percent: Option<u8>,
    /// Unknown until the exact native held-item selection routine is verified.
    pub held_percent: Option<f64>,
    pub held_context: Option<crate::wild_items::HeldContext>,
    pub held_issue: Option<String>,
    pub encounter_method: Option<String>,
    pub periods: Vec<String>,
    pub conditions: Vec<ConditionCheck>,
    pub requirements: Vec<crate::rom::EvolutionRequirement>,
    pub evolution: Option<Evolution>,
    pub status: &'static str,
    pub receipt_flag: Option<u16>,
    pub receipt: Option<crate::map_events::ReceiptEvidence>,
    pub script_source: Option<crate::script_pokemon::PokemonSource>,
    pub trade_context: Option<TradeContext>,
    pub teaching_source: Option<crate::script_teaching::TeachingSource>,
    pub repeatable: Option<bool>,
    pub offset: usize,
    pub partial: bool,
    pub in_scenario: Option<bool>,
}
#[derive(Serialize)]
pub struct AcquisitionReport {
    pub clock: Option<crate::clock::ClockReport>,
    pub target: Target,
    pub sources: Vec<AcquisitionSource>,
    pub partial: bool,
}
#[derive(Default)]
pub(crate) struct WildCache {
    data: Option<Arc<Vec<u8>>>,
    values: BTreeMap<(u16, u16, Vec<u8>), crate::wild_items::HeldDistribution>,
}
pub struct AcquisitionIndex {
    pub(crate) wild_cache: RefCell<WildCache>,
    pub world: World,
    pub species: Vec<Species>,
    pub evolutions: BTreeMap<u16, Vec<Evolution>>,
    pub learnsets: BTreeMap<u16, Vec<LearnSource>>,
}
fn source(kind: &str, offset: usize) -> AcquisitionSource {
    AcquisitionSource {
        kind: kind.into(),
        map_id: None,
        region: None,
        x: None,
        y: None,
        underfoot: None,
        related: vec![],
        quantity: None,
        min_level: None,
        max_level: None,
        encounter_percent: None,
        held_percent: None,
        held_context: None,
        held_issue: None,
        encounter_method: None,
        periods: vec![],
        conditions: vec![],
        requirements: vec![],
        evolution: None,
        status: "unknown",
        receipt_flag: None,
        receipt: None,
        script_source: None,
        trade_context: None,
        teaching_source: None,
        repeatable: None,
        offset,
        partial: true,
        in_scenario: None,
    }
}
fn check(
    state: Option<&EventSnapshot>,
    rom: Option<&Rom>,
    condition: &EventCondition,
) -> ConditionCheck {
    // Neither ROM-only queries nor SAV overlays can infer unsaved native context.
    let unresolved = if matches!(condition.kind, "bag_item_runtime" | "money_runtime") {
        Some("script_changes_resource")
    } else if condition.kind == "bag_item"
        && rom
            .and_then(|r| r.profile.resource_checks)
            .is_some_and(|r| r.alternate_bag)
    {
        Some("alternate_bag_unresolved")
    } else {
        None
    };
    let mut present = None;
    let actual = state.and_then(|s| match condition.kind {
        "flag" => s.flag(condition.id).map(u32::from),
        "variable" => s.variable(condition.id).map(u32::from),
        "money" => s.money(),
        "bag_item" => {
            let rom = rom?;
            let (quantity, found) = s.normal_bag_item(rom, condition.id)?;
            present = Some(found);
            Some(quantity)
        }
        "bag_item_runtime" | "money_runtime" => None,
        _ => None,
    });
    let satisfied = if unresolved.is_some() {
        None
    } else {
        actual.and_then(|v| {
            Some(
                (match condition.comparison {
                    0 => v < condition.value,
                    1 => v == condition.value,
                    2 => v > condition.value,
                    3 => v <= condition.value,
                    4 => v >= condition.value,
                    5 => v != condition.value,
                    _ => return None,
                } && present != Some(false))
                    == condition.taken,
            )
        })
    };
    ConditionCheck {
        condition: condition.clone(),
        actual,
        satisfied,
        unresolved,
    }
}

fn scripted_source(
    rom: &Rom,
    save: Option<&Save>,
    state: Option<&EventSnapshot>,
    map: &crate::world::Map,
    marker: Option<&crate::map_events::MapMarker>,
    mon: &crate::script_pokemon::PokemonSource,
) -> Result<AcquisitionSource> {
    let mut s = source(mon.method, mon.offset);
    s.map_id = Some(map.id.clone());
    s.region = Some(map.region);
    s.x = marker.map(|m| m.x);
    s.y = marker.map(|m| m.y);
    s.min_level = mon.level;
    s.max_level = mon.level;
    s.conditions = mon
        .conditions
        .iter()
        .map(|c| check(state, Some(rom), c))
        .collect();
    s.script_source = Some(mon.clone());
    if let Some(item) = mon.held_item.filter(|i| *i != 0) {
        s.related.push(Target {
            kind: TargetKind::Item,
            id: item,
        });
    }
    if let Some(trade) = &mon.trade {
        s.related.push(Target {
            kind: TargetKind::Species,
            id: trade.requested_species,
        });
        s.trade_context = save
            .map(|save| TradeContext::read(save, rom, trade.requested_species))
            .transpose()?;
        if s.trade_context
            .as_ref()
            .is_some_and(|c| c.party_levels.is_empty())
        {
            s.status = "blocked";
        }
    }
    if s.conditions.iter().any(|c| c.satisfied == Some(false)) {
        s.status = "blocked";
    }
    // These are native command inputs, not a receipt, current access,
    // or a complete generated individual. Visibility never proves receipt.
    s.partial = true;
    Ok(s)
}
impl AcquisitionIndex {
    fn wild_distribution(
        &self,
        rom: &Rom,
        species: u16,
        layout: u16,
        lead: Option<&[u8]>,
    ) -> Result<crate::wild_items::HeldDistribution> {
        let key = (
            species,
            rom.held_layout(layout)?,
            lead.unwrap_or(&[]).to_vec(),
        );
        let mut cache = self.wild_cache.borrow_mut();
        if cache
            .data
            .as_ref()
            .is_none_or(|data| !Arc::ptr_eq(data, &rom.data))
        {
            cache.values.clear();
            cache.data = Some(rom.data.clone());
        }
        if let Some(distribution) = cache.values.get(&key) {
            return Ok(distribution.clone());
        }
        let distribution = rom.wild_item_distribution(species, layout, lead)?;
        // Bounded runtime cache; changing either ROM bytes or leading individual
        // changes the key. No cached names, artwork or results become release data.
        if cache.values.len() >= 512 {
            cache.values.clear();
        }
        cache.values.insert(key, distribution.clone());
        Ok(distribution)
    }
    fn encounter_source(
        &self,
        rom: &Rom,
        state: Option<&EventSnapshot>,
        e: &crate::world::Encounter,
    ) -> AcquisitionSource {
        let mut s = source(&e.method, e.offset);
        s.map_id = Some(e.map_id.clone());
        s.region = Some(e.region);
        s.min_level = Some(e.min_level);
        s.max_level = Some(e.max_level);
        s.encounter_percent = e.weight;
        s.periods = e.periods.iter().map(|v| v.to_string()).collect();
        if let Some(selector) = &e.selector {
            s.conditions.push(check(
                state,
                Some(rom),
                &EventCondition {
                    kind: "variable",
                    id: selector.variable,
                    value: selector.value as u32,
                    comparison: if selector.fallback { 5 } else { 1 },
                    taken: true,
                },
            ));
        }
        s.repeatable = if matches!(
            e.method.as_str(),
            "grass"
                | "cave"
                | "surf"
                | "dive"
                | "old_rod"
                | "good_rod"
                | "super_rod"
                | "rock_smash"
        ) {
            Some(true)
        } else {
            None
        };
        s.partial = e.conditional || !e.periods.is_empty();
        if s.conditions.iter().any(|c| c.satisfied == Some(false)) {
            s.status = "blocked";
        }
        s
    }
    pub fn build(rom: &Rom) -> Result<Self> {
        let species: Vec<_> = (1..rom.profile.species.count as u16)
            .filter_map(|id| rom.valid_species(id).ok())
            .collect();
        let mut evolutions = BTreeMap::new();
        let mut learnsets = BTreeMap::new();
        for s in &species {
            evolutions.insert(s.id, rom.evolutions(s.id)?);
            learnsets.insert(s.id, rom.learnset(s.id)?);
        }
        Ok(Self {
            wild_cache: RefCell::default(),
            world: rom.world()?,
            species,
            evolutions,
            learnsets,
        })
    }
    /// Indexed receiving scripts locate services, not proof of access or an egg.
    pub fn daycare_sources(&self, rom: &Rom, save: Option<&Save>) -> Vec<AcquisitionSource> {
        let state = save
            .zip(rom.profile.event_state)
            .map(|(save, layout)| EventSnapshot::new(save, layout));
        let mut sources = Vec::new();
        for report in &self.world.map_events {
            let Some(map) = self.world.maps.iter().find(|m| m.id == report.map_id) else {
                continue;
            };
            for (marker, offer) in report
                .markers
                .iter()
                .flat_map(|m| m.daycare.iter().map(move |d| (Some(m), d)))
                .chain(report.unplaced_daycare.iter().map(|d| (None, d)))
            {
                let mut s = source("daycare", offer.offset);
                s.map_id = Some(map.id.clone());
                s.region = Some(map.region);
                s.x = marker.map(|m| m.x);
                s.y = marker.map(|m| m.y);
                s.conditions = offer
                    .conditions
                    .iter()
                    .map(|c| check(state.as_ref(), Some(rom), c))
                    .collect();
                sources.push(s);
            }
        }
        sources
    }
    pub fn query_scenario(
        &self,
        rom: &Rom,
        save: Option<&Save>,
        target: Target,
        hour: Option<u8>,
        use_save_clock: bool,
    ) -> Result<AcquisitionReport> {
        let mut report = self.query(rom, save, target)?;
        if hour.is_some() || use_save_clock {
            let clock = rom.clock_query_with_save(
                if use_save_clock { save } else { None },
                crate::clock::ClockScenario {
                    hour,
                    weekday: None,
                },
            )?;
            self.mark_period(&mut report.sources, clock.period);
            report.clock = Some(clock);
        }
        Ok(report)
    }
    pub(crate) fn mark_period(&self, sources: &mut [AcquisitionSource], period: Option<&str>) {
        if let Some(period) = period {
            for source in sources {
                if source.periods.is_empty() {
                    continue;
                }
                let base_fallback = source.periods.iter().any(|p| p == "base")
                    && !self.world.encounters.iter().any(|e| {
                        Some(&e.map_id) == source.map_id.as_ref()
                            && e.method
                                == source.encounter_method.as_deref().unwrap_or(&source.kind)
                            && e.periods.contains(&period)
                    });
                source.in_scenario =
                    Some(source.periods.iter().any(|p| p == period) || base_fallback);
            }
        }
    }
    pub fn query(
        &self,
        rom: &Rom,
        save: Option<&Save>,
        target: Target,
    ) -> Result<AcquisitionReport> {
        match target.kind {
            TargetKind::Species => {
                rom.valid_species(target.id)?;
            }
            TargetKind::Item => {
                rom.item(target.id)?;
            }
            TargetKind::Move => {
                rom.move_info(target.id)?;
            }
        }
        let state = save
            .zip(rom.profile.event_state)
            .map(|(save, layout)| EventSnapshot::new(save, layout));
        let mut sources = Vec::new();
        match target.kind {
            TargetKind::Item => {
                for report in &self.world.map_events {
                    let map = self
                        .world
                        .maps
                        .iter()
                        .find(|m| m.id == report.map_id)
                        .ok_or_else(|| err("map_id", &report.map_id))?;
                    for (marker, reward) in report
                        .markers
                        .iter()
                        .flat_map(|m| m.rewards.iter().map(move |r| (Some(m), r)))
                        .chain(report.unplaced_rewards.iter().map(|r| (None, r)))
                        .filter(|(_, r)| r.item == target.id)
                    {
                        let mut s = source(reward.via, reward.offset);
                        s.map_id = Some(map.id.clone());
                        s.region = Some(map.region);
                        s.quantity = reward.quantity;
                        s.x = marker.map(|m| m.x);
                        s.y = marker.map(|m| m.y);
                        s.underfoot = marker.and_then(|m| m.underfoot);
                        s.conditions = reward
                            .conditions
                            .iter()
                            .map(|c| check(state.as_ref(), Some(rom), c))
                            .collect();
                        s.receipt = reward.receipt.clone();
                        s.receipt_flag = s.receipt.as_ref().map(|r| r.flag);
                        if matches!(reward.via, "hidden" | "pickup") {
                            s.receipt_flag = marker.and_then(|m| m.receipt_flag);
                            // Receipt protocols do not prove the absence of flag-reset scripts.
                        } else if reward.via == "shop" {
                            s.repeatable = Some(true);
                        }
                        let receipt = state
                            .as_ref()
                            .and_then(|state| s.receipt_flag.and_then(|f| state.flag(f)));
                        s.partial = marker.is_none_or(|m| !m.stopped_at.is_empty())
                            || !report.stopped_at.is_empty()
                            || rom.profile.event_state.is_none()
                            || (matches!(reward.via, "hidden" | "pickup" | "gift" | "pc")
                                && (s.receipt_flag.is_none()
                                    || (save.is_some() && receipt.is_none())));
                        s.status = if receipt == Some(1) {
                            "completed"
                        } else if !s.partial
                            && s.conditions.iter().any(|c| c.satisfied == Some(false))
                        {
                            "blocked"
                        } else if save.is_some()
                            && !s.partial
                            && s.conditions.iter().all(|c| c.satisfied == Some(true))
                        {
                            "available"
                        } else {
                            "unknown"
                        };
                        sources.push(s);
                    }
                }
                for report in &self.world.map_events {
                    let map = self
                        .world
                        .maps
                        .iter()
                        .find(|m| m.id == report.map_id)
                        .ok_or_else(|| err("map_id", &report.map_id))?;
                    for (marker, mon) in report
                        .markers
                        .iter()
                        .flat_map(|m| m.pokemon.iter().map(move |p| (Some(m), p)))
                        .chain(report.unplaced_pokemon.iter().map(|p| (None, p)))
                        .filter(|(_, p)| {
                            p.trade.is_some() && p.held_item == Some(target.id) && target.id != 0
                        })
                    {
                        let mut s = scripted_source(rom, save, state.as_ref(), map, marker, mon)?;
                        s.kind = "npc_trade_item".into();
                        s.quantity = Some(1);
                        s.related
                            .retain(|t| !(t.kind == TargetKind::Item && t.id == target.id));
                        s.related.insert(
                            0,
                            Target {
                                kind: TargetKind::Species,
                                id: mon.species,
                            },
                        );
                        sources.push(s);
                    }
                }
                if target.id != 0 {
                    let mut candidates: BTreeSet<_> = self
                        .species
                        .iter()
                        .filter(|s| s.items.contains(&target.id))
                        .map(|s| s.id)
                        .collect();
                    if rom.profile.wild_items.is_some() {
                        candidates.extend(
                            rom.special_held_species(target.id)?
                                .into_iter()
                                .filter(|id| rom.valid_species(*id).is_ok()),
                        );
                    }
                    let lead = save
                        .filter(|s| s.party_count() > 0)
                        .map(|s| s.raw(crate::save::Location::Party { slot: 0 }))
                        .transpose()?;
                    for species in candidates {
                        let mut referenced = false;
                        for e in self.world.encounters.iter().filter(|e| {
                            e.species == species
                                && matches!(
                                    e.method.as_str(),
                                    "grass"
                                        | "cave"
                                        | "dive"
                                        | "surf"
                                        | "old_rod"
                                        | "good_rod"
                                        | "super_rod"
                                        | "rock_smash"
                                )
                        }) {
                            referenced = true;
                            let mut s = self.encounter_source(rom, state.as_ref(), e);
                            s.kind = "wild_held".into();
                            s.encounter_method = Some(e.method.clone());
                            s.quantity = Some(1);
                            s.partial = true; // Static references do not establish access or all encounter overrides.
                            s.related.push(Target {
                                kind: TargetKind::Species,
                                id: species,
                            });
                            let map = self
                                .world
                                .maps
                                .iter()
                                .find(|m| m.id == e.map_id)
                                .ok_or_else(|| err("map_id", &e.map_id))?;
                            let layout = crate::binary::u16(&rom.data, map.header + 18)?;
                            let context = (|| {
                                let baseline =
                                    self.wild_distribution(rom, species, layout, None)?;
                                let current_party = lead
                                    .as_deref()
                                    .map(|raw| {
                                        self.wild_distribution(rom, species, layout, Some(raw))
                                    })
                                    .transpose()?;
                                Ok::<_, crate::Error>(crate::wild_items::HeldContext {
                                    species,
                                    layout,
                                    routine: rom.profile.wild_items.unwrap().routine,
                                    baseline,
                                    current_party,
                                })
                            })();
                            match context {
                                Ok(context) => {
                                    let used =
                                        context.current_party.as_ref().unwrap_or(&context.baseline);
                                    // Normal species fields can be overridden in an exceptional layout.
                                    // A zero native chance is not an obtainable source in this context.
                                    if used.count(target.id) == 0
                                        && context.baseline.count(target.id) == 0
                                    {
                                        continue;
                                    }
                                    s.held_percent = Some(used.percent(target.id));
                                    s.held_context = Some(context);
                                }
                                Err(error) => {
                                    s.held_issue = Some(error.to_string());
                                }
                            }
                            sources.push(s);
                        }
                        if !referenced
                            && self
                                .species
                                .iter()
                                .any(|s| s.id == species && s.items.contains(&target.id))
                        {
                            let mut s =
                                source("wild_held_unreferenced", rom.species(species)?.offset);
                            s.related.push(Target {
                                kind: TargetKind::Species,
                                id: species,
                            });
                            // A table field without a random encounter reference is not a catchable source.
                            sources.push(s);
                        }
                    }
                }
            }
            TargetKind::Species => {
                for e in self.world.encounters.iter().filter(|e| {
                    e.species == target.id
                        && !matches!(
                            e.method.as_str(),
                            "static" | "gift" | "egg" | "special_battle"
                        )
                }) {
                    sources.push(self.encounter_source(rom, state.as_ref(), e));
                }
                for report in &self.world.map_events {
                    let map = self
                        .world
                        .maps
                        .iter()
                        .find(|m| m.id == report.map_id)
                        .ok_or_else(|| err("map_id", &report.map_id))?;
                    for (marker, mon) in report
                        .markers
                        .iter()
                        .flat_map(|m| m.pokemon.iter().map(move |p| (Some(m), p)))
                        .chain(report.unplaced_pokemon.iter().map(|p| (None, p)))
                        .filter(|(_, p)| p.species == target.id)
                    {
                        sources.push(scripted_source(
                            rom,
                            save,
                            state.as_ref(),
                            map,
                            marker,
                            mon,
                        )?);
                    }
                }
                for (parent, evos) in &self.evolutions {
                    for evo in evos.iter().filter(|e| e.target == target.id) {
                        let mut s = source("evolution", evo.offset);
                        s.related.push(Target {
                            kind: TargetKind::Species,
                            id: *parent,
                        });
                        s.evolution = Some(evo.clone());
                        s.requirements = evo.requirements.clone();
                        s.related.extend(evolution_targets(evo));
                        sources.push(s);
                    }
                }
                let mon = self.species.iter().find(|s| s.id == target.id).unwrap();
                if !mon.egg_groups.iter().any(|g| matches!(g, 0 | 13 | 15))
                    && !self
                        .evolutions
                        .values()
                        .flatten()
                        .any(|e| e.target == target.id)
                {
                    let mut s = source("breeding_candidate", mon.offset);
                    s.related.push(target.clone());
                    sources.push(s);
                }
            }
            TargetKind::Move => {
                // Physical teachers appear before broad compatibility rows so they
                // remain visible in the default compact query result page.
                for report in &self.world.map_events {
                    let map = self
                        .world
                        .maps
                        .iter()
                        .find(|m| m.id == report.map_id)
                        .ok_or_else(|| err("map_id", &report.map_id))?;
                    for (marker, offer) in report
                        .markers
                        .iter()
                        .flat_map(|m| m.teaching.iter().map(move |t| (Some(m), t)))
                        .chain(report.unplaced_teaching.iter().map(|t| (None, t)))
                        .filter(|(_, t)| t.move_id == target.id)
                    {
                        let mut s = source("move_tutor", offer.offset);
                        s.map_id = Some(map.id.clone());
                        s.region = Some(map.region);
                        s.x = marker.map(|m| m.x);
                        s.y = marker.map(|m| m.y);
                        s.conditions = offer
                            .conditions
                            .iter()
                            .map(|c| check(state.as_ref(), Some(rom), c))
                            .collect();
                        s.teaching_source = Some(offer.clone());
                        if s.conditions.iter().any(|c| c.satisfied == Some(false)) {
                            s.status = "blocked";
                        }
                        sources.push(s);
                    }
                }
                for (species, list) in &self.learnsets {
                    for l in list.iter().filter(|l| l.move_id == target.id) {
                        let mut s = source(&format!("learn_{}", l.source), l.offset);
                        s.related.push(Target {
                            kind: TargetKind::Species,
                            id: *species,
                        });
                        s.min_level = l.level;
                        sources.push(s);
                    }
                }
                for id in 1..rom.profile.items.count as u16 {
                    if let Ok(item) = rom.item(id) {
                        if item.tm_move == Some(target.id) {
                            let mut s = source("machine", item.offset);
                            s.related.push(Target {
                                kind: TargetKind::Item,
                                id,
                            });
                            sources.push(s);
                        }
                    }
                }
            }
        }
        for source in &mut sources {
            if source.conditions.iter().any(|c| {
                matches!(
                    c.condition.kind,
                    "bag_item" | "bag_item_runtime" | "money" | "money_runtime"
                )
            }) {
                source.partial = true;
                if source.status != "completed"
                    && source.conditions.iter().any(|c| {
                        c.satisfied == Some(false)
                            && matches!(c.condition.kind, "bag_item" | "money")
                    })
                {
                    source.status = "blocked";
                } else if source.status == "available" {
                    source.status = "unknown";
                }
            }
            for check in &source.conditions {
                if matches!(check.condition.kind, "bag_item" | "bag_item_runtime") {
                    let item = check.condition.id;
                    if item != 0
                        && !source
                            .related
                            .iter()
                            .any(|r| r.kind == TargetKind::Item && r.id == item)
                    {
                        source.related.push(Target {
                            kind: TargetKind::Item,
                            id: item,
                        });
                    }
                }
            }
        }
        Ok(AcquisitionReport {
            clock: None,
            target,
            sources,
            partial: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evolution_cross_links_keep_compound_operands_and_do_not_guess_by_number() {
        let mut e = Evolution {
            method: 0,
            condition: "held_day",
            requirements: vec![],
            parameter: 17,
            auxiliary: 23,
            target: 2,
            offset: 100,
        };
        for condition in [
            "held_day",
            "held_night",
            "male_item",
            "female_item",
            "trade_item",
            "item_night",
            "item_location",
        ] {
            e.condition = condition;
            let targets = evolution_targets(&e);
            assert!(
                targets.len() == 1 && targets[0].kind == TargetKind::Item && targets[0].id == 17
            );
        }
        e.condition = "item_hold_item";
        assert_eq!(
            evolution_targets(&e)
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            [17, 23]
        );
        e.condition = "level_hold_item";
        assert_eq!(evolution_targets(&e)[0].id, 23, "level is not an item ID");
        for condition in ["move", "move_male", "move_female"] {
            e.condition = condition;
            assert!(evolution_targets(&e)
                .iter()
                .all(|t| t.kind == TargetKind::Move && t.id == 17));
        }
        for condition in ["party_species", "trade_species"] {
            e.condition = condition;
            assert!(evolution_targets(&e)
                .iter()
                .all(|t| t.kind == TargetKind::Species && t.id == 17));
        }
        for condition in [
            "level",
            "unknown",
            "flag",
            "move_type",
            "party_type",
            "nature_high",
            "map",
        ] {
            e.condition = condition;
            assert!(
                evolution_targets(&e).is_empty(),
                "numeric values must not be guessed into resource IDs"
            );
        }
        e.condition = "item";
        e.requirements = vec![
            crate::rom::EvolutionRequirement {
                kind: "item",
                value: 17,
            },
            crate::rom::EvolutionRequirement {
                kind: "held_item",
                value: 23,
            },
        ];
        assert_eq!(
            evolution_targets(&e)
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            [17, 23]
        );
    }
    #[test]
    fn time_table_fallback_is_per_map_and_method_not_per_target_species() {
        use crate::world::{Encounter, TrainerLocationIndex};
        let encounter = Encounter {
            selector: None,
            periods: vec!["night"],
            species: 2,
            map_id: "0-0".into(),
            map_name: "Synthetic map".into(),
            region: 1,
            method: "grass".into(),
            min_level: 1,
            max_level: 1,
            weight: Some(100),
            encounter_rate: Some(20),
            slot: Some(0),
            offset: 0,
            conditional: true,
        };
        let index = AcquisitionIndex {
            wild_cache: RefCell::default(),
            world: World {
                maps: vec![],
                map_events: vec![],
                encounters: vec![encounter],
                trainers: vec![],
                trainer_locations: TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: vec![],
            evolutions: BTreeMap::new(),
            learnsets: BTreeMap::new(),
        };
        let mut sources: Vec<_> = ["grass", "surf", "grass", "static"]
            .into_iter()
            .map(|kind| {
                let mut s = source(kind, 0);
                s.map_id = Some("0-0".into());
                s.periods = vec!["base".into()];
                s
            })
            .collect();
        sources[2].map_id = Some("0-1".into());
        sources[3].periods.clear();
        index.mark_period(&mut sources, Some("night"));
        assert_eq!(
            sources.iter().map(|s| s.in_scenario).collect::<Vec<_>>(),
            [Some(false), Some(true), Some(true), None]
        );
        // The target species need not occur in the overriding table: its base slots still lose priority.
        index.mark_period(&mut sources, Some("day"));
        assert_eq!(sources[0].in_scenario, Some(true));
    }
    #[test]
    fn predicates_are_tristate_and_compare_numeric_values() {
        use crate::event_state::{EventBlock, EventRange};
        let layout = crate::profile::EventStateLayout {
            flags: &[EventRange {
                first: 0,
                count: 0x4000,
                block: EventBlock::Main,
                offset: 0,
            }],
            variables: &[EventRange {
                first: 0x4000,
                count: 8,
                block: EventBlock::Main,
                offset: 32,
            }],
            pickup_receipt: true,
            gift_result: true,
        };
        let mut b = vec![0u8; 64];
        b[1] = 4;
        b[32..34].copy_from_slice(&7u16.to_le_bytes());
        let state = EventSnapshot::fixture(b, vec![], vec![], layout);
        let c = EventCondition {
            kind: "flag",
            id: 10,
            value: 1,
            comparison: 1,
            taken: true,
        };
        assert_eq!(check(Some(&state), None, &c).satisfied, Some(true));
        assert_eq!(check(None, None, &c).satisfied, None);
        let c = EventCondition {
            kind: "variable",
            id: 0x4000,
            value: 6,
            comparison: 2,
            taken: false,
        };
        assert_eq!(check(Some(&state), None, &c).satisfied, Some(false));
        let c = EventCondition { id: 0x800d, ..c };
        assert_eq!(check(Some(&state), None, &c).satisfied, None);
    }
}
