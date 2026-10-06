//! Shared command surface for the desktop shell and the opt-in development server.
use crate::{
    err,
    pokemon::Policy,
    rom::Rom,
    save::Location,
    session::{atomic_write, Action, PokemonFile, Session},
    Result,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
pub struct Request {
    pub command: String,
    #[serde(default)]
    pub payload: Value,
}
#[derive(Default)]
pub struct App {
    pub session: Option<Session>,
    pub(crate) event_dependency_cache:
        Option<(std::sync::Arc<Vec<u8>>, crate::event_dependencies::Index)>,
    pub(crate) acquisition_cache: Option<(
        std::sync::Arc<Vec<u8>>,
        crate::acquisition::AcquisitionIndex,
    )>,
    pub(crate) editor_cheat_cache: Option<(std::sync::Arc<Vec<u8>>, crate::cheats::CheatRom)>,
}
fn required(v: &Value, key: &str) -> Result<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| err("arguments", key))
}
fn read_file(p: &Value) -> Result<(Vec<u8>, Option<PathBuf>)> {
    if let Some(s) = p.get("bytes").and_then(Value::as_str) {
        if s.len() > 48_000_000 {
            return Err(err("file_size", s.len()));
        }
        Ok((STANDARD.decode(s).map_err(|e| err("base64", e))?, None))
    } else {
        let path = PathBuf::from(required(p, "path")?);
        Ok((fs::read(&path)?, Some(path)))
    }
}
impl App {
    fn session(&self) -> Result<&Session> {
        self.session
            .as_ref()
            .ok_or_else(|| err("no_rom", "open a supported ROM"))
    }
    fn session_mut(&mut self) -> Result<&mut Session> {
        self.session
            .as_mut()
            .ok_or_else(|| err("no_rom", "open a supported ROM"))
    }
    pub fn dispatch(&mut self, input: Request) -> Result<Value> {
        let p = input.payload;
        match input.command.as_str() {
            "cheats" | "cheat_code" => {
                let md5 = required(&p, "expected_rom_md5")?;
                let rom =
                    if let Some(s) = self.session.as_ref().filter(|s| s.rom.profile.md5 == md5) {
                        if let Some((_, cached)) = self
                            .editor_cheat_cache
                            .as_ref()
                            .filter(|(data, _)| std::sync::Arc::ptr_eq(data, &s.rom.data))
                        {
                            cached.clone()
                        } else {
                            let rom = crate::cheats::CheatRom::open(&s.rom.data)?;
                            self.editor_cheat_cache = Some((s.rom.data.clone(), rom.clone()));
                            rom
                        }
                    } else {
                        return Err(err(
                            "cheat_context_mismatch",
                            "load the matching ROM before requesting cheats",
                        ));
                    };
                if input.command == "cheats" {
                    Ok(serde_json::to_value(rom.catalog())?)
                } else {
                    let request = serde_json::from_value(p)?;
                    Ok(serde_json::to_value(rom.generate(&request)?)?)
                }
            }
            "profiles" => Ok(serde_json::to_value(crate::profile::PROFILES)?),
            "open_rom" => {
                let (data, _) = read_file(&p)?;
                let session = Session::new(Rom::open(data)?);
                let catalog = session.rom.catalog()?;
                self.session = Some(session);
                self.editor_cheat_cache = None;
                self.acquisition_cache = None;
                self.event_dependency_cache = None;
                Ok(json!({"catalog":catalog,"save":null}))
            }
            "open_save" => {
                let (data, path) = read_file(&p)?;
                let s = self.session_mut()?;
                s.load(data, path)?;
                Ok(serde_json::to_value(s.snapshot()?)?)
            }
            "state" => {
                let profiles: Vec<_> = crate::profile::PROFILES
                    .iter()
                    .map(|profile| {
                        json!({
                            "id": profile.id,
                            "label": profile.label,
                            "md5": profile.md5,
                        })
                    })
                    .collect();
                if let Some(s) = &self.session {
                    Ok(
                        json!({"catalog":s.rom.catalog()?,"save":if s.save.is_some(){Some(s.snapshot()?)}else{None},"profiles":profiles}),
                    )
                } else {
                    Ok(json!({"catalog":null,"save":null,"profiles":profiles}))
                }
            }
            "action" => {
                let action: Action = serde_json::from_value(
                    p.get("action")
                        .cloned()
                        .ok_or_else(|| err("arguments", "action"))?,
                )?;
                let policy: Policy =
                    serde_json::from_value(p.get("policy").cloned().unwrap_or(json!("standard")))?;
                let s = self.session_mut()?;
                let change = s.apply(action, policy)?;
                Ok(json!({"save":s.snapshot()?,"change":change}))
            }
            "undo" | "redo" => {
                let s = self.session_mut()?;
                if input.command == "undo" {
                    s.undo()?;
                } else {
                    s.redo()?;
                }
                Ok(serde_json::to_value(s.snapshot()?)?)
            }
            "export_save" => {
                let path = PathBuf::from(required(&p, "path")?);
                let s = self.session_mut()?;
                let backup = s.export(&path)?;
                Ok(json!({"save":s.snapshot()?,"backup":backup,"path":path}))
            }
            "save_bytes" => {
                let s = self.session()?;
                s.rom
                    .profile
                    .capabilities
                    .require(s.rom.profile.capabilities.save_edit, "save_edit")?;
                s.save_ref()?.validate(&s.rom)?;
                Ok(json!({"bytes":STANDARD.encode(&s.save_ref()?.data)}))
            }
            "species" => {
                let id = serde_json::from_value(p["id"].clone())?;
                Ok(serde_json::to_value(self.session()?.rom.detail(id)?)?)
            }
            "event_dependencies" | "event_search" | "trainer_references" => {
                let expected = required(&p, "expected_rom_md5")?;
                let search_request = if input.command == "event_search" {
                    Some(serde_json::from_value::<
                        crate::event_dependencies::SearchRequest,
                    >(p.clone())?)
                } else {
                    None
                };
                let trainer_request = if input.command == "trainer_references" {
                    Some(serde_json::from_value::<
                        crate::event_dependencies::TrainerRequest,
                    >(p.clone())?)
                } else {
                    None
                };
                let dependency_request = if input.command == "event_dependencies" {
                    Some(serde_json::from_value::<crate::event_dependencies::Request>(p)?)
                } else {
                    None
                };
                let session = self.session()?;
                if expected != session.rom.profile.md5 {
                    return Err(err("rom_mismatch", expected));
                }
                if self
                    .event_dependency_cache
                    .as_ref()
                    .is_none_or(|(data, _)| !std::sync::Arc::ptr_eq(data, &session.rom.data))
                {
                    let maps = session.rom.maps()?;
                    let index = crate::event_dependencies::Index::build(&session.rom, &maps)?;
                    self.event_dependency_cache = Some((session.rom.data.clone(), index));
                }
                let session = self.session()?;
                let index = &self.event_dependency_cache.as_ref().unwrap().1;
                if let Some(request) = trainer_request {
                    return Ok(serde_json::to_value(index.trainer(
                        &session.rom,
                        session.save.as_ref(),
                        request,
                    )?)?);
                }
                if let Some(request) = search_request {
                    return Ok(serde_json::to_value(index.search(
                        &session.rom,
                        session.save.as_ref(),
                        request,
                    )?)?);
                }
                let request = dependency_request.expect("validated dependency request");
                Ok(serde_json::to_value(index.query(
                    &session.rom,
                    session.save.as_ref(),
                    request,
                )?)?)
            }
            "world" | "acquisition" | "collection" | "collection_export" | "daycare_sources" => {
                let session = self.session()?;
                if self
                    .acquisition_cache
                    .as_ref()
                    .is_none_or(|(data, _)| !std::sync::Arc::ptr_eq(data, &session.rom.data))
                {
                    let index = crate::acquisition::AcquisitionIndex::build(&session.rom)?;
                    self.acquisition_cache = Some((session.rom.data.clone(), index));
                }
                let index = &self.acquisition_cache.as_ref().unwrap().1;
                if input.command == "world" {
                    Ok(serde_json::to_value(&index.world)?)
                } else if input.command == "daycare_sources" {
                    let session = self.session()?;
                    Ok(serde_json::to_value(
                        index.daycare_sources(&session.rom, session.save.as_ref()),
                    )?)
                } else if input.command == "collection_export" {
                    #[derive(Deserialize)]
                    #[serde(deny_unknown_fields)]
                    struct Input {
                        expected_rom_md5: String,
                        query: crate::collection::CollectionRequest,
                    }
                    let request: Input = serde_json::from_value(p)?;
                    let session = self.session()?;
                    if request.expected_rom_md5 != session.rom.profile.md5 {
                        return Err(err("rom_mismatch", request.expected_rom_md5));
                    }
                    let mut plan =
                        index.collection(&session.rom, session.save_ref()?, request.query)?;
                    if self
                        .event_dependency_cache
                        .as_ref()
                        .is_none_or(|(data, _)| !std::sync::Arc::ptr_eq(data, &session.rom.data))
                    {
                        let deps = crate::event_dependencies::Index::build(
                            &session.rom,
                            &index.world.maps,
                        )?;
                        self.event_dependency_cache = Some((session.rom.data.clone(), deps));
                    }
                    let session = self.session()?;
                    plan.prerequisites =
                        Some(self.event_dependency_cache.as_ref().unwrap().1.trace_plan(
                            &session.rom,
                            session.save_ref()?,
                            &self.acquisition_cache.as_ref().unwrap().1.world.maps,
                            &plan,
                        )?);
                    Ok(serde_json::to_value(plan)?)
                } else if input.command == "collection" {
                    let request = serde_json::from_value(p)?;
                    let session = self.session()?;
                    Ok(serde_json::to_value(index.collection(
                        &session.rom,
                        session.save_ref()?,
                        request,
                    )?)?)
                } else {
                    #[derive(Deserialize)]
                    #[serde(deny_unknown_fields)]
                    struct Input {
                        kind: crate::acquisition::TargetKind,
                        id: u16,
                        hour: Option<u8>,
                        #[serde(default)]
                        use_save_clock: bool,
                    }
                    let query: Input = serde_json::from_value(p)?;
                    let session = self.session()?;
                    Ok(serde_json::to_value(index.query_scenario(
                        &session.rom,
                        session.save.as_ref(),
                        crate::acquisition::Target {
                            kind: query.kind,
                            id: query.id,
                        },
                        query.hour,
                        query.use_save_clock,
                    )?)?)
                }
            }
            "breeding_preview" => {
                let session = self.session()?;
                Ok(serde_json::to_value(crate::breeding::preview(
                    &session.rom,
                    session.save.as_ref(),
                    &serde_json::from_value(p)?,
                )?)?)
            }
            "daycare_state" => {
                let session = self.session()?;
                let state = session
                    .save
                    .as_ref()
                    .map(|s| crate::daycare_state::snapshot(&session.rom, s))
                    .transpose()?
                    .flatten();
                Ok(serde_json::to_value(state)?)
            }
            "trainer_native_preview" => Ok(serde_json::to_value(crate::native_trainer::preview(
                &self.session()?.rom,
                &serde_json::from_value(p)?,
            )?)?),
            "trainer_ev_preview" => {
                let request: crate::ultimate_ev::TrainerEvRequest = serde_json::from_value(p)?;
                Ok(serde_json::to_value(crate::ultimate_ev::preview(
                    &self.session()?.rom,
                    &request,
                )?)?)
            }
            "trainer_battle_preview" => {
                let request: crate::ultimate_battle::TrainerBattleRequest =
                    serde_json::from_value(p)?;
                Ok(serde_json::to_value(crate::ultimate_battle::preview(
                    &self.session()?.rom,
                    &request,
                )?)?)
            }
            "contest_check" => {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Input {
                    condition: [u8; 6],
                    nature: u8,
                    expected_rom_md5: String,
                }
                let p: Input = serde_json::from_value(p)?;
                let r = &self.session()?.rom;
                if r.profile.md5 != p.expected_rom_md5 {
                    return Err(err("rom_mismatch", "contest check"));
                }
                Ok(serde_json::to_value(crate::contest::check_npc(
                    r,
                    p.nature,
                    p.condition,
                )?)?)
            }
            "clock_rtc_preview" => {
                let session = self.session()?;
                Ok(serde_json::to_value(session.rom.hardware_clock_preview(
                    session.save.as_ref(),
                    serde_json::from_value(p)?,
                )?)?)
            }
            "clock_query" => {
                let session = self.session()?;
                Ok(serde_json::to_value(session.rom.clock_query_with_save(
                    session.save.as_ref(),
                    serde_json::from_value(p)?,
                )?)?)
            }
            "fishing_spots" => {
                let s = self.session()?;
                Ok(serde_json::to_value(s.rom.fishing_spots(s.save.as_ref())?)?)
            }
            "map_navigation" => {
                let id = required(&p, "id")?;
                Ok(serde_json::to_value(
                    self.session()?.rom.map_navigation(&id)?,
                )?)
            }
            "map_report" => {
                let id = required(&p, "id")?;
                let r = &self.session()?.rom;
                let m = r
                    .maps()?
                    .into_iter()
                    .find(|m| m.id == id)
                    .ok_or_else(|| err("map_id", id))?;
                Ok(serde_json::to_value(r.script_report(&m)?)?)
            }
            "sprite" => {
                let id = serde_json::from_value(p["id"].clone())?;
                let shiny = p["shiny"].as_bool().unwrap_or(false);
                let pid: u32 = match p.get("pid") {
                    None => 0,
                    Some(value) => serde_json::from_value(value.clone())?,
                };
                Ok(
                    json!({"url":format!("data:image/png;base64,{}",STANDARD.encode(self.session()?.rom.pokemon_sprite(id,shiny,pid)?))}),
                )
            }
            "trainer_sprite" | "object_sprite" => {
                let id = serde_json::from_value(p["id"].clone())?;
                let rom = &self.session()?.rom;
                let png = if input.command == "trainer_sprite" {
                    rom.trainer_sprite(id)?
                } else {
                    rom.object_sprite(id)?
                };
                Ok(json!({"url":format!("data:image/png;base64,{}", STANDARD.encode(png))}))
            }
            "map_image" => {
                let id = required(&p, "id")?;
                Ok(
                    json!({"url":format!("data:image/png;base64,{}",STANDARD.encode(self.session()?.rom.map_image(&id)?)),
                        "warnings": self.session()?.rom.map_image_warnings(&id)?}),
                )
            }
            "export_pokemon" => {
                let loc: Location = serde_json::from_value(p["location"].clone())?;
                let file = self.session()?.export_pokemon(loc)?;
                if let Some(path) = p["path"].as_str() {
                    atomic_write(
                        &PathBuf::from(path),
                        &serde_json::to_vec_pretty(&file)?,
                        |_| Ok(()),
                    )?;
                }
                Ok(serde_json::to_value(file)?)
            }
            "import_pokemon" => {
                let (data, _) = read_file(&p)?;
                let file: PokemonFile = serde_json::from_slice(&data)?;
                if file.format != "gen3-pokemon-1" {
                    return Err(err("pokemon_format", file.format));
                }
                let loc = serde_json::from_value(p["location"].clone())?;
                let s = self.session_mut()?;
                s.apply(
                    Action::Import {
                        location: loc,
                        rom_md5: file.rom_md5,
                        bytes: file.bytes,
                    },
                    Policy::Free,
                )?;
                Ok(serde_json::to_value(s.snapshot()?)?)
            }
            _ => Err(err("command", input.command)),
        }
    }
}
