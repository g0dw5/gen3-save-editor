//! Shared command surface for the desktop shell and the opt-in development server.
use crate::{
    err,
    pokemon::Policy,
    rom::{Rom, RomEdit},
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
            "profiles" => Ok(serde_json::to_value(crate::profile::PROFILES)?),
            "open_rom" => {
                let (data, _) = read_file(&p)?;
                let session = Session::new(Rom::open(data)?);
                let catalog = session.rom.catalog()?;
                self.session = Some(session);
                Ok(json!({"catalog":catalog,"save":null}))
            }
            "open_save" => {
                let (data, path) = read_file(&p)?;
                let s = self.session_mut()?;
                s.load(data, path)?;
                Ok(serde_json::to_value(s.snapshot()?)?)
            }
            "state" => {
                if let Some(s) = &self.session {
                    Ok(
                        json!({"catalog":s.rom.catalog()?,"save":if s.save.is_some(){Some(s.snapshot()?)}else{None}}),
                    )
                } else {
                    Ok(json!({"catalog":null,"save":null}))
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
                s.save_ref()?.validate(&s.rom)?;
                Ok(json!({"bytes":STANDARD.encode(&s.save_ref()?.data)}))
            }
            "species" => {
                let id = serde_json::from_value(p["id"].clone())?;
                Ok(serde_json::to_value(self.session()?.rom.detail(id)?)?)
            }
            "world" => {
                let r = &self.session()?.rom;
                Ok(serde_json::to_value(r.world()?)?)
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
                    json!({"url":format!("data:image/png;base64,{}",STANDARD.encode(self.session()?.rom.map_image(&id)?))}),
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
            "patch_rom" => {
                let edits: Vec<RomEdit> = serde_json::from_value(p["edits"].clone())?;
                let r = &self.session()?.rom;
                let (bytes, manifest) = r.patch(&edits)?;
                if let Some(path) = p["path"].as_str() {
                    let path = PathBuf::from(path);
                    if path.exists() && crate::binary::hash(&fs::read(&path)?) == r.profile.md5 {
                        return Err(err("rom_overwrite", "choose a separate file"));
                    }
                    atomic_write(&path, &bytes, |_| Ok(()))?;
                    atomic_write(
                        &path.with_extension("patch.json"),
                        &serde_json::to_vec_pretty(&manifest)?,
                        |_| Ok(()),
                    )?;
                    Ok(json!({"manifest":manifest}))
                } else {
                    Ok(json!({"manifest":manifest,"bytes":STANDARD.encode(bytes)}))
                }
            }
            _ => Err(err("command", input.command)),
        }
    }
}
