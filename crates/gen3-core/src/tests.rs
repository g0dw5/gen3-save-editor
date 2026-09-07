use crate::{
    binary::*,
    graphics,
    pokemon::{self, PokemonPatch, Policy},
    profile,
    rom::Rom,
    save::{sector_checksum, Location, Save},
    session::{Action, Session},
    text::Codec,
};
use std::sync::OnceLock;

fn rom() -> Rom {
    static ROM: OnceLock<Rom> = OnceLock::new();
    ROM.get_or_init(|| {
        let mut b = vec![0; 0x2000000];
        let p = profile::BW;
        let codec = Codec::new();
        for id in 1..4 {
            let name = format!("MON{id}");
            b[p.species.offset + id * 11..p.species.offset + (id + 1) * 11]
                .copy_from_slice(&codec.encode(&name, 11).unwrap());
            let o = p.base_stats.offset + id * 28;
            b[o..o + 6].copy_from_slice(&[45, 49, 49, 45, 65, 65]);
            b[o + 16] = 127;
            b[o + 18] = 70;
            b[o + 19] = 0;
            b[o + 22] = 1;
            b[o + 23] = 2;
            put32(&mut b, p.learnsets + (id + 1) * 4, 0x08020000);
        }
        put16(&mut b, 0x20000, (1 << 9) | 1);
        put16(&mut b, 0x20002, (30 << 9) | 2);
        put16(&mut b, 0x20004, 0xffff);
        put16(&mut b, p.eggs, 0xffff);
        for id in 0..3 {
            b[p.moves.offset + id * 12 + 4] = if id == 0 { 0 } else { 35 };
        }
        b[p.items.offset + 44 + 26] = 1;
        Rom::fixture(b)
    })
    .clone()
}
fn save_bytes(r: &Rom) -> Vec<u8> {
    let mut b = vec![0; 0x20000];
    let layout = r.profile.save;
    let raw = pokemon::create(r, 1, 0x12345678, "ASH", 50, 12345).unwrap();
    let party = pokemon::to_party(&raw, r).unwrap();
    for bank in 0..2 {
        for physical in 0..14 {
            let id = (physical + 4) % 14;
            let o = bank * 14 * 4096 + physical * 4096;
            // Non-payload bytes make accidental normalization detectable.
            b[o + 0xff0..o + 0xff4].copy_from_slice(&[0xa1, 0xb2, 0xc3, 0xd4]);
            if id == 0 {
                b[o..o + 7].copy_from_slice(&r.codec.encode("ASH", 7).unwrap());
                put32(&mut b, o + 0xac, 0x87654321);
                put16(&mut b, o + 10, 0x5678);
                put16(&mut b, o + 12, 0x1234);
            }
            if id == 1 {
                b[o + 0x234] = 1;
                b[o + 0x238..o + 0x238 + 100].copy_from_slice(&party);
                put32(&mut b, o + 0x490, 1000 ^ 0x87654321);
                put16(&mut b, o + 0x494, 50 ^ 0x4321);
            }
            put16(&mut b, o + 0xff4, id as u16);
            put32(&mut b, o + 0xff8, 0x08012025);
            put32(&mut b, o + 0xffc, if bank == 0 { 10 } else { 9 });
            let sum = sector_checksum(&b[o..o + layout.sizes[id]]);
            put16(&mut b, o + 0xff6, sum);
        }
    }
    b
}
fn session() -> Session {
    let r = rom();
    let bytes = save_bytes(&r);
    let mut s = Session::new(r);
    s.load(bytes, None).unwrap();
    s
}
fn party() -> Location {
    Location::Party { slot: 0 }
}
fn loc(n: usize) -> Location {
    Location::Box {
        box_index: n / 30,
        slot: n % 30,
    }
}

#[test]
fn text_encoding_preserves_double_byte_zero() {
    let c = Codec::new();
    assert_eq!(c.decode(&[4, 0, 255]), "肤");
    assert_eq!(c.encode("肤", 2).unwrap(), vec![4, 0]);
    assert!(c.encode("肤肤", 3).is_err());
    assert!(c.encode("🙂", 10).is_err());
    assert_eq!(c.decode(&[0x71, 255]), "\u{2009}");
}
#[test]
fn crypto_all_permutations_and_known_checksum() {
    let mut canonical = [0; 48];
    for (i, v) in canonical.iter_mut().enumerate() {
        *v = i as u8;
    }
    assert_eq!(pokemon::checksum(&canonical), 0x4228);
    for pid in 0..24 {
        let mut raw = vec![0; 80];
        put32(&mut raw, 0, pid);
        put32(&mut raw, 4, 0xdeadbeef);
        pokemon::pack(&mut raw, &canonical);
        assert_eq!(pokemon::unpack(&raw).unwrap(), canonical);
        assert_eq!(u16(&raw, 28).unwrap(), 0x4228);
    }
}
#[test]
fn known_stat_and_growth_vectors() {
    assert_eq!(
        pokemon::stats([45, 49, 49, 45, 65, 65], [31; 6], [0; 6], 50, 0),
        [120, 69, 69, 65, 85, 85]
    );
    assert_eq!(
        pokemon::stats([45, 49, 49, 45, 65, 65], [31; 6], [0; 6], 50, 3),
        [120, 75, 69, 65, 76, 85]
    );
    for (g, total) in [
        (0, 1000000),
        (1, 600000),
        (2, 1640000),
        (3, 1059860),
        (4, 800000),
        (5, 1250000),
    ] {
        assert_eq!(pokemon::experience(g, 100), total);
        let mut previous = 0;
        for lv in 1..=100 {
            let e = pokemon::experience(g, lv);
            assert!(e >= previous);
            assert_eq!(pokemon::level(g, e), lv);
            previous = e;
        }
    }
}
#[test]
fn identity_edit_reencrypts_and_preserves_unowned_bytes() {
    let r = rom();
    let raw = pokemon::create(&r, 1, 0x12345678, "ASH", 50, 12345).unwrap();
    let mut extended = pokemon::to_party(&raw, &r).unwrap();
    extended[30] = 0xa5;
    extended[31] = 0x5a;
    extended[85] = 0xff;
    let patch = PokemonPatch {
        ot_id: Some(1),
        nickname: Some("肤".into()),
        nature: Some(3),
        gender: Some("female".into()),
        shiny: Some(true),
        ivs: Some([31; 6]),
        level: Some(70),
        egg: Some(true),
        ..Default::default()
    };
    let (out, _) = pokemon::edit(&extended, &patch, &r, Policy::Standard).unwrap();
    let p = pokemon::decode(&out, &r).unwrap();
    assert!(p.checksum_ok && p.shiny && p.egg);
    assert_eq!(p.nature, 3);
    assert_eq!(p.gender, "female");
    assert_eq!(p.nickname, "肤");
    assert_eq!(&out[30..32], &[0xa5, 0x5a]);
    assert_eq!(out[19] & 4, 4);
    assert_eq!(out[84], 70);
    assert_eq!(u16(&out, 88).unwrap(), p.stats[0]);
    assert_eq!(out[85], 255);
}
#[test]
fn exact_roundtrip_and_counter_rollover() {
    let r = rom();
    let mut bytes = save_bytes(&r);
    let save = Save::open(bytes.clone(), r.profile.save).unwrap();
    assert_eq!(save.active_slot, 0);
    save.validate(&r).unwrap();
    assert_eq!(save.data, bytes);
    for i in 0..14 {
        put32(&mut bytes, i * 4096 + 0xffc, u32::MAX);
        put32(&mut bytes, 14 * 4096 + i * 4096 + 0xffc, 0);
    }
    let save = Save::open(bytes, r.profile.save).unwrap();
    assert_eq!(save.active_slot, 1);
    assert_eq!(save.counter, 0);
}
#[test]
fn corrupt_and_mixed_slots_are_not_selected() {
    let r = rom();
    let mut bytes = save_bytes(&r);
    put32(&mut bytes, 0xffc, 11);
    let s = Save::open(bytes.clone(), r.profile.save).unwrap();
    assert_eq!(s.active_slot, 1);
    assert!(!s.backup_valid);
    bytes[14 * 4096 + 100] ^= 1;
    assert!(Save::open(bytes, r.profile.save).is_err());
    assert!(Save::open(vec![0; 100], r.profile.save).is_err());
}
#[test]
fn crossing_sector_storage_and_undo_are_lossless() {
    let mut s = session();
    let original = s.save_ref().unwrap().data.clone();
    s.apply(
        Action::Transfer {
            from: party(),
            to: loc(49),
            copy: true,
        },
        Policy::Standard,
    )
    .unwrap();
    s.save_ref().unwrap().validate(&s.rom).unwrap();
    assert!(s
        .save_ref()
        .unwrap()
        .pokemon(loc(49), &s.rom)
        .unwrap()
        .is_some());
    let new = s.save_ref().unwrap().data.clone();
    s.undo().unwrap();
    assert_eq!(s.save_ref().unwrap().data, original);
    s.redo().unwrap();
    assert_eq!(s.save_ref().unwrap().data, new);
}
#[test]
fn batch_failure_rolls_back_entire_transaction() {
    let mut s = session();
    let original = s.save_ref().unwrap().data.clone();
    let err = s
        .apply(
            Action::Batch {
                locations: vec![party(), loc(0)],
                patch: PokemonPatch {
                    level: Some(99),
                    ..Default::default()
                },
            },
            Policy::Standard,
        )
        .unwrap_err();
    assert_eq!(err.code, "empty_slot");
    assert_eq!(s.save_ref().unwrap().data, original);
    assert!(!s.snapshot().unwrap().can_undo);
}
#[test]
fn policy_never_bypasses_structure_and_preserves_free_values() {
    let mut s = session();
    let a = Action::Pokemon {
        location: party(),
        patch: PokemonPatch {
            evs: Some([255; 6]),
            ..Default::default()
        },
    };
    assert!(s.apply(a.clone(), Policy::Standard).is_err());
    assert!(s.apply(a, Policy::Free).is_ok());
    s.apply(
        Action::Pokemon {
            location: party(),
            patch: PokemonPatch {
                nickname: Some("TEST".into()),
                ..Default::default()
            },
        },
        Policy::Standard,
    )
    .unwrap();
    assert_eq!(
        s.save_ref()
            .unwrap()
            .pokemon(party(), &s.rom)
            .unwrap()
            .unwrap()
            .evs,
        [255; 6]
    );
    assert!(s
        .apply(
            Action::Pokemon {
                location: party(),
                patch: PokemonPatch {
                    ivs: Some([32; 6]),
                    ..Default::default()
                }
            },
            Policy::Free
        )
        .is_err());
}
#[test]
fn transfer_invariants_and_import_profile() {
    let mut s = session();
    assert_eq!(
        s.apply(Action::Delete { location: party() }, Policy::Free)
            .unwrap_err()
            .code,
        "last_party"
    );
    assert!(s
        .apply(
            Action::Transfer {
                from: party(),
                to: loc(0),
                copy: false
            },
            Policy::Free
        )
        .is_err());
    s.apply(
        Action::Transfer {
            from: party(),
            to: loc(0),
            copy: true,
        },
        Policy::Standard,
    )
    .unwrap();
    s.apply(
        Action::Transfer {
            from: loc(0),
            to: Location::Party { slot: 5 },
            copy: false,
        },
        Policy::Standard,
    )
    .unwrap();
    assert_eq!(s.save_ref().unwrap().party_count(), 2);
    s.apply(Action::Delete { location: party() }, Policy::Standard)
        .unwrap();
    assert_eq!(s.save_ref().unwrap().party_count(), 1);
    let f = s.export_pokemon(party()).unwrap();
    assert!(s
        .apply(
            Action::Import {
                location: loc(0),
                rom_md5: "wrong".into(),
                bytes: f.bytes
            },
            Policy::Free
        )
        .is_err());
}
#[test]
fn full_box_validation_detects_bad_checksum() {
    let r = rom();
    let mut s = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let raw = pokemon::create(&r, 1, 42, "ASH", 50, 7).unwrap();
    s.insert(loc(419), &raw, &r).unwrap();
    let offset = 4 + 419 * 80;
    let physical = s.sections[5 + offset / 3968] + offset % 3968;
    s.data[physical + 28] ^= 1;
    let sec = s.sections[13];
    let sum = sector_checksum(&s.data[sec..sec + s.layout.sizes[13]]);
    put16(&mut s.data, sec + 0xff6, sum);
    assert_eq!(s.validate(&r).unwrap_err().code, "pokemon_checksum");
}
#[test]
fn money_changes_only_owned_bytes() {
    let mut s = session();
    let before = s.save_ref().unwrap().data.clone();
    let sector = s.save_ref().unwrap().sections[1];
    s.apply(
        Action::Trainer {
            patch: crate::save::TrainerPatch {
                money: Some(999999),
                ..Default::default()
            },
        },
        Policy::Standard,
    )
    .unwrap();
    let after = &s.save_ref().unwrap().data;
    for (i, (a, b)) in before.iter().zip(after).enumerate() {
        if a != b {
            assert!(
                (sector + 0x490..sector + 0x494).contains(&i)
                    || (sector + 0xff6..sector + 0xff8).contains(&i)
            );
        }
    }
    assert_eq!(s.snapshot().unwrap().trainer.money, 999999);
}
#[test]
fn atomic_export_backup_and_external_conflict() {
    let mut s = session();
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("game.sav");
    let original = s.save_ref().unwrap().data.clone();
    std::fs::write(&p, &original).unwrap();
    s.load(original.clone(), Some(p.clone())).unwrap();
    s.apply(
        Action::Pokemon {
            location: party(),
            patch: PokemonPatch {
                level: Some(80),
                ..Default::default()
            },
        },
        Policy::Standard,
    )
    .unwrap();
    let backup = s.export(&p).unwrap().unwrap();
    assert_eq!(std::fs::read(backup).unwrap(), original);
    let modified = std::fs::read(&p).unwrap();
    Save::open(modified.clone(), s.rom.profile.save)
        .unwrap()
        .validate(&s.rom)
        .unwrap();
    std::fs::write(&p, original).unwrap();
    assert_eq!(s.export(&p).unwrap_err().code, "save_conflict");
}
#[test]
fn binary_and_compression_boundaries() {
    assert!(bytes(&[1], usize::MAX, 2).is_err());
    assert!(pointer(&[0; 4], 0).is_err());
    assert!(graphics::lz77(&[0x10, 4, 0, 0, 0x80, 0, 0], 0).is_err());
    assert_eq!(
        graphics::lz77(&[0x10, 3, 0, 0, 0, 1, 2, 3], 0).unwrap(),
        vec![1, 2, 3]
    );
    assert!(profile::identify(&[0; 100]).is_err());
}

#[test]
fn pp_ups_alone_updates_pp_and_box_names_terminate() {
    let mut s = session();
    s.apply(
        Action::Pokemon {
            location: party(),
            patch: PokemonPatch {
                pp_ups: Some([3; 4]),
                ..Default::default()
            },
        },
        Policy::Standard,
    )
    .unwrap();
    let mon = s
        .save_ref()
        .unwrap()
        .pokemon(party(), &s.rom)
        .unwrap()
        .unwrap();
    assert_eq!(mon.pps[0], 56);
    s.apply(
        Action::Box {
            index: 0,
            name: "ABCDEFGH".into(),
            wallpaper: 3,
        },
        Policy::Standard,
    )
    .unwrap();
    assert_eq!(s.snapshot().unwrap().boxes[0].name, "ABCDEFGH");
    assert!(s
        .apply(
            Action::Box {
                index: 0,
                name: "ABCDEFGHI".into(),
                wallpaper: 3
            },
            Policy::Free
        )
        .is_err());
}
#[test]
fn changes_record_actual_before_and_after() {
    let mut s = session();
    let change = s
        .apply(
            Action::Trainer {
                patch: crate::save::TrainerPatch {
                    money: Some(12345),
                    ..Default::default()
                },
            },
            Policy::Standard,
        )
        .unwrap();
    let diff = change
        .fields
        .iter()
        .find(|d| d.path == "/trainer/money")
        .unwrap();
    assert_eq!(diff.before, 1000);
    assert_eq!(diff.after, 12345);
}
/// Optional local regression; no copyrighted fixture is embedded or downloaded.
#[test]
#[ignore = "set GEN3_ROM_BW and GEN3_ROM_DP to user-supplied exact ROMs"]
fn local_rom_regression() {
    for key in ["GEN3_ROM_BW", "GEN3_ROM_DP"] {
        let path = std::env::var(key).expect("local ROM path required");
        let r = Rom::open(std::fs::read(path).unwrap()).unwrap();
        let catalog = r.catalog().unwrap();
        assert_eq!(catalog.moves.len(), 472);
        let maps = r.maps().unwrap();
        assert_eq!(maps.len(), 707);
        let encounters = r.encounters().unwrap();
        assert!(encounters.len() > 1000);
        let trainers = r.trainers().unwrap();
        assert!(trainers.len() > 1300);
        let mut s = Session::new(r.clone());
        s.load(save_bytes(&r), None).unwrap();
        for species in &catalog.species {
            if species.stats[0] == 0 {
                continue;
            }
            r.level_moves(species.id).unwrap();
            r.learnset(species.id).unwrap();
            r.sprite(species.id, false).unwrap();
            r.sprite(species.id, true).unwrap();
        }
        for id in ["0-0", "0-16", "24-0", "34-0", "35-46", "37-38"] {
            r.map_image(id).unwrap();
        }
        for (i, species) in catalog
            .species
            .iter()
            .filter(|s| s.stats[0] > 0)
            .take(48)
            .enumerate()
        {
            s.apply(
                Action::Create {
                    location: loc(i),
                    species: species.id,
                    level: 50,
                    met_location: Some(16),
                    egg: false,
                },
                Policy::Standard,
            )
            .unwrap();
        }
        s.apply(
            Action::Pokemon {
                location: loc(0),
                patch: PokemonPatch {
                    moves: Some([206, 0, 0, 0]),
                    ivs: Some([31; 6]),
                    evs: Some([255; 6]),
                    shiny: Some(true),
                    ..Default::default()
                },
            },
            Policy::Free,
        )
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("roundtrip.sav");
        s.export(&output).unwrap();
        let bytes = std::fs::read(output).unwrap();
        Save::open(bytes, r.profile.save)
            .unwrap()
            .validate(&r)
            .unwrap();
        if key == "GEN3_ROM_BW" {
            if let Ok(path) = std::env::var("GEN3_TEST_SAVE") {
                std::fs::write(path, &s.save_ref().unwrap().data).unwrap();
            }
        }
        eprintln!(
            "{}: {} species, {} maps, {} encounters, {} trainers; {} trainer diagnostics",
            r.profile.id,
            catalog.species.len(),
            maps.len(),
            encounters.len(),
            trainers.len(),
            trainers
                .iter()
                .filter(|t| !t.diagnostics.is_empty())
                .count()
        );
    }
}

#[test]
fn trainer_level_does_not_consume_adjacent_byte() {
    let mut r = rom();
    let profile = r.profile;
    let bytes = std::sync::Arc::make_mut(&mut r.data);
    let header = profile.trainers.offset + 40;
    bytes[header] = 3;
    bytes[header + 32] = 1;
    put32(bytes, header + 36, 0x08022000);
    put16(bytes, 0x22000, 255);
    bytes[0x22002] = 44;
    bytes[0x22003] = 0x26;
    put16(bytes, 0x22004, 1);
    let trainers = r.trainers().unwrap();
    assert_eq!(trainers[0].party[0].level, 44);
    assert!(trainers[0].diagnostics.is_empty());
}

#[test]
fn free_editing_cannot_create_dangling_mail_links() {
    let mut s = session();
    let before = s.save_ref().unwrap().data.clone();
    let result = s.apply(
        Action::Pokemon {
            location: party(),
            patch: PokemonPatch {
                held_item: Some(0x79),
                ..Default::default()
            },
        },
        Policy::Free,
    );
    assert_eq!(result.unwrap_err().code, "mail_attachment");
    assert_eq!(s.save_ref().unwrap().data, before);
}

#[test]
fn rom_patch_is_bounded_reproducible_and_keeps_the_baseline() {
    use crate::rom::RomEdit;
    let r = rom();
    let baseline = hash(&r.data);
    let edit = RomEdit {
        table: "species".into(),
        id: 1,
        field: "attack".into(),
        value: 120,
    };
    let (out, manifest) = r.patch(std::slice::from_ref(&edit)).unwrap();
    let changed: Vec<_> = r
        .data
        .iter()
        .zip(&out)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(changed, vec![r.profile.base_stats.offset + 28 + 1]);
    assert_eq!(manifest.base_md5, baseline);
    assert_eq!(manifest.output_md5, hash(&out));
    assert_eq!(manifest.output_sha256, sha256(&out));
    assert_eq!(hash(&r.data), baseline);
    let (again, _) = r.patch(std::slice::from_ref(&edit)).unwrap();
    assert_eq!(out, again);
    assert!(r
        .patch(&[RomEdit {
            value: 256,
            ..edit.clone()
        }])
        .is_err());
    assert!(r
        .patch(&[RomEdit {
            field: "offset".into(),
            ..edit
        }])
        .is_err());
}
