//! Every edit is a transaction over a cloned save. Files change only on export.
use crate::{
    err,
    pokemon::{self, Finding, PokemonPatch, Policy},
    rom::Rom,
    save::{BagEntry, BoxInfo, DexFlag, Location, Save, StoredPokemon, Trainer, TrainerPatch},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Pokemon {
        location: Location,
        patch: PokemonPatch,
    },
    Batch {
        locations: Vec<Location>,
        patch: PokemonPatch,
    },
    Create {
        location: Location,
        species: u16,
        level: u8,
        #[serde(default)]
        met_location: Option<u8>,
        #[serde(default)]
        egg: bool,
    },
    Transfer {
        from: Location,
        to: Location,
        #[serde(default)]
        copy: bool,
    },
    Delete {
        location: Location,
    },
    Trainer {
        patch: TrainerPatch,
    },
    Bag {
        pocket: String,
        slot: usize,
        item: u16,
        quantity: u16,
    },
    Box {
        index: usize,
        name: String,
        wallpaper: u8,
    },
    Sort {
        index: usize,
    },
    Dex {
        number: u16,
        seen: bool,
        owned: bool,
    },
    Import {
        location: Location,
        rom_md5: String,
        bytes: Vec<u8>,
    },
}
#[derive(Clone, Debug, Serialize)]
pub struct FieldChange {
    pub path: String,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
}
#[derive(Clone, Debug, Serialize)]
pub struct Change {
    pub fields: Vec<FieldChange>,
    pub action: Action,
    pub bytes_changed: usize,
    pub findings: Vec<Finding>,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub trainer: Trainer,
    pub pokemon: Vec<StoredPokemon>,
    pub boxes: Vec<BoxInfo>,
    pub bag: Vec<BagEntry>,
    pub dex: Vec<DexFlag>,
    pub active_slot: usize,
    pub counter: u32,
    pub backup_valid: bool,
    pub dirty: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub changes: Vec<Change>,
}
pub struct Session {
    pub rom: Rom,
    pub save: Option<Save>,
    pub source: Option<PathBuf>,
    original: Vec<u8>,
    undo: Vec<(Vec<u8>, Change)>,
    redo: Vec<(Vec<u8>, Change)>,
}
#[derive(Serialize, Deserialize)]
pub struct PokemonFile {
    pub format: String,
    pub rom_md5: String,
    pub bytes: Vec<u8>,
}
impl Session {
    pub fn new(rom: Rom) -> Self {
        Self {
            rom,
            save: None,
            source: None,
            original: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }
    pub fn load(&mut self, data: Vec<u8>, source: Option<PathBuf>) -> Result<()> {
        let save = Save::open(data.clone(), self.rom.profile.save)?;
        save.validate(&self.rom)?;
        self.original = data;
        self.save = Some(save);
        self.source = source;
        self.undo.clear();
        self.redo.clear();
        Ok(())
    }
    pub fn save_ref(&self) -> Result<&Save> {
        self.save
            .as_ref()
            .ok_or_else(|| err("no_save", "load a battery save"))
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        let s = self.save_ref()?;
        Ok(Snapshot {
            trainer: s.trainer(&self.rom)?,
            pokemon: s.all(&self.rom)?,
            boxes: s.boxes(&self.rom)?,
            bag: s.bag()?,
            dex: s.dex()?,
            active_slot: s.active_slot,
            counter: s.counter,
            backup_valid: s.backup_valid,
            dirty: s.data != self.original,
            can_undo: !self.undo.is_empty(),
            can_redo: !self.redo.is_empty(),
            changes: self.undo.iter().map(|(_, c)| c.clone()).collect(),
        })
    }
    pub fn apply(&mut self, action: Action, policy: Policy) -> Result<Change> {
        self.rom
            .profile
            .capabilities
            .require(self.rom.profile.capabilities.save_edit, "save_edit")?;
        let before = self.save_ref()?.data.clone();
        let mut save = self.save_ref()?.clone();
        let mut findings = Vec::new();
        let rom = &self.rom;
        match &action {
            Action::Pokemon { location, patch } => {
                findings = save.edit(*location, patch, rom, policy)?
            }
            Action::Batch { locations, patch } => {
                if locations.len() > 426 {
                    return Err(err("batch_size", locations.len()));
                }
                let mut seen = Vec::new();
                for loc in locations {
                    if seen.contains(loc) {
                        return Err(err("batch_duplicate", format!("{loc:?}")));
                    }
                    seen.push(*loc);
                    findings.extend(save.edit(*loc, patch, rom, policy)?);
                }
            }
            Action::Create {
                location,
                species,
                level,
                met_location,
                egg,
            } => {
                if policy == Policy::Standard && rom.is_battle_species(*species)? {
                    return Err(err("battle_species", species));
                }
                let t = save.trainer(rom)?;
                let pid = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos();
                let raw = pokemon::create(
                    rom,
                    *species,
                    (t.sid as u32) << 16 | t.tid as u32,
                    &t.name,
                    *level,
                    pid,
                )?;
                let patch = PokemonPatch {
                    met_location: *met_location,
                    ot_gender: Some(t.gender),
                    egg: Some(*egg),
                    ..Default::default()
                };
                let (raw, w) = pokemon::edit(&raw, &patch, rom, policy)?;
                findings = w;
                save.insert(*location, &raw, rom)?;
            }
            Action::Transfer { from, to, copy } => save.transfer(*from, *to, *copy, rom)?,
            Action::Delete { location } => save.remove(*location, rom)?,
            Action::Trainer { patch } => save.edit_trainer(patch, rom)?,
            Action::Bag {
                pocket,
                slot,
                item,
                quantity,
            } => save.edit_bag(pocket, *slot, *item, *quantity, rom, policy)?,
            Action::Box {
                index,
                name,
                wallpaper,
            } => save.edit_box(*index, name, *wallpaper, rom)?,
            Action::Sort { index } => save.sort_box(*index, rom)?,
            Action::Dex {
                number,
                seen,
                owned,
            } => save.edit_dex(*number, *seen, *owned)?,
            Action::Import {
                location,
                rom_md5,
                bytes,
            } => {
                if rom_md5 != rom.profile.md5 {
                    return Err(err("profile_mismatch", rom_md5));
                }
                save.insert(*location, bytes, rom)?;
            }
        }
        save.validate(rom)?;
        let mut fields = Vec::new();
        diff_values(
            "",
            &editable_view(self.save_ref()?, rom)?,
            &editable_view(&save, rom)?,
            &mut fields,
        );
        let change = Change {
            fields,
            action,
            bytes_changed: before
                .iter()
                .zip(&save.data)
                .filter(|(a, b)| a != b)
                .count(),
            findings,
        };
        if change.bytes_changed > 0 {
            if self.undo.len() >= 128 {
                self.undo.remove(0);
            }
            self.undo.push((before, change.clone()));
            self.redo.clear();
            self.save = Some(save);
        }
        Ok(change)
    }
    pub fn undo(&mut self) -> Result<()> {
        if let Some((data, change)) = self.undo.pop() {
            let current = self.save_ref()?.data.clone();
            let save = Save::open(data, self.rom.profile.save)?;
            self.redo.push((current, change));
            self.save = Some(save);
        }
        Ok(())
    }
    pub fn redo(&mut self) -> Result<()> {
        if let Some((data, change)) = self.redo.pop() {
            let current = self.save_ref()?.data.clone();
            let save = Save::open(data, self.rom.profile.save)?;
            self.undo.push((current, change));
            self.save = Some(save);
        }
        Ok(())
    }
    pub fn export_pokemon(&self, location: Location) -> Result<PokemonFile> {
        let s = self.save_ref()?;
        if s.pokemon(location, &self.rom)?.is_none() {
            return Err(err("empty_slot", format!("{location:?}")));
        }
        Ok(PokemonFile {
            format: "gen3-pokemon-1".into(),
            rom_md5: self.rom.profile.md5.into(),
            bytes: s.raw(location)?[..80].to_vec(),
        })
    }
    pub fn export(&mut self, path: &Path) -> Result<Option<PathBuf>> {
        self.rom
            .profile
            .capabilities
            .require(self.rom.profile.capabilities.save_edit, "save_edit")?;
        let save = self.save_ref()?;
        save.validate(&self.rom)?;
        if let Some(src) = &self.source {
            if fs::read(src)? != self.original {
                return Err(err("save_conflict", src.display()));
            }
        }
        let data = save.data.clone();
        let layout = self.rom.profile.save;
        let backup = atomic_write(path, &data, |b| {
            Save::open(b.to_vec(), layout)?.validate(&self.rom)
        })?;
        self.source = Some(path.to_path_buf());
        self.original = data;
        self.undo.clear();
        self.redo.clear();
        Ok(backup)
    }
}
/// Write in the target directory, verify the staged bytes, then atomically rename.
/// Any existing destination is backed up before replacement.
pub fn atomic_write(
    path: &Path,
    data: &[u8],
    validate: impl Fn(&[u8]) -> Result<()>,
) -> Result<Option<PathBuf>> {
    validate(data)?;
    if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(err("symlink_target", path.display()));
    }
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| err("output_path", path.display()))?
        .to_string_lossy();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp = dir.join(format!(".{name}.{stamp}.tmp"));
    let backup = if path.exists() {
        let dest = dir.join(format!("{name}.bak-{stamp}"));
        let previous = fs::read(path)?;
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&dest)?;
        f.write_all(&previous)?;
        f.sync_all()?;
        Some(dest)
    } else {
        None
    };
    let result = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        file.write_all(data)?;
        file.sync_all()?;
        drop(file);
        let staged = fs::read(&tmp)?;
        if staged != data {
            return Err(err("write_verify", tmp.display()));
        }
        validate(&staged)?;
        fs::rename(&tmp, path)?;
        #[cfg(unix)]
        {
            fs::File::open(dir)?.sync_all()?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result?;
    Ok(backup)
}

fn editable_view(s: &Save, r: &Rom) -> Result<serde_json::Value> {
    Ok(
        serde_json::json!({"pokemon":s.all(r)?,"trainer":s.trainer(r)?,"boxes":s.boxes(r)?,"bag":s.bag()?,"dex":s.dex()?}),
    )
}
fn diff_values(
    path: &str,
    before: &serde_json::Value,
    after: &serde_json::Value,
    out: &mut Vec<FieldChange>,
) {
    use serde_json::Value;
    if before == after {
        return;
    }
    match (before, after) {
        (Value::Object(a), Value::Object(b)) => {
            for key in a
                .keys()
                .chain(b.keys())
                .collect::<std::collections::BTreeSet<_>>()
            {
                diff_values(
                    &format!("{path}/{key}"),
                    a.get(key).unwrap_or(&Value::Null),
                    b.get(key).unwrap_or(&Value::Null),
                    out,
                );
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                diff_values(&format!("{path}/{i}"), a, b, out);
            }
        }
        _ => out.push(FieldChange {
            path: path.into(),
            before: before.clone(),
            after: after.clone(),
        }),
    }
}
