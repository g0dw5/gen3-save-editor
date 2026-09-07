use gen3_core::{
    binary, err,
    pokemon::Policy,
    profile::PROFILES,
    rom::Rom,
    save::Save,
    session::{Action, Session},
    Result,
};
use std::{env, fs, path::Path};
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", serde_json::to_string(&e).unwrap());
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let a: Vec<_> = env::args().skip(1).collect();
    let command = a.first().map(String::as_str).unwrap_or("help");
    let print = |v: serde_json::Value| {
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    };
    if command == "help" {
        println!("gen3 profiles\ngen3 identify ROM\ngen3 catalog ROM\ngen3 species ROM ID\ngen3 world ROM\ngen3 inspect ROM SAVE\ngen3 validate ROM SAVE\ngen3 patch-save ROM SAVE ACTIONS.json OUTPUT.sav [--free] [--dry-run]\ngen3 patch-rom ROM EDITS.json OUTPUT.gba");
        return Ok(());
    }
    if command == "profiles" {
        print(serde_json::to_value(PROFILES)?);
        return Ok(());
    }
    let arg = |i| a.get(i).ok_or_else(|| err("arguments", "run gen3 help"));
    let data = fs::read(arg(1)?)?;
    if command == "identify" {
        print(
            serde_json::json!({"md5":binary::hash(&data),"sha256":binary::sha256(&data),"bytes":data.len(),"profile":gen3_core::profile::identify(&data).ok()}),
        );
        return Ok(());
    }
    let rom = Rom::open(data)?;
    match command {
        "catalog" => print(serde_json::to_value(rom.catalog()?)?),
        "species" => print(serde_json::to_value(
            rom.detail(
                arg(2)?
                    .parse()
                    .map_err(|_| err("arguments", "species ID"))?,
            )?,
        )?),
        "world" => print(serde_json::to_value(rom.world()?)?),
        "inspect" => {
            let mut session = Session::new(rom);
            session.load(fs::read(arg(2)?)?, Some(arg(2)?.into()))?;
            print(serde_json::to_value(session.snapshot()?)?);
        }
        "validate" => {
            let save = Save::open(fs::read(arg(2)?)?, rom.profile.save)?;
            save.validate(&rom)?;
            print(
                serde_json::json!({"valid":true,"active_slot":save.active_slot,"backup_valid":save.backup_valid}),
            );
        }
        "patch-save" => {
            let actions: Vec<Action> = serde_json::from_slice(&fs::read(arg(3)?)?)?;
            let mut session = Session::new(rom);
            session.load(fs::read(arg(2)?)?, Some(arg(2)?.into()))?;
            let policy = if a.iter().any(|a| a == "--free") {
                Policy::Free
            } else {
                Policy::Standard
            };
            for action in actions {
                session.apply(action, policy)?;
            }
            let snapshot = session.snapshot()?;
            let backup = if a.iter().any(|a| a == "--dry-run") {
                None
            } else {
                session.export(Path::new(arg(4)?))?
            };
            print(
                serde_json::json!({"changes":snapshot.changes,"backup":backup,"dry_run":a.iter().any(|a|a=="--dry-run")}),
            );
        }
        "patch-rom" => {
            let edits: Vec<gen3_core::rom::RomEdit> = serde_json::from_slice(&fs::read(arg(2)?)?)?;
            let (output, manifest) = rom.patch(&edits)?;
            let path = Path::new(arg(3)?);
            if path == Path::new(arg(1)?) {
                return Err(err("rom_overwrite", "choose a separate output"));
            }
            gen3_core::session::atomic_write(path, &output, |b| {
                if binary::hash(b) == manifest.output_md5 {
                    Ok(())
                } else {
                    Err(err("write_verify", "ROM"))
                }
            })?;
            let meta = path.with_extension("patch.json");
            gen3_core::session::atomic_write(
                &meta,
                &serde_json::to_vec_pretty(&manifest)?,
                |_| Ok(()),
            )?;
            print(serde_json::to_value(manifest)?);
        }
        _ => return Err(err("arguments", command)),
    }
    Ok(())
}
