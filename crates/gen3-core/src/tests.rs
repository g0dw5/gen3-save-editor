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
            if layout.sizes[id] <= 0xff0 {
                b[o + 0xff0..o + 0xff4].copy_from_slice(&[0xa1, 0xb2, 0xc3, 0xd4]);
            }
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

// An independent physical-byte oracle: don't use Save's logical block writer.
fn expected_box_write(save: &Save, output: &mut [u8], index: usize, raw: &[u8]) {
    for (i, byte) in raw.iter().enumerate() {
        let logical = 4 + index * 80 + i;
        output[save.sections[5 + logical / 3968] + logical % 3968] = *byte;
    }
    for section in 5..=13 {
        let start = save.sections[section];
        let checksum = sector_checksum(&output[start..start + save.layout.sizes[section]]);
        put16(output, start + 0xff6, checksum);
    }
}

#[test]
fn transfers_preserve_every_record_byte_across_all_box_slots() {
    let rom = rom();
    let fixture = save_bytes(&rom);
    let mut source = pokemon::create(&rom, 1, 0xdeadbeef, "ASH", 46, 0xbadc5647).unwrap();
    // Header padding is outside the Pokemon checksum and must still survive.
    source[30..32].copy_from_slice(&[0xa7, 0x2b]);
    let other = pokemon::create(&rom, 2, 0x87654321, "ASH", 37, 0xfeed1234).unwrap();
    for target in 0..420 {
        // Exercise both active banks and all fourteen physical sector rotations.
        let mut rotated = fixture.clone();
        for bank in 0..2 {
            for physical in 0..14 {
                let start = bank * 14 * 4096;
                let dest = start + ((physical + target % 14) % 14) * 4096;
                let original = start + physical * 4096;
                rotated[dest..dest + 4096].copy_from_slice(&fixture[original..original + 4096]);
                put32(
                    &mut rotated,
                    dest + 0xffc,
                    if bank == target % 2 { 12 } else { 11 },
                );
            }
        }
        let mut baseline = Save::open(rotated, rom.profile.save).unwrap();
        baseline.insert(loc(69), &source, &rom).unwrap();
        for (occupied, copy) in [(false, false), (true, false), (false, true)] {
            let mut save = baseline.clone();
            if occupied && target != 69 {
                save.insert(loc(target), &other, &rom).unwrap();
            }
            let before = save.data.clone();
            let mut expected = before.clone();
            if target != 69 {
                if !copy {
                    expected_box_write(
                        &save,
                        &mut expected,
                        69,
                        if occupied { &other } else { &[0; 80] },
                    );
                }
                expected_box_write(&save, &mut expected, target, &source);
            }
            save.transfer(loc(69), loc(target), copy, &rom).unwrap();
            assert_eq!(
                save.data, expected,
                "target={target} occupied={occupied} copy={copy}"
            );
            let reopened = Save::open(save.data.clone(), rom.profile.save).unwrap();
            reopened.validate(&rom).unwrap();
            assert_eq!(reopened.raw(loc(target)).unwrap(), source);
            if !copy && target != 69 {
                save.transfer(loc(target), loc(69), false, &rom).unwrap();
                assert_eq!(save.data, before, "round trip to {target}");
            }
        }
    }
}

#[test]
fn transfer_transactions_keep_pid_and_ciphertext_through_party_and_history() {
    let mut session = session();
    let raw = pokemon::create(&session.rom, 2, 0xdeadbeef, "ASH", 46, 0xbadc5647).unwrap();
    session
        .apply(
            Action::Import {
                location: loc(69),
                rom_md5: session.rom.profile.md5.into(),
                bytes: raw.clone(),
            },
            Policy::Standard,
        )
        .unwrap();
    for (from, to, copy) in [
        (loc(69), loc(49), false), // Record crosses a flash-sector payload boundary.
        (loc(49), party(), false), // Occupied party slot: swap with box conversion.
        (party(), loc(419), true),
        (loc(419), Location::Party { slot: 5 }, false), // Empty slot appends to party.
        (Location::Party { slot: 1 }, loc(198), true),
    ] {
        let before = session.save_ref().unwrap().data.clone();
        session
            .apply(Action::Transfer { from, to, copy }, Policy::Standard)
            .unwrap();
        let after = session.save_ref().unwrap().data.clone();
        let actual_to = if to == (Location::Party { slot: 5 }) {
            Location::Party { slot: 1 }
        } else {
            to
        };
        assert_eq!(
            &session.save_ref().unwrap().raw(actual_to).unwrap()[..80],
            &raw
        );
        session.undo().unwrap();
        assert_eq!(session.save_ref().unwrap().data, before);
        session.redo().unwrap();
        assert_eq!(session.save_ref().unwrap().data, after);
    }
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("transfer.sav");
    session.export(&output).unwrap();
    let exported = Save::open(std::fs::read(output).unwrap(), session.rom.profile.save).unwrap();
    assert_eq!(exported.raw(loc(198)).unwrap(), raw);
    exported.validate(&session.rom).unwrap();
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
fn explicit_pid_edits_reencrypt_across_all_substructure_orders() {
    let r = rom();
    let original_pid = 0x9abc1357;
    let raw = pokemon::create(&r, 1, 0x12345678, "ASH", 50, original_pid).unwrap();
    let canonical = pokemon::unpack(&raw).unwrap();
    // Cover a truncated high half, the u32 limit, and every destination order.
    let targets = [original_pid & 0xffff, u32::MAX]
        .into_iter()
        .chain((0..24).map(|order| (0x87650000 / 24) * 24 + order));
    for pid in targets {
        let (edited, _) = pokemon::edit(
            &raw,
            &PokemonPatch {
                pid: Some(pid),
                ..Default::default()
            },
            &r,
            Policy::Free,
        )
        .unwrap();
        assert_eq!(u32(&edited, 0).unwrap(), pid);
        assert_eq!(pokemon::checked_unpack(&edited).unwrap(), canonical);
        assert_eq!(&edited[4..32], &raw[4..32]);
        assert_ne!(&edited[32..80], &raw[32..80]);
    }
}

// Damage a populated record while retaining a valid outer sector checksum,
// as happens when a game saves a record that was damaged in memory.
fn overwrite_box_record(s: &mut Save, index: usize, raw: &[u8]) {
    let offset = 4 + index * 80;
    let id = 5 + offset / 3968;
    let pos = offset % 3968;
    assert!(pos + raw.len() <= s.layout.sizes[id]);
    let sector = s.sections[id];
    s.data[sector + pos..sector + pos + raw.len()].copy_from_slice(raw);
    let sum = sector_checksum(&s.data[sector..sector + s.layout.sizes[id]]);
    put16(&mut s.data, sector + 0xff6, sum);
}

#[test]
fn truncated_pid_is_rejected_on_load_and_export_without_writing() {
    let r = rom();
    let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let raw = pokemon::create(&r, 1, 0x12345678, "ASH", 50, 0x9abc1357).unwrap();
    save.insert(loc(69), &raw, &r).unwrap();
    let good = save.data.clone();
    let mut damaged = raw.clone();
    damaged[2..4].fill(0);
    overwrite_box_record(&mut save, 69, &damaged);
    let error = save.validate(&r).unwrap_err();
    assert_eq!(error.code, "pokemon_checksum");
    assert!(error.detail.contains("box_index: 2, slot: 9"));
    assert!(pokemon::edit(&damaged, &PokemonPatch::default(), &r, Policy::Free).is_err());

    let mut session = Session::new(r);
    session.load(good.clone(), None).unwrap();
    assert_eq!(
        session.load(save.data.clone(), None).unwrap_err().code,
        "pokemon_checksum"
    );
    assert_eq!(session.save_ref().unwrap().data, good);
    // An invalid in-memory record must also be blocked at the final write gate.
    session.save = Some(save);
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("existing.sav");
    std::fs::write(&output, &good).unwrap();
    assert_eq!(
        session.export(&output).unwrap_err().code,
        "pokemon_checksum"
    );
    assert_eq!(std::fs::read(&output).unwrap(), good);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn damaged_empty_looking_record_cannot_be_skipped_or_overwritten() {
    let r = rom();
    let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let mut raw = vec![0; 80];
    put32(&mut raw, 0, 0x9abc1357);
    put32(&mut raw, 4, 0x12345678);
    pokemon::pack(&mut raw, &[0; 48]);
    // Species zero and hasSpecies unset used to bypass checksum validation.
    raw[28] ^= 1;
    overwrite_box_record(&mut save, 69, &raw);
    let before = save.data.clone();
    assert_eq!(
        save.pokemon(loc(69), &r).unwrap_err().code,
        "pokemon_checksum"
    );
    assert_eq!(save.validate(&r).unwrap_err().code, "pokemon_checksum");
    let replacement = pokemon::create(&r, 1, 42, "ASH", 50, 7).unwrap();
    assert_eq!(
        save.insert(loc(69), &replacement, &r).unwrap_err().code,
        "pokemon_checksum"
    );
    assert_eq!(save.data, before);
}

#[test]
fn bad_egg_header_is_rejected_even_with_valid_checksum() {
    let r = rom();
    let mut raw = pokemon::create(&r, 1, 42, "ASH", 50, 7).unwrap();
    raw[19] |= 1;
    assert!(pokemon::decode(&raw, &r).unwrap().checksum_ok);
    assert_eq!(
        pokemon::checked_unpack(&raw).unwrap_err().code,
        "pokemon_bad_egg"
    );
    assert_eq!(
        pokemon::edit(&raw, &PokemonPatch::default(), &r, Policy::Free)
            .unwrap_err()
            .code,
        "pokemon_bad_egg"
    );
    assert_eq!(
        pokemon::to_party(&raw, &r).unwrap_err().code,
        "pokemon_bad_egg"
    );
    let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    assert_eq!(
        save.insert(loc(0), &raw, &r).unwrap_err().code,
        "pokemon_bad_egg"
    );
    overwrite_box_record(&mut save, 69, &raw);
    assert_eq!(save.validate(&r).unwrap_err().code, "pokemon_bad_egg");
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
        let reports: Vec<_> = maps.iter().map(|m| r.map_events(m).unwrap()).collect();
        let markers: Vec<_> = reports.iter().flat_map(|r| &r.markers).collect();
        assert_eq!(markers.iter().filter(|m| m.kind == "hidden").count(), 112);
        assert_eq!(markers.iter().filter(|m| m.kind == "pickup").count(), 122);
        let city = reports.iter().find(|r| r.map_id == "0-0").unwrap();
        let hidden = city.markers.iter().find(|m| m.kind == "hidden").unwrap();
        assert_eq!((hidden.x, hidden.y, hidden.flag), (11, 29, Some(0x253)));
        let gifts = reports.iter().find(|r| r.map_id == "6-0").unwrap();
        let gift = gifts
            .markers
            .iter()
            .find(|m| m.local_id == Some(2))
            .unwrap();
        assert_eq!(gift.kind, "gift");
        assert!(gift.rewards.iter().any(|r| r.item == 333)); // TM45

        let encounters = r.encounters().unwrap();
        assert!(encounters.len() > 1000);
        let vaporeon = r.origin_options(134).unwrap();
        assert_eq!(vaporeon.ancestors, [133, 134]);
        assert!(vaporeon.can_hatch);
        assert!(vaporeon
            .encounters
            .iter()
            .all(|e| [133, 134].contains(&e.species)));
        assert!(!r.origin_options(150).unwrap().can_hatch);
        assert!(!r.origin_options(132).unwrap().can_hatch);
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

#[test]
fn hidden_power_profiles_expose_the_verified_ui_rule() {
    for profile in [profile::BW, profile::DP] {
        assert_eq!(
            serde_json::to_value(profile.hidden_power).unwrap(),
            serde_json::json!({"move_id": 237, "formula": "gen3_to5"})
        );
    }
}

#[test]
fn origins_follow_ancestors_without_sibling_encounters_and_allow_babies() {
    let mut r = rom();
    r.profile.map_counts = &[1];
    let b = std::sync::Arc::make_mut(&mut r.data);
    put32(b, r.profile.maps, 0x08021000);
    put32(b, 0x21000, 0x08021100);
    put32(b, 0x21100, 0x08021200);
    put32(b, 0x21200, 2);
    put32(b, 0x21204, 2);
    b[0x21114] = 1;
    let evo = r.profile.evolutions.offset + r.profile.evolutions.stride;
    // Species 1 branches into 2 and 3; only species 3 can breed.
    for (i, target) in [2, 3].into_iter().enumerate() {
        put16(b, evo + i * 8, 4);
        put16(b, evo + i * 8 + 4, target);
    }
    for (id, group) in [(1, 15), (2, 15), (3, 5)] {
        let o = r.profile.base_stats.offset + id * 28;
        b[o + 20..o + 22].fill(group);
    }
    let wild = r.profile.wild;
    put32(b, wild + 4, 0x08021300);
    b[wild + 20..wild + 22].fill(255);
    put32(b, 0x21300, 20);
    put32(b, 0x21304, 0x08021400);
    for i in 0..12 {
        b[0x21400 + i * 4..0x21402 + i * 4].fill(5);
        put16(b, 0x21402 + i * 4, (i % 3 + 1) as u16);
    }
    let origins = r.origin_options(2).unwrap();
    assert_eq!(origins.ancestors, [1, 2]);
    assert_eq!(origins.encounters.len(), 8);
    assert!(origins.encounters.iter().all(|e| e.species != 3));
    assert!(origins.can_hatch);
    assert_eq!(origins.hatch_regions, [1]);
    assert!(r.origin_options(1).unwrap().can_hatch);
    // Undiscovered and Ditto groups cannot themselves produce this lineage.
    for group in [15, 13] {
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[r.profile.base_stats.offset + 3 * 28 + 20..r.profile.base_stats.offset + 3 * 28 + 22]
            .fill(group);
        let origins = r.origin_options(2).unwrap();
        assert!(!origins.can_hatch);
        assert!(origins.hatch_regions.is_empty());
    }
    // A malformed cycle terminates rather than expanding indefinitely.
    let b = std::sync::Arc::make_mut(&mut r.data);
    let evo = r.profile.evolutions.offset + 2 * r.profile.evolutions.stride;
    put16(b, evo, 4);
    put16(b, evo + 4, 1);
    assert_eq!(
        r.ancestors(2).unwrap().into_iter().collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn item_event_layers_keep_coordinates_and_hidden_flags() {
    let mut r = rom();
    let b = std::sync::Arc::make_mut(&mut r.data);
    let ev = 0x25000;
    let objects = 0x25100;
    let bg = 0x25200;
    let script = 0x25300;
    b[ev] = 1;
    b[ev + 3] = 1;
    put32(b, ev + 4, 0x08000000 + objects as u32);
    put32(b, ev + 16, 0x08000000 + bg as u32);
    b[objects] = 8;
    b[objects + 1] = 59;
    put16(b, objects + 4, 11);
    put16(b, objects + 6, 7);
    b[objects + 8] = 3;
    put16(b, objects + 20, 0x400);
    put32(b, objects + 16, 0x08000000 + script as u32);
    b[script..script + 13].copy_from_slice(&[0x1a, 0, 0x80, 1, 0, 0x1a, 1, 0x80, 2, 0, 9, 1, 2]);
    put16(b, bg, 5);
    put16(b, bg + 2, 9);
    b[bg + 5] = 7;
    put16(b, bg + 8, 1);
    put16(b, bg + 10, 10);
    let map = crate::world::Map {
        id: "0-0".into(),
        group: 0,
        number: 0,
        name: "Test".into(),
        region: 0,
        width: 20,
        height: 20,
        map_type: 0,
        header: 0,
        layout: 0,
        events: Some(ev),
        scripts: vec![script],
        objects: vec![],
    };
    let events = r.map_events(&map).unwrap();
    assert_eq!(events.markers.len(), 2);
    assert!(events.unplaced_rewards.is_empty());
    let ball = &events.markers[0];
    assert_eq!(
        (ball.kind, ball.x, ball.y, ball.flag),
        ("pickup", 11, 7, Some(0x400))
    );
    assert_eq!(ball.rewards[0].quantity, Some(2));
    let hidden = &events.markers[1];
    assert_eq!(
        (hidden.kind, hidden.x, hidden.y, hidden.flag),
        ("hidden", 5, 9, Some(0x1fe))
    );
    // Bad pointers must fail safely, never reinterpret an item's ID as a script.
    let b = std::sync::Arc::make_mut(&mut r.data);
    put32(b, ev + 16, 0xfffffff0);
    assert!(r.map_events(&map).is_err());
}

#[test]
fn reward_scripts_follow_calls_and_preserve_branch_evidence() {
    let mut r = rom();
    let b = std::sync::Arc::make_mut(&mut r.data);
    let p = 0x25000;
    // Call sets item/amount. Then a receipt flag selects either exit or gift.
    b[p] = 4;
    put32(b, p + 1, 0x08025100);
    b[p + 5] = 0x2b;
    put16(b, p + 6, 0x117);
    b[p + 8] = 6;
    b[p + 9] = 1;
    put32(b, p + 10, 0x08025200);
    b[p + 14] = 9;
    b[p + 15] = 0;
    b[p + 16] = 2;
    b[0x25200] = 2;
    b[0x25100..0x2510b].copy_from_slice(&[0x16, 0, 0x80, 1, 0, 0x16, 1, 0x80, 3, 0, 3]);
    let (rewards, stops) = r.item_script(p).unwrap();
    assert!(stops.is_empty());
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0].quantity, Some(3));
    assert_eq!(rewards[0].via, "gift");
    assert_eq!(rewards[0].conditions[0].id, 0x117);
    assert!(!rewards[0].conditions[0].taken);
    // Decoration IDs belong to a different catalog and must not become items.
    let b = std::sync::Arc::make_mut(&mut r.data);
    b[p + 15] = 7;
    assert!(r.item_script(p).unwrap().0.is_empty());
}

#[test]
fn reward_parser_stops_unknown_commands_and_invalidates_native_values() {
    let mut r = rom();
    let b = std::sync::Arc::make_mut(&mut r.data);
    let p = 0x25000;
    b[p..p + 16].copy_from_slice(&[
        0x16, 0, 0x80, 1, 0, 0x16, 1, 0x80, 1, 0, 0x25, 0, 0, 9, 0, 2,
    ]);
    let (rewards, stops) = r.item_script(p).unwrap();
    assert!(rewards.is_empty());
    assert!(stops.contains(&(p + 10)));
    let b = std::sync::Arc::make_mut(&mut r.data);
    b[p] = 0xff;
    b[p + 1..p + 6].copy_from_slice(&[0x44, 1, 0, 1, 0]);
    assert_eq!(r.item_script(p).unwrap(), (vec![], vec![p]));
}

/// Synthetic tables intentionally use the same public model with different byte formats.
fn adapter_rom(p: profile::Profile) -> Rom {
    let mut r = rom();
    if p.save.pokemon_codec == crate::adapter::PokemonCodec::Gen3 {
        // Synthetic BW/DP tables share test addresses, while identity/capabilities differ.
        r.profile.id = p.id;
        r.profile.md5 = p.md5;
        return r;
    }
    let mut b = vec![0; p.size];
    for id in 1..4 {
        let o = p.base_stats.offset + id * 36;
        b[o..o + 6].copy_from_slice(&[45, 49, 49, 45, 65, 65]);
        b[o + 18] = 127;
        b[o + 20] = 70;
        put16(&mut b, o + 24, 1);
        put16(&mut b, o + 26, 2);
        put16(&mut b, o + 28, 267); // Must not truncate to u8.
        put32(&mut b, p.learnsets + id * 4, 0x08020000);
    }
    put16(&mut b, 0x20000, 1);
    put16(&mut b, 0x20002, 1);
    put16(&mut b, 0x20004, 0xffff);
    for id in 1..3 {
        b[p.moves.offset + id * 20 + 6] = 35;
    }
    r.data = std::sync::Arc::new(b);
    r.profile = p;
    r
}

#[test]
fn adapter_matrix_bit_ownership_all_pid_orders() {
    for profile in profile::PROFILES {
        let r = adapter_rom(profile);
        let rocket = profile.save.pokemon_codec == crate::adapter::PokemonCodec::Rocket21;
        for pid in 0..24 {
            let mut raw = pokemon::create(&r, 1, 0x12345678, "ASH", 50, pid).unwrap();
            let mut c = pokemon::unpack(&raw).unwrap();
            if rocket {
                // Independent literal masks: sentinel bits belong to unrelated fields.
                c[6] |= 0x80;
                c[10] |= 0xfc;
                c[11] = 0xa5;
                c[39] |= 0x78;
                c[43] |= 0x80;
                c[47] = 0xa8;
                raw[18] |= 0xc0;
                raw[26] = 0xb0;
                raw[27] = 0x57;
            }
            pokemon::pack(&mut raw, &c);
            let unchanged = pokemon::edit(&raw, &PokemonPatch::default(), &r, Policy::Free)
                .unwrap()
                .0;
            assert_eq!(unchanged, raw, "no-op {} PID {pid}", profile.id);
            let patch = PokemonPatch {
                ball: Some(if rocket { 31 } else { 12 }),
                friendship: Some(201),
                experience: Some(234567),
                pp_ups: Some([3, 2, 1, 0]),
                ability_slot: Some(if rocket { 2 } else { 1 }),
                nature_override: rocket.then_some(3),
                ..Default::default()
            };
            let (edited, _) = pokemon::edit(&raw, &patch, &r, Policy::Free).unwrap();
            let after = pokemon::decode(&edited, &r).unwrap();
            assert_eq!(after.experience, 234567);
            assert_eq!(after.pp_ups, [3, 2, 1, 0]);
            assert_eq!(after.friendship, 201);
            assert_eq!(after.ball, if rocket { 31 } else { 12 });
            assert_eq!(after.ability_id, if rocket { 267 } else { 2 });
            assert_eq!(
                after.effective_nature,
                if rocket { 3 } else { (pid % 25) as u8 }
            );
            assert_eq!(after.ot_name, "ASH");
            assert_eq!(&edited[..28], &raw[..28]);
            assert!(after.checksum_ok);
            let canonical = pokemon::unpack(&edited).unwrap();
            let mut expected = c;
            if rocket {
                expected[4..7].copy_from_slice(&[0x47, 0x94, 0x83]); // 234567 + preserved bit 23.
                expected[7] = 0x1b;
                expected[8] = 201;
                expected[9] = 0x7f; // nature 3 and ball 31.
                expected[10] &= 0xfc;
                expected[47] = (expected[47] & !3) | 2;
            } else {
                put32(&mut expected, 4, 234567);
                expected[8] = 0x1b;
                expected[9] = 201;
                let origins = (u16(&expected, 38).unwrap() & !0x7800) | (12 << 11);
                put16(&mut expected, 38, origins);
                expected[43] |= 0x80;
            }
            for i in 0..4 {
                let id = u16(&expected, 12 + i * 2).unwrap();
                expected[20 + i] =
                    (r.move_info(id).unwrap().pp as u16 * (5 + [3, 2, 1, 0][i]) / 5) as u8;
            }
            assert_eq!(canonical, expected, "unowned bits {} PID {pid}", profile.id);
        }
    }
}

#[test]
fn adapter_capabilities_reject_writes_without_mutation() {
    let r = adapter_rom(profile::ROCKET);
    let mut s = Session::new(r.clone());
    let original = save_bytes(&r);
    s.load(original.clone(), None).unwrap();
    for policy in [Policy::Standard, Policy::Free] {
        let error = s
            .apply(
                Action::Pokemon {
                    location: party(),
                    patch: PokemonPatch {
                        ball: Some(4),
                        ..Default::default()
                    },
                },
                policy,
            )
            .err()
            .unwrap();
        assert_eq!(error.code, "unsupported_feature");
        assert_eq!(s.save_ref().unwrap().data, original);
        assert!(!s.snapshot().unwrap().can_undo);
    }
    let mut save = s.save_ref().unwrap().clone();
    assert_eq!(
        save.transfer(party(), loc(0), false, &r).unwrap_err().code,
        "unsupported_feature"
    );
    assert_eq!(
        save.edit_bag("pc", 0, 1, 1, &r, Policy::Free)
            .unwrap_err()
            .code,
        "unsupported_feature"
    );
    assert_eq!(
        save.edit_dex(1, true, true).unwrap_err().code,
        "unsupported_feature"
    );
    assert_eq!(save.data, original);
    assert!(save.dex().unwrap().is_empty());
    assert_eq!(
        save.bag()
            .unwrap()
            .iter()
            .filter(|p| p.pocket == "pc")
            .count(),
        10
    );
    assert_eq!(r.world().err().unwrap().code, "unsupported_feature");
    assert_eq!(r.patch(&[]).err().unwrap().code, "unsupported_feature");
    let mut app = crate::app::App { session: Some(s) };
    assert_eq!(
        app.dispatch(crate::app::Request {
            command: "save_bytes".into(),
            payload: serde_json::json!({})
        })
        .unwrap_err()
        .code,
        "unsupported_feature"
    );
    assert_eq!(app.session.unwrap().save_ref().unwrap().data, original);
}

#[test]
fn battle_transformations_are_not_evolution_or_ancestry() {
    let mut r = adapter_rom(profile::ROCKET);
    let b = std::sync::Arc::make_mut(&mut r.data);
    let o = r.profile.evolutions.offset + 80;
    for (slot, method) in [0xffff, 0xfffe, 0xfffd].into_iter().enumerate() {
        put16(b, o + slot * 8, method);
        put16(b, o + slot * 8 + 2, 1);
        put16(b, o + slot * 8 + 4, 2);
    }
    put16(b, o + 24, 4);
    put16(b, o + 26, 16);
    put16(b, o + 28, 3);
    assert_eq!(r.evolutions(1).unwrap().len(), 1);
    assert_eq!(r.evolutions(1).unwrap()[0].target, 3);
    assert_eq!(r.ancestors(2).unwrap().into_iter().collect::<Vec<_>>(), [2]);
    assert_eq!(
        r.ancestors(3).unwrap().into_iter().collect::<Vec<_>>(),
        [1, 3]
    );
    let forms = r.battle_forms(1).unwrap();
    assert_eq!(forms.len(), 3);
    assert_eq!(forms[1].trigger, crate::forms::BattleTrigger::KnownMove(1));
    assert_eq!(forms[2].kind, crate::forms::BattleFormKind::Primal);
    assert_eq!(r.battle_forms(2).unwrap().len(), 3);
    assert!(rom().battle_forms(1).unwrap().is_empty());
}

#[test]
#[ignore = "set GEN3_ROM_ROCKET; optionally GEN3_SAVE_ROCKET and GEN3_ADAPTER_PROBES"]
fn local_rocket_adapter_regression() {
    let path = std::env::var("GEN3_ROM_ROCKET").unwrap();
    let r = Rom::open(std::fs::read(path).unwrap()).unwrap();
    assert_eq!(r.profile.id, profile::ROCKET.id);
    let catalog = r.catalog().unwrap();
    assert_eq!(
        (
            catalog.species.len(),
            catalog.moves.len(),
            catalog.items.len()
        ),
        (1394, 755, 923)
    );
    assert_eq!(r.species(1).unwrap().abilities.len(), 3);
    assert!(r.move_info(755).is_err()); // Internal Z attacks cannot become ordinary moves.
    assert_eq!(r.battle_forms(6).unwrap().len(), 2);
    let mut associations = 0;
    for species in 1..1395 {
        r.level_moves(species).unwrap();
        associations += r
            .battle_forms(species)
            .unwrap()
            .iter()
            .filter(|f| f.source == species)
            .count();
        assert!(r
            .evolutions(species)
            .unwrap()
            .iter()
            .all(|e| e.method < 0xfffd));
    }
    assert_eq!(associations, 59);
    for id in [1, 6, 41, 330] {
        for shiny in [false, true] {
            assert!(r
                .pokemon_sprite(id, shiny, 0)
                .unwrap()
                .starts_with(b"\x89PNG"));
        }
    }
    let mut probes = Vec::new();
    if let Ok(path) = std::env::var("GEN3_SAVE_ROCKET") {
        let bytes = std::fs::read(&path).unwrap();
        let before_hash = sha256(&bytes);
        let save = Save::open(bytes.clone(), r.profile.save).unwrap();
        save.validate(&r).unwrap();
        for row in save.all(&r).unwrap() {
            let raw = save.raw(row.location).unwrap();
            probes.push(serde_json::json!({"before":raw,"after":raw,"pokemon":row.pokemon}));
        }
        assert_eq!(save.data, bytes);
        assert_eq!(sha256(&std::fs::read(path).unwrap()), before_hash);
    }
    // Synthetic in-memory codec probes; the public writer remains disabled.
    for pid in 0..24 {
        let raw = pokemon::create(&r, 1, 0x12345678, "TEST", 50, pid).unwrap();
        for patch in [
            PokemonPatch {
                ball: Some(17),
                ..Default::default()
            },
            PokemonPatch {
                nature_override: Some(3),
                ..Default::default()
            },
            PokemonPatch {
                ability_slot: Some(2),
                ..Default::default()
            },
            PokemonPatch {
                experience: Some(234567),
                ..Default::default()
            },
            PokemonPatch {
                friendship: Some(201),
                ..Default::default()
            },
            PokemonPatch {
                pp_ups: Some([3, 2, 1, 0]),
                pps: Some(pokemon::decode(&raw, &r).unwrap().pps),
                ..Default::default()
            },
        ] {
            let (edited, _) = pokemon::edit(&raw, &patch, &r, Policy::Free).unwrap();
            probes.push(serde_json::json!({"before":raw,"after":edited,"pokemon":pokemon::decode(&edited,&r).unwrap(),"patch":patch}));
        }
    }
    if let Ok(path) = std::env::var("GEN3_ADAPTER_PROBES") {
        std::fs::write(path, serde_json::to_vec(&probes).unwrap()).unwrap();
    }
}

#[test]
fn table_formats_can_be_composed_without_changing_the_pokemon_codec() {
    let mut r = rom();
    r.profile.formats.moves = crate::adapter::MoveFormat::Expanded20;
    r.profile.moves.stride = 20;
    let b = std::sync::Arc::make_mut(&mut r.data);
    let o = r.profile.moves.offset + 20;
    put16(b, o, 356);
    put16(b, o + 2, 500);
    b[o + 6] = 10;
    b[o + 16] = 1;
    r.profile.move_category_offset = 16;
    let mv = r.move_info(1).unwrap();
    assert_eq!((mv.effect, mv.power, mv.pp, mv.category), (356, 500, 10, 1));
    assert_eq!(r.species(1).unwrap().abilities.len(), 2);
    assert_eq!(r.level_moves(1).unwrap()[0].level, Some(1));
    assert_eq!(
        r.profile.save.pokemon_codec,
        crate::adapter::PokemonCodec::Gen3
    );
}

#[test]
fn bad_egg_and_checksum_rejection_follow_each_codec() {
    for profile in profile::PROFILES {
        let r = adapter_rom(profile);
        let raw = pokemon::create(&r, 1, 1, "TEST", 10, 11).unwrap();
        let mut bad = raw.clone();
        let flag = if profile.save.pokemon_codec == crate::adapter::PokemonCodec::Gen3 {
            (19, 1)
        } else {
            (18, 8)
        };
        bad[flag.0] |= flag.1;
        assert!(pokemon::checked_unpack_with(&bad, profile.save.pokemon_codec).is_err());
        bad = raw;
        bad[32] ^= 1;
        assert_eq!(
            pokemon::checked_unpack_with(&bad, profile.save.pokemon_codec)
                .unwrap_err()
                .code,
            "pokemon_checksum"
        );
    }
}
