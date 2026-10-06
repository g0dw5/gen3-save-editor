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
    /// Standard jump/call, return, wait/message, yes/no, delay and close-message.
    /// Addresses identify verified native behavior, never standard-script content.
    pub presentation_handlers: [usize; 8],
    /// Optional native message-preparation gate in the standard-call dispatcher.
    pub standard_call_hook: Option<usize>,
    /// Verified player-gender reader, SaveBlock2 pointer and VAR_RESULT address.
    pub player_gender: (usize, u32, u32),
    /// Species/item/move/number/literal string-buffer handlers, independently verified.
    pub buffer_handlers: [usize; 5],
    /// Native standalone string variables; never names or extracted dialogue.
    pub buffer_destinations: [u32; 3],
    /// Optional native item-format branch reading an unsaved/custom berry name.
    /// Read the compared ID from this ROM instruction; do not bundle item content.
    pub buffer_item_dynamic_compare: Option<usize>,
    /// Native applymovement/applymovementat/waitmovement/waitmovementat.
    /// Identifies commands; does not certify background actions or current tiles.
    pub movement_handlers: [usize; 4],
}
const OPCODES: [u8; 8] = [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f];
pub const EMERALD: Rules = Rules {
    movement_handlers: [0x9a5e8, 0x9a62c, 0x9a698, 0x9a6ec],
    buffer_item_dynamic_compare: None,
    buffer_destinations: [0x02021cc4, 0x02021dc4, 0x02021ec4],
    buffer_handlers: [0x9afbc, 0x9b090, 0x9b150, 0x9b190, 0x9b248],
    player_gender: (0x9b88c, 0x03005d90, 0x020375f0),
    standard_call_hook: None,
    presentation_handlers: [
        0x99508, 0x99538, 0x99380, 0x9ac78, 0x9abd4, 0x9acd4, 0x99db4, 0x9ac8c,
    ],
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
    movement_handlers: [0xd0530, 0xd0574, 0xd05e0, 0xd0634],
    buffer_item_dynamic_compare: None,
    buffer_destinations: EMERALD.buffer_destinations,
    buffer_handlers: [0xd0ef0, 0xd0fc4, 0xd1084, 0xd10c4, 0xd117c],
    player_gender: (0xd185c, 0x03005250, 0x020385b0),
    standard_call_hook: None,
    presentation_handlers: [
        0xcf3f8, 0xcf428, 0xcf270, 0xd0bac, 0xd0b08, 0xd0c08, 0xcfcfc, 0xd0bc0,
    ],
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
    movement_handlers: [0x6b200, 0x6b244, 0x6b2b0, 0x6b304],
    buffer_item_dynamic_compare: Some(0x99e98),
    buffer_destinations: [0x02021cd0, 0x02021cf0, 0x02021d04],
    buffer_handlers: [0x6bc88, 0x6bd5c, 0x6be50, 0x6be90, 0x6bf14],
    player_gender: (0x6c4f0, 0x0300500c, 0x020370d0),
    standard_call_hook: None,
    presentation_handlers: [
        0x6a150, 0x6a180, 0x69fc8, 0x6b878, 0x1d5df26, 0x6ba80, 0x6a9b0, 0x6b88c,
    ],
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
pub const EXPANDED: Rules = Rules {
    standard_call_hook: Some(0x14a332c),
    ..EMERALD
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
    pub fn navigation_graph(
        &self,
        rom: &Rom,
        maps: &[Map],
        save: Option<&Save>,
    ) -> Result<crate::navigation::Graph> {
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
        Ok(crate::navigation::Graph {
            edges,
            diagnostics,
            coverage: self.coverage.clone(),
        })
    }

    pub fn map_navigation(
        &self,
        rom: &Rom,
        maps: &[Map],
        id: &str,
        save: Option<&Save>,
    ) -> Result<crate::navigation::MapNavigation> {
        let graph = self.navigation_graph(rom, maps, save)?;
        crate::navigation::report(maps, &graph.edges, graph.diagnostics, id)
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
