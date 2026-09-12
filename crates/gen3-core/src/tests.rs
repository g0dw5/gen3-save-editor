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
        // The split is per move, not the original Gen III type-based split.
        for (id, category) in [(7, 0), (53, 1), (14, 2), (247, 1), (174, 3)] {
            assert_eq!(catalog.moves[id].category, category);
        }
        assert!(catalog.moves.iter().all(|m| m.category <= 3));
        let maps = r.maps().unwrap();
        assert_eq!(maps.len(), 707);
        let encounters = r.encounters().unwrap();
        assert!(encounters.len() > 1000);
        let trainers = r.trainers().unwrap();
        assert!(trainers.len() > 1300);
        // Both hacks use the relocated class table, including expanded IDs.
        assert_eq!(u32(&r.data, 0x183b4).unwrap(), 0x0919a000);
        assert_eq!(u32(&r.data, 0x6f0ac).unwrap(), 0x0919a000);
        for (id, class_name) in [
            (261, "四天王"),
            (271, "馆主"),
            (335, "联盟冠军"),
            (957, "四天王"),
            (973, "联盟冠军"),
            (1003, "水舰队首领"),
        ] {
            assert_eq!(
                trainers
                    .iter()
                    .find(|t| t.id == id)
                    .unwrap()
                    .class_name
                    .as_deref(),
                Some(class_name)
            );
        }
        let locations = r.trainer_locations_for_maps(&maps).unwrap();
        assert_eq!(
            catalog
                .met_locations
                .iter()
                .find(|l| l.id == maps.iter().find(|m| m.id == "14-0").unwrap().region)
                .unwrap()
                .name,
            "绿岭市"
        );
        assert!(catalog.met_locations.iter().all(|l| l.id < 213));
        for (id, map, expected) in [
            (271, "14-0", vec![131, 132]),
            (335, "16-4", vec![133]),
            (973, "36-42", vec![133]),
        ] {
            let location = locations
                .locations
                .iter()
                .find(|l| l.trainer_id == id && l.map_id == map)
                .unwrap();
            let mut graphics: Vec<_> = location.actors.iter().map(|a| a.graphics_id).collect();
            graphics.sort();
            graphics.dedup();
            assert_eq!(graphics, expected, "trainer {id}");
        }
        assert_eq!(u32(&r.data, 0x5df78).unwrap(), 0x09198000);
        assert_eq!(u32(&r.data, 0x5df80).unwrap(), 0x09199000);
        for id in 0..r.profile.trainer_sprites.count {
            if id == 165 {
                // Unreferenced resource with 63 tiles; do not invent the missing tile.
                assert!(!trainers.iter().any(|t| t.portrait == 165));
                assert!(r.trainer_sprite(id as u16).is_err());
            } else {
                r.trainer_sprite(id as u16).unwrap();
            }
        }
        // Both original and extended overworld banks are read from the ROM.
        for id in [33, 121, 131, 132, 133, 0x121, 0x179, 0x185] {
            r.object_sprite(id).unwrap();
        }
        assert_eq!(locations.locations.len(), 680);
        assert_eq!(
            locations
                .locations
                .iter()
                .map(|l| l.trainer_id)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            636
        );
        assert!(locations
            .locations
            .iter()
            .any(|l| l.trainer_id == 656 && l.map_id == "0-2"));
        for (id, map, offset) in [
            (261, "16-0", 0x227f7b),
            (262, "16-1", 0x2281e2),
            (263, "16-2", 0x228480),
            (264, "16-3", 0x22870a),
            (335, "16-4", 0x228a51),
            (903, "24-106", 0xdfbf00),
            (904, "24-106", 0xdfbf10),
            (993, "36-46", 0xe0241c),
            (999, "37-38", 0xe069e7),
            (1000, "37-38", 0xe068f7),
        ] {
            assert!(locations.locations.iter().any(|l| l.trainer_id == id
                && l.map_id == map
                && l.battle_offsets.contains(&offset)));
        }
        let unusual: Vec<_> = trainers
            .iter()
            .filter(|t| t.party.iter().any(|p| p.level == 0 || p.level > 100))
            .collect();
        assert_eq!(unusual.len(), 96);
        let missing: Vec<_> = unusual
            .iter()
            .filter(|t| !locations.locations.iter().any(|l| l.trainer_id == t.id))
            .map(|t| t.id)
            .collect();
        assert_eq!(missing, [683, 684, 685]);
        // These remaining IDs are indirect rematch references, not orphan rows.
        for (i, id) in [681, 682, 683, 684, 685, 24, 1].iter().enumerate() {
            assert_eq!(u16(&r.data, 0x550204 + i * 2).unwrap(), *id);
        }
        assert_eq!(u32(&r.data, 0xb1ebc).unwrap(), 0x085500a4);
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
        for id in [
            "0-0", "0-16", "24-0", "26-1", "34-0", "35-24", "35-46", "37-38",
        ] {
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

#[test]
fn trainer_map_index_follows_branches_and_keeps_evidence() {
    let mut r = rom();
    let b = std::sync::Arc::make_mut(&mut r.data);
    // Conditional branch: trainer 7 on fallthrough, trainer 8 on the branch.
    b[0x23000] = 6;
    b[0x23001] = 1;
    put32(b, 0x23002, 0x08023100);
    for (offset, id) in [(0x23006, 7), (0x23100, 8)] {
        b[offset] = 0x5c;
        b[offset + 1] = 0;
        put16(b, offset + 2, id);
        b[offset + 14] = 2;
    }
    // A shared root and cycle must not duplicate a battle reference.
    b[0x2310e] = 5;
    put32(b, 0x2310f, 0x08023000);
    // Unknown opcode: battle-looking bytes after it must not be scanned.
    b[0x23200] = 0xff;
    b[0x23201] = 0x5c;
    b[0x23202] = 0;
    put16(b, 0x23203, 9);
    let first = crate::world::Map {
        id: "0-0".into(),
        group: 0,
        number: 0,
        name: "First".into(),
        region: 0,
        width: 1,
        height: 1,
        map_type: 0,
        header: 0,
        layout: 0,
        events: None,
        objects: vec![],
        scripts: vec![0x23000, 0x23100],
    };
    let second = crate::world::Map {
        id: "0-1".into(),
        number: 1,
        name: "Second".into(),
        scripts: vec![0x23200],
        ..first.clone()
    };
    let third = crate::world::Map {
        id: "0-2".into(),
        number: 2,
        name: "Third".into(),
        ..first.clone()
    };
    let index = r
        .trainer_locations_for_maps(&[first, second, third])
        .unwrap();
    assert_eq!(index.locations.len(), 4);
    assert_eq!(index.locations[0].trainer_id, 7);
    assert_eq!(index.locations[0].battle_offsets, vec![0x23006]);
    assert_eq!(index.locations[1].battle_offsets, vec![0x23100]);
    assert_eq!(
        index.locations.iter().filter(|l| l.trainer_id == 7).count(),
        2
    );
    assert!(index.locations.iter().all(|l| l.trainer_id != 9));
    assert_eq!(index.unresolved_maps.len(), 1);
    assert_eq!(index.unresolved_maps[0].map_id, "0-1");
    assert_eq!(index.unresolved_maps[0].stopped_at, vec![0x23200]);
}

#[test]
fn trainer_scripts_cross_menus_music_and_native_calls() {
    let mut r = rom();
    let b = std::sync::Arc::make_mut(&mut r.data);
    // Arguments deliberately contain battle-looking bytes. Only instruction
    // boundaries count; the native address must not become a script root.
    let script = [
        0x16, 0x04, 0x80, 1, 0, // setvar species
        0x16, 0x05, 0x80, 20, 0, // setvar level
        0x6f, 0x5c, 0, 7, 0,    // multichoice
        0xa0, // checkplayergender
        0x36, 0x5c, 0, // fadenewbgm
        0x23, 1, 0x34, 2, 8, // callnative 0x08023401
        0x25, 0xe2, 1, // unknown native effects invalidate special-battle inputs
        0x5c, 3, 42, 0, 0, 0, 0, 0, 0, 0, // trainerbattle
        2,
    ];
    b[0x23000..0x23000 + script.len()].copy_from_slice(&script);
    let map = crate::world::Map {
        id: "0-0".into(),
        group: 0,
        number: 0,
        name: "Test".into(),
        region: 0,
        width: 1,
        height: 1,
        map_type: 0,
        header: 0,
        layout: 0,
        events: None,
        objects: vec![],
        scripts: vec![0x23000],
    };
    let report = r.script_report(&map).unwrap();
    assert_eq!(report.trainer_ids, vec![42]);
    assert_eq!(report.trainer_battles.len(), 1);
    assert_eq!(report.trainer_battles[0].offset, 0x2301b);
    assert!(report.encounters.is_empty());
    assert!(report.stopped_at.is_empty());
}

#[test]
fn trainer_generation_uses_quality_and_cumulative_names() {
    let mut r = rom();
    let b = std::sync::Arc::make_mut(&mut r.data);
    let h = r.profile.trainers.offset + 40;
    b[h] = 1;
    b[h + 4..h + 16].fill(0xff);
    b[h + 4] = 10;
    b[h + 32] = 2;
    put32(b, h + 36, 0x08024000);
    for (i, quality) in [255, 250].iter().enumerate() {
        let p = 0x24000 + i * 16;
        put16(b, p, *quality);
        b[p + 2] = 26;
        put16(b, p + 4, 1);
    }
    let s = r.profile.species.offset + r.profile.species.stride;
    b[s..s + 11].fill(0xff);
    b[s] = 20;
    b[r.profile.base_stats.offset + 28 + 16] = 254; // Always female species.
    let t = r.trainers().unwrap().remove(0);
    let first = t.party[0].generation.as_ref().unwrap();
    let second = t.party[1].generation.as_ref().unwrap();
    assert_eq!(first.ivs, Some([31; 6]));
    assert_eq!(second.ivs, Some([30; 6]));
    assert_eq!(first.evs, [0; 6]);
    assert_eq!(first.gender, "female"); // The trainer is male.
    assert_eq!(first.nature, ((30 * 256 + 0x88) % 25) as u8);
    assert_eq!(second.nature, ((60 * 256 + 0x88) % 25) as u8);
    assert_eq!(first.ability_id, 1);
    // The nonzero parameter preserves nature while forcing ability parity.
    for parameter in 1..50u8 {
        let pid = crate::world::trainer_personality(123, parameter, false, false);
        assert_eq!(pid % 25, parameter as u32 % 25);
        assert_eq!(pid & 1, u32::from(parameter >= 25));
    }
}

#[test]
fn trainer_scripts_skip_music_arguments() {
    let mut r = rom();
    let b = std::sync::Arc::make_mut(&mut r.data);
    let script = [
        0x31, 0x5c, 0, 0x33, 0xc2, 1, 0, 0x5c, 3, 42, 0, 0, 0, 0, 0, 0, 0, 2,
    ];
    b[0x23000..0x23000 + script.len()].copy_from_slice(&script);
    let map = crate::world::Map {
        id: "0-0".into(),
        group: 0,
        number: 0,
        name: "Test".into(),
        region: 0,
        width: 1,
        height: 1,
        map_type: 0,
        header: 0,
        layout: 0,
        events: None,
        objects: vec![],
        scripts: vec![0x23000],
    };
    let report = r.script_report(&map).unwrap();
    assert_eq!(report.trainer_ids, vec![42]);
    assert_eq!(report.trainer_battles[0].offset, 0x23007);
    assert!(report.stopped_at.is_empty());
}

#[test]
fn trainer_class_names_follow_configured_table_and_preserve_unknown_ids() {
    let mut r = rom();
    // Relocation must work without changing trainer IDs or UI classifications.
    r.profile.trainer_classes = profile::Table {
        offset: 0x25000,
        count: 2,
        stride: 13,
    };
    let b = std::sync::Arc::make_mut(&mut r.data);
    let h = r.profile.trainers.offset + 40;
    b[h + 1] = 1;
    b[h + 32] = 1;
    put32(b, h + 36, 0x08024000);
    b[0x24002] = 20;
    put16(b, 0x24004, 1);
    b[0x2500d..0x2501a].copy_from_slice(&r.codec.encode("SCOUT", 13).unwrap());
    let trainer = r.trainers().unwrap().remove(0);
    assert_eq!(trainer.class, 1);
    assert_eq!(trainer.class_name.as_deref(), Some("SCOUT"));
    std::sync::Arc::make_mut(&mut r.data)[h + 1] = 255;
    let unknown = r.trainers().unwrap().remove(0);
    assert_eq!(unknown.class, 255);
    assert_eq!(unknown.class_name, None);
    assert_eq!(unknown.party.len(), 1);
}

#[test]
fn party_and_box_share_current_pp_and_bonus_storage() {
    let r = rom();
    let raw = pokemon::create(&r, 1, 123, "ASH", 20, 456).unwrap();
    let patch = PokemonPatch {
        moves: Some([1, 0, 0, 0]),
        pp_ups: Some([3, 0, 0, 0]),
        pps: Some([7, 0, 0, 0]),
        ..Default::default()
    };
    let (boxed, _) = pokemon::edit(&raw, &patch, &r, Policy::Standard).unwrap();
    let party = pokemon::to_party(&boxed, &r).unwrap();
    assert_eq!(&party[..80], &boxed);
    for bytes in [&boxed, &party] {
        let p = pokemon::decode(bytes, &r).unwrap();
        assert_eq!(p.pps, [7, 0, 0, 0]);
        assert_eq!(p.pp_ups, [3, 0, 0, 0]);
        assert!(p.checksum_ok);
    }
}

#[test]
fn tiled_sprite_validates_bounds_and_decodes_tile_order() {
    let mut tiles = [0; 64];
    tiles[0] = 0x21; // Low nibble is the left pixel.
    tiles[32] = 3; // First pixel of the second tile.
    let mut palette = [0; 32];
    put16(&mut palette, 2, 0x001f);
    put16(&mut palette, 4, 0x03e0);
    put16(&mut palette, 6, 0x7c00);
    let encoded = graphics::tiled_sprite(&tiles, &palette, 16, 8).unwrap();
    let mut reader = png::Decoder::new(std::io::Cursor::new(encoded))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; 16 * 8 * 4];
    reader.next_frame(&mut pixels).unwrap();
    assert_eq!(&pixels[..8], &[255, 0, 0, 255, 0, 255, 0, 255]);
    assert_eq!(&pixels[32..36], &[0, 0, 255, 255]);
    assert_eq!(pixels[11], 0);
    assert!(graphics::tiled_sprite(&tiles, &palette, 17, 8).is_err());
    assert!(graphics::tiled_sprite(&tiles[..32], &palette, 16, 8).is_err());
    let r = rom();
    assert!(r.object_sprite(240).is_err()); // Variable graphics ID, not an image index.
    assert!(r.trainer_sprite(203).is_err());
}

#[test]
fn pc_items_are_unencrypted_bounded_and_undoable() {
    let mut s = session();
    let original = s.save_ref().unwrap().data.clone();
    let base = s.save_ref().unwrap().sections[1];
    let action = |slot, item, quantity| Action::Bag {
        pocket: "pc".into(),
        slot,
        item,
        quantity,
    };
    s.apply(action(49, 1, 999), Policy::Standard).unwrap();
    let data = &s.save_ref().unwrap().data;
    assert_eq!(u16(data, base + 0x498 + 49 * 4).unwrap(), 1);
    assert_eq!(u16(data, base + 0x498 + 49 * 4 + 2).unwrap(), 999);
    for (i, (&before, &after)) in original.iter().zip(data).enumerate() {
        if before != after {
            assert!(
                (base + 0x498 + 49 * 4..base + 0x560).contains(&i)
                    || (base + 0xff6..base + 0xff8).contains(&i)
            );
        }
    }
    let reloaded = Save::open(data.clone(), s.rom.profile.save).unwrap();
    assert_eq!(
        reloaded
            .bag()
            .unwrap()
            .iter()
            .find(|e| e.pocket == "pc" && e.slot == 49)
            .unwrap()
            .quantity,
        999
    );
    s.undo().unwrap();
    assert_eq!(s.save_ref().unwrap().data, original);
    s.redo().unwrap();
    let edited = s.save_ref().unwrap().data.clone();
    assert!(s.apply(action(50, 1, 1), Policy::Standard).is_err());
    assert!(s.apply(action(49, 1, 1000), Policy::Standard).is_err());
    assert_eq!(s.save_ref().unwrap().data, edited);
    s.apply(action(49, 0, 1), Policy::Standard).unwrap();
    assert_eq!(
        u32(&s.save_ref().unwrap().data, base + 0x498 + 49 * 4).unwrap(),
        0
    );
}

#[test]
fn unown_appearance_uses_all_four_pid_bytes() {
    let mut counts = [0; 28];
    for bits in 0u32..256 {
        let pid = (bits & 3) | ((bits & 12) << 6) | ((bits & 48) << 12) | ((bits & 192) << 18);
        let letter = graphics::unown_letter(pid);
        assert_eq!(letter, (bits % 28) as u16);
        assert_eq!(letter, graphics::unown_letter(pid | 0xfcfcfcfc));
        counts[letter as usize] += 1;
    }
    assert!(counts.iter().all(|count| *count >= 9));
}
