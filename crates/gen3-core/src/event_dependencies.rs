//! Read-only potential writers of persistent prerequisites, never a task catalog.
//! Referenced map roots and text come from the loaded ROM. Native/special writes,
//! dynamic roots and current reachability remain outside this bounded static index.
use crate::{
    acquisition::{check, ConditionCheck},
    binary::*,
    err,
    event_state::EventSnapshot,
    map_events::EventCondition,
    rom::Rom,
    save::Save,
    world::Map,
    Result,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rules {
    pub commands: usize,
    /// setvar, addvar, subvar, copyvar, setorcopyvar, setflag, clearflag, loadpointer.
    pub handlers: [usize; 8],
    pub battle_handler: usize,
    /// Native operand layouts, including opcode. Zero means not yet verified.
    pub battle_lengths: [u8; 17],
    pub battle_roles: [&'static str; 17],
    /// Native warp, silent, door, teleport, setwarp, spin and gym dispatch.
    /// Zero means unsupported/unverified; destination setters are not passages.
    pub warp_handlers: [usize; 7],
}
const OPCODES: [u8; 8] = [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f];
pub const EMERALD: Rules = Rules {
    warp_handlers: [
        0x99ebc, 0x99f44, 0x99fcc, 0x9a0c8, 0x9a1d8, 0x9bcdc, 0x9a150,
    ],
    battle_roles: [
        "primary",
        "primary",
        "primary",
        "primary",
        "primary",
        "rematch_base",
        "primary",
        "rematch_base",
        "primary",
        "unresolved",
        "setup",
        "secondary_setup",
        "unresolved",
        "unresolved",
        "unresolved",
        "unresolved",
        "unresolved",
    ],
    commands: 0x1db67c,
    battle_handler: 0x9b5d0,
    battle_lengths: [
        14, 18, 18, 10, 18, 14, 22, 18, 22, 14, 14, 14, 14, 0, 0, 0, 0,
    ],
    handlers: [
        0x99720, 0x99914, 0x9993c, 0x99744, 0x99770, 0x99c14, 0x99c28, 0x99644,
    ],
};
pub const ROCKET: Rules = Rules {
    warp_handlers: [
        0xcfe04, 0xcfe8c, 0xcff14, 0xd0010, 0xd0120, 0xd1cac, 0xd0098,
    ],
    battle_roles: EMERALD.battle_roles,
    commands: 0x22b218,
    battle_handler: 0xd1530,
    battle_lengths: [
        14, 18, 18, 10, 18, 14, 22, 18, 22, 14, 14, 14, 14, 0, 0, 0, 0,
    ],
    handlers: [
        0xcf610, 0xcf804, 0xcf82c, 0xcf634, 0xcf660, 0xcfb5c, 0xcfb70, 0xcf534,
    ],
};
pub const MERCURY: Rules = Rules {
    warp_handlers: [0x6aa64, 0x6aaec, 0x6ab74, 0x6ac70, 0x6ad8c, 0x6acf8, 0],
    battle_roles: [
        "primary",
        "primary",
        "primary",
        "primary",
        "primary",
        "rematch_base",
        "primary",
        "rematch_base",
        "primary",
        "primary",
        "primary",
        "unresolved",
        "primary",
        "primary",
        "primary",
        "primary",
        "unresolved",
    ],
    commands: 0x15f9b4,
    battle_handler: 0x6c2c4,
    battle_lengths: [
        14, 18, 18, 10, 18, 14, 22, 18, 22, 14, 20, 0, 14, 14, 18, 10, 0,
    ],
    handlers: [
        0x6a390, 0x6a584, 0x6a5ac, 0x6a3b4, 0x6a3e0, 0x6a82c, 0x6a840, 0x6a2b4,
    ],
};
pub(crate) fn validate(rom: &Rom) -> Result<()> {
    let rules = rom
        .profile
        .event_state
        .and_then(|r| r.effects)
        .ok_or_else(|| err("event_dependencies_unverified", rom.profile.id))?;
    for (opcode, target) in OPCODES.into_iter().zip(rules.handlers) {
        if (pointer(&rom.data, rules.commands + opcode as usize * 4)? & !1) != target {
            return Err(err("event_dependency_dispatch", format!("{opcode:02x}")));
        }
    }
    Ok(())
}
pub(crate) fn persistent(rom: &Rom, kind: &str, id: u16) -> bool {
    let Some(layout) = rom.profile.event_state else {
        return false;
    };
    let ranges = match kind {
        "flag" if id != 0 => layout.flags,
        "variable" => layout.variables,
        _ => return false,
    };
    ranges
        .iter()
        .any(|r| id >= r.first && (id - r.first) < r.count)
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Effect {
    pub kind: &'static str,
    pub id: u16,
    pub operation: &'static str,
    pub operand: Option<u16>,
    /// Known after this instruction along this static path, not the SAV value.
    pub value: Option<u16>,
    pub offset: usize,
    pub conditions: Vec<EventCondition>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextReference {
    pub offset: usize,
    pub text: String,
}
pub(crate) struct ScriptEffects {
    pub warps: Vec<crate::navigation::ScriptWarp>,
    pub effects: Vec<Effect>,
    pub text: Vec<TextReference>,
    pub battles: Vec<BattleSource>,
    pub stopped_at: Vec<usize>,
    pub complete: bool,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct BattleSource {
    pub trainer_id: u16,
    pub battle_type: u8,
    pub offset: usize,
    pub conditions: Vec<EventCondition>,
    pub role: &'static str,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainerRequest {
    pub expected_rom_md5: String,
    pub trainer_id: u16,
    #[serde(default)]
    pub offset: usize,
}
#[derive(Serialize)]
pub struct TrainerReference {
    pub clue_id: String,
    pub battle: BattleSource,
    pub reference: Reference,
    pub conditions: Vec<ConditionCheck>,
    pub visibility: Vec<ConditionCheck>,
    pub text: Vec<TextReference>,
    pub stopped_at: Vec<usize>,
}
#[derive(Serialize)]
pub struct TrainerReport {
    pub rom_md5: String,
    pub trainer_id: u16,
    pub references: Vec<TrainerReference>,
    pub total_matches: usize,
    pub next_offset: Option<usize>,
    pub coverage: Coverage,
    pub partial: bool,
}
#[derive(Clone, Serialize)]
pub struct Reference {
    pub map_id: String,
    pub map_name: String,
    pub region: u8,
    pub kind: &'static str,
    pub x: Option<i16>,
    pub y: Option<i16>,
    pub local_id: Option<u8>,
    pub offset: usize,
    pub root: usize,
    pub conditions: Vec<EventCondition>,
    /// Entry selectors, movement and script activation are not access proofs.
    pub entry_unresolved: bool,
}
struct IndexedEffect {
    effect: Effect,
    reference: Reference,
    root: usize,
}
pub struct Index {
    rom_md5: &'static str,
    rom_data: std::sync::Arc<Vec<u8>>,
    scripts: BTreeMap<usize, ScriptEffects>,
    references: Vec<Reference>,
    writers: BTreeMap<(&'static str, u16), Vec<IndexedEffect>>,
    pub coverage: Coverage,
}
#[derive(Clone, Serialize)]
pub struct Coverage {
    pub checked_scripts: usize,
    pub total_scripts: usize,
    pub failed_scripts: usize,
    pub truncated: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Flag,
    Variable,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub kind: Kind,
    pub id: u16,
    pub value: u32,
    pub comparison: u8,
    pub taken: bool,
    pub expected_rom_md5: String,
    #[serde(default)]
    pub offset: usize,
}
#[derive(Serialize)]
pub struct Writer {
    pub effect: Effect,
    pub reference: Reference,
    pub conditions: Vec<ConditionCheck>,
    pub text: Vec<TextReference>,
    pub stopped_at: Vec<usize>,
    pub path_complete: bool,
}
#[derive(Serialize)]
pub struct Report {
    pub rom_md5: String,
    pub condition: ConditionCheck,
    pub writers: Vec<Writer>,
    pub coverage: Coverage,
    pub total_matches: usize,
    pub next_offset: Option<usize>,
    /// Native/special/dynamic/unreferenced writes are not exhaustively covered.
    pub partial: bool,
}
#[derive(Serialize)]
pub struct Bundle {
    pub reports: Vec<Report>,
    pub routes: Vec<PlanningRoute>,
    pub entrances: Vec<crate::collection::EntranceSuggestion>,
    pub skipped_conditions: usize,
    pub truncated: bool,
    pub partial: bool,
}

/// A guarded writer is one alternative, not an additional mandatory task.
#[derive(Serialize)]
pub struct PlanningCandidate {
    pub writer_index: usize,
    pub requires: Vec<usize>,
    pub untraced_conditions: Vec<ConditionCheck>,
    /// The indexed alternatives contain a path back to the requested condition.
    /// This does not make the condition impossible: another alternative may exit.
    pub recursive: bool,
}
#[derive(Serialize)]
pub struct PlanningRoute {
    pub report_index: usize,
    /// Region/task indices in this exact plan snapshot, never persistent task IDs.
    pub goals: Vec<[usize; 2]>,
    pub candidates: Vec<PlanningCandidate>,
}

pub(crate) fn planning_routes(
    reports: &[Report],
    plan: &crate::collection::CollectionPlan,
) -> Vec<PlanningRoute> {
    let indices: BTreeMap<_, _> = reports
        .iter()
        .enumerate()
        .map(|(i, r)| (r.condition.condition.clone(), i))
        .collect();
    let mut routes: Vec<_> = reports
        .iter()
        .enumerate()
        .map(|(report_index, report)| PlanningRoute {
            report_index,
            goals: Vec::new(),
            candidates: report
                .writers
                .iter()
                .enumerate()
                .map(|(writer_index, writer)| {
                    let mut requires = BTreeSet::new();
                    let mut untraced_conditions = Vec::new();
                    for guard in writer
                        .conditions
                        .iter()
                        .filter(|c| c.satisfied != Some(true))
                    {
                        if let Some(i) = indices.get(&guard.condition) {
                            requires.insert(*i);
                        } else {
                            untraced_conditions.push(guard.clone());
                        }
                    }
                    PlanningCandidate {
                        writer_index,
                        requires: requires.into_iter().collect(),
                        untraced_conditions,
                        recursive: false,
                    }
                })
                .collect(),
        })
        .collect();
    let edges: Vec<Vec<usize>> = routes
        .iter()
        .map(|r| {
            if reports[r.report_index].condition.satisfied == Some(true) {
                return Vec::new();
            }
            r.candidates
                .iter()
                .flat_map(|c| &c.requires)
                .copied()
                .collect()
        })
        .collect();
    let reachable = |starts: Vec<usize>| {
        let mut seen = BTreeSet::new();
        let mut pending = starts;
        while let Some(i) = pending.pop() {
            if seen.insert(i) {
                pending.extend(&edges[i]);
            }
        }
        seen
    };
    for (region_index, region) in plan.regions.iter().enumerate() {
        for (task_index, task) in region.tasks.iter().enumerate() {
            let starts = [
                task.source.as_ref(),
                task.preparation.as_ref().and_then(|p| p.source.as_ref()),
            ]
            .into_iter()
            .flatten()
            .flat_map(|s| &s.conditions)
            .filter_map(|c| indices.get(&c.condition).copied())
            .collect();
            for i in reachable(starts) {
                routes[i].goals.push([region_index, task_index]);
            }
        }
    }
    for route in &mut routes {
        for candidate in &mut route.candidates {
            candidate.recursive =
                reachable(candidate.requires.clone()).contains(&route.report_index);
        }
    }
    routes
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    pub expected_rom_md5: String,
    #[serde(default)]
    pub search: String,
    pub map_id: Option<String>,
    #[serde(default)]
    pub offset: usize,
    pub selected_id: Option<String>,
}
#[derive(Serialize)]
pub struct ClueEffect {
    pub effect: Effect,
    pub conditions: Vec<ConditionCheck>,
    /// Whether this write's known value currently matches, not receipt/completion.
    pub observed: Option<bool>,
}
#[derive(Serialize)]
pub struct Clue {
    pub id: String,
    pub reference: Reference,
    pub text: Vec<TextReference>,
    pub visibility: Vec<ConditionCheck>,
    pub effects: Vec<ClueEffect>,
    pub battles: Vec<BattleSource>,
    pub effects_truncated: bool,
    pub stopped_at: Vec<usize>,
    pub path_complete: bool,
}
#[derive(Serialize)]
pub struct SearchReport {
    pub rom_md5: String,
    pub entries: Vec<Clue>,
    pub selected: Option<Clue>,
    pub total_matches: usize,
    pub next_offset: Option<usize>,
    pub coverage: Coverage,
    pub partial: bool,
}

fn reference_id(reference: &Reference) -> String {
    format!(
        "{}:{}:{:x}:{:x}",
        reference.map_id, reference.kind, reference.offset, reference.root
    )
}

/// Keep positioned non-reward triggers/signs too; reward markers alone miss them.
fn references(rom: &Rom, maps: &[Map]) -> Result<Vec<Reference>> {
    let mut out = Vec::new();
    for map in maps {
        let mut positioned = BTreeSet::new();
        if let Some(ev) = map.events {
            for (count_off, ptr_off, stride, script_off, kind) in [
                (0, 4, 24, 16, "npc"),
                (2, 12, 16, 12, "trigger"),
                (3, 16, 12, 8, "sign"),
            ] {
                let count = bytes(&rom.data, ev + count_off, 1)?[0] as usize;
                if count == 0 {
                    continue;
                }
                let table = pointer(&rom.data, ev + ptr_off)?;
                bytes(&rom.data, table, count * stride)?;
                for i in 0..count {
                    let offset = table + i * stride;
                    if count_off == 3 && rom.data[offset + 5] > 4 {
                        continue;
                    }
                    let Ok(root) = pointer(&rom.data, offset + script_off) else {
                        continue;
                    };
                    let xy = if count_off == 0 { 4 } else { 0 };
                    let mut conditions = Vec::new();
                    if count_off == 0 {
                        let flag = u16(&rom.data, offset + 20)?;
                        if flag != 0 {
                            conditions.push(EventCondition {
                                kind: "flag",
                                id: flag,
                                value: 1,
                                comparison: 1,
                                taken: false,
                            });
                        }
                    }
                    let x = u16(&rom.data, offset + xy)? as i16;
                    let y = u16(&rom.data, offset + xy + 2)? as i16;
                    let placed =
                        x >= 0 && y >= 0 && (x as u32) < map.width && (y as u32) < map.height;
                    positioned.insert(root);
                    out.push(Reference {
                        map_id: map.id.clone(),
                        map_name: map.name.clone(),
                        region: map.region,
                        kind,
                        x: placed.then_some(x),
                        y: placed.then_some(y),
                        local_id: (count_off == 0).then_some(rom.data[offset]),
                        offset,
                        root,
                        conditions,
                        entry_unresolved: true,
                    });
                }
            }
        }
        for &root in &map.scripts {
            if positioned.contains(&root) {
                continue;
            }
            out.push(Reference {
                map_id: map.id.clone(),
                map_name: map.name.clone(),
                region: map.region,
                kind: "map_script",
                x: None,
                y: None,
                local_id: None,
                offset: map.header,
                root,
                conditions: Vec::new(),
                entry_unresolved: true,
            });
        }
    }
    Ok(out)
}
impl Index {
    /// Reuse the loaded-ROM script index; never cache SAV eligibility in it.
    pub fn map_navigation(
        &self,
        rom: &Rom,
        maps: &[Map],
        id: &str,
        save: Option<&Save>,
    ) -> Result<crate::navigation::MapNavigation> {
        self.check_rom(rom)?;
        let state = save
            .zip(rom.profile.event_state)
            .map(|(s, layout)| EventSnapshot::new(s, layout));
        let (mut edges, mut diagnostics) = crate::navigation::links(&rom.data, maps)?;
        let mut failures = 0;
        for reference in &self.references {
            let Some(script) = self.scripts.get(&reference.root) else {
                continue;
            };
            for warp in &script.warps {
                match crate::navigation::script_link(
                    &rom.data,
                    maps,
                    reference,
                    warp,
                    &script.stopped_at,
                ) {
                    Ok(mut link) => {
                        if let Some(script) = &mut link.script {
                            script.checks = script
                                .conditions
                                .iter()
                                .map(|c| check(state.as_ref(), Some(rom), c))
                                .collect();
                        }
                        edges.push(link);
                    }
                    Err(_) => failures += 1,
                }
            }
        }
        if failures > 0 || self.coverage.failed_scripts > 0 || self.coverage.truncated {
            diagnostics.push(format!("Partial script passages: {failures} failed destinations; {} failed roots; bounded={}", self.coverage.failed_scripts, self.coverage.truncated));
        }
        diagnostics.push("Script passages retain branch/visibility guards; activation and current reachability remain unverified. Hole, native/special and extended transitions are not fully covered. Destination setters do not create edges.".into());
        crate::navigation::report(maps, &edges, diagnostics, id)
    }

    pub fn trainer(
        &self,
        rom: &Rom,
        save: Option<&Save>,
        request: TrainerRequest,
    ) -> Result<TrainerReport> {
        self.check_rom(rom)?;
        if request.expected_rom_md5 != rom.profile.md5 {
            return Err(err("rom_mismatch", request.expected_rom_md5));
        }
        if request.trainer_id as usize >= rom.profile.trainers.count {
            return Err(err("trainer_id", request.trainer_id));
        }
        let state = save
            .zip(rom.profile.event_state)
            .map(|(s, l)| EventSnapshot::new(s, l));
        let matches: Vec<_> = self
            .references
            .iter()
            .flat_map(|r| {
                self.scripts
                    .get(&r.root)
                    .into_iter()
                    .flat_map(move |s| s.battles.iter().map(move |b| (r, s, b)))
            })
            .filter(|(_, _, b)| b.trainer_id == request.trainer_id)
            .collect();
        if request.offset > matches.len() {
            return Err(err("trainer_reference_offset", request.offset));
        }
        let references: Vec<_> = matches
            .iter()
            .skip(request.offset)
            .take(32)
            .map(|(r, s, b)| TrainerReference {
                clue_id: reference_id(r),
                battle: (*b).clone(),
                reference: (*r).clone(),
                text: s.text.clone(),
                stopped_at: s.stopped_at.clone(),
                conditions: b
                    .conditions
                    .iter()
                    .map(|c| check(state.as_ref(), Some(rom), c))
                    .collect(),
                visibility: r
                    .conditions
                    .iter()
                    .map(|c| check(state.as_ref(), Some(rom), c))
                    .collect(),
            })
            .collect();
        let next = request.offset + references.len();
        Ok(TrainerReport {
            rom_md5: rom.profile.md5.into(),
            trainer_id: request.trainer_id,
            references,
            total_matches: matches.len(),
            next_offset: (next < matches.len()).then_some(next),
            coverage: self.coverage.clone(),
            partial: true,
        })
    }
    fn check_rom(&self, rom: &Rom) -> Result<()> {
        if self.rom_md5 != rom.profile.md5 || !std::sync::Arc::ptr_eq(&self.rom_data, &rom.data) {
            return Err(err(
                "rom_mismatch",
                "prerequisite index belongs to another ROM input",
            ));
        }
        Ok(())
    }
    /// A bounded dependency appendix for the same immutable SAV used by a plan.
    /// Guards are potential prerequisites, never a complete story dependency DAG.
    pub fn trace_plan(
        &self,
        rom: &Rom,
        save: &Save,
        maps: &[Map],
        plan: &crate::collection::CollectionPlan,
    ) -> Result<Bundle> {
        self.check_rom(rom)?;
        if plan.rom_md5 != rom.profile.md5 {
            return Err(err("rom_mismatch", plan.rom_md5));
        }
        use std::collections::VecDeque;
        let mut queue = VecDeque::new();
        for task in plan.regions.iter().flat_map(|r| &r.tasks) {
            for source in [
                task.source.as_ref(),
                task.preparation.as_ref().and_then(|p| p.source.as_ref()),
            ]
            .into_iter()
            .flatten()
            {
                queue.extend(source.conditions.iter().map(|c| (c.condition.clone(), 0)));
            }
        }
        let mut seen = BTreeSet::new();
        let mut reports = Vec::new();
        let mut skipped_conditions = 0;
        let mut truncated = false;
        while let Some((condition, depth)) = queue.pop_front() {
            if !seen.insert(condition.clone()) {
                continue;
            }
            if !matches!(condition.kind, "flag" | "variable") {
                continue;
            }
            if !persistent(rom, condition.kind, condition.id)
                || condition.comparison > 5
                || condition.value > u16::MAX as u32
            {
                skipped_conditions += 1;
                continue;
            }
            if reports.len() >= 128 {
                truncated = true;
                break;
            }
            let report = self.query(
                rom,
                Some(save),
                Request {
                    kind: if condition.kind == "flag" {
                        Kind::Flag
                    } else {
                        Kind::Variable
                    },
                    id: condition.id,
                    value: condition.value,
                    comparison: condition.comparison,
                    taken: condition.taken,
                    expected_rom_md5: rom.profile.md5.into(),
                    offset: 0,
                },
            )?;
            truncated |= report.next_offset.is_some();
            for guard in report
                .writers
                .iter()
                .flat_map(|w| &w.conditions)
                .filter(|g| report.condition.satisfied != Some(true) && g.satisfied != Some(true))
            {
                if depth < 3 {
                    queue.push_back((guard.condition.clone(), depth + 1));
                } else if !seen.contains(&guard.condition) {
                    truncated = true;
                }
            }
            reports.push(report);
        }
        let ids: BTreeSet<_> = reports
            .iter()
            .flat_map(|r| &r.writers)
            .map(|w| w.reference.map_id.clone())
            .collect();
        let (edges, _) = crate::navigation::links(&rom.data, maps)?;
        truncated |= ids.len() > 256;
        let entrances = ids
            .into_iter()
            .take(256)
            .map(|id| {
                let (chains, truncated) = crate::navigation::approaches(maps, &edges, &id);
                crate::collection::EntranceSuggestion {
                    map_id: id,
                    chains,
                    truncated,
                }
            })
            .collect();
        let routes = planning_routes(&reports, plan);
        Ok(Bundle {
            reports,
            routes,
            entrances,
            skipped_conditions,
            truncated,
            partial: true,
        })
    }
    pub fn build(rom: &Rom, maps: &[Map]) -> Result<Self> {
        validate(rom)?;
        let references = references(rom, maps)?;
        let roots: BTreeSet<_> = references.iter().map(|r| r.root).collect();
        let mut scripts = BTreeMap::new();
        let mut failed_scripts = 0;
        // Bound index work separately from each walk's 8192-instruction limit.
        for root in roots.iter().take(16_384) {
            match rom.event_effect_script(*root) {
                Ok(report) => {
                    scripts.insert(*root, report);
                }
                Err(_) => failed_scripts += 1,
            }
        }
        let coverage = Coverage {
            checked_scripts: scripts.len(),
            total_scripts: roots.len(),
            failed_scripts,
            truncated: roots.len() > 16_384,
        };
        let mut writers: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for reference in &references {
            let Some(report) = scripts.get(&reference.root) else {
                continue;
            };
            for effect in &report.effects {
                writers
                    .entry((effect.kind, effect.id))
                    .or_default()
                    .push(IndexedEffect {
                        effect: effect.clone(),
                        root: reference.root,
                        reference: reference.clone(),
                    });
            }
        }
        Ok(Self {
            rom_md5: rom.profile.md5,
            rom_data: rom.data.clone(),
            scripts,
            references,
            writers,
            coverage,
        })
    }
    /// Search loaded-ROM context, never a bundled quest catalog. Visibility,
    /// branch guards and observed writes do not establish quest completion.
    pub fn search(
        &self,
        rom: &Rom,
        save: Option<&Save>,
        request: SearchRequest,
    ) -> Result<SearchReport> {
        self.check_rom(rom)?;
        if request.expected_rom_md5 != rom.profile.md5 {
            return Err(err("rom_mismatch", request.expected_rom_md5));
        }
        if request.search.len() > 512
            || request.map_id.as_ref().is_some_and(|v| v.len() > 32)
            || request.selected_id.as_ref().is_some_and(|v| v.len() > 128)
        {
            return Err(err("event_search_arguments", "query exceeds bounds"));
        }
        let search = request.search.trim().to_lowercase();
        let matches: Vec<_> = self
            .references
            .iter()
            .filter(|reference| {
                if request
                    .map_id
                    .as_ref()
                    .is_some_and(|id| id != &reference.map_id)
                {
                    return false;
                }
                let Some(script) = self.scripts.get(&reference.root) else {
                    return false;
                };
                // Empty scripts have no readable clue; do not invent a task for them.
                if script.text.is_empty() && script.effects.is_empty() && script.battles.is_empty()
                {
                    return false;
                }
                search.is_empty()
                    || reference.map_name.to_lowercase().contains(&search)
                    || reference.map_id.contains(&search)
                    || script
                        .text
                        .iter()
                        .any(|t| t.text.to_lowercase().contains(&search))
            })
            .collect();
        if request.offset > matches.len() {
            return Err(err("event_search_offset", request.offset));
        }
        let state = save
            .zip(rom.profile.event_state)
            .map(|(s, l)| EventSnapshot::new(s, l));
        let clue = |reference: &&Reference| {
            let script = &self.scripts[&reference.root];
            Clue {
                id: reference_id(reference),
                reference: (*reference).clone(),
                text: script.text.clone(),
                visibility: reference
                    .conditions
                    .iter()
                    .map(|c| check(state.as_ref(), Some(rom), c))
                    .collect(),
                effects: script
                    .effects
                    .iter()
                    .take(64)
                    .map(|effect| {
                        let guards: BTreeSet<_> = effect.conditions.iter().cloned().collect();
                        let observed = effect.value.and_then(|value| {
                            check(
                                state.as_ref(),
                                Some(rom),
                                &EventCondition {
                                    kind: effect.kind,
                                    id: effect.id,
                                    value: value as u32,
                                    comparison: 1,
                                    taken: true,
                                },
                            )
                            .satisfied
                        });
                        ClueEffect {
                            effect: effect.clone(),
                            conditions: guards
                                .iter()
                                .map(|c| check(state.as_ref(), Some(rom), c))
                                .collect(),
                            observed,
                        }
                    })
                    .collect(),
                battles: script.battles.iter().take(64).cloned().collect(),
                effects_truncated: script.effects.len() > 64 || script.battles.len() > 64,
                stopped_at: script.stopped_at.clone(),
                path_complete: script.complete,
            }
        };
        let selected = request
            .selected_id
            .as_ref()
            .and_then(|id| matches.iter().find(|r| reference_id(r) == *id))
            .map(clue);
        let entries: Vec<_> = matches
            .iter()
            .skip(request.offset)
            .take(32)
            .map(clue)
            .collect();
        let next = request.offset + entries.len();
        Ok(SearchReport {
            rom_md5: rom.profile.md5.into(),
            entries,
            selected,
            total_matches: matches.len(),
            next_offset: (next < matches.len()).then_some(next),
            coverage: self.coverage.clone(),
            partial: true,
        })
    }
    pub fn query(&self, rom: &Rom, save: Option<&Save>, request: Request) -> Result<Report> {
        self.check_rom(rom)?;
        if request.expected_rom_md5 != rom.profile.md5 {
            return Err(err("rom_mismatch", request.expected_rom_md5));
        }
        let kind = match request.kind {
            Kind::Flag => "flag",
            Kind::Variable => "variable",
        };
        if request.comparison > 5
            || request.value > u16::MAX as u32
            || !persistent(rom, kind, request.id)
        {
            return Err(err("event_dependency_condition", request.id));
        }
        let condition = EventCondition {
            kind,
            id: request.id,
            value: request.value,
            comparison: request.comparison,
            taken: request.taken,
        };
        let state = save
            .zip(rom.profile.event_state)
            .map(|(s, l)| EventSnapshot::new(s, l));
        let matches: Vec<_> = self
            .writers
            .get(&(kind, request.id))
            .into_iter()
            .flatten()
            .filter(|w| {
                w.effect.value.is_none_or(|value| {
                    crate::acquisition::compare(value as u32, &condition) != Some(false)
                })
            })
            .collect();
        if request.offset > matches.len() {
            return Err(err("event_dependency_offset", request.offset));
        }
        let writers = matches
            .iter()
            .skip(request.offset)
            .take(64)
            .map(|w| {
                let script = &self.scripts[&w.root];
                let guards: BTreeSet<_> = w
                    .effect
                    .conditions
                    .iter()
                    .chain(&w.reference.conditions)
                    .cloned()
                    .collect();
                Writer {
                    effect: w.effect.clone(),
                    reference: w.reference.clone(),
                    conditions: guards
                        .iter()
                        .map(|c| check(state.as_ref(), Some(rom), c))
                        .collect(),
                    text: script.text.clone(),
                    stopped_at: script.stopped_at.clone(),
                    path_complete: script.complete,
                }
            })
            .collect::<Vec<_>>();
        let next = request.offset + writers.len();
        Ok(Report {
            rom_md5: rom.profile.md5.into(),
            condition: check(state.as_ref(), Some(rom), &condition),
            writers,
            coverage: self.coverage.clone(),
            total_matches: matches.len(),
            next_offset: (next < matches.len()).then_some(next),
            partial: true,
        })
    }
}

#[cfg(test)]
mod route_tests {
    use super::*;
    fn guard(id: u16, satisfied: Option<bool>) -> ConditionCheck {
        ConditionCheck {
            condition: EventCondition {
                kind: "flag",
                id,
                value: 1,
                comparison: 1,
                taken: true,
            },
            satisfied,
            actual: satisfied.map(|s| u32::from(s)),
            unresolved: None,
        }
    }
    fn report(id: u16, guards: Vec<Vec<ConditionCheck>>) -> Report {
        Report {
            rom_md5: "fixture".into(),
            condition: guard(id, Some(false)),
            writers: guards
                .into_iter()
                .map(|conditions| Writer {
                    effect: Effect {
                        kind: "flag",
                        id,
                        operation: "set",
                        operand: None,
                        value: Some(1),
                        offset: 100,
                        conditions: vec![],
                    },
                    reference: Reference {
                        map_id: "0-0".into(),
                        map_name: "Fixture".into(),
                        region: 1,
                        kind: "npc",
                        x: Some(1),
                        y: Some(1),
                        local_id: Some(1),
                        offset: 200,
                        root: 100,
                        conditions: vec![],
                        entry_unresolved: true,
                    },
                    conditions,
                    text: vec![],
                    stopped_at: vec![],
                    path_complete: true,
                })
                .collect(),
            coverage: Coverage {
                checked_scripts: 2,
                total_scripts: 2,
                failed_scripts: 0,
                truncated: false,
            },
            total_matches: 2,
            next_offset: None,
            partial: true,
        }
    }
    #[test]
    fn prerequisite_routes_keep_alternatives_cycles_and_untraced_guards_separate() {
        let plan = crate::collection::CollectionPlan {
            prerequisites: None,
            clock: None,
            rom_md5: "fixture",
            basis: crate::collection::CollectionBasis::Individuals,
            families: true,
            owned_count: 0,
            missing_count: 0,
            regions: vec![],
            entrances: vec![],
            breeding_coverage: None,
            partial: true,
        };
        let mut resource = guard(30, None);
        resource.condition.kind = "bag_item";
        let reports = vec![
            report(
                11,
                vec![
                    vec![
                        guard(12, Some(false)),
                        guard(12, Some(false)),
                        guard(13, Some(true)),
                        resource,
                    ],
                    vec![],
                ],
            ),
            report(12, vec![vec![guard(11, Some(false))]]),
            report(13, vec![]),
        ];
        let routes = planning_routes(&reports, &plan);
        assert_eq!(routes[0].candidates.len(), 2);
        let first = &routes[0].candidates[0];
        assert_eq!(first.requires, vec![1]);
        assert_eq!(first.untraced_conditions.len(), 1);
        assert_eq!(first.untraced_conditions[0].condition.kind, "bag_item");
        assert!(first.recursive);
        assert!(routes[1].candidates[0].recursive);
        let alternate = &routes[0].candidates[1];
        assert!(alternate.requires.is_empty());
        assert!(!alternate.recursive);
        assert!(routes[2].candidates.is_empty());
        assert!(routes.iter().all(|r| r.goals.is_empty()));
    }
}
