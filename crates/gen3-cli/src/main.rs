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
        println!("gen3 profiles\ngen3 identify ROM\ngen3 cheats ROM\ngen3 cheat-code ROM CHEAT_ID gameshark_v1_v2|codebreaker [PARAMETERS.json]\ngen3 catalog ROM\ngen3 contest-check ROM NATURE_ID CONDITION.json\ngen3 species ROM ID\ngen3 sprite ROM ID (PNG to stdout)\ngen3 world ROM\ngen3 adventure-guide ROM [SAVE]\ngen3 fishing-spots ROM [SAVE]\ngen3 daycare-state ROM SAVE\ngen3 event-dependencies ROM CONDITION.json [SAVE]\ngen3 inspect ROM SAVE\ngen3 validate ROM SAVE\ngen3 patch-save ROM SAVE ACTIONS.json OUTPUT.sav [--free] [--dry-run]");
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
    if command == "cheats" || command == "cheat-code" {
        let rom = gen3_core::cheats::CheatRom::open(&data)?;
        if command == "cheats" {
            if a.len() != 2 {
                return Err(err("arguments", "gen3 cheats ROM"));
            }
            print(serde_json::to_value(rom.catalog())?);
        } else {
            if a.len() != 4 && a.len() != 5 {
                return Err(err(
                    "arguments",
                    "gen3 cheat-code ROM CHEAT_ID gameshark_v1_v2|codebreaker [PARAMETERS.json]",
                ));
            }
            let request = serde_json::from_value(serde_json::json!({
                "expected_rom_md5": binary::hash(&data), "cheat_id": arg(2)?, "format": arg(3)?,
                "parameters": a.get(4).map(|path| -> gen3_core::Result<serde_json::Value> { Ok(serde_json::from_slice(&fs::read(path)?)?) }).transpose()?
            }))?;
            print(serde_json::to_value(rom.generate(&request)?)?);
        }
        return Ok(());
    }
    let rom = Rom::open(data)?;
    match command {
        "catalog" => print(serde_json::to_value(rom.catalog()?)?),
        "adventure-guide" => {
            let md5 = rom.profile.md5;
            let mut session = gen3_core::session::Session::new(rom);
            if let Some(path) = a.get(2) {
                session.load(fs::read(path)?, None)?;
            }
            // App keeps its caches private; use its public session entry point.
            #[allow(clippy::field_reassign_with_default)]
            let mut app = {
                let mut app = gen3_core::app::App::default();
                app.session = Some(session);
                app
            };
            print(app.dispatch(gen3_core::app::Request {
                command: "adventure_guide".into(),
                payload: serde_json::json!({"expected_rom_md5": md5}),
            })?);
        }
        "contest-check" => {
            let nature = arg(2)?.parse().map_err(|_| err("arguments", "nature ID"))?;
            let condition: [u8; 6] = serde_json::from_slice(&fs::read(arg(3)?)?)?;
            print(serde_json::to_value(gen3_core::contest::check_npc(
                &rom, nature, condition,
            )?)?);
        }
        "species" => print(serde_json::to_value(
            rom.detail(
                arg(2)?
                    .parse()
                    .map_err(|_| err("arguments", "species ID"))?,
            )?,
        )?),
        "sprite" => {
            use std::io::Write;
            let id = arg(2)?
                .parse()
                .map_err(|_| err("arguments", "species ID"))?;
            std::io::stdout().write_all(&rom.sprite(id, false)?)?;
        }
        "world" => print(serde_json::to_value(rom.world()?)?),

        "event-dependencies" => {
            if !matches!(a.len(), 3 | 4) {
                return Err(err(
                    "arguments",
                    "gen3 event-dependencies ROM CONDITION.json [SAVE]",
                ));
            }
            let mut input: serde_json::Value = serde_json::from_slice(&fs::read(arg(2)?)?)?;
            input
                .as_object_mut()
                .ok_or_else(|| err("arguments", "query object"))?
                .entry("expected_rom_md5")
                .or_insert_with(|| serde_json::json!(rom.profile.md5));
            let save = a
                .get(3)
                .map(|path| {
                    let save = Save::open(fs::read(path)?, rom.profile.save)?;
                    save.validate(&rom)?;
                    Ok::<_, gen3_core::Error>(save)
                })
                .transpose()?;
            let index = gen3_core::event_dependencies::Index::build(&rom, &rom.maps()?)?;
            print(serde_json::to_value(index.query(
                &rom,
                save.as_ref(),
                serde_json::from_value(input)?,
            )?)?);
        }
        "fishing-spots" => {
            let save = a
                .get(2)
                .map(|path| {
                    let save = Save::open(fs::read(path)?, rom.profile.save)?;
                    save.validate(&rom)?;
                    Ok::<_, gen3_core::Error>(save)
                })
                .transpose()?;
            print(serde_json::to_value(rom.fishing_spots(save.as_ref())?)?);
        }

        "daycare-state" => {
            if a.len() != 3 {
                return Err(err("arguments", "gen3 daycare-state ROM SAVE"));
            }
            let save = Save::open(fs::read(arg(2)?)?, rom.profile.save)?;
            save.validate(&rom)?;
            print(serde_json::to_value(gen3_core::daycare_state::snapshot(
                &rom, &save,
            )?)?);
        }
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
        _ => return Err(err("arguments", command)),
    }
    Ok(())
}
