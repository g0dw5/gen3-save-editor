//! Referenced ROM topology. An edge is never a proof of current access.
use crate::{
    binary::{bytes, pointer, u16, u32},
    err,
    rom::Rom,
    world::Map,
    Result,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const WARP_OPCODES: [u8; 7] = [0x39, 0x3a, 0x3b, 0x3d, 0x3e, 0xd1, 0xd7];

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ScriptWarp {
    pub opcode: u8,
    pub offset: usize,
    pub group: u8,
    pub number: u8,
    pub warp: u8,
    pub x: Option<i16>,
    pub y: Option<i16>,
    pub conditions: Vec<crate::map_events::EventCondition>,
}

impl Rom {
    pub(crate) fn script_warp_instruction(
        &self,
        pc: usize,
        resolve: impl Fn(u16) -> Option<u16>,
    ) -> Result<Option<ScriptWarp>> {
        let opcode = bytes(&self.data, pc, 1)?[0];
        let Some(index) = WARP_OPCODES.iter().position(|v| *v == opcode) else {
            return Ok(None);
        };
        // setwarp only stores a destination. No edge is implied until a native
        // transition consumes it; dynamic/dive/hole/escape setters are excluded.
        if opcode == 0x3e {
            return Ok(None);
        }
        let rules = self
            .profile
            .event_state
            .and_then(|r| r.effects)
            .ok_or_else(|| err("script_warp_unverified", pc))?;
        let handler = rules.warp_handlers[index];
        if handler == 0
            || (pointer(&self.data, rules.commands + opcode as usize * 4)? & !1) != handler
        {
            return Err(err("script_warp_dispatch", pc));
        }
        let code = bytes(&self.data, pc, 8)?;
        // Native VarGet(u16), then s8 narrowing before SetWarpDestination.
        // Preserve unknown variables; never resolve them from a different SAV.
        let coordinate = |at| -> Result<Option<i16>> {
            Ok(resolve(u16(code, at)?).map(|v| v as u8 as i8 as i16))
        };
        Ok(Some(ScriptWarp {
            opcode,
            offset: pc,
            group: code[1],
            number: code[2],
            warp: code[3],
            x: coordinate(4)?,
            y: coordinate(6)?,
            conditions: Vec::new(),
        }))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ScriptPassage {
    pub root: usize,
    pub source_kind: &'static str,
    pub local_id: Option<u8>,
    pub opcode: u8,
    pub conditions: Vec<crate::map_events::EventCondition>,
    pub checks: Vec<crate::acquisition::ConditionCheck>,
    pub stopped_at: Vec<usize>,
    /// Activation selectors, movement and branch execution remain unproven.
    pub entry_unresolved: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct MapLink {
    pub from: String,
    pub to: Option<String>,
    pub kind: &'static str,
    pub x: Option<i16>,
    pub y: Option<i16>,
    pub target_x: Option<i16>,
    pub target_y: Option<i16>,
    pub warp_index: Option<u8>,
    pub target_warp: Option<u8>,
    pub direction: Option<u32>,
    pub displacement: Option<i32>,
    pub offset: usize,
    pub unresolved: Option<&'static str>,
    pub script: Option<ScriptPassage>,
}
#[derive(Serialize)]
pub struct MapNavigation {
    pub map_id: String,
    pub outgoing: Vec<MapLink>,
    pub incoming: Vec<MapLink>,
    /// Ordered from an exterior map to the selected map; alternatives are bounded.
    pub approaches: Vec<Vec<MapLink>>,
    pub truncated: bool,
    pub diagnostics: Vec<String>,
}

pub struct Graph {
    pub edges: Vec<MapLink>,
    pub diagnostics: Vec<String>,
    pub coverage: crate::event_dependencies::Coverage,
}

pub(crate) fn links(data: &[u8], maps: &[Map]) -> Result<(Vec<MapLink>, Vec<String>)> {
    let by_id: BTreeMap<_, _> = maps.iter().map(|m| (m.id.clone(), m)).collect();
    let mut all = Vec::new();
    let mut diagnostics = Vec::new();
    for map in maps {
        if map.invalid_events {
            diagnostics.push(format!("{}: invalid event table", map.id));
        }
        if let Some(ev) = map.events {
            let count = bytes(data, ev + 1, 1)?[0];
            if count > 0 {
                let p = pointer(data, ev + 8)?;
                bytes(data, p, count as usize * 8)?;
                for i in 0..count {
                    let o = p + i as usize * 8;
                    let w = bytes(data, o, 8)?;
                    let id = format!("{}-{}", w[7], w[6]);
                    let target = by_id.get(&id);
                    let mut link = MapLink {
                        from: map.id.clone(),
                        to: target.map(|_| id),
                        kind: "warp",
                        x: Some(u16(data, o)? as i16),
                        y: Some(u16(data, o + 2)? as i16),
                        target_x: None,
                        target_y: None,
                        warp_index: Some(i),
                        target_warp: Some(w[5]),
                        direction: None,
                        displacement: None,
                        offset: o,
                        script: None,
                        unresolved: if target.is_none() {
                            Some("dynamic_or_missing_map")
                        } else {
                            None
                        },
                    };
                    if let Some(target) = target {
                        if let Some(te) = target.events {
                            let n = bytes(data, te + 1, 1)?[0];
                            if w[5] < n {
                                let tw = pointer(data, te + 8)? + w[5] as usize * 8;
                                link.target_x = Some(u16(data, tw)? as i16);
                                link.target_y = Some(u16(data, tw + 2)? as i16);
                                if link
                                    .target_x
                                    .is_some_and(|x| x < 0 || x as u32 >= target.width)
                                    || link
                                        .target_y
                                        .is_some_and(|y| y < 0 || y as u32 >= target.height)
                                {
                                    link.unresolved = Some("target_outside_layout");
                                }
                            } else {
                                link.unresolved = Some("dynamic_or_missing_warp");
                            }
                        } else {
                            link.unresolved = Some("missing_target_events");
                        }
                    }
                    if link.x.is_some_and(|x| x < 0 || x as u32 >= map.width)
                        || link.y.is_some_and(|y| y < 0 || y as u32 >= map.height)
                    {
                        link.unresolved = Some("source_outside_layout");
                    }
                    all.push(link);
                }
            }
        }
        // Native MapHeader +12 -> {count, pointer}, 12-byte connection entries.
        if let Ok(c) = pointer(data, map.header + 12) {
            let count = u32(data, c)? as usize;
            if count > 64 {
                diagnostics.push(format!("{}: invalid connection count {count}", map.id));
                continue;
            }
            if count == 0 {
                continue;
            }
            let Ok(p) = pointer(data, c + 4) else {
                diagnostics.push(format!("{}: invalid connection pointer", map.id));
                continue;
            };
            if bytes(data, p, count * 12).is_err() {
                diagnostics.push(format!("{}: truncated connections", map.id));
                continue;
            }
            for i in 0..count {
                let o = p + i * 12;
                let entry = bytes(data, o, 12)?;
                let id = format!("{}-{}", entry[8], entry[9]);
                let direction = u32(data, o)?;
                all.push(MapLink {
                    from: map.id.clone(),
                    to: by_id.contains_key(&id).then_some(id),
                    kind: "connection",
                    x: None,
                    y: None,
                    target_x: None,
                    target_y: None,
                    warp_index: None,
                    target_warp: None,
                    direction: Some(direction),
                    displacement: Some(u32(data, o + 4)? as i32),
                    offset: o,
                    script: None,
                    unresolved: if !by_id.contains_key(&format!("{}-{}", entry[8], entry[9])) {
                        Some("dynamic_or_missing_map")
                    } else if !(1..=6).contains(&direction) {
                        Some("unknown_direction")
                    } else {
                        None
                    },
                });
            }
        }
    }
    Ok((all, diagnostics))
}

pub(crate) fn script_link(
    data: &[u8],
    maps: &[Map],
    reference: &crate::event_dependencies::Reference,
    warp: &ScriptWarp,
    stopped_at: &[usize],
) -> Result<MapLink> {
    let id = format!("{}-{}", warp.group, warp.number);
    // Negative native group/number operands remain unresolved here. Do not
    // reinterpret them as unsigned map-table indices to fabricate a destination.
    let target = maps
        .iter()
        .find(|m| m.id == id && warp.group < 128 && warp.number < 128);
    let mut link = MapLink {
        from: reference.map_id.clone(),
        to: target.map(|_| id),
        kind: "script_warp",
        x: reference.x,
        y: reference.y,
        target_x: None,
        target_y: None,
        warp_index: None,
        target_warp: (warp.warp != 255).then_some(warp.warp),
        direction: None,
        displacement: None,
        offset: warp.offset,
        unresolved: target.is_none().then_some("dynamic_or_missing_map"),
        script: Some(ScriptPassage {
            root: reference.root,
            source_kind: reference.kind,
            local_id: reference.local_id,
            opcode: warp.opcode,
            conditions: reference
                .conditions
                .iter()
                .chain(&warp.conditions)
                .cloned()
                .collect(),
            checks: Vec::new(),
            stopped_at: stopped_at.to_vec(),
            entry_unresolved: true,
        }),
    };
    if let Some(target) = target {
        if warp.warp == 255 {
            link.target_x = warp.x;
            link.target_y = warp.y;
            if warp.x.is_none() || warp.y.is_none() {
                link.unresolved = Some("dynamic_coordinates");
            }
        } else if warp.warp >= 128 {
            link.unresolved = Some("dynamic_or_missing_warp");
        } else if let Some(events) = target.events {
            if warp.warp < bytes(data, events + 1, 1)?[0] {
                let entry = pointer(data, events + 8)? + warp.warp as usize * 8;
                link.target_x = Some(u16(data, entry)? as i16);
                link.target_y = Some(u16(data, entry + 2)? as i16);
            } else {
                link.unresolved = Some("dynamic_or_missing_warp");
            }
        } else {
            link.unresolved = Some("missing_target_events");
        }
        if link
            .target_x
            .is_some_and(|v| v < 0 || v as u32 >= target.width)
            || link
                .target_y
                .is_some_and(|v| v < 0 || v as u32 >= target.height)
        {
            link.unresolved = Some("target_outside_layout");
        }
    }
    Ok(link)
}

pub(crate) fn approaches(maps: &[Map], all: &[MapLink], id: &str) -> (Vec<Vec<MapLink>>, bool) {
    let outdoor: BTreeSet<_> = maps
        .iter()
        .filter(|m| matches!(m.map_type, 1..=3))
        .map(|m| m.id.as_str())
        .collect();
    if outdoor.contains(id) {
        return (Vec::new(), false);
    }
    let mut incoming: BTreeMap<&str, Vec<&MapLink>> = BTreeMap::new();
    for link in all.iter().filter(|e| e.unresolved.is_none()) {
        if let Some(to) = &link.to {
            incoming.entry(to).or_default().push(link);
        }
    }
    let mut queue = VecDeque::from([(
        id.to_owned(),
        Vec::<MapLink>::new(),
        BTreeSet::from([id.to_owned()]),
    )]);
    let mut paths = Vec::new();
    let mut steps = 0;
    let mut truncated = false;
    while let Some((current, path, seen)) = queue.pop_front() {
        steps += 1;
        if steps > 2048 || paths.len() >= 8 {
            truncated = true;
            break;
        }
        if path.len() >= 12 {
            truncated = true;
            continue;
        }
        for edge in incoming.get(current.as_str()).into_iter().flatten() {
            if seen.contains(&edge.from) {
                continue;
            }
            let mut next = path.clone();
            next.push((*edge).clone());
            if outdoor.contains(edge.from.as_str()) {
                next.reverse();
                paths.push(next);
            } else {
                let mut visited = seen.clone();
                visited.insert(edge.from.clone());
                queue.push_back((edge.from.clone(), next, visited));
            }
        }
    }
    (paths, truncated)
}
impl Rom {
    pub fn map_navigation(&self, id: &str) -> Result<MapNavigation> {
        let maps = self.maps()?;
        let index = crate::event_dependencies::Index::build(self, &maps)?;
        index.map_navigation(self, &maps, id, None)
    }
}

pub(crate) fn report(
    maps: &[Map],
    all: &[MapLink],
    diagnostics: Vec<String>,
    id: &str,
) -> Result<MapNavigation> {
    if !maps.iter().any(|m| m.id == id) {
        return Err(err("map_id", id));
    }
    let (approaches, truncated) = approaches(maps, all, id);
    Ok(MapNavigation {
        map_id: id.to_owned(),
        outgoing: all.iter().filter(|e| e.from == id).cloned().collect(),
        incoming: all
            .iter()
            .filter(|e| e.to.as_deref() == Some(id))
            .cloned()
            .collect(),
        approaches,
        truncated,
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn map(id: &str, kind: u8, header: usize, events: Option<usize>) -> Map {
        Map {
            id: id.into(),
            group: 0,
            number: 0,
            name: id.into(),
            region: 0,
            width: 20,
            height: 20,
            map_type: kind,
            header,
            layout: 0,
            events,
            invalid_events: false,
            scripts: vec![],
            objects: vec![],
        }
    }
    fn ptr(data: &mut [u8], at: usize, to: usize) {
        data[at..at + 4].copy_from_slice(&(0x08000000 + to as u32).to_le_bytes());
    }
    #[test]
    fn entrances_dynamic_targets_and_cycles() {
        let mut data = vec![0; 512];
        let maps = vec![
            map("0-0", 1, 0, Some(96)),
            map("0-1", 4, 32, Some(116)),
            map("0-2", 4, 64, Some(136)),
        ];
        for (ev, p, count) in [(96, 160, 2), (116, 176, 2), (136, 192, 1)] {
            data[ev + 1] = count;
            ptr(&mut data, ev + 8, p);
        }
        for (o, x, y, dest, warp) in [
            (160, 3, 4, 1, 0),
            (168, 7, 8, 255, 255),
            (176, 1, 2, 0, 0),
            (184, 5, 6, 2, 0),
            (192, 9, 10, 1, 1),
        ] {
            data[o] = x;
            data[o + 2] = y;
            data[o + 5] = warp;
            data[o + 6] = dest;
        }
        let (edges, diagnostics) = links(&data, &maps).unwrap();
        assert!(diagnostics.is_empty());
        assert_eq!(edges[0].target_x, Some(1));
        assert_eq!(edges[1].to, None);
        assert_eq!(edges[1].unresolved, Some("dynamic_or_missing_map"));
        let (paths, truncated) = approaches(&maps, &edges, "0-2");
        assert!(!truncated);
        assert_eq!(paths.len(), 1);
        assert_eq!(
            paths[0].iter().map(|e| e.from.as_str()).collect::<Vec<_>>(),
            ["0-0", "0-1"]
        );
    }
    #[test]
    fn malformed_connection_is_reported_not_followed() {
        let mut data = vec![0; 128];
        ptr(&mut data, 12, 64);
        data[64..68].copy_from_slice(&1000u32.to_le_bytes());
        let (edges, diagnostics) = links(&data, &[map("0-0", 1, 0, None)]).unwrap();
        assert!(edges.is_empty());
        assert_eq!(diagnostics.len(), 1);
    }
    #[test]
    #[ignore = "requires all five exact local ROMs"]
    fn local_query_navigation_all_profiles() {
        for name in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
            let rom = Rom::open(
                std::fs::read(std::env::var(format!("GEN3_ROM_{name}")).unwrap()).unwrap(),
            )
            .unwrap();
            let maps = rom.maps().unwrap();
            let (edges, diagnostics) = links(&rom.data, &maps).unwrap();
            assert!(
                edges.iter().filter(|e| e.kind == "warp").count() > 100,
                "{name}"
            );
            assert!(
                edges.iter().filter(|e| e.kind == "connection").count() > 10,
                "{name}"
            );
            for e in edges
                .iter()
                .filter(|e| e.unresolved.is_none() && e.kind == "warp")
            {
                let dest = maps.iter().find(|m| Some(&m.id) == e.to.as_ref()).unwrap();
                assert!(
                    e.target_x.unwrap() >= 0 && (e.target_x.unwrap() as u32) < dest.width,
                    "{name}: {:?}",
                    e
                );
                assert!(
                    e.target_y.unwrap() >= 0 && (e.target_y.unwrap() as u32) < dest.height,
                    "{name}: {:?}",
                    e
                );
            }
            let path_count = maps
                .iter()
                .filter(|m| !approaches(&maps, &edges, &m.id).0.is_empty())
                .count();
            assert!(path_count > 50, "{name}: {path_count}");
            eprintln!("{name}: {} maps, {} warps, {} connections, {} unresolved, {} entrance chains, {} diagnostics", maps.len(), edges.iter().filter(|e| e.kind == "warp").count(), edges.iter().filter(|e| e.kind == "connection").count(), edges.iter().filter(|e| e.unresolved.is_some()).count(), path_count, diagnostics.len());
        }
    }
}
