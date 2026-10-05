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
}
const OPCODES: [u8; 8] = [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f];
pub const EMERALD: Rules = Rules {
    commands: 0x1db67c,
    handlers: [
        0x99720, 0x99914, 0x9993c, 0x99744, 0x99770, 0x99c14, 0x99c28, 0x99644,
    ],
};
pub const ROCKET: Rules = Rules {
    commands: 0x22b218,
    handlers: [
        0xcf610, 0xcf804, 0xcf82c, 0xcf634, 0xcf660, 0xcfb5c, 0xcfb70, 0xcf534,
    ],
};
pub const MERCURY: Rules = Rules {
    commands: 0x15f9b4,
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
    pub effects: Vec<Effect>,
    pub text: Vec<TextReference>,
    pub stopped_at: Vec<usize>,
    pub complete: bool,
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
    pub entrances: Vec<crate::collection::EntranceSuggestion>,
    pub skipped_conditions: usize,
    pub truncated: bool,
    pub partial: bool,
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
                .filter(|g| g.satisfied != Some(true))
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
        Ok(Bundle {
            reports,
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
        for reference in references {
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
            writers,
            coverage,
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
