use crate::{
    adapter::{Field, PokemonCodec},
    binary::*,
    err,
    rom::Rom,
    Result,
};
use serde::{Deserialize, Serialize};

const ORDERS: [&str; 24] = [
    "GAEM", "GAME", "GEAM", "GEMA", "GMAE", "GMEA", "AGEM", "AGME", "AEGM", "AEMG", "AMGE", "AMEG",
    "EGAM", "EGMA", "EAGM", "EAMG", "EMGA", "EMAG", "MGAE", "MGEA", "MAGE", "MAEG", "MEGA", "MEAG",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pokemon {
    pub pid: u32,
    pub ot_id: u32,
    pub nickname: String,
    pub ot_name: String,
    pub language: u8,
    pub markings: u8,
    pub species: u16,
    pub held_item: u16,
    pub experience: u32,
    pub pp_ups: [u8; 4],
    pub friendship: u8,
    pub moves: [u16; 4],
    pub pps: [u8; 4],
    pub evs: [u8; 6],
    pub condition: [u8; 6],
    pub ivs: [u8; 6],
    pub ability_slot: u8,
    pub ability_id: u16,
    pub egg: bool,
    pub pokerus: u8,
    pub met_location: u8,
    pub met_level: u8,
    pub origin_game: u8,
    pub ball: u8,
    pub ot_gender: u8,
    pub ribbons: u32,
    pub nature: u8,
    pub effective_nature: u8,
    pub nature_override: Option<u8>,
    pub gender: String,
    pub shiny: bool,
    pub level: u8,
    pub stats: [u16; 6],
    pub current_hp: Option<u16>,
    pub status: Option<u32>,
    pub checksum_ok: bool,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PokemonPatch {
    pub pid: Option<u32>,
    pub ot_id: Option<u32>,
    pub nickname: Option<String>,
    pub ot_name: Option<String>,
    pub language: Option<u8>,
    pub markings: Option<u8>,
    pub species: Option<u16>,
    pub held_item: Option<u16>,
    pub experience: Option<u32>,
    pub level: Option<u8>,
    pub pp_ups: Option<[u8; 4]>,
    pub friendship: Option<u8>,
    pub moves: Option<[u16; 4]>,
    pub pps: Option<[u8; 4]>,
    pub evs: Option<[u8; 6]>,
    pub condition: Option<[u8; 6]>,
    pub ivs: Option<[u8; 6]>,
    pub ability_slot: Option<u8>,
    pub egg: Option<bool>,
    pub pokerus: Option<u8>,
    pub met_location: Option<u8>,
    pub met_level: Option<u8>,
    pub origin_game: Option<u8>,
    pub ball: Option<u8>,
    pub ot_gender: Option<u8>,
    pub ribbons: Option<u32>,
    pub nature: Option<u8>,
    /// Rocket uses 26 to restore PID-derived nature. Not supported by Gen III.
    pub nature_override: Option<u8>,
    pub gender: Option<String>,
    pub shiny: Option<bool>,
    pub current_hp: Option<u16>,
    pub status: Option<u32>,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    #[default]
    Standard,
    Free,
}
#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    pub code: String,
    pub field: String,
    pub severity: String,
    pub detail: String,
}
fn finding(code: &str, field: &str, detail: impl ToString) -> Finding {
    Finding {
        code: code.into(),
        field: field.into(),
        severity: "warning".into(),
        detail: detail.to_string(),
    }
}

pub fn checksum(b: &[u8]) -> u16 {
    b.as_chunks::<2>().0.iter().fold(0u16, |a, c| {
        a.wrapping_add(u16::from_le_bytes([c[0], c[1]]))
    })
}
pub fn unpack(raw: &[u8]) -> Result<[u8; 48]> {
    if raw.len() != 80 && raw.len() != 100 {
        return Err(err("pokemon_size", raw.len()));
    }
    let pid = u32(raw, 0)?;
    let key = pid ^ u32(raw, 4)?;
    let mut plain = [0u8; 48];
    for i in 0..12 {
        put32(&mut plain, i * 4, u32(raw, 32 + i * 4)? ^ key);
    }
    let mut canonical = [0; 48];
    for (i, ch) in ORDERS[(pid % 24) as usize].chars().enumerate() {
        let j = "GAEM".find(ch).unwrap();
        canonical[j * 12..j * 12 + 12].copy_from_slice(&plain[i * 12..i * 12 + 12]);
    }
    Ok(canonical)
}
/// Check integrity before interpreting decrypted fields or treating a slot as empty.
/// `unpack` remains available for read-only forensic inspection of damaged records.
pub fn checked_unpack(raw: &[u8]) -> Result<[u8; 48]> {
    checked_unpack_with(raw, PokemonCodec::Gen3)
}
pub fn checked_unpack_with(raw: &[u8], codec: PokemonCodec) -> Result<[u8; 48]> {
    let canonical = unpack(raw)?;
    let stored = u16(raw, 28)?;
    let calculated = checksum(&canonical);
    if stored != calculated {
        return Err(err(
            "pokemon_checksum",
            format!("stored {stored:#06x}, calculated {calculated:#06x}"),
        ));
    }
    if codec.fields().bad_egg.read(raw)? != 0 {
        return Err(err("pokemon_bad_egg", "record is marked as a Bad Egg"));
    }
    Ok(canonical)
}
pub fn pack(raw: &mut [u8], canonical: &[u8; 48]) {
    let pid = u32(raw, 0).unwrap();
    let key = pid ^ u32(raw, 4).unwrap();
    let mut plain = [0; 48];
    for (i, ch) in ORDERS[(pid % 24) as usize].chars().enumerate() {
        let j = "GAEM".find(ch).unwrap();
        plain[i * 12..i * 12 + 12].copy_from_slice(&canonical[j * 12..j * 12 + 12]);
    }
    put16(raw, 28, checksum(canonical));
    for i in 0..12 {
        put32(raw, 32 + i * 4, u32(&plain, i * 4).unwrap() ^ key);
    }
}
pub fn shiny(pid: u32, ot: u32) -> bool {
    ((pid >> 16) ^ (pid & 65535) ^ (ot >> 16) ^ (ot & 65535)) < 8
}
pub fn gender(ratio: u8, pid: u32) -> &'static str {
    match ratio {
        255 => "genderless",
        254 => "female",
        0 => "male",
        _ => {
            if (pid as u8) < ratio {
                "female"
            } else {
                "male"
            }
        }
    }
}
pub fn experience(growth: u8, level: u8) -> u32 {
    let n = level as i64;
    if n <= 1 {
        return 0;
    }
    let cube = n * n * n;
    let x = match growth {
        0 => cube,
        1 => {
            if n <= 50 {
                cube * (100 - n) / 50
            } else if n <= 68 {
                cube * (150 - n) / 100
            } else if n <= 98 {
                cube * ((1911 - 10 * n) / 3) / 500
            } else {
                cube * (160 - n) / 100
            }
        }
        2 => {
            if n <= 15 {
                cube * ((n + 1) / 3 + 24) / 50
            } else if n <= 36 {
                cube * (n + 14) / 50
            } else {
                cube * (n / 2 + 32) / 50
            }
        }
        3 => 6 * cube / 5 - 15 * n * n + 100 * n - 140,
        4 => 4 * cube / 5,
        5 => 5 * cube / 4,
        _ => cube,
    };
    x.max(0) as u32
}
pub fn level(growth: u8, exp: u32) -> u8 {
    (1..=100)
        .rev()
        .find(|l| experience(growth, *l) <= exp)
        .unwrap_or(1)
}
/// Expanded engines may replace the growth tables, including levels above 100.
pub fn rom_experience(rom: &Rom, growth: u8, level: u8) -> Result<u32> {
    if level > rom.profile.max_level {
        return Err(err("range", "level"));
    }
    if let Some(table) = rom.profile.experience_table {
        if growth as usize >= table.count {
            return Err(err("range", "growth"));
        }
        u32(
            &rom.data,
            table.offset + growth as usize * table.stride + level as usize * 4,
        )
    } else {
        Ok(experience(growth, level))
    }
}
pub fn rom_level(rom: &Rom, growth: u8, xp: u32) -> Result<u8> {
    for level in 1..rom.profile.max_level {
        if rom_experience(rom, growth, level + 1)? > xp {
            return Ok(level);
        }
    }
    Ok(rom.profile.max_level)
}
pub fn stats(base: [u8; 6], ivs: [u8; 6], evs: [u8; 6], level: u8, nature: u8) -> [u16; 6] {
    let mut out = [0; 6];
    for i in 0..6 {
        let x = ((2 * base[i] as u32 + ivs[i] as u32 + evs[i] as u32 / 4) * level as u32) / 100;
        out[i] = if i == 0 {
            if base[0] == 1 {
                1
            } else {
                (x + level as u32 + 10) as u16
            }
        } else {
            let up = nature / 5;
            let down = nature % 5;
            let v = x + 5;
            if up != down && i as u8 == up + 1 {
                (v * 110 / 100) as u16
            } else if up != down && i as u8 == down + 1 {
                (v * 90 / 100) as u16
            } else {
                v as u16
            }
        };
    }
    out
}
pub fn decode(raw: &[u8], rom: &Rom) -> Result<Pokemon> {
    let c = unpack(raw)?;
    let pid = u32(raw, 0)?;
    let ot_id = u32(raw, 4)?;
    let s = rom.species(u16(&c, 0)?)?;
    let fields = rom.profile.save.pokemon_codec.fields();
    let xp = fields.experience.read(&c)?;
    let pp_ups = fields.pp_ups.read(&c)? as u8;
    let nature_override = fields
        .nature_override
        .map(|f| f.read(&c))
        .transpose()?
        .map(|v| v as u8);
    let effective_nature = nature_override
        .filter(|v| *v < 25)
        .unwrap_or((pid % 25) as u8);
    let ivword = u32(&c, 40)?;
    let origin = u16(&c, 38)?;
    let ivs = std::array::from_fn(|i| ((ivword >> (i * 5)) & 31) as u8);
    let evs = c[24..30].try_into().unwrap();
    let lv = rom_level(rom, s.growth, xp)?;
    let slot = fields.ability.read(&c)? as u8;
    Ok(Pokemon {
        pid,
        ot_id,
        nickname: rom.codec.decode(&raw[8..18]),
        ot_name: rom.codec.decode(&raw[fields.ot_name..fields.ot_name + 7]),
        language: fields.language.read(raw)? as u8,
        markings: if rom.profile.save.pokemon_codec == PokemonCodec::Gen3 {
            raw[27]
        } else {
            fields.markings.read(raw)? as u8
        },
        species: s.id,
        held_item: u16(&c, 2)?,
        experience: xp,
        pp_ups: std::array::from_fn(|i| (pp_ups >> (i * 2)) & 3),
        friendship: fields.friendship.read(&c)? as u8,
        moves: std::array::from_fn(|i| u16(&c, 12 + i * 2).unwrap()),
        pps: c[20..24].try_into().unwrap(),
        evs,
        condition: c[30..36].try_into().unwrap(),
        ivs,
        ability_slot: slot,
        ability_id: s
            .abilities
            .get(slot as usize)
            .copied()
            .filter(|id| *id != 0)
            .unwrap_or(if slot == 1 && s.abilities.len() == 2 {
                s.abilities[0]
            } else {
                0
            }),
        egg: ivword & (1 << 30) != 0,
        pokerus: c[36],
        met_location: c[37],
        met_level: (origin & 127) as u8,
        origin_game: ((origin >> 7) & 15) as u8,
        ball: fields.ball.read(&c)? as u8,
        ot_gender: (origin >> 15) as u8,
        ribbons: rom.profile.save.pokemon_codec.read_ribbons(&c)?,
        nature: (pid % 25) as u8,
        effective_nature,
        nature_override,
        gender: gender(s.gender_ratio, pid).into(),
        shiny: shiny(pid, ot_id),
        level: lv,
        stats: stats(s.stats, ivs, evs, lv, effective_nature),
        current_hp: if raw.len() == 100 {
            Some(u16(raw, 86)?)
        } else {
            None
        },
        status: if raw.len() == 100 {
            Some(u32(raw, 80)?)
        } else {
            None
        },
        checksum_ok: checksum(&c) == u16(raw, 28)?,
    })
}
fn check(v: bool, field: &str) -> Result<()> {
    if v {
        Ok(())
    } else {
        Err(err("range", field))
    }
}
pub fn findings(p: &Pokemon, rom: &Rom) -> Result<Vec<Finding>> {
    let mut out = Vec::new();
    let s = rom.valid_species(p.species)?;
    rom.item(p.held_item)?;
    if p.evs.iter().map(|v| *v as u16).sum::<u16>() > 510 {
        out.push(finding("ev_total", "evs", "total > 510"));
    }
    if (s.abilities.len() == 2 && p.ability_slot == 1 && s.abilities[1] == 0)
        || (s.abilities.len() != 2
            && s.abilities
                .get(p.ability_slot as usize)
                .is_none_or(|a| *a == 0))
    {
        out.push(finding(
            "ability_slot",
            "ability_slot",
            "selected ability absent",
        ));
    }
    let known = rom.known_moves(p.species, p.level)?;
    for (i, id) in p.moves.iter().enumerate() {
        let mv = rom.move_info(*id)?;
        let max = mv.pp as u16 * (5 + p.pp_ups[i] as u16) / 5;
        if p.pps[i] as u16 > max {
            out.push(finding("pp_limit", &format!("pps.{i}"), max));
        }
        if *id != 0 && !known.contains(id) {
            out.push(finding("move_source_unknown", &format!("moves.{i}"), id));
        }
    }
    if p.experience > rom_experience(rom, s.growth, rom.profile.max_level)? {
        out.push(finding("experience_limit", "experience", p.experience));
    }
    Ok(out)
}
fn solve(start: u32, ot: u32, ratio: u8, nature: u8, sex: &str, is_shiny: bool) -> Result<u32> {
    if !matches!(sex, "male" | "female" | "genderless") {
        return Err(err("gender", sex));
    }
    let matches =
        |p| p % 25 == nature as u32 && gender(ratio, p) == sex && shiny(p, ot) == is_shiny;
    if matches(start) {
        return Ok(start);
    }
    if is_shiny {
        for low in 0..65536u32 {
            for v in 0..8 {
                let high = ((ot >> 16) ^ (ot & 65535) ^ low ^ v) & 65535;
                let p = (high << 16) | low;
                if matches(p) {
                    return Ok(p);
                }
            }
        }
    } else {
        for i in 0..2_000_000u32 {
            let p = start.wrapping_add(i);
            if matches(p) {
                return Ok(p);
            }
        }
    }
    Err(err(
        "pid_constraints",
        format!("nature={nature}, gender={sex}, shiny={is_shiny}"),
    ))
}
pub fn edit(
    raw: &[u8],
    patch: &PokemonPatch,
    rom: &Rom,
    policy: Policy,
) -> Result<(Vec<u8>, Vec<Finding>)> {
    let mut c = checked_unpack_with(raw, rom.profile.save.pokemon_codec)?;
    let fields = rom.profile.save.pokemon_codec.fields();
    let before = decode(raw, rom)?;
    let mut out = raw.to_vec();
    let mut species = patch.species.unwrap_or(before.species);
    let item = patch.held_item.unwrap_or(before.held_item);
    let moves = patch.moves.unwrap_or(before.moves);
    if item != before.held_item {
        species = rom.storage_form(species, item, &moves, 1)?;
    }
    if moves != before.moves {
        species = rom.storage_form(species, item, &moves, 3)?;
    }
    let s = rom.valid_species(species)?;
    if policy == Policy::Standard && species != before.species && rom.is_battle_species(species)? {
        return Err(err("battle_species", species));
    }
    if let Some(v) = patch.held_item {
        if v != before.held_item && (rom.is_mail(v) || rom.is_mail(before.held_item)) {
            return Err(err(
                "mail_attachment",
                "linked mail editing is not implemented",
            ));
        }
        rom.item(v)?;
        put16(&mut c, 2, v);
    }
    put16(&mut c, 0, species);
    if let Some(v) = patch.experience {
        fields.experience.write(&mut c, v)?;
    }
    if let Some(v) = patch.level {
        check((1..=rom.profile.max_level).contains(&v), "level")?;
        if patch.experience.is_some() && rom_level(rom, s.growth, patch.experience.unwrap())? != v {
            return Err(err("level_experience", "inconsistent patch"));
        }
        fields
            .experience
            .write(&mut c, rom_experience(rom, s.growth, v)?)?;
    }
    if let Some(v) = patch.friendship {
        fields.friendship.write(&mut c, v as u32)?;
    }
    if let Some(v) = patch.pp_ups {
        check(v.iter().all(|x| *x <= 3), "pp_ups")?;
        let value = v
            .iter()
            .enumerate()
            .fold(0u8, |a, (i, v)| a | (*v << (i * 2)));
        fields.pp_ups.write(&mut c, value as u32)?;
    }
    if patch.moves.is_some() || patch.pp_ups.is_some() {
        let v = patch.moves.unwrap_or(before.moves);
        for (i, id) in v.iter().enumerate() {
            let mv = rom.move_info(*id)?;
            put16(&mut c, 12 + i * 2, *id);
            if patch.pps.is_none() && (*id != before.moves[i] || patch.pp_ups.is_some()) {
                c[20 + i] = (mv.pp as u16 * (5 + ((fields.pp_ups.read(&c)? >> (i * 2)) & 3) as u16)
                    / 5)
                .min(255) as u8;
            }
        }
    }
    if let Some(v) = patch.pps {
        c[20..24].copy_from_slice(&v);
    }
    if let Some(v) = patch.evs {
        c[24..30].copy_from_slice(&v);
    }
    if let Some(v) = patch.condition {
        c[30..36].copy_from_slice(&v);
    }
    let ivs = patch.ivs.unwrap_or(before.ivs);
    check(ivs.iter().all(|x| *x <= 31), "ivs")?;
    let ability = patch.ability_slot.unwrap_or(before.ability_slot);
    if patch.ability_slot.is_some() {
        check((ability as usize) < s.abilities.len(), "ability_slot")?;
    }
    let egg = patch.egg.unwrap_or(before.egg);
    let word = ivs
        .iter()
        .enumerate()
        .fold(0u32, |w, (i, v)| w | ((*v as u32) << (5 * i)))
        | ((egg as u32) << 30);
    Field::new(40, 0, 31).write(&mut c, word)?;
    fields.ability.write(&mut c, ability as u32)?;
    if let Some(v) = patch.pokerus {
        c[36] = v;
    }
    if let Some(v) = patch.met_location {
        c[37] = v;
    }
    let met = patch.met_level.unwrap_or(before.met_level);
    let game = patch.origin_game.unwrap_or(before.origin_game);
    let ball = patch.ball.unwrap_or(before.ball);
    let ot_gender = patch.ot_gender.unwrap_or(before.ot_gender);
    check(met <= 127, "met_level")?;
    check(game <= 15, "origin_game")?;
    check(ball as u32 <= fields.ball.max(), "ball")?;
    check(ot_gender <= 1, "ot_gender")?;
    Field::new(38, 0, 7).write(&mut c, met as u32)?;
    Field::new(38, 7, 4).write(&mut c, game as u32)?;
    Field::new(39, 7, 1).write(&mut c, ot_gender as u32)?;
    fields.ball.write(&mut c, ball as u32)?;
    if let Some(value) = patch.nature_override {
        check(value < 25 || value == 26, "nature_override")?;
        fields
            .nature_override
            .ok_or_else(|| err("unsupported_feature", "nature_override"))?
            .write(&mut c, value as u32)?;
    }
    if let Some(v) = patch.ribbons {
        rom.profile.save.pokemon_codec.write_ribbons(&mut c, v)?;
    }
    if let Some(v) = patch.language {
        check((1..=7).contains(&v) && v != 6, "language")?;
        fields.language.write(&mut out, v as u32)?;
    }
    if let Some(v) = patch.markings {
        check(v <= 15, "markings")?;
        fields.markings.write(&mut out, v as u32)?;
    }
    if let Some(v) = &patch.nickname {
        out[8..18].copy_from_slice(&rom.codec.encode(v, 10)?);
    }
    if let Some(v) = &patch.ot_name {
        out[fields.ot_name..fields.ot_name + 7].copy_from_slice(&rom.codec.encode(v, 7)?);
    }
    let ot = patch.ot_id.unwrap_or(before.ot_id);
    put32(&mut out, 4, ot);
    let mut pid = patch.pid.unwrap_or(before.pid);
    if patch.pid.is_none()
        && (patch.nature.is_some()
            || patch.gender.is_some()
            || patch.shiny.is_some()
            || patch.ot_id.is_some()
            || patch.species.is_some())
    {
        let nature = patch.nature.unwrap_or(before.nature);
        check(nature < 25, "nature")?;
        let sex = patch
            .gender
            .as_deref()
            .unwrap_or_else(|| gender(s.gender_ratio, before.pid));
        pid = solve(
            pid,
            ot,
            s.gender_ratio,
            nature,
            sex,
            patch.shiny.unwrap_or(before.shiny),
        )?;
    } else if patch.pid.is_some()
        && (patch.nature.is_some() || patch.gender.is_some() || patch.shiny.is_some())
    {
        return Err(err(
            "pid_constraints",
            "explicit PID cannot be combined with derived fields",
        ));
    }
    put32(&mut out, 0, pid);
    fields.has_species.write(&mut out, 1)?;
    fields.header_egg.write(&mut out, egg as u32)?;
    pack(&mut out, &c);
    let mut after = decode(&out, rom)?;
    if out.len() == 100 {
        out[84] = after.level;
        for (i, v) in after.stats.iter().enumerate() {
            put16(&mut out, 88 + i * 2, *v);
        }
        let old_max = u16(raw, 88)?;
        let hp = before.current_hp.unwrap_or(0);
        let inferred = if hp == 0 {
            0
        } else {
            (hp as i32 + after.stats[0] as i32 - old_max as i32).clamp(1, after.stats[0] as i32)
                as u16
        };
        let new_hp = patch.current_hp.unwrap_or(inferred);
        check(new_hp <= after.stats[0], "current_hp")?;
        put16(&mut out, 86, new_hp);
        if let Some(v) = patch.status {
            check(v <= 255, "status")?;
            put32(&mut out, 80, v);
        }
        after = decode(&out, rom)?;
    } else if patch.current_hp.is_some() || patch.status.is_some() {
        return Err(err("party_only", "HP/status"));
    }
    let warnings = findings(&after, rom)?;
    if policy == Policy::Standard {
        // Unknown sources are warnings, never proof of impossibility. Do not make
        // unrelated edits fail because an existing free-mode value is preserved.
        if patch.evs.is_some() && warnings.iter().any(|w| w.code == "ev_total") {
            return Err(err("ev_total", "free editing required"));
        }
        if patch.ability_slot.is_some() && warnings.iter().any(|w| w.code == "ability_slot") {
            return Err(err("ability_slot", "free editing required"));
        }
        if patch.pps.is_some() && warnings.iter().any(|w| w.code == "pp_limit") {
            return Err(err("pp_limit", "free editing required"));
        }
    }
    Ok((out, warnings))
}
pub fn create(
    rom: &Rom,
    species: u16,
    ot: u32,
    ot_name: &str,
    level: u8,
    pid: u32,
) -> Result<Vec<u8>> {
    let s = rom.valid_species(species)?;
    check((1..=rom.profile.max_level).contains(&level), "level")?;
    let mut raw = vec![0; 80];
    put32(&mut raw, 0, pid);
    put32(&mut raw, 4, ot);
    let fields = rom.profile.save.pokemon_codec.fields();
    fields.language.write(&mut raw, 2)?;
    fields.has_species.write(&mut raw, 1)?;
    raw[8..18].copy_from_slice(&rom.codec.encode(&s.name, 10)?);
    raw[fields.ot_name..fields.ot_name + 7].copy_from_slice(&rom.codec.encode(ot_name, 7)?);
    let mut c = [0; 48];
    put16(&mut c, 0, species);
    fields
        .experience
        .write(&mut c, rom_experience(rom, s.growth, level)?)?;
    fields.friendship.write(&mut c, s.friendship as u32)?;
    put16(&mut c, 38, level.min(127) as u16 | (3 << 7));
    fields.ball.write(&mut c, fields.default_ball as u32)?;
    if let Some(field) = fields.nature_override {
        field.write(&mut c, 26)?;
    }
    let mut moves = Vec::new();
    for r in rom.level_moves(species)? {
        if r.level.unwrap_or(0) <= level {
            moves.retain(|id| *id != r.move_id);
            moves.push(r.move_id);
        }
    }
    let start = moves.len().saturating_sub(4);
    for (i, id) in moves[start..].iter().enumerate() {
        put16(&mut c, 12 + i * 2, *id);
        c[20 + i] = rom.move_info(*id)?.pp;
    }
    pack(&mut raw, &c);
    Ok(raw)
}
pub fn to_party(raw: &[u8], rom: &Rom) -> Result<Vec<u8>> {
    checked_unpack_with(raw, rom.profile.save.pokemon_codec)?;
    let mut out = raw[..80].to_vec();
    out.resize(100, 0);
    out[85] = 255;
    let p = decode(&out, rom)?;
    out[84] = p.level;
    put16(&mut out, 86, p.stats[0]);
    for (i, v) in p.stats.iter().enumerate() {
        put16(&mut out, 88 + i * 2, *v);
    }
    Ok(out)
}
