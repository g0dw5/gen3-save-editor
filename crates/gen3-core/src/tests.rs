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

#[test]
fn start_state_lists_every_supported_rom_without_loading_game_data() {
    use crate::app::{App, Request};
    let mut app = App::default();
    let state = app
        .dispatch(Request {
            command: "state".into(),
            payload: serde_json::Value::Null,
        })
        .unwrap();
    assert!(state["catalog"].is_null());
    let profiles = state["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), profile::PROFILES.len());
    for expected in profile::PROFILES {
        assert!(profiles.iter().any(|actual| {
            actual["id"] == expected.id
                && actual["label"] == expected.label
                && actual["md5"] == expected.md5
        }));
    }
    assert_eq!(
        app.dispatch(Request {
            command: "open_cheat_rom".into(),
            payload: serde_json::Value::Null,
        })
        .unwrap_err()
        .code,
        "command"
    );
    assert_eq!(
        app.dispatch(Request {
            command: "patch_rom".into(),
            payload: serde_json::Value::Null,
        })
        .unwrap_err()
        .code,
        "command"
    );
}

#[test]
fn contest_api_rejects_stale_rom_and_malformed_values() {
    use crate::app::{App, Request};
    let mut app = App {
        session: Some(Session::new(rom())),
        ..Default::default()
    };
    let bad_values = [
        serde_json::json!([0, null, 0, 0, 0, 0]),
        serde_json::json!([0, 256, 0, 0, 0, 0]),
        serde_json::json!([0, -1, 0, 0, 0, 0]),
        serde_json::json!([0, 1.5, 0, 0, 0, 0]),
        serde_json::json!([0, 0]),
    ];
    for condition in bad_values {
        let request = Request {
            command: "contest_check".into(),
            payload: serde_json::json!({"condition": condition, "nature": 9, "expected_rom_md5": profile::BW.md5}),
        };
        assert_eq!(app.dispatch(request).unwrap_err().code, "json");
    }
    let request = Request {
        command: "contest_check".into(),
        payload: serde_json::json!({"condition": [0, 0, 0, 0, 0, 0], "nature": 9, "expected_rom_md5": profile::ROCKET.md5}),
    };
    assert_eq!(app.dispatch(request).unwrap_err().code, "rom_mismatch");
}

#[test]
fn contest_feeding_caps_after_the_last_block_and_obeys_gain_direction() {
    use crate::contest::feed;
    assert_eq!(
        feed([0, 250, 0, 0, 0, 254], [0, 20, 0, 0, 0, 30], [0; 5]).unwrap(),
        [0, 255, 0, 0, 0, 255]
    );
    assert_eq!(
        feed([0, 250, 0, 0, 0, 255], [0, 20, 0, 0, 0, 30], [0; 5])
            .unwrap_err()
            .code,
        "contest_full"
    );
    // Equal liked/disliked flavors have zero gain: neither receives a modifier.
    assert_eq!(
        feed([0; 6], [0, 0, 0, 25, 25, 20], [0, 0, 0, -1, 1]).unwrap(),
        [0, 0, 0, 25, 25, 20]
    );
    assert_eq!(
        feed([0; 6], [0, 0, 0, 25, 35, 20], [0, 0, 0, -1, 1]).unwrap(),
        [0, 0, 0, 25, 39, 20]
    );
    assert_eq!(
        feed([0; 6], [0, 0, 0, 35, 25, 20], [0, 0, 0, -1, 1]).unwrap(),
        [0, 0, 0, 31, 25, 20]
    );
}

#[test]
fn contest_fields_are_independent_bytes_across_codecs_and_pid_orders() {
    for profile in profile::PROFILES {
        let r = adapter_rom(profile);
        for pid in 0..24 {
            let before =
                pokemon::create(&r, 1, 0x12345678, "TEST", 50, 0xb4f35640 / 24 * 24 + pid).unwrap();
            let condition = [255, 255, 255, 255, 255, 255];
            let (after, _) = pokemon::edit(
                &before,
                &PokemonPatch {
                    condition: Some(condition),
                    ..Default::default()
                },
                &r,
                Policy::Standard,
            )
            .unwrap();
            let a = pokemon::checked_unpack_with(&after, profile.save.pokemon_codec).unwrap();
            let b = pokemon::unpack_with(&before, profile.save.pokemon_codec).unwrap();
            assert_eq!(&a[..30], &b[..30]);
            assert_eq!(&a[30..36], &condition);
            assert_eq!(&a[36..], &b[36..]);
            assert_eq!(&after[..28], &before[..28]);
            assert_eq!(&after[30..32], &before[30..32]);
            assert_eq!(pokemon::decode(&after, &r).unwrap().condition, condition);
        }
    }
    for invalid in [
        "[0,256,0,0,0,0]",
        "[0,-1,0,0,0,0]",
        "[0,null,0,0,0,0]",
        "[0,1.5,0,0,0,0]",
        "[0,1,0,0,0]",
    ] {
        assert!(
            serde_json::from_str::<PokemonPatch>(&format!("{{\"condition\":{invalid}}}")).is_err()
        );
    }
}

#[test]
#[ignore = "requires exact local ROMs and native contest probes"]
fn contest_matches_native_feeding_and_npc_bounds() {
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_CONTEST_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for name in ["BW", "DP", "ROCKET"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{name}")).unwrap()).unwrap())
                .unwrap();
        let entry = probes
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["profile"] == name)
            .unwrap();
        assert_eq!(entry["md5"], r.profile.md5);
        for case in entry["cases"].as_array().unwrap() {
            let nature = case["nature"].as_u64().unwrap() as u8;
            let before: [u8; 6] = serde_json::from_value(case["before"].clone()).unwrap();
            let block = serde_json::from_value(case["block"].clone()).unwrap();
            let after: [u8; 6] = serde_json::from_value(case["after"].clone()).unwrap();
            let result = crate::contest::feed(
                before,
                block,
                crate::contest::preferences(&r, nature).unwrap(),
            );
            if before[5] == 255 {
                assert!(result.is_err());
                assert_eq!(before, after);
            } else {
                assert_eq!(result.unwrap(), after);
            }
        }
        if name == "ROCKET" {
            assert_eq!(
                crate::contest::check_npc(&r, 9, [255; 6]).unwrap_err().code,
                "unsupported_feature"
            );
            continue;
        }
        let result = crate::contest::check_npc(&r, 9, [255; 6]).unwrap();
        assert_eq!(result.status, "outside_npc_bound");
        assert_eq!(result.minimum_sheen_lower_bound, None);
        assert_eq!(
            crate::contest::check_npc(&r, 9, [0, 255, 0, 0, 0, 0])
                .unwrap()
                .status,
            "outside_npc_bound"
        );
        assert_eq!(
            crate::contest::check_npc(&r, 9, [0, 255, 0, 0, 0, 227])
                .unwrap()
                .status,
            "not_disproved"
        );
        let before = pokemon::create(&r, 328, 1, "TEST", 24, 0xb4f35647).unwrap();
        let patch = PokemonPatch {
            condition: Some([0, 255, 0, 0, 0, 0]),
            contest_scope: Some(crate::contest::ContestScope::Npc),
            ..Default::default()
        };
        assert_eq!(
            pokemon::edit(&before, &patch, &r, Policy::Standard)
                .unwrap_err()
                .code,
            "contest_npc_unreachable"
        );
        let (exception, findings) = pokemon::edit(&before, &patch, &r, Policy::Free).unwrap();
        assert!(findings.iter().any(|f| f.code == "contest_npc_unreachable"));
        // Existing exceptional data must not prevent an unrelated edit.
        let (renamed, _) = pokemon::edit(
            &exception,
            &PokemonPatch {
                nickname: Some("TEST".into()),
                ..Default::default()
            },
            &r,
            Policy::Standard,
        )
        .unwrap();
        assert_eq!(
            pokemon::decode(&renamed, &r).unwrap().condition,
            patch.condition.unwrap()
        );
        // Re-read modified table bytes; no cached catalog may override the ROM.
        let mut modified = r.clone();
        let config = modified.profile.contest.unwrap().npc_blender.unwrap();
        std::sync::Arc::make_mut(&mut modified.data)[config.berries + config.flavors_offset + 5] =
            0;
        assert!(
            crate::contest::npc_upper_blocks(&modified, 9).is_err()
                || crate::contest::npc_upper_blocks(&modified, 9).unwrap()
                    != crate::contest::npc_upper_blocks(&r, 9).unwrap()
        );
    }
}

/// Synthetic labels deliberately differ from every supported game's names.
fn label_fixture(b: &mut [u8], p: profile::Profile) {
    if let Some(rules) = p.event_state.and_then(|r| r.effects) {
        for (opcode, handler) in [8, 9, 3, 0x66, 0x67, 0x6e, 0x28, 0x68]
            .into_iter()
            .zip(rules.presentation_handlers)
        {
            put32(b, rules.commands + opcode * 4, 0x08000001 + handler as u32);
        }
        for (opcode, handler) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
            .into_iter()
            .zip(rules.handlers)
        {
            put32(b, rules.commands + opcode * 4, 0x08000001 + handler as u32);
        }
        put32(
            b,
            rules.commands + 0x5c * 4,
            0x08000001 + rules.battle_handler as u32,
        );
    }
    if let Some(rules) = p.resource_checks {
        let address = (rules.item_sanitizer - 0x08000000) as usize;
        // Synthetic identity sanitizer; native redirects are tested separately.
        put16(b, address, 0x4770);
    }
    put16(b, p.script_pokemon.egg_level_instruction, 0x2201);
    let codec = Codec::new();
    for n in 0..25 {
        let address = 0x18000 + n * 64;
        put32(b, p.nature_names + n * 4, 0x08000000 + address as u32);
        b[address..address + 64].copy_from_slice(&codec.encode(&format!("NATURE{n}"), 64).unwrap());
        for stat in 0..5 {
            b[p.nature_effects + n * 5 + stat] = if n / 5 == n % 5 {
                0
            } else if stat == n / 5 {
                1
            } else if stat == n % 5 {
                255
            } else {
                0
            };
        }
    }
    for id in 0..p.type_names.count {
        let address = p.type_names.offset + id * p.type_names.stride;
        b[address..address + p.type_names.stride].copy_from_slice(
            &codec
                .encode(&format!("T{id}"), p.type_names.stride)
                .unwrap(),
        );
    }
}

fn rom() -> Rom {
    static ROM: OnceLock<Rom> = OnceLock::new();
    ROM.get_or_init(|| {
        let mut b = vec![0; 0x2000000];
        let p = profile::BW;
        let codec = Codec::new();
        label_fixture(&mut b, p);
        if let Some(table) = p.experience_table {
            for growth in 0..6 {
                for level in 0..=100 {
                    put32(
                        &mut b,
                        table.offset + growth * table.stride + level * 4,
                        if level == 1 {
                            1
                        } else {
                            pokemon::experience(growth as u8, level as u8)
                        },
                    );
                }
            }
        }
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
                put32(&mut b, o + layout.key, 0x87654321);
                put16(&mut b, o + 10, 0x5678);
                put16(&mut b, o + 12, 0x1234);
            }
            if id == 1 {
                b[o + layout.party_count] = 1;
                b[o + layout.party..o + layout.party + 100].copy_from_slice(&party);
                put32(&mut b, o + layout.money, 1000 ^ 0x87654321);
                put16(&mut b, o + layout.coins, 50 ^ 0x4321);
            }
            put16(&mut b, o + 0xff4, id as u16);
            put32(&mut b, o + 0xff8, 0x08012025);
            put32(&mut b, o + 0xffc, if bank == 0 { 10 } else { 9 });
            let sum = if layout.sector_checksum == profile::SectorChecksum::NativeConstantOne {
                1
            } else {
                sector_checksum(&b[o..o + layout.sizes[id]])
            };
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
        pokemon::stats_with_changes([45, 49, 49, 45, 65, 65], [31; 6], [0; 6], 50, [0; 5], true),
        [120, 69, 69, 65, 85, 85]
    );
    assert_eq!(
        pokemon::stats_with_changes(
            [45, 49, 49, 45, 65, 65],
            [31; 6],
            [0; 6],
            50,
            [1, 0, 0, -1, 0],
            true
        ),
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
fn save_roundtrip_keeps_speed_ivs_evs_and_ability_in_one_edit() {
    let mut session = session();
    session
        .apply(
            Action::Pokemon {
                location: party(),
                patch: PokemonPatch {
                    ivs: Some([10, 10, 10, 31, 10, 10]),
                    evs: Some([0, 0, 0, 252, 0, 0]),
                    ability_slot: Some(1),
                    ..Default::default()
                },
            },
            Policy::Standard,
        )
        .unwrap();
    let bytes = session.save_ref().unwrap().data.clone();
    let exported = Save::open(bytes, session.rom.profile.save).unwrap();
    exported.validate(&session.rom).unwrap();
    let pokemon = exported.pokemon(party(), &session.rom).unwrap().unwrap();
    assert_eq!(pokemon.ivs[3], 31);
    assert_eq!(pokemon.evs[3], 252);
    assert_eq!(pokemon.ability_slot, 1);
}

#[test]
fn save_export_rejects_a_rom_path_without_writing() {
    let mut session = session();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("game.GBA");
    std::fs::write(&path, b"untouched ROM placeholder").unwrap();
    assert_eq!(session.export(&path).unwrap_err().code, "rom_write");
    assert_eq!(std::fs::read(&path).unwrap(), b"untouched ROM placeholder");
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
    let payload = save.layout.sizes[5];
    for (i, byte) in raw.iter().enumerate() {
        let logical = 4 + index * 80 + i;
        output[save.sections[5 + logical / payload] + logical % payload] = *byte;
    }
    for section in 5..=13 {
        let start = save.sections[section];
        let checksum = if save.layout.sector_checksum == profile::SectorChecksum::NativeConstantOne
        {
            1
        } else {
            sector_checksum(&output[start..start + save.layout.sizes[section]])
        };
        put16(output, start + 0xff6, checksum);
    }
}

#[test]
fn transfers_preserve_every_record_byte_across_all_box_slots() {
    for profile in profile::PROFILES {
        let rom = adapter_rom(profile);
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
        let family = r.species_relations(9).unwrap();
        assert!(family.species.starts_with(&[7, 8, 9]));
        assert!(family
            .evolutions
            .iter()
            .any(|e| e.source == 8 && e.evolution.target == 9));
        assert!(family.form_families.is_empty());
        assert_eq!(pokemon::rom_experience(&r, 0, 1).unwrap(), 1);
        assert_eq!(pokemon::rom_experience(&r, 0, 100).unwrap(), 1_000_000);
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
        invalid_events: false,
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
        invalid_events: false,
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
    assert_eq!(first.evs, Some([0; 6]));
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
        invalid_events: false,
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
        invalid_events: false,
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
    // An unverified adapter must not inherit another profile's collection index protocol.
    r.profile.formats.scripts = crate::adapter::ScriptFormat::FireRed;
    r.profile.hidden_items = None;
    let fire_red = r.map_events(&map).unwrap();
    let hidden = fire_red
        .markers
        .iter()
        .find(|m| m.kind == "hidden")
        .unwrap();
    assert_eq!(hidden.flag, None);
    assert_eq!(hidden.rewards[0].quantity, None);
    assert!(hidden.stopped_at.contains(&bg));
    // Exact-adapter rules own the format, including Mercury's runtime region override.
    r.profile.hidden_items = Some(crate::profile::HiddenItemRules {
        packed: true,
        flag_base: 0x3e8,
        region_override: Some((0x25400, 0xd00)),
    });
    {
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[0x25400..0x25402].copy_from_slice(&[0, 0xff]);
        b[bg + 10] = 10;
        b[bg + 11] = 5;
    }
    let hidden = r.map_events(&map).unwrap().markers.remove(1);
    assert_eq!(
        (hidden.flag, hidden.underfoot, hidden.rewards[0].quantity),
        (Some(0xd0a), Some(false), Some(5))
    );
    std::sync::Arc::make_mut(&mut r.data)[bg + 11] = 0x85;
    let hidden = r.map_events(&map).unwrap().markers.remove(1);
    assert_eq!(
        (hidden.flag, hidden.underfoot, hidden.rewards[0].quantity),
        (Some(0xd0a), Some(true), Some(1))
    );
    // Editing the synthetic ROM table changes the result; the override is not bundled.
    std::sync::Arc::make_mut(&mut r.data)[0x25400] = 1;
    assert_eq!(r.map_events(&map).unwrap().markers[1].flag, Some(0x3f2));
    // Bad pointers must fail safely, never reinterpret an item's ID as a script.
    let b = std::sync::Arc::make_mut(&mut r.data);
    put32(b, ev + 16, 0xfffffff0);
    assert!(r.map_events(&map).is_err());
}

#[test]
fn pokemon_script_operands_follow_each_adapter_and_runtime_egg_level() {
    use crate::script_pokemon::WildCommand;
    for p in profile::PROFILES {
        let mut r = adapter_rom(p);
        let pc = 0x25000;
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[pc..pc + 18].fill(0);
        b[pc] = 0xb6;
        put16(b, pc + 1, 1);
        b[pc + 3] = 20;
        let resolve = |v| {
            if v == 0x8004 {
                Some(2)
            } else if v < 0x4000 {
                Some(v)
            } else {
                None
            }
        };
        assert_eq!(
            r.wild_command_length(pc).unwrap(),
            match p.script_pokemon.wild {
                WildCommand::RocketExtended => 11,
                _ => 6,
            }
        );
        let source = r.script_pokemon_instruction(pc, resolve).unwrap();
        assert_eq!(
            (source[0].species, source[0].level),
            (1, Some(20)),
            "{}",
            p.id
        );
        put16(
            std::sync::Arc::make_mut(&mut r.data).as_mut_slice(),
            pc + 1,
            0x8004,
        );
        let source = r.script_pokemon_instruction(pc, resolve).unwrap();
        if matches!(p.script_pokemon.wild, WildCommand::MercuryDouble) {
            assert_eq!(source[0].species, 2);
            let b = std::sync::Arc::make_mut(&mut r.data);
            put16(b, pc + 1, 0xffff);
            put16(b, pc + 7, 0x8004);
            b[pc + 9] = 21;
            put16(b, pc + 13, 3);
            b[pc + 15] = 22;
            assert_eq!(r.wild_command_length(pc).unwrap(), 18);
            let source = r.script_pokemon_instruction(pc, resolve).unwrap();
            assert_eq!(
                source
                    .iter()
                    .map(|s| (s.species, s.level, s.member))
                    .collect::<Vec<_>>(),
                [(2, Some(21), 0), (3, Some(22), 1)]
            );
        } else {
            assert!(
                source.is_empty(),
                "literal native operands must not become VarGet: {}",
                p.id
            );
        }
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[pc] = 0x7a;
        put16(b, pc + 1, 1);
        put16(b, p.script_pokemon.egg_level_instruction, 0x2201);
        assert_eq!(
            r.script_pokemon_instruction(pc, resolve).unwrap()[0].level,
            Some(1)
        );
        put16(
            std::sync::Arc::make_mut(&mut r.data).as_mut_slice(),
            p.script_pokemon.egg_level_instruction,
            0x2205,
        );
        assert_eq!(
            r.script_pokemon_instruction(pc, resolve).unwrap()[0].level,
            Some(5)
        );
        put16(
            std::sync::Arc::make_mut(&mut r.data).as_mut_slice(),
            p.script_pokemon.egg_level_instruction,
            0x46c0,
        );
        assert!(r.script_pokemon_instruction(pc, resolve).is_err());
    }
}

#[test]
fn scripted_pokemon_keep_npc_tile_and_unplaced_sources_separate() {
    let mut r = rom();
    let ev = 0x25000;
    let object = 0x25100;
    let gift = 0x25200;
    let egg = 0x25300;
    let b = std::sync::Arc::make_mut(&mut r.data);
    b[ev] = 1;
    put32(b, ev + 4, 0x08000000 + object as u32);
    b[object] = 1;
    put16(b, object + 4, 11);
    put16(b, object + 6, 7);
    put32(b, object + 16, 0x08000000 + gift as u32);
    b[gift] = 0x79;
    put16(b, gift + 1, 1);
    b[gift + 3] = 20;
    b[gift + 15] = 2;
    b[egg] = 0x7a;
    put16(b, egg + 1, 2);
    b[egg + 3] = 2;
    let map = crate::world::Map {
        id: "0-0".into(),
        group: 0,
        number: 0,
        name: "Synthetic".into(),
        region: 0,
        width: 20,
        height: 20,
        map_type: 0,
        header: 0,
        layout: 0,
        invalid_events: false,
        events: Some(ev),
        scripts: vec![gift, egg],
        objects: vec![],
    };
    let report = r.map_events(&map).unwrap();
    assert_eq!(report.markers.len(), 1);
    let marker = &report.markers[0];
    assert_eq!((marker.kind, marker.x, marker.y), ("gift", 11, 7));
    assert_eq!(marker.pokemon[0].species, 1);
    assert!(marker.rewards.is_empty());
    assert_eq!(report.unplaced_pokemon.len(), 1);
    assert_eq!(report.unplaced_pokemon[0].species, 2);
    assert_eq!(report.unplaced_pokemon[0].method, "egg");
}

fn npc_trade_fixture(p: profile::Profile) -> (Rom, crate::world::Map) {
    let mut r = adapter_rom(p);
    let rule = r.profile.script_pokemon.trade.unwrap();
    let b = std::sync::Arc::make_mut(&mut r.data);
    put32(
        b,
        rule.specials + rule.information_special as usize * 4,
        0x08000001 + rule.information_code as u32,
    );
    put32(b, rule.table_pointer, 0x08027000);
    put16(b, 0x27000 + 12, 2);
    put16(b, 0x27000 + 40, 1);
    put16(b, 0x27000 + 56, 1);
    b[0x26000..0x2600b].copy_from_slice(&[
        0x16,
        4,
        0x80,
        0,
        0,
        0x26,
        0x0d,
        0x80,
        rule.information_special as u8,
        (rule.information_special >> 8) as u8,
        2,
    ]);
    b[0x25000] = 1;
    put32(b, 0x25004, 0x08025100);
    b[0x25100] = 1;
    put16(b, 0x25104, 3);
    put16(b, 0x25106, 2);
    put32(b, 0x25110, 0x08026000);
    let map = crate::world::Map {
        id: "0-0".into(),
        group: 0,
        number: 0,
        name: "Synthetic trade room".into(),
        region: 1,
        width: 4,
        height: 4,
        map_type: 1,
        header: 0,
        layout: 0,
        invalid_events: false,
        events: Some(0x25000),
        scripts: vec![0x26000],
        objects: vec![],
    };
    (r, map)
}

fn npc_tutor_fixture(p: profile::Profile) -> (Rom, crate::world::Map) {
    let (mut r, map) = npc_trade_fixture(p);
    let rules = r.profile.tutor_scripts.unwrap();
    let b = std::sync::Arc::make_mut(&mut r.data);
    put32(
        b,
        rules.specials + rules.special as usize * 4,
        0x08000001 + rules.code as u32,
    );
    if let crate::script_teaching::TutorParameter::Index { getter, .. } = rules.parameter {
        // Native Thumb getter returns the synthetic move 2. No catalogs are injected.
        let o = (getter - 0x08000000) as usize;
        b[o..o + 4].copy_from_slice(&[2, 0x20, 0x70, 0x47]);
    }
    let parameter = if matches!(
        rules.parameter,
        crate::script_teaching::TutorParameter::MoveId
    ) {
        2
    } else {
        1
    };
    // Conditional offer, followed by a native call; the flag is an access guard,
    // not a teaching receipt, despite also being the object's visibility flag.
    b[0x26000..0x26016].copy_from_slice(&[
        0x2b,
        0x17,
        1,
        6,
        1,
        0x15,
        0x60,
        2,
        8,
        0x16,
        5,
        0x80,
        parameter,
        0,
        0x25,
        rules.special as u8,
        (rules.special >> 8) as u8,
        2,
        0,
        0,
        0,
        2,
    ]);
    put16(b, 0x25114, 0x117);
    (r, map)
}
#[test]
fn tutor_sources_preserve_native_selector_guards_tiles_and_read_only_queries() {
    use crate::acquisition::{AcquisitionIndex, Target, TargetKind};
    for p in profile::PROFILES {
        let (mut r, map) = npc_tutor_fixture(p);
        let original = r.data.clone();
        let report = r.map_events(&map).unwrap();
        let marker = &report.markers[0];
        assert_eq!(marker.kind, "npc");
        assert_eq!((marker.x, marker.y), (3, 2));
        assert_eq!(marker.teaching.len(), 1);
        assert_eq!(marker.teaching[0].move_id, 2);
        assert!(marker.teaching[0]
            .conditions
            .iter()
            .any(|c| c.kind == "flag" && c.id == 0x117 && !c.taken));
        assert!(report.unplaced_teaching.is_empty());
        assert_eq!(marker.receipt_flag, None);
        let index = AcquisitionIndex {
            wild_cache: std::cell::RefCell::default(),
            breeding_cache: Default::default(),
            world: crate::world::World {
                maps: vec![map.clone()],
                map_events: vec![report],
                encounters: vec![],
                trainers: vec![],
                trainer_locations: crate::world::TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: vec![],
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let target = Target {
            kind: TargetKind::Move,
            id: 2,
        };
        let query = index.query(&r, None, target.clone()).unwrap();
        assert_eq!(query.sources[0].kind, "move_tutor");
        assert_eq!(query.sources[0].status, "unknown");
        assert_eq!(
            query.sources[0].teaching_source.as_ref().unwrap().move_id,
            2
        );
        let save = Save::open(save_bytes(&r), p.save).unwrap();
        let before = save.data.clone();
        let saved_query = index.query(&r, Some(&save), target).unwrap();
        assert_ne!(saved_query.sources[0].status, "completed");
        assert!(saved_query.sources[0].partial);
        assert_eq!(saved_query.sources[0].repeatable, None);
        assert_eq!(save.data, before);
        assert_eq!(r.data, original);
        let rules = r.profile.tutor_scripts.unwrap();
        if let crate::script_teaching::TutorParameter::Index { count, .. } = rules.parameter {
            assert_eq!(r.tutor_move(256).unwrap(), 2);
            assert_eq!(r.tutor_move(count).unwrap_err().code, "tutor_index");
        }
        let b = std::sync::Arc::make_mut(&mut r.data);
        put32(b, rules.specials + rules.special as usize * 4, 0x08028001);
        let failed = r.map_events(&map).unwrap();
        assert!(failed.markers[0].teaching.is_empty());
        assert!(failed.markers[0].stopped_at.contains(&0x2600e));
        // A map-level invocation cannot borrow a nearby NPC's tile.
        let (r, mut map) = npc_tutor_fixture(p);
        map.events = None;
        let unplaced = r.map_events(&map).unwrap();
        assert!(unplaced.markers.is_empty());
        assert_eq!(unplaced.unplaced_teaching.len(), 1);
    }
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_TUTOR_PROBES native lookup/menu evidence"]
fn local_teaching_sources_match_native_getters_and_crosslinks() {
    use crate::acquisition::{AcquisitionIndex, Target, TargetKind};
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_TUTOR_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        assert_eq!(probes[key]["md5"], r.profile.md5);
        let rows = probes[key]["rows"].as_array().unwrap();
        for row in rows {
            let index = row["parameter"].as_u64().unwrap() as u16;
            let native = row["move_id"].as_u64().unwrap() as u16;
            assert_eq!(r.tutor_move(index).unwrap(), native);
            // Independently validate the ordinary learnset table against native lookup.
            if index < r.profile.teaching.tutor_count as u16 {
                assert_eq!(
                    u16(&r.data, r.profile.tutor_moves + index as usize * 2).unwrap(),
                    native
                );
            }
        }
        for row in probes[key]["direct_compatibility"].as_array().unwrap() {
            let move_id = row["parameter"].as_u64().unwrap() as u16;
            assert_eq!(r.tutor_move(move_id).unwrap(), move_id);
        }
        let index = AcquisitionIndex::build(&r).unwrap();
        let offers: Vec<_> = index
            .world
            .map_events
            .iter()
            .flat_map(|r| {
                r.markers
                    .iter()
                    .flat_map(|m| &m.teaching)
                    .chain(&r.unplaced_teaching)
            })
            .collect();
        assert!(
            !offers.is_empty(),
            "{key}: teaching references must be discovered"
        );
        for offer in &offers {
            assert_eq!(r.tutor_move(offer.parameter).unwrap(), offer.move_id);
            let query = index
                .query(
                    &r,
                    None,
                    Target {
                        kind: TargetKind::Move,
                        id: offer.move_id,
                    },
                )
                .unwrap();
            assert!(query.sources.iter().any(|s| s.kind == "move_tutor"
                && s.offset == offer.offset
                && s.teaching_source.as_ref().unwrap() == *offer));
            assert_eq!(query.sources[0].kind, "move_tutor");
            assert!(query
                .sources
                .iter()
                .filter(|s| s.kind == "move_tutor")
                .all(|s| s.status == "unknown" && s.partial && s.receipt_flag.is_none()));
        }
        eprintln!(
            "{key}: {} native indexed getters and {} parsed teaching references",
            rows.len(),
            offers.len()
        );
    }
}

#[test]
fn npc_trade_sources_use_native_dispatch_runtime_records_and_mode_guards() {
    for p in profile::PROFILES {
        let (mut r, map) = npc_trade_fixture(p);
        let rule = p.script_pokemon.trade.unwrap();
        let report = r.map_events(&map).unwrap();
        assert_eq!(report.markers.len(), 1, "{}", p.id);
        assert!(report.unplaced_pokemon.is_empty());
        let marker = &report.markers[0];
        assert_eq!((marker.x, marker.y), (3, 2));
        let source = &marker.pokemon[0];
        assert_eq!(
            (
                source.species,
                source.method,
                source.level,
                source.held_item
            ),
            (2, "npc_trade", None, Some(1))
        );
        assert_eq!(source.trade.as_ref().unwrap().requested_species, 1);
        assert!(r.trade_offer(rule.count).is_err());
        if let Some(flag) = rule.alternate_flag {
            assert!(source.conditions.iter().any(|c| c.id == flag && !c.taken));
            let b = std::sync::Arc::make_mut(&mut r.data);
            b[0x25ffd] = 0x29;
            put16(b, 0x25ffe, flag);
            put32(b, 0x25110, 0x08025ffd);
            assert!(r.map_events(&map).unwrap().markers[0].pokemon.is_empty());
            std::sync::Arc::make_mut(&mut r.data)[0x25ffd] = 0x2a;
            assert!(r.map_events(&map).unwrap().markers[0].pokemon[0]
                .conditions
                .is_empty());
        }
        put16(
            std::sync::Arc::make_mut(&mut r.data).as_mut_slice(),
            0x27000 + 12,
            3,
        );
        assert_eq!(r.trade_offer(0).unwrap().0, 3);
        put32(
            std::sync::Arc::make_mut(&mut r.data).as_mut_slice(),
            rule.specials + rule.information_special as usize * 4,
            0x08000001,
        );
        assert_eq!(r.trade_offer(0).unwrap_err().code, "trade_dispatch");
    }
}

#[test]
fn npc_trade_queries_and_plans_keep_donor_requirements_and_save_read_only() {
    use crate::{
        acquisition::{AcquisitionIndex, Target, TargetKind},
        collection::{CollectionBasis, CollectionRequest},
    };
    for p in profile::PROFILES {
        let (r, map) = npc_trade_fixture(p);
        let report = r.map_events(&map).unwrap();
        let index = AcquisitionIndex {
            wild_cache: std::cell::RefCell::default(),
            breeding_cache: Default::default(),
            world: crate::world::World {
                maps: vec![map],
                map_events: vec![report],
                encounters: vec![],
                trainers: vec![],
                trainer_locations: crate::world::TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: (1..=3).map(|id| r.valid_species(id).unwrap()).collect(),
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let target = Target {
            kind: TargetKind::Species,
            id: 2,
        };
        let rom_only = index.query(&r, None, target.clone()).unwrap();
        assert_eq!(rom_only.sources[0].status, "unknown");
        assert!(rom_only.sources[0].trade_context.is_none());
        let mut save = Save::open(save_bytes(&r), p.save).unwrap();
        let before = save.data.clone();
        let report = index.query(&r, Some(&save), target.clone()).unwrap();
        assert_eq!(report.sources[0].status, "unknown");
        assert_eq!(
            report.sources[0]
                .trade_context
                .as_ref()
                .unwrap()
                .party_levels,
            [50]
        );
        assert_eq!(report.sources[0].receipt_flag, None);
        let items = index
            .query(
                &r,
                Some(&save),
                Target {
                    kind: TargetKind::Item,
                    id: 1,
                },
            )
            .unwrap();
        assert!(items.sources.iter().any(|s| s.kind == "npc_trade_item"
            && s.quantity == Some(1)
            && s.related
                .iter()
                .any(|t| t.kind == TargetKind::Species && t.id == 2)));
        let plan = index
            .collection(
                &r,
                &save,
                CollectionRequest {
                    basis: CollectionBasis::Individuals,
                    families: true,
                    include_unknown_rewards: true,
                },
            )
            .unwrap();
        assert!(plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .any(|t| t.target.id == 2
                && t.source
                    .as_ref()
                    .unwrap()
                    .script_source
                    .as_ref()
                    .unwrap()
                    .trade
                    .is_some()));
        assert!(plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .any(|t| t.target.kind == TargetKind::Item
                && t.target.id == 1
                && t.source.as_ref().unwrap().kind == "npc_trade_item"));
        assert_eq!(save.data, before);
        save.transfer(party(), loc(5), true, &r).unwrap();
        save.edit(
            party(),
            &PokemonPatch {
                species: Some(3),
                ..Default::default()
            },
            &r,
            Policy::Free,
        )
        .unwrap();
        let report = index.query(&r, Some(&save), target.clone()).unwrap();
        let c = report.sources[0].trade_context.as_ref().unwrap();
        assert!(c.party_levels.is_empty());
        assert!(!c.box_levels.is_empty());
        assert_eq!(report.sources[0].status, "blocked");
        let known_only = index
            .collection(
                &r,
                &save,
                CollectionRequest {
                    basis: CollectionBasis::Individuals,
                    families: true,
                    include_unknown_rewards: false,
                },
            )
            .unwrap();
        assert!(known_only
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .all(|t| t.target.kind != TargetKind::Item));
        // Eggs are not eligible donors; the query must use the exact requested species.
        for stored in save
            .all(&r)
            .unwrap()
            .into_iter()
            .filter(|s| s.pokemon.species == 1)
        {
            save.edit(
                stored.location,
                &PokemonPatch {
                    egg: Some(true),
                    ..Default::default()
                },
                &r,
                Policy::Free,
            )
            .unwrap();
        }
        let before = save.data.clone();
        let report = index.query(&r, Some(&save), target).unwrap();
        let c = report.sources[0].trade_context.as_ref().unwrap();
        assert!(c.party_levels.is_empty() && c.box_levels.is_empty());
        assert_eq!(save.data, before);
    }
}

#[test]
#[ignore = "requires five exact private ROMs and GEN3_TRADE_PROBES native vectors"]
fn local_npc_trades_match_native_quote_and_generation() {
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_TRADE_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        assert_eq!(probes[key]["md5"], r.profile.md5);
        let rows = probes[key]["rows"].as_array().unwrap();
        assert_eq!(
            rows.len(),
            r.profile.script_pokemon.trade.unwrap().count as usize
        );
        for row in rows {
            let (received, held, trade) = r
                .trade_offer(row["index"].as_u64().unwrap() as u16)
                .unwrap();
            assert_eq!(
                trade.requested_species as u64,
                row["requested_species"].as_u64().unwrap()
            );
            for case in row["cases"].as_array().unwrap() {
                assert_eq!(received as u64, case["species"].as_u64().unwrap());
                assert_eq!(held as u64, case["held_item"].as_u64().unwrap());
                assert_eq!(case["level"], case["offered_level"]);
            }
        }
        let index = crate::acquisition::AcquisitionIndex::build(&r).unwrap();
        let sources: Vec<_> = index
            .world
            .map_events
            .iter()
            .flat_map(|m| {
                m.markers
                    .iter()
                    .flat_map(|m| &m.pokemon)
                    .chain(&m.unplaced_pokemon)
            })
            .filter(|m| m.trade.is_some())
            .collect();
        assert!(
            !sources.is_empty(),
            "{key}: referenced offers must be indexed"
        );
        assert!(sources
            .iter()
            .all(|m| m.level.is_none() && m.method == "npc_trade"));
        for mon in &sources {
            let query = index
                .query(
                    &r,
                    None,
                    crate::acquisition::Target {
                        kind: crate::acquisition::TargetKind::Species,
                        id: mon.species,
                    },
                )
                .unwrap();
            assert!(query.sources.iter().any(|s| s.kind == "npc_trade"
                && s.offset == mon.offset
                && s.script_source.as_ref().unwrap().trade == mon.trade));
            if let Some(item) = mon.held_item.filter(|id| *id != 0) {
                let query = index
                    .query(
                        &r,
                        None,
                        crate::acquisition::Target {
                            kind: crate::acquisition::TargetKind::Item,
                            id: item,
                        },
                    )
                    .unwrap();
                assert!(query.sources.iter().any(|s| s.kind == "npc_trade_item"
                    && s.offset == mon.offset
                    && s.related
                        .iter()
                        .any(|t| t.kind == crate::acquisition::TargetKind::Species
                            && t.id == mon.species)));
            }
        }
        if let Some(flag) = r.profile.script_pokemon.trade.unwrap().alternate_flag {
            assert_eq!(
                probes[key]["alternate"]["flag"].as_u64().unwrap(),
                flag as u64
            );
            assert_eq!(probes[key]["alternate"]["different_constructor"], true);
        }
        eprintln!(
            "{key}: {} table rows match native generation; {} parsed map offer references",
            rows.len(),
            sources.len()
        );
    }
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
    r.profile.wild_items = None;
    if p.id == profile::BW.id || p.id == profile::DP.id {
        // Synthetic BW/DP tables share test addresses, while identity/capabilities differ.
        r.profile.id = p.id;
        r.profile.md5 = p.md5;
        return r;
    }
    let mut b = vec![0; p.size];
    label_fixture(&mut b, p);
    if let Some(table) = p.experience_table {
        for growth in 0..table.count {
            for level in 0..=p.max_level {
                put32(
                    &mut b,
                    table.offset + growth * table.stride + level as usize * 4,
                    pokemon::experience(growth as u8, level),
                );
            }
        }
    }
    let expanded = p.formats.species == crate::adapter::SpeciesFormat::Expanded36;
    for id in 1..4 {
        let o = p.base_stats.offset + id * p.base_stats.stride;
        b[o..o + 6].copy_from_slice(&[45, 49, 49, 45, 65, 65]);
        let tail = if expanded { 2 } else { 0 };
        b[o + 16 + tail] = 127;
        b[o + 18 + tail] = 70;
        if let Some(table) = p.species_abilities {
            for (slot, ability) in [1, 2, 267].iter().enumerate() {
                put16(
                    &mut b,
                    table.offset + id * table.stride + slot * 2,
                    *ability,
                );
            }
        } else if p.formats.species == crate::adapter::SpeciesFormat::Cfru28 {
            b[o + 22] = 1;
            b[o + 23] = 2;
            b[o + 26] = 3;
        } else {
            put16(&mut b, o + 24, 1);
            put16(&mut b, o + 26, 2);
            put16(&mut b, o + 28, 267);
        }
        put32(&mut b, p.learnsets + id * 4, 0x08020000);
        if let Some(table) = p.teaching.shared_lists {
            put32(&mut b, table + id * 4, 0x08020100);
        }
    }
    put16(&mut b, 0x20000, 1);
    put16(&mut b, 0x20002, 1);
    put16(&mut b, 0x20004, 0xffff);
    if p.formats.learnsets == crate::adapter::LearnsetFormat::Move16Level8 {
        b[0x20000..0x20006].copy_from_slice(&[1, 0, 1, 0, 0, 255]);
        for index in 0..5 {
            put32(&mut b, 0x1f08268 + index * 4, 0x08021001 + index as u32 * 2);
            put16(&mut b, 0x21000 + index * 2, 0x2301 + index as u16);
        }
    }
    for id in 1..3 {
        b[p.moves.offset + id * p.moves.stride + if expanded { 6 } else { 4 }] = 35;
    }
    r.data = std::sync::Arc::new(b);
    r.profile = profile::Profile {
        wild_items: None,
        ..p
    };
    r
}

#[test]
fn adapter_matrix_bit_ownership_all_pid_orders() {
    for profile in profile::PROFILES {
        let r = adapter_rom(profile);
        let rocket = profile.save.pokemon_codec == crate::adapter::PokemonCodec::Rocket21;
        for pid in 0..24 {
            let mut raw = pokemon::create(&r, 1, 0x12345678, "ASH", 50, pid).unwrap();
            let mut c = pokemon::unpack_with(&raw, profile.save.pokemon_codec).unwrap();
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
            pokemon::pack_with(&mut raw, &c, profile.save.pokemon_codec);
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
            if profile.save.pokemon_codec == crate::adapter::PokemonCodec::Cfru {
                // CFRU chooses the two ordinary abilities by PID parity. Its
                // ability capsule also changes PID while retaining nature and sex.
                assert_eq!(after.pid & 1, 1);
                assert_eq!(&edited[4..28], &raw[4..28]);
            } else {
                assert_eq!(&edited[..28], &raw[..28]);
            }
            assert!(after.checksum_ok);
            let canonical = pokemon::unpack_with(&edited, profile.save.pokemon_codec).unwrap();
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
                if profile.save.pokemon_codec == crate::adapter::PokemonCodec::Ultimate55 {
                    expected[39] = (expected[39] & !0x7c) | (12 << 2);
                } else if profile.save.pokemon_codec == crate::adapter::PokemonCodec::Cfru {
                    expected[10] = 12;
                } else {
                    let origins = (u16(&expected, 38).unwrap() & !0x7800) | (12 << 11);
                    put16(&mut expected, 38, origins);
                }
                if profile.save.pokemon_codec == crate::adapter::PokemonCodec::Cfru {
                    expected[43] &= !0x80;
                } else {
                    expected[43] |= 0x80;
                }
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
fn cfru_ball_hidden_ability_and_gigantamax_bits_do_not_overlap() {
    let rom = adapter_rom(crate::mercury::PROFILE);
    let codec = crate::adapter::PokemonCodec::Cfru;
    let mut raw = pokemon::create(&rom, 1, 0x1234_5678, "ASH", 50, 100).unwrap();
    assert_eq!(pokemon::decode(&raw, &rom).unwrap().origin_game, 4);
    let mut canonical = pokemon::unpack_with(&raw, codec).unwrap();
    canonical[39] |= 0x08; // CFRU Gigantamax flag, formerly Gen III ball bits.
    canonical[43] |= 0x80; // Hidden ability, not ordinary ability 2.
    pokemon::pack_with(&mut raw, &canonical, codec);
    let hidden = pokemon::decode(&raw, &rom).unwrap();
    assert_eq!((hidden.ability_slot, hidden.ability_id), (2, 3));
    let (ball_changed, _) = pokemon::edit(
        &raw,
        &PokemonPatch {
            ball: Some(12),
            ..PokemonPatch::default()
        },
        &rom,
        Policy::Free,
    )
    .unwrap();
    let c = pokemon::unpack_with(&ball_changed, codec).unwrap();
    assert_eq!(c[10], 12);
    assert_eq!(c[39] & 0x08, 0x08);
    assert_eq!(c[43] & 0x80, 0x80);
    let (ordinary, _) = pokemon::edit(
        &ball_changed,
        &PokemonPatch {
            ability_slot: Some(1),
            ..PokemonPatch::default()
        },
        &rom,
        Policy::Free,
    )
    .unwrap();
    let after = pokemon::decode(&ordinary, &rom).unwrap();
    assert_eq!((after.ability_slot, after.ability_id), (1, 2));
    assert_eq!(after.pid & 1, 1);
    assert_eq!(after.nature, hidden.nature);
    assert_eq!(after.gender, hidden.gender);
    assert_eq!(after.shiny, hidden.shiny);
    let c = pokemon::unpack_with(&ordinary, codec).unwrap();
    assert_eq!(c[39] & 0x08, 0x08);
    assert_eq!(c[43] & 0x80, 0);
}

#[test]
fn cfru_single_ordinary_ability_does_not_require_even_pid() {
    let mut rom = adapter_rom(crate::mercury::PROFILE);
    let offset = rom.profile.base_stats.offset + rom.profile.base_stats.stride;
    std::sync::Arc::make_mut(&mut rom.data)[offset + 23] = 0;
    let raw = pokemon::create(&rom, 1, 0x1234_5678, "ASH", 50, 101).unwrap();
    let before = pokemon::decode(&raw, &rom).unwrap();
    assert_eq!(before.ability_slot, 0);
    let (edited, _) = pokemon::edit(
        &raw,
        &PokemonPatch {
            ability_slot: Some(0),
            ..PokemonPatch::default()
        },
        &rom,
        Policy::Standard,
    )
    .unwrap();
    assert_eq!(pokemon::decode(&edited, &rom).unwrap().pid, before.pid);
    assert!(pokemon::edit(
        &raw,
        &PokemonPatch {
            ability_slot: Some(1),
            ..PokemonPatch::default()
        },
        &rom,
        Policy::Free,
    )
    .is_err());
    // A stored hidden-ability flag can survive an evolution into a species
    // without that slot. Native fallback does not authorize clearing it.
    std::sync::Arc::make_mut(&mut rom.data)[offset + 26] = 0;
    let mut flagged = raw.clone();
    flagged[75] |= 0x80;
    assert_eq!(pokemon::decode(&flagged, &rom).unwrap().ability_slot, 0);
    let (edited, _) = pokemon::edit(
        &flagged,
        &PokemonPatch {
            ball: Some(12),
            ..PokemonPatch::default()
        },
        &rom,
        Policy::Free,
    )
    .unwrap();
    assert_eq!(edited[75] & 0x80, 0x80);
}

#[test]
fn speed_ivs_evs_and_ability_survive_each_save_codec() {
    for profile in profile::PROFILES {
        let rom = adapter_rom(profile);
        let slot = if profile.save.pokemon_codec == crate::adapter::PokemonCodec::Rocket21 {
            2
        } else {
            1
        };
        for pid in 0..24 {
            let raw = pokemon::create(&rom, 1, 0x12345678, "ASH", 50, pid).unwrap();
            let (edited, _) = pokemon::edit(
                &raw,
                &PokemonPatch {
                    ivs: Some([10, 10, 10, 31, 10, 10]),
                    evs: Some([0, 0, 0, 252, 0, 0]),
                    ability_slot: Some(slot),
                    ..Default::default()
                },
                &rom,
                Policy::Free,
            )
            .unwrap();
            let after = pokemon::decode(&edited, &rom).unwrap();
            assert_eq!(after.ivs[3], 31, "{} PID {pid}", profile.id);
            assert_eq!(after.evs[3], 252, "{} PID {pid}", profile.id);
            assert_eq!(after.ability_slot, slot, "{} PID {pid}", profile.id);
        }
    }
}

#[test]
#[ignore = "requires GEN3_ROM_BW, GEN3_ROM_DP, GEN3_ROM_ROCKET and GEN3_ROM_ULTIMATE"]
fn gyarados_save_fields_roundtrip_with_exact_roms() {
    for name in ["BW", "DP", "ROCKET", "ULTIMATE"] {
        let path = std::env::var(format!("GEN3_ROM_{name}")).unwrap();
        let rom = Rom::open(std::fs::read(path).unwrap()).unwrap();
        let species = rom.species(130).unwrap();
        assert!(species.name.contains("暴鲤龙"));
        let slot = if species.abilities.len() > 2 { 2 } else { 1 };
        let raw = pokemon::create(&rom, 130, 0x12345678, "TEST", 50, 12345).unwrap();
        let (edited, _) = pokemon::edit(
            &raw,
            &PokemonPatch {
                ivs: Some([10, 10, 10, 31, 10, 10]),
                evs: Some([0, 0, 0, 252, 0, 0]),
                ability_slot: Some(slot),
                ..Default::default()
            },
            &rom,
            Policy::Standard,
        )
        .unwrap();
        let after = pokemon::decode(&edited, &rom).unwrap();
        assert_eq!(after.ivs[3], 31, "{name}");
        assert_eq!(after.evs[3], 252, "{name}");
        assert_eq!(after.ability_slot, slot, "{name}");
        assert_eq!(after.ability_id, species.abilities[slot as usize], "{name}");
    }
}

#[test]
fn adapter_capabilities_reject_writes_without_mutation() {
    let mut disabled = profile::ROCKET;
    disabled.capabilities = crate::adapter::Capabilities {
        save_edit: false,
        world: false,
        dex: false,
        complete_learnsets: false,
        individual_sprites: false,
        battle_forms: true,
    };
    disabled.save.dex = None;
    let r = adapter_rom(profile::ROCKET);
    let mut s = Session::new(r.clone());
    let original = save_bytes(&r);
    s.load(original.clone(), None).unwrap();
    s.rom.profile = disabled;
    s.save.as_mut().unwrap().layout.dex = None;
    let r = s.rom.clone();
    assert_eq!(
        Session::new(r.clone())
            .load(original.clone(), None)
            .unwrap_err()
            .code,
        "unsupported_feature"
    );
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
    let mut app = crate::app::App {
        session: Some(s),
        ..Default::default()
    };
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
    let raw = pokemon::create(&r, 1, 1, "TEST", 50, 7).unwrap();
    let patch = PokemonPatch {
        species: Some(2),
        ..Default::default()
    };
    assert_eq!(
        pokemon::edit(&raw, &patch, &r, Policy::Standard)
            .unwrap_err()
            .code,
        "battle_species"
    );
    assert!(pokemon::edit(&raw, &patch, &r, Policy::Free).is_ok());
}

#[test]
fn persistent_forms_follow_only_changed_item_or_move_triggers() {
    let mut r = adapter_rom(profile::ROCKET);
    let b = std::sync::Arc::make_mut(&mut r.data);
    for id in [1, 2] {
        put32(b, 0x6193ac + id * 4, 0x08021000);
    }
    // Native format: trigger, target, item/move, comparison.
    for (index, row) in [[1, 1, 0, 0], [1, 2, 2, 0], [3, 2, 1, 0]]
        .iter()
        .enumerate()
    {
        for (i, value) in row.iter().enumerate() {
            put16(b, 0x21000 + index * 8 + i * 2, *value);
        }
    }
    let raw = pokemon::create(&r, 1, 1, "TEST", 50, 7).unwrap();
    let (held, _) = pokemon::edit(
        &raw,
        &PokemonPatch {
            held_item: Some(2),
            ..Default::default()
        },
        &r,
        Policy::Standard,
    )
    .unwrap();
    assert_eq!(pokemon::decode(&held, &r).unwrap().species, 2);
    let (removed, _) = pokemon::edit(
        &held,
        &PokemonPatch {
            held_item: Some(0),
            ..Default::default()
        },
        &r,
        Policy::Standard,
    )
    .unwrap();
    assert_eq!(pokemon::decode(&removed, &r).unwrap().species, 1);
    let (unrelated, _) = pokemon::edit(
        &raw,
        &PokemonPatch {
            friendship: Some(99),
            ..Default::default()
        },
        &r,
        Policy::Standard,
    )
    .unwrap();
    assert_eq!(pokemon::decode(&unrelated, &r).unwrap().species, 1);
    let (moved, _) = pokemon::edit(
        &raw,
        &PokemonPatch {
            moves: Some([1, 2, 0, 0]),
            ..Default::default()
        },
        &r,
        Policy::Standard,
    )
    .unwrap();
    assert_eq!(pokemon::decode(&moved, &r).unwrap().species, 2);
}

#[test]
#[ignore = "set GEN3_ROM_ROCKET; optionally GEN3_SAVE_ROCKET and GEN3_ADAPTER_PROBES"]
fn local_rocket_adapter_regression() {
    let path = std::env::var("GEN3_ROM_ROCKET").unwrap();
    let r = Rom::open(std::fs::read(path).unwrap()).unwrap();
    assert_eq!(r.profile.id, profile::ROCKET.id);
    for id in [9, 899, 980] {
        let family = r.species_relations(id).unwrap();
        assert_eq!(family.species, [7, 8, 9, 899, 980]);
        assert_eq!(family.evolutions.len(), 2);
        assert!(family
            .battle_forms
            .iter()
            .any(|f| f.source == 9 && f.target == 980));
        assert!(family.form_families.iter().any(|f| f.species == [9, 980]));
        assert!(family
            .name_relations
            .iter()
            .any(|f| f.source == 9 && f.target == 899));
    }
    assert_eq!(
        r.ancestors(899).unwrap().into_iter().collect::<Vec<_>>(),
        [899]
    );
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
            .all(|e| e.method < 0xfffd && e.condition != "unknown"));
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
    for growth in 0..6 {
        let Some(species) = catalog
            .species
            .iter()
            .find(|s| s.growth == growth)
            .map(|s| s.id)
        else {
            // Some growth curves are not assigned to any species in this ROM.
            continue;
        };
        for level in [100, 101, 127, 150] {
            let raw = pokemon::create(&r, species, 0x12345678, "TEST", level, 23).unwrap();
            let p = pokemon::decode(&raw, &r).unwrap();
            assert_eq!(p.level, level);
            probes.push(serde_json::json!({"after":raw,"pokemon":p}));
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
    // Evolution semantics can be changed without changing the species layout.
    r.profile.formats.evolutions = crate::adapter::EvolutionFormat::Expanded;
    let evo = r.profile.evolutions.offset + r.profile.evolutions.stride;
    let b = std::sync::Arc::make_mut(&mut r.data);
    put16(b, evo, 24);
    put16(b, evo + 2, 18);
    put16(b, evo + 4, 2);
    assert_eq!(r.evolutions(1).unwrap()[0].condition, "move_type");
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
        let flag = if profile.save.pokemon_codec != crate::adapter::PokemonCodec::Rocket21 {
            (19, 1)
        } else {
            (18, 8)
        };
        bad[flag.0] |= flag.1;
        assert!(pokemon::checked_unpack_with(&bad, profile.save.pokemon_codec).is_err());
        bad = raw;
        bad[32] ^= 1;
        if profile.save.pokemon_codec.plain_substructures() {
            assert!(pokemon::checked_unpack_with(&bad, profile.save.pokemon_codec).is_ok());
            continue;
        }
        assert_eq!(
            pokemon::checked_unpack_with(&bad, profile.save.pokemon_codec)
                .unwrap_err()
                .code,
            "pokemon_checksum"
        );
    }
}

#[test]
#[ignore = "set GEN3_ROM_ROCKET and GEN3_SAVE_ROCKET to private fixtures"]
fn local_rocket_parity_regression() {
    let r = Rom::open(std::fs::read(std::env::var("GEN3_ROM_ROCKET").unwrap()).unwrap()).unwrap();
    let world = r.world().unwrap();
    assert_eq!(world.maps.len(), 1363);
    assert_eq!(world.trainers.len(), 2558);
    let mut errors = Vec::new();
    for id in 1..r.profile.species.count as u16 {
        if let Err(e) = r.learnset(id) {
            errors.push(format!("learnset {id}: {e:?}"));
        }
        if let Err(e) = r.pokemon_sprite(id, false, 0) {
            errors.push(format!("sprite {id}: {e:?}"));
        }
    }
    eprintln!("species checked");
    for id in 0..r.profile.trainer_sprites.count as u16 {
        if let Err(e) = r.trainer_sprite(id) {
            errors.push(format!("trainer sprite {id}: {e:?}"));
        }
    }
    let graphics: std::collections::BTreeSet<_> = world
        .maps
        .iter()
        .flat_map(|m| m.objects.iter().map(|o| o.graphics_id))
        .collect();
    for id in graphics.into_iter().filter(|id| *id < 542) {
        if let Err(e) = r.object_sprite(id) {
            errors.push(format!("object {id}: {e:?}"));
        }
    }
    eprintln!("objects checked");
    for map in &world.maps {
        if let Err(e) = r.map_image(&map.id) {
            errors.push(format!("map {}: {e:?}", map.id));
        }
    }
    eprintln!("all maps checked; {} errors: {errors:?}", errors.len());
    assert!(errors.is_empty());
}

#[test]
#[ignore = "requires user-supplied exact Rocket ROM and save"]
fn local_rocket_write_regression() {
    let r = Rom::open(std::fs::read(std::env::var("GEN3_ROM_ROCKET").unwrap()).unwrap()).unwrap();
    let original = std::fs::read(std::env::var("GEN3_SAVE_ROCKET").unwrap()).unwrap();
    let mut session = Session::new(r.clone());
    session.load(original.clone(), None).unwrap();
    for pocket in r.profile.save.pockets {
        let item = (1..r.profile.items.count as u16)
            .find(|id| r.item(*id).unwrap().pocket == pocket.category || pocket.category == 0)
            .unwrap();
        session
            .apply(
                Action::Bag {
                    pocket: pocket.id.into(),
                    slot: 0,
                    item,
                    quantity: 1,
                },
                Policy::Standard,
            )
            .unwrap();
        session.undo().unwrap();
        assert_eq!(session.save_ref().unwrap().data, original);
    }
    for number in [1, 416, 417, 944, 955] {
        let mut save = session.save_ref().unwrap().clone();
        save.edit_dex(number, true, true).unwrap();
        let flags = save.dex().unwrap();
        assert!(flags[number as usize - 1].owned);
        save.validate(&r).unwrap();
        let before = session.save_ref().unwrap().logical(1..=4);
        let after = save.logical(1..=4);
        let changed: Vec<_> = before
            .iter()
            .zip(&after)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i)
            .collect();
        assert!(changed.iter().all(|i| [
            0x2ee4 + (number as usize - 1) / 8,
            0x2f5c + (number as usize - 1) / 8
        ]
        .contains(i)));
    }
    for destination in [
        loc(0),
        Location::Box {
            box_index: 13,
            slot: 29,
        },
    ] {
        session
            .apply(
                Action::Transfer {
                    from: party(),
                    to: destination,
                    copy: false,
                },
                Policy::Standard,
            )
            .unwrap();
        session.undo().unwrap();
        assert_eq!(session.save_ref().unwrap().data, original);
    }
}

#[test]
fn adapter_inventory_crosses_logical_sectors_without_touching_other_bytes() {
    for profile in profile::PROFILES {
        let r = adapter_rom(profile);
        let original = save_bytes(&r);
        for pocket in r.profile.save.pockets {
            for index in 0..pocket.count {
                let mut save = Save::open(original.clone(), r.profile.save).unwrap();
                let before = if pocket.block == crate::save::PocketBlock::SectorExtensions {
                    save.extensions()
                } else {
                    save.logical(1..=4)
                };
                save.edit_bag(pocket.id, index, 1, 17, &r, Policy::Free)
                    .unwrap();
                let reopened = Save::open(save.data.clone(), r.profile.save).unwrap();
                reopened.validate(&r).unwrap();
                let entry = reopened
                    .bag()
                    .unwrap()
                    .into_iter()
                    .find(|e| e.pocket == pocket.id && e.slot == index)
                    .unwrap();
                assert_eq!((entry.item, entry.quantity), (1, 17));
                let mut expected = before;
                let offset = pocket.offset + index * 4;
                put16(&mut expected, offset, 1);
                put16(
                    &mut expected,
                    offset + 2,
                    17 ^ if pocket.encrypted { 0x4321 } else { 0 },
                );
                assert_eq!(
                    if pocket.block == crate::save::PocketBlock::SectorExtensions {
                        reopened.extensions()
                    } else {
                        reopened.logical(1..=4)
                    },
                    expected,
                    "{} {} {index}",
                    profile.id,
                    pocket.id
                );
                let mut expected_file = original.clone();
                // Permit only the addressed slot and necessary main-bank checksum.
                let mut remaining = 0;
                let spans: Vec<(usize, usize)> =
                    if pocket.block == crate::save::PocketBlock::SectorExtensions {
                        (0..14)
                            .map(|id| {
                                (
                                    save.sections[id] + profile.save.sizes[id],
                                    0xff0 - profile.save.sizes[id],
                                )
                            })
                            .chain(
                                profile
                                    .save
                                    .extension_sectors
                                    .unwrap()
                                    .iter()
                                    .map(|id| (id * 4096, 0xff0)),
                            )
                            .collect()
                    } else {
                        (1..=4)
                            .map(|id| (save.sections[id], profile.save.sizes[id]))
                            .collect()
                    };
                for (off, len) in spans {
                    for i in 0..len {
                        if (offset..offset + 4).contains(&(remaining + i)) {
                            expected_file[off + i] = save.data[off + i];
                        }
                    }
                    remaining += len;
                }
                for id in 0..14 {
                    let off = save.sections[id] + 0xff6;
                    expected_file[off..off + 2].copy_from_slice(&save.data[off..off + 2]);
                }
                assert_eq!(
                    save.data, expected_file,
                    "{} {} {index} unrelated bytes",
                    profile.id, pocket.id
                );
            }
        }
    }
}

#[test]
fn rocket_ribbons_preserve_ivs_egg_ability_and_reserved_bits() {
    let codec = crate::adapter::PokemonCodec::Rocket21;
    // Literal native field locations; deliberately independent of codec fields.
    let fields = [
        (351usize, 1, 0),
        (352, 1, 3),
        (353, 3, 6),
        (356, 3, 9),
        (359, 3, 12),
        (362, 12, 15),
        (374, 2, 27),
        (378, 1, 31),
    ];
    for (bit, width, logical) in fields {
        for value in [0, (1u32 << width) - 1] {
            let mut canonical = [0xa5; 48];
            let mut expected = canonical;
            let prior = codec.read_ribbons(&canonical).unwrap();
            let mask = ((1u32 << width) - 1) << logical;
            codec
                .write_ribbons(&mut canonical, (prior & !mask) | value << logical)
                .unwrap();
            for i in 0..width {
                let mask = 1 << ((bit + i) % 8);
                expected[(bit + i) / 8] = (expected[(bit + i) / 8] & !mask)
                    | if value & (1 << i) != 0 { mask } else { 0 };
            }
            assert_eq!(canonical, expected);
        }
    }
}

#[test]
fn relation_graph_uses_runtime_tables_and_does_not_expand_legal_ancestry_from_names() {
    for profile in profile::PROFILES
        .into_iter()
        .filter(|p| p.capabilities.save_edit)
    {
        let mut r = adapter_rom(profile);
        r.profile.species.count = 5;
        r.profile.evolutions.count = 5;
        let p = r.profile;
        let b = std::sync::Arc::make_mut(&mut r.data);
        // Growth thresholds come from the supplied ROM, not a bundled formula.
        let table = p.experience_table.unwrap();
        put32(b, table.offset + 50 * 4, 123456);
        assert_eq!(pokemon::rom_experience(&r, 0, 50).unwrap(), 123456);
        let b = std::sync::Arc::make_mut(&mut r.data);
        let first_stats = b[p.base_stats.offset + p.base_stats.stride
            ..p.base_stats.offset + 2 * p.base_stats.stride]
            .to_vec();
        let fourth = p.base_stats.offset + 4 * p.base_stats.stride;
        b[fourth..fourth + p.base_stats.stride].copy_from_slice(&first_stats);
        for (id, name) in [(1, "Root"), (2, "Branch"), (3, "Sibling"), (4, "RootZ")] {
            let name_offset = p.species.offset + id * p.species.stride;
            b[name_offset..name_offset + p.species.stride]
                .copy_from_slice(&r.codec.encode(name, p.species.stride).unwrap());
        }
        let evolution = p.evolutions.offset + p.evolutions.stride;
        for (row, target) in [(0, 2), (1, 3)] {
            put16(b, evolution + row * 8, 4);
            put16(b, evolution + row * 8 + 2, 16);
            put16(b, evolution + row * 8 + 4, target);
        }
        let graph = r.species_relations(2).unwrap();
        assert_eq!(graph.species, [1, 2, 3, 4]);
        assert_eq!(graph.evolutions.len(), 2);
        assert_eq!(graph.name_relations.len(), 1);
        assert_eq!(r.ancestors(4).unwrap().into_iter().collect::<Vec<_>>(), [4]);
        // A newly edited table row must immediately change the graph.
        let b = std::sync::Arc::make_mut(&mut r.data);
        put16(b, evolution + 12, 4);
        assert_eq!(
            r.species_relations(2).unwrap().evolutions[1]
                .evolution
                .target,
            4
        );
        if let Some(table) = p.form_families {
            let b = std::sync::Arc::make_mut(&mut r.data);
            put32(b, table + 2 * 4, 0x08029000);
            put16(b, 0x29000, 2);
            put16(b, 0x29002, 3);
            put16(b, 0x29004, 0xffff);
            assert_eq!(r.form_families().unwrap()[0].species, [2, 3]);
            let b = std::sync::Arc::make_mut(&mut r.data);
            for i in 0..5 {
                put16(b, 0x29000 + i * 2, 2);
            }
            assert_eq!(r.form_families().err().unwrap().code, "form_terminator");
        }
    }
}

#[test]
fn runtime_labels_and_nature_effects_follow_each_rom_without_cached_fallbacks() {
    for profile in profile::PROFILES
        .into_iter()
        .filter(|p| p.capabilities.save_edit)
    {
        let mut r = adapter_rom(profile);
        let original = r.nature(15).unwrap();
        assert_eq!(original.name, "NATURE15");
        assert_eq!(original.stat_changes, [-1, 0, 0, 1, 0]);
        let raw = pokemon::create(&r, 1, 1, "TEST", 50, 15).unwrap();
        let before = pokemon::decode(&raw, &r).unwrap();
        let nature = before.effective_nature;
        let name = r.codec.encode("CUSTOM", 64).unwrap();
        let data = std::sync::Arc::make_mut(&mut r.data);
        data[0x18000 + 15 * 64..0x18000 + 16 * 64].copy_from_slice(&name);
        data[profile.nature_effects + nature as usize * 5
            ..profile.nature_effects + nature as usize * 5 + 5]
            .copy_from_slice(&[0, 1, 0, 0, 255]);
        // A newly read catalog must reflect ROM changes; no locale/profile name fallback.
        let catalog = r.catalog().unwrap();
        assert_eq!(catalog.natures[15].name, "CUSTOM");
        assert_eq!(catalog.type_names[0], "T0");
        assert_eq!(catalog.type_names.len(), profile.type_names.count);
        let after = pokemon::decode(&raw, &r).unwrap();
        assert_eq!(after.stats[0], before.stats[0]);
        assert!(after.stats[2] > before.stats[2]);
        assert!(after.stats[5] < before.stats[5]);
        assert_eq!(raw.len(), 80);
        assert!(r.nature(25).is_err());
        put32(
            std::sync::Arc::make_mut(&mut r.data).as_mut_slice(),
            profile.nature_names,
            0,
        );
        assert!(r.nature(0).is_err());
    }
}

#[test]
fn nature_product_width_matches_each_native_engine() {
    for p in profile::PROFILES {
        let modified = pokemon::stats_with_changes(
            [255; 6],
            [31; 6],
            [252; 6],
            100,
            [1, 0, 0, 0, 0],
            p.nature_product_u16,
        );
        // Unmodified attack is 609. The BW/DP engine truncates 609*110 to u16.
        assert_eq!(modified[1], if p.nature_product_u16 { 14 } else { 669 });
        assert_eq!(modified[2], 609);
    }
}

#[test]
#[ignore = "requires GEN3_ROM_ULTIMATE; native probes stay in GEN3_ULTIMATE_PROBES"]
fn local_ultimate_adapter_regression() {
    let r = Rom::open(std::fs::read(std::env::var("GEN3_ROM_ULTIMATE").unwrap()).unwrap()).unwrap();
    let catalog = r.catalog().unwrap();
    assert_eq!(catalog.profile.md5, crate::ultimate::PROFILE.md5);
    assert_eq!(
        (
            catalog.species.len(),
            catalog.moves.len(),
            catalog.items.len()
        ),
        (1199, 938, 800)
    );
    assert_eq!(r.species(25).unwrap().abilities, [9, 9, 31]);
    assert_eq!(catalog.type_names[23], "妖精");
    assert!(catalog.battle_forms.iter().any(|form| {
        form.source == 6
            && form.target == 252
            && form.kind == crate::forms::BattleFormKind::Gigantamax
            && form.trigger == crate::forms::BattleTrigger::HeldItem(702)
    }));
    assert!(catalog.editor_rules.hyper_training);
    assert!(!catalog.editor_rules.pokemon_checksum);
    assert_eq!(
        catalog
            .editor_rules
            .ball_options
            .iter()
            .find(|b| b.value == 0)
            .unwrap()
            .item,
        4
    );
    let world = r.world().unwrap();
    assert_eq!(world.maps.len(), 922);
    assert_eq!(world.trainers.len(), 1336);
    for id in 1..1200 {
        r.level_moves(id).unwrap();
    }
    let mut probes = Vec::new();
    for species in [1, 25, 150, 201, 303, 328, 902, 1000, 1199] {
        for nature in 0..25 {
            let before =
                pokemon::create(&r, species, 0xdeadbeef, "TEST", 50, 0xbad00000 + 23).unwrap();
            for hyper_trained in [
                [false; 6],
                [true; 6],
                [true, false, true, false, true, false],
            ] {
                let (after, _) = pokemon::edit(
                    &before,
                    &PokemonPatch {
                        nature_override: Some(nature),
                        ivs: Some([1, 2, 3, 4, 5, 6]),
                        evs: Some([252, 0, 0, 0, 252, 6]),
                        ability_slot: Some(2),
                        hyper_trained: Some(hyper_trained),
                        ..Default::default()
                    },
                    &r,
                    Policy::Free,
                )
                .unwrap();
                let pokemon = pokemon::decode(&after, &r).unwrap();
                probes.push(serde_json::json!({"before":before,"after":after,"pokemon":pokemon}));
            }
        }
    }
    if let Ok(path) = std::env::var("GEN3_ULTIMATE_PROBES") {
        std::fs::write(path, serde_json::to_vec(&probes).unwrap()).unwrap();
    }
}

#[test]
#[ignore = "requires five exact local ROMs; optional private saves for planning"]
fn local_query_acquisition_and_collection_all_profiles() {
    use crate::{
        acquisition::{AcquisitionIndex, Target, TargetKind},
        collection::{CollectionBasis, CollectionRequest},
    };
    for name in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{name}")).unwrap()).unwrap())
                .unwrap();
        let index = AcquisitionIndex::build(&r).unwrap();
        let item = index
            .world
            .map_events
            .iter()
            .flat_map(|m| m.markers.iter())
            .flat_map(|m| m.rewards.iter())
            .next()
            .unwrap()
            .item;
        let report = index
            .query(
                &r,
                None,
                Target {
                    kind: TargetKind::Item,
                    id: item,
                },
            )
            .unwrap();
        assert!(!report.sources.is_empty(), "{name}");
        assert!(report.sources.iter().all(|s| s.status == "unknown"));
        for s in report.sources.iter().filter(|s| s.x.is_some()) {
            let map = index
                .world
                .maps
                .iter()
                .find(|m| Some(&m.id) == s.map_id.as_ref())
                .unwrap();
            // Initial event locations outside a dynamic layout must not be normalized.
            assert!(s.x.unwrap().abs() < 32767 && map.width > 0);
        }
        // A real grass/cave/surf/dive/fishing reference must close the held-item
        // query, with the independent native chance rather than an orphan species row.
        let encounter = index
            .world
            .encounters
            .iter()
            .find(|e| {
                matches!(
                    e.method.as_str(),
                    "grass"
                        | "cave"
                        | "surf"
                        | "dive"
                        | "rock_smash"
                        | "old_rod"
                        | "good_rod"
                        | "super_rod"
                ) && r
                    .species(e.species)
                    .is_ok_and(|s| s.items.iter().any(|i| *i != 0))
            })
            .unwrap();
        let held = r
            .species(encounter.species)
            .unwrap()
            .items
            .into_iter()
            .find(|i| *i != 0)
            .unwrap();
        let held_report = index
            .query(
                &r,
                None,
                Target {
                    kind: TargetKind::Item,
                    id: held,
                },
            )
            .unwrap();
        let held_source = held_report
            .sources
            .iter()
            .find(|s| {
                s.kind == "wild_held"
                    && s.map_id.as_deref() == Some(encounter.map_id.as_str())
                    && s.related
                        .iter()
                        .any(|t| t.kind == TargetKind::Species && t.id == encounter.species)
            })
            .unwrap();
        assert!(
            held_source
                .held_percent
                .is_some_and(|p| p > 0.0 && p <= 100.0),
            "{name}"
        );
        assert!(held_source.held_context.is_some());
        assert_eq!(held_source.encounter_percent, encounter.weight);
        assert_eq!(
            held_source.encounter_method.as_deref(),
            Some(encounter.method.as_str())
        );
        assert!(held_source.partial);
        // Current raw party context belongs to the cache key; no quantity, identity
        // or unrelated byte can change during a reference query.
        let synthetic = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let original = synthetic.data.clone();
        let current = index
            .query(
                &r,
                Some(&synthetic),
                Target {
                    kind: TargetKind::Item,
                    id: held,
                },
            )
            .unwrap();
        assert!(current
            .sources
            .iter()
            .filter(|s| s.kind == "wild_held")
            .all(|s| s
                .held_context
                .as_ref()
                .is_some_and(|c| c.current_party.is_some())));
        assert_eq!(synthetic.data, original);
        let species = index.world.encounters.first().unwrap().species;
        let mon = index
            .query(
                &r,
                None,
                Target {
                    kind: TargetKind::Species,
                    id: species,
                },
            )
            .unwrap();
        assert!(mon.sources.iter().any(|s| s.map_id.is_some()), "{name}");
        let move_id = index.learnsets.values().flatten().next().unwrap().move_id;
        assert!(!index
            .query(
                &r,
                None,
                Target {
                    kind: TargetKind::Move,
                    id: move_id
                }
            )
            .unwrap()
            .sources
            .is_empty());
        let shops = index
            .world
            .map_events
            .iter()
            .flat_map(|m| m.markers.iter())
            .flat_map(|m| m.rewards.iter())
            .filter(|r| r.via == "shop")
            .count();
        if let Ok(path) = std::env::var(format!("GEN3_SAVE_{name}")) {
            let original = std::fs::read(path).unwrap();
            let s = Save::open(original.clone(), r.profile.save).unwrap();
            s.validate(&r).unwrap();
            let plan = index
                .collection(
                    &r,
                    &s,
                    CollectionRequest {
                        basis: CollectionBasis::Individuals,
                        families: true,
                        include_unknown_rewards: true,
                    },
                )
                .unwrap();
            assert_eq!(s.data, original, "planning must be read-only");
            assert!(plan.missing_count > 0);
            let current = s.all(&r).unwrap();
            for task in plan.regions.iter().flat_map(|r| &r.tasks) {
                if let Some(p) = &task.preparation {
                    assert!(p.partial && !p.steps.is_empty());
                    let mut from = p.origin;
                    for step in &p.steps {
                        assert_eq!(step.from, from);
                        assert!(index.evolutions[&from]
                            .iter()
                            .any(|e| e.offset == step.evolution.offset
                                && e.target == step.evolution.target));
                        from = step.evolution.target;
                    }
                    assert_eq!(from, task.target.id);
                    assert_eq!(
                        p.current_count,
                        current
                            .iter()
                            .filter(|v| v.pokemon.species == p.origin
                                && !v.pokemon.egg
                                && v.pokemon.checksum_ok)
                            .count()
                    );
                    if let Some(source) = &p.source {
                        let id = source.map_id.as_ref().unwrap();
                        assert!(plan.entrances.iter().any(|e| &e.map_id == id));
                        assert_ne!(source.kind, "breeding_candidate");
                    }
                }
            }
            if r.profile.save.dex.is_none() && r.profile.save.dex_read.is_none() {
                assert_eq!(
                    index
                        .collection(
                            &r,
                            &s,
                            CollectionRequest {
                                basis: CollectionBasis::Dex,
                                families: true,
                                include_unknown_rewards: false
                            }
                        )
                        .err()
                        .unwrap()
                        .code,
                    "collection_dex_unverified"
                );
            }
            if r.profile.save.dex_read.is_some() {
                let before = s.data.clone();
                let dex_plan = index
                    .collection(
                        &r,
                        &s,
                        CollectionRequest {
                            basis: CollectionBasis::Dex,
                            families: false,
                            include_unknown_rewards: false,
                        },
                    )
                    .unwrap();
                assert!(dex_plan.dex_status.as_ref().unwrap().read_only);
                assert_eq!(dex_plan.dex_status.as_ref().unwrap().count, 1027);
                assert!(dex_plan.missing_count > 0);
                assert_eq!(s.data, before);
                eprintln!("{name}: read-only native Dex planning passed, {} owned species, {} missing goals", dex_plan.owned_count, dex_plan.missing_count);
            }
            eprintln!("{name}: {} species, {} shop rows, {} missing family goals, {} regions, {} entrance reports",index.species.len(),shops,plan.missing_count,plan.regions.len(),plan.entrances.len());
        } else {
            eprintln!(
                "{name}: {} species, {} shop rows; ROM-only queries passed",
                index.species.len(),
                shops
            );
        }
    }
}

#[cfg(test)]
pub(crate) fn query_fixture_rom() -> Rom {
    rom()
}

#[test]
fn standard_dialogues_preserve_position_and_keep_choices_unknown_across_profiles() {
    use crate::event_dependencies::Index;
    for profile in profile::PROFILES {
        let (mut rom, map) = npc_trade_fixture(profile);
        let rules = rom.profile.event_state.unwrap().effects.unwrap();
        let b = std::sync::Arc::make_mut(&mut rom.data);
        for (op, handler) in [8, 9, 3, 0x66, 0x67, 0x6e, 0x28, 0x68]
            .into_iter()
            .zip(rules.presentation_handlers)
        {
            put32(b, rules.commands + op * 4, 0x08000001 + handler as u32);
        }
        for (op, handler) in [0x39, 0x3a, 0x3b, 0x3d, 0x3e, 0xd1, 0xd7]
            .into_iter()
            .zip(rules.warp_handlers)
        {
            if handler != 0 {
                put32(b, rules.commands + op * 4, 0x08000001 + handler as u32);
            }
        }
        for handler in &rules.presentation_handlers[..2] {
            put32(b, handler + 0x28, 0x08027500);
            put32(b, handler + 0x2c, 0x08027520);
        }
        if let Some(hook) = rules.standard_call_hook {
            let handler = rules.presentation_handlers[1];
            put16(b, handler + 0x12, 0x4b06);
            put16(b, handler + 0x14, 0x4718);
            put32(b, handler + 0x2c, 0x08000001 + hook as u32);
            put16(b, hook + 6, 0x2807);
            for (offset, index) in [(0x0a, 2), (0x0e, 3), (0x12, 4), (0x16, 6)] {
                put16(b, hook + offset, 0x2800 + index);
            }
        }
        // A runtime body in slot 7, not a hardcoded standard ID or body address.
        put32(b, 0x27500 + 7 * 4, 0x08027600);
        b[0x27600..0x2760a].copy_from_slice(&[0x67, 0, 0, 0, 0, 0x66, 0x6e, 20, 8, 3]);
        let root = 0x26000;
        let entry = 0x26100;
        let yes = 0x26200;
        b[root..root + 5].copy_from_slice(&[0x2b, 11, 0, 6, 1]);
        put32(b, root + 5, 0x08000000 + entry as u32);
        b[root + 9] = 2;
        b[entry..entry + 24].copy_from_slice(&[
            0x16, 8, 0x80, 1, 0, 0x16, 9, 0x80, 2, 0, 0x16, 0x0d, 0x80, 1, 0, 9, 7, 0x21, 0x0d,
            0x80, 1, 0, 6, 1,
        ]);
        put32(b, entry + 24, 0x08000000 + yes as u32);
        b[entry + 28..entry + 33].copy_from_slice(&[0x16, 8, 0x80, 3, 0]);
        let passage = [0x68, 0x28, 30, 0, 0x3b, 0, 1, 255, 8, 0x80, 9, 0x80, 2];
        b[entry + 33..entry + 46].copy_from_slice(&passage);
        b[yes..yes + 13].copy_from_slice(&passage);
        let mut target = map.clone();
        target.id = "0-1".into();
        target.name = "Synthetic interior".into();
        target.map_type = 4;
        target.events = None;
        target.scripts.clear();
        let maps = vec![map.clone(), target];
        let before = rom.data.clone();
        let index = Index::build(&rom, &maps).unwrap();
        let report = index.map_navigation(&rom, &maps, "0-1", None).unwrap();
        assert_eq!(report.incoming.len(), 2, "{}", profile.id);
        let coordinates: std::collections::BTreeSet<_> = report
            .incoming
            .iter()
            .map(|edge| (edge.target_x, edge.target_y))
            .collect();
        assert_eq!(coordinates, [(Some(1), Some(2)), (Some(3), Some(2))].into());
        for edge in &report.incoming {
            assert!(edge.unresolved.is_none());
            let script = edge.script.as_ref().unwrap();
            assert!(script.entry_unresolved);
            assert!(script
                .stopped_at
                .iter()
                .all(|offset| [entry + 37, yes + 4].contains(offset)));
            assert!(script.checks.iter().any(|check| check.condition.id == 11));
            assert!(script
                .checks
                .iter()
                .any(|check| { check.condition.id == 0x800d && check.satisfied.is_none() }));
        }
        let save = Save::open(save_bytes(&rom), rom.profile.save).unwrap();
        let saved_before = save.data.clone();
        let saved = index
            .map_navigation(&rom, &maps, "0-1", Some(&save))
            .unwrap();
        assert_eq!(saved.incoming.len(), 2);
        assert!(saved.incoming.iter().all(|edge| edge
            .script
            .as_ref()
            .unwrap()
            .checks
            .iter()
            .any(|check| check.condition.id == 0x800d && check.satisfied.is_none())));
        assert_eq!(save.data, saved_before);
        assert_eq!(rom.data, before);

        // Script comparisons are cached by the native context. Changing the
        // later menu result does not erase a previously computed flag comparison.
        let mut cached = rom.clone();
        let b = std::sync::Arc::make_mut(&mut cached.data);
        b[root..root + 19].copy_from_slice(&[
            0x16, 8, 0x80, 1, 0, 0x16, 9, 0x80, 2, 0, 0x2b, 11, 0, 9, 7, 6, 1, 0, 0,
        ]);
        put32(b, root + 17, 0x08000000 + yes as u32);
        b[root + 21] = 2;
        let graph = Index::build(&cached, &maps)
            .unwrap()
            .navigation_graph(&cached, &maps, None)
            .unwrap();
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].target_x, Some(1));
        assert!(graph.edges[0]
            .script
            .as_ref()
            .unwrap()
            .checks
            .iter()
            .any(|check| check.condition.id == 11));

        // Out-of-range gotoStd consumes its operand and continues; it must not
        // return out of this root. Valid gotoStd returns out of the current call.
        let mut ignored = rom.clone();
        let b = std::sync::Arc::make_mut(&mut ignored.data);
        b[entry + 15..entry + 17].copy_from_slice(&[8, 255]);
        let graph = Index::build(&ignored, &maps)
            .unwrap()
            .navigation_graph(&ignored, &maps, None)
            .unwrap();
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].target_x, Some(1));
        std::sync::Arc::make_mut(&mut ignored.data)[entry + 16] = 7;
        let graph = Index::build(&ignored, &maps)
            .unwrap()
            .navigation_graph(&ignored, &maps, None)
            .unwrap();
        assert!(graph.edges.is_empty());

        // Unknown bodies/native dispatches invalidate positions. A stale index
        // rejects mutated inputs even when the profile's stored MD5 is unchanged.
        let mut unknown = rom.clone();
        let b = std::sync::Arc::make_mut(&mut unknown.data);
        b[0x27600] = 0x25;
        let graph = Index::build(&unknown, &maps)
            .unwrap()
            .navigation_graph(&unknown, &maps, None)
            .unwrap();
        assert!(graph.edges.iter().any(|edge| edge.unresolved.is_some()));
        assert!(index.navigation_graph(&unknown, &maps, None).is_err());
        let b = std::sync::Arc::make_mut(&mut unknown.data);
        b[0x27600] = 0x67;
        put32(b, rules.commands + 0x6e * 4, 0x08001001);
        let graph = Index::build(&unknown, &maps)
            .unwrap()
            .navigation_graph(&unknown, &maps, None)
            .unwrap();
        assert!(graph.edges.iter().any(|edge| edge.unresolved.is_some()));

        let mut retargeted = rom.clone();
        let b = std::sync::Arc::make_mut(&mut retargeted.data);
        put32(b, 0x27500 + 7 * 4, 0x08027700);
        b[0x27700..0x27706].copy_from_slice(&[0x67, 0, 0, 0, 0, 3]);
        let graph = Index::build(&retargeted, &maps)
            .unwrap()
            .navigation_graph(&retargeted, &maps, None)
            .unwrap();
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].target_x, Some(1));

        // Item/quantity constants likewise survive a prompt. No receipt flag
        // or delivery status is fabricated from those readable operands.
        let mut gift = rom.clone();
        let b = std::sync::Arc::make_mut(&mut gift.data);
        b[root..root + 18].copy_from_slice(&[
            0x16, 0, 0x80, 2, 0, 0x16, 1, 0x80, 1, 0, 9, 7, 0x44, 0, 0x80, 1, 0x80, 2,
        ]);
        let rewards = gift.map_events(&map).unwrap();
        let reward = &rewards.markers[0].rewards[0];
        assert_eq!((reward.item, reward.quantity), (2, Some(1)));
        assert!(reward.receipt.is_none());
    }
}

#[test]
#[ignore = "requires all five exact ROMs and GEN3_DIALOGUE_PROBES native mGBA vectors"]
fn local_standard_dialogues_match_native_and_resolve_referenced_coordinates() {
    use crate::event_dependencies::Index;
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_DIALOGUE_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let original = std::fs::read(&path).unwrap();
        let rom = Rom::open(original.clone()).unwrap();
        let proof = &probes[key];
        assert_eq!(proof["md5"].as_str().unwrap(), rom.profile.md5);
        assert_eq!(proof["sha256"].as_str().unwrap(), sha256(&original));
        let rules = rom.profile.event_state.unwrap().effects.unwrap();
        for (op, expected) in [8, 9, 3, 0x66, 0x67, 0x6e, 0x28, 0x68]
            .into_iter()
            .zip(rules.presentation_handlers)
        {
            assert_eq!(
                proof["handlers"][op.to_string()].as_u64().unwrap() as usize,
                expected
            );
        }
        assert_eq!(proof["dispatch"].as_array().unwrap().len(), 512);
        let mut classified = 0;
        for row in proof["dispatch"].as_array().unwrap() {
            let op = row["opcode"].as_u64().unwrap() as u8;
            let index = row["index"].as_u64().unwrap() as u8;
            // These are operand-decoding contexts, not assertions that arbitrary
            // matching bytes are referenced scripts or accessible game content.
            let Some(pc) = original.windows(2).position(|v| v == [op, index]) else {
                continue;
            };
            if let Some(effect) = rom.script_presentation(pc) {
                classified += 1;
                assert_eq!(effect.returns, row["valid"].as_bool().unwrap());
            }
        }
        assert!(classified > 100);
        let prompt = original.windows(2).position(|v| v == [9, 5]).unwrap();
        let effect = rom.script_presentation(prompt).unwrap();
        assert!(effect.choice && effect.returns);
        for row in proof["choices"].as_array().unwrap() {
            for i in 0..16 {
                if i != 13 {
                    assert_eq!(row["before"][i], row["after"][i]);
                }
            }
        }
        assert!(proof["existing_menu_preserved"].as_bool().unwrap());
        let maps = rom.maps().unwrap();
        let index = Index::build(&rom, &maps).unwrap();
        let graph = index.navigation_graph(&rom, &maps, None).unwrap();
        let coordinates: Vec<_> = graph
            .edges
            .iter()
            .filter(|edge| {
                edge.script.is_some()
                    && original[edge.offset + 3] == 255
                    && (u16(&original, edge.offset + 4).unwrap() >= 0x4000
                        || u16(&original, edge.offset + 6).unwrap() >= 0x4000)
            })
            .collect();
        assert_eq!(coordinates.len(), if key == "MERCURY12" { 0 } else { 12 });
        assert!(coordinates.iter().all(|edge| {
            edge.unresolved.is_none() && edge.target_x.is_some() && edge.target_y.is_some()
        }));
        let save = Save::open(save_bytes(&rom), rom.profile.save).unwrap();
        let before = save.data.clone();
        let saved = index.navigation_graph(&rom, &maps, Some(&save)).unwrap();
        for edge in &coordinates {
            let current = saved
                .edges
                .iter()
                .find(|e| {
                    e.from == edge.from
                        && e.offset == edge.offset
                        && e.script.as_ref().unwrap().root == edge.script.as_ref().unwrap().root
                })
                .unwrap();
            assert_eq!(
                (edge.target_x, edge.target_y),
                (current.target_x, current.target_y)
            );
            assert!(current.script.as_ref().unwrap().entry_unresolved);
        }
        assert_eq!(save.data, before);
        assert_eq!(rom.data.as_ref(), &original);
        assert_eq!(std::fs::read(path).unwrap(), original);
        println!("{key}: native presentation parity and {} referenced variable-coordinate entrances; coordinates preserved across SAV snapshots, access unproven", coordinates.len());
    }
}

#[test]
fn script_passages_keep_guards_native_coordinate_width_and_rom_bound_state() {
    use crate::event_dependencies::Index;
    for profile in profile::PROFILES {
        let (mut r, map) = npc_trade_fixture(profile);
        let rules = r.profile.event_state.unwrap().effects.unwrap();
        let b = std::sync::Arc::make_mut(&mut r.data);
        for (opcode, handler) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
            .into_iter()
            .zip(rules.handlers)
        {
            put32(
                b,
                rules.commands + opcode as usize * 4,
                0x08000001 + handler as u32,
            );
        }
        for (opcode, handler) in [0x39, 0x3a, 0x3b, 0x3d, 0x3e, 0xd1, 0xd7]
            .into_iter()
            .zip(rules.warp_handlers)
        {
            if handler != 0 {
                put32(
                    b,
                    rules.commands + opcode as usize * 4,
                    0x08000001 + handler as u32,
                );
            }
        }
        let root = 0x26000;
        let branch = 0x26100;
        // The NPC is hidden by flag 12. Flag 11 selects the passage branch.
        put16(b, 0x25114, 12);
        b[root..root + 3].copy_from_slice(&[0x2b, 11, 0]);
        b[root + 3..root + 5].copy_from_slice(&[6, 1]);
        put32(b, root + 5, 0x08000000 + branch as u32);
        // setwarp cannot add a passage, including a seemingly valid destination.
        b[root + 9..root + 18].copy_from_slice(&[0x3e, 0, 1, 255, 1, 0, 2, 0, 2]);
        // 256 narrows to zero, 511 to -1. Warp index 0 overrides these coords.
        b[branch..branch + 5].copy_from_slice(&[0x16, 1, 0x40, 0, 1]);
        b[branch + 5..branch + 14].copy_from_slice(&[0x3b, 0, 1, 0, 1, 0x40, 255, 1, 2]);
        // Unreferenced apparent passage must not appear in any map.
        b[0x26200..0x26209].copy_from_slice(&[0x3b, 0, 1, 255, 1, 0, 2, 0, 2]);
        b[0x25301] = 1;
        put32(b, 0x25308, 0x08025400);
        put16(b, 0x25400, 1);
        put16(b, 0x25402, 2);
        b[0x25406..0x25408].fill(255);
        // A writer inside the destination itself adds a guarded/cyclic clue.
        // Trace it as an alternative, never as proof that entering is possible.
        b[0x26400..0x26405].copy_from_slice(&[0x2b, 13, 0, 6, 1]);
        put32(b, 0x26405, 0x08026500);
        b[0x26409] = 2;
        b[0x26500..0x26504].copy_from_slice(&[0x29, 11, 0, 2]);
        let mut target = map.clone();
        target.id = "0-1".into();
        target.map_type = 4;
        target.events = Some(0x25300);
        target.scripts = vec![0x26400];
        let maps = vec![map, target];
        let original = r.data.clone();
        let index = Index::build(&r, &maps).unwrap();
        let rom_only = index.map_navigation(&r, &maps, "0-1", None).unwrap();
        assert_eq!(rom_only.incoming.len(), 1, "{}", profile.id);
        let edge = &rom_only.incoming[0];
        assert_eq!(
            (edge.x, edge.y, edge.target_x, edge.target_y),
            (Some(3), Some(2), Some(1), Some(2))
        );
        assert_eq!(edge.kind, "script_warp");
        assert_eq!(edge.offset, branch + 5);
        assert!(edge.unresolved.is_none());
        assert_eq!(rom_only.approaches.len(), 1);
        let script = edge.script.as_ref().unwrap();
        assert!(script.entry_unresolved);
        assert_eq!(script.checks.len(), 2);
        assert!(script.checks.iter().all(|c| c.satisfied.is_none()));
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let before = save.data.clone();
        let saved = index.map_navigation(&r, &maps, "0-1", Some(&save)).unwrap();
        assert!(saved.incoming[0]
            .script
            .as_ref()
            .unwrap()
            .checks
            .iter()
            .any(|c| c.condition.id == 11 && c.satisfied == Some(false)));
        // The planner must use the same referenced edges and fresh saved guards
        // as the map page. A missing acquisition guard does not hide entrance guards.
        let acquisition = crate::acquisition::AcquisitionIndex {
            wild_cache: Default::default(),
            breeding_cache: Default::default(),
            world: crate::world::World {
                maps: maps.clone(),
                map_events: vec![],
                encounters: vec![crate::world::Encounter {
                    selector: None,
                    periods: vec![],
                    species: 3,
                    map_id: "0-1".into(),
                    map_name: "Interior".into(),
                    region: 1,
                    method: "grass".into(),
                    min_level: 5,
                    max_level: 5,
                    weight: Some(20),
                    encounter_rate: Some(20),
                    slot: Some(0),
                    offset: 0x27000,
                    conditional: false,
                }],
                trainers: vec![],
                trainer_locations: crate::world::TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: vec![r.valid_species(3).unwrap()],
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let request = || crate::collection::CollectionRequest {
            basis: crate::collection::CollectionBasis::Individuals,
            families: false,
            include_unknown_rewards: false,
        };
        let plan = acquisition
            .collection_with_passages(&r, &save, request(), &index)
            .unwrap();
        let entry = plan.entrances.iter().find(|e| e.map_id == "0-1").unwrap();
        assert_eq!(
            serde_json::to_value(&entry.chains).unwrap(),
            serde_json::to_value(&saved.approaches).unwrap()
        );
        assert!(entry.unresolved_incoming.is_empty());
        assert!(plan.entrance_coverage.is_some());
        let bundle = index.trace_plan(&r, &save, &maps, &plan).unwrap();
        let clue = bundle
            .reports
            .iter()
            .position(|r| r.condition.condition.id == 11)
            .unwrap();
        assert!(!bundle.routes[clue].goals.is_empty());
        assert_eq!(bundle.reports[clue].condition.satisfied, Some(false));
        let writer = &bundle.routes[clue].candidates[0];
        assert!(writer
            .requires
            .iter()
            .any(|i| bundle.reports[*i].condition.condition.id == 13));
        assert!(writer
            .entry_requires
            .iter()
            .flatten()
            .any(|i| bundle.reports[*i].condition.condition.id == 11));
        assert!(writer.recursive);

        assert_eq!(save.data, before);
        let range = &profile.event_state.unwrap().flags[0];
        let bit = (11 - range.first) as usize;
        let mut main = save.logical(1..=4);
        main[range.offset + bit / 8] |= 1 << (bit % 8);
        synthetic_daycare_main(&mut save, &main);
        let changed = save.data.clone();
        let saved = index.map_navigation(&r, &maps, "0-1", Some(&save)).unwrap();
        assert!(saved.incoming[0]
            .script
            .as_ref()
            .unwrap()
            .checks
            .iter()
            .any(|c| c.condition.id == 11 && c.satisfied == Some(true)));
        let refreshed = acquisition
            .collection_with_passages(&r, &save, request(), &index)
            .unwrap();
        let bundle = index.trace_plan(&r, &save, &maps, &refreshed).unwrap();
        assert!(bundle
            .reports
            .iter()
            .any(|r| r.condition.condition.id == 11 && r.condition.satisfied == Some(true)));
        assert!(
            !bundle
                .reports
                .iter()
                .any(|r| r.condition.condition.id == 13),
            "already-met conditions do not expand writer or entrance guards"
        );
        assert_eq!(save.data, changed);
        assert_ne!(save.data, before);
        let reverse = index.map_navigation(&r, &maps, "0-1", None).unwrap();
        assert!(
            reverse
                .outgoing
                .iter()
                .all(|e| e.to.as_deref() != Some("0-0")),
            "never fabricate a return passage"
        );
        assert_eq!(r.data, original);
        let mut stale = r.clone();
        std::sync::Arc::make_mut(&mut stale.data)[0] ^= 1;
        assert_eq!(
            acquisition
                .collection_with_passages(&stale, &save, request(), &index)
                .err()
                .unwrap()
                .code,
            "rom_mismatch"
        );
        assert_eq!(
            index
                .map_navigation(&stale, &maps, "0-1", None)
                .err()
                .unwrap()
                .code,
            "rom_mismatch"
        );
        // Direct coordinates use the native s8 result; unknown variables survive.
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[branch + 8] = 255;
        let warp = r
            .script_warp_instruction(branch + 5, |id| {
                (id == 0x4001)
                    .then_some(256)
                    .or_else(|| (id < 0x4000).then_some(id))
            })
            .unwrap()
            .unwrap();
        assert_eq!((warp.x, warp.y), (Some(0), Some(-1)));
        let unresolved = r
            .script_warp_instruction(branch + 5, |id| (id < 0x4000).then_some(id))
            .unwrap()
            .unwrap();
        assert_eq!(unresolved.x, None);
        assert_eq!(unresolved.y, Some(-1));
        // An unknown native call invalidates a previously known coordinate.
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[0x26400..0x26413].copy_from_slice(&[
            0x16, 1, 0x40, 2, 0, 0x23, 0, 0, 0, 8, 0x3b, 0, 1, 255, 1, 0x40, 1, 0, 2,
        ]);
        let unknown = r.event_effect_script(0x26400).unwrap();
        assert_eq!(unknown.warps.len(), 1);
        assert_eq!(unknown.warps[0].x, None);
        assert!(unknown.stopped_at.contains(&0x26405));
        let b = std::sync::Arc::make_mut(&mut r.data);
        put32(
            b,
            rules.commands + 0x3b * 4,
            0x08000003 + rules.warp_handlers[2] as u32,
        );
        assert_eq!(
            r.script_warp_instruction(branch + 5, Some)
                .err()
                .unwrap()
                .code,
            "script_warp_dispatch"
        );
    }
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_WARP_PROBES independent mGBA vectors"]
fn local_script_warp_operands_match_native_all_profiles() {
    let probes: serde_json::Value =
        serde_json::from_slice(&std::fs::read(std::env::var("GEN3_WARP_PROBES").unwrap()).unwrap())
            .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let original = Rom::open(std::fs::read(&path).unwrap()).unwrap();
        let hash = crate::binary::sha256(&original.data);
        assert_eq!(probes[key]["md5"], original.profile.md5);
        let mut fixture = original.clone();
        let pc = fixture.data.len();
        std::sync::Arc::make_mut(&mut fixture.data).extend_from_slice(&[0; 8]);
        let mut count = 0;
        for row in probes[key]["rows"].as_array().unwrap() {
            let op = row["opcode"].as_u64().unwrap() as u8;
            let operands: Vec<u8> = serde_json::from_value(row["operands"].clone()).unwrap();
            let expected: Vec<i32> =
                serde_json::from_value(row["expected_arguments"].clone()).unwrap();
            let resolved: Vec<u16> = serde_json::from_value(row["resolved"].clone()).unwrap();
            let b = std::sync::Arc::make_mut(&mut fixture.data);
            b[pc] = op;
            b[pc + 1..pc + 8].copy_from_slice(&operands);
            let parsed = fixture
                .script_warp_instruction(pc, |v| match v {
                    0x4001 => Some(resolved[0]),
                    0x4002 => Some(resolved[1]),
                    v if v < 0x4000 => Some(v),
                    _ => None,
                })
                .unwrap();
            if op == 0x3e {
                assert!(parsed.is_none());
            } else {
                let parsed = parsed.unwrap();
                assert_eq!(parsed.group as i8 as i32, expected[0]);
                assert_eq!(parsed.number as i8 as i32, expected[1]);
                assert_eq!(parsed.warp as i8 as i32, expected[2]);
                assert_eq!(parsed.x.map(i32::from), Some(expected[3]));
                assert_eq!(parsed.y.map(i32::from), Some(expected[4]));
            }
            count += 1;
        }
        let maps = original.maps().unwrap();
        let index = crate::event_dependencies::Index::build(&original, &maps).unwrap();
        let mut passages = 0;
        for map in maps.iter().take(50) {
            let report = index
                .map_navigation(&original, &maps, &map.id, None)
                .unwrap();
            for edge in report.outgoing.iter().filter(|e| e.kind == "script_warp") {
                passages += 1;
                assert!(edge.script.as_ref().unwrap().entry_unresolved);
                assert_ne!(edge.script.as_ref().unwrap().opcode, 0x3e);
            }
        }
        assert_eq!(crate::binary::sha256(&original.data), hash);
        assert_eq!(crate::binary::sha256(&std::fs::read(path).unwrap()), hash);
        println!("{key}: {count} native operand vectors; first 50 maps have {passages} referenced script passages; access remains unverified");
    }
}

#[test]
fn native_extension_configuration_rejects_overlaps_and_invalid_sector_bounds() {
    let r = adapter_rom(crate::mercury::PROFILE);
    let bytes = save_bytes(&r);
    for sectors in [&[27_usize][..], &[32][..], &[30, 30][..]] {
        let sectors: &'static [usize] = Box::leak(sectors.to_vec().into_boxed_slice());
        let mut layout = r.profile.save;
        layout.extension_sectors = Some(sectors);
        assert_eq!(
            Save::open(bytes.clone(), layout).err().unwrap().code,
            "save_layout"
        );
    }
    let mut layout = r.profile.save;
    layout.sizes[0] = 0xff4;
    assert_eq!(Save::open(bytes, layout).err().unwrap().code, "save_layout");
}

#[test]
#[ignore = "requires private exact Mercury 1.2 ROM and current SAV"]
fn local_mercury_storage_roundtrip() {
    let r =
        Rom::open(std::fs::read(std::env::var("GEN3_ROM_MERCURY12").unwrap()).unwrap()).unwrap();
    let original = std::fs::read(std::env::var("GEN3_SAVE_MERCURY12").unwrap()).unwrap();
    let save = Save::open(original.clone(), r.profile.save).unwrap();
    save.validate(&r).unwrap();
    assert_eq!(save.bag().unwrap().len(), 808);
    let mut session = Session::new(r.clone());
    session.load(original.clone(), None).unwrap();
    for pocket in r.profile.save.pockets {
        let item = (1..r.profile.items.count as u16)
            .find(|id| r.item(*id).unwrap().pocket == pocket.category || pocket.category == 0)
            .unwrap();
        for slot in [0, pocket.count - 1] {
            let before = session.snapshot().unwrap();
            session
                .apply(
                    Action::Bag {
                        pocket: pocket.id.into(),
                        slot,
                        item,
                        quantity: 1,
                    },
                    Policy::Standard,
                )
                .unwrap();
            let after = session.snapshot().unwrap();
            assert_eq!(
                serde_json::to_value(before.pokemon).unwrap(),
                serde_json::to_value(after.pokemon).unwrap()
            );
            Save::open(session.save_ref().unwrap().data.clone(), r.profile.save)
                .unwrap()
                .validate(&r)
                .unwrap();
            session.undo().unwrap();
            assert_eq!(session.save_ref().unwrap().data, original);
        }
    }
    assert_eq!(r.region_name(143), "若叶镇");
    assert_eq!(
        r.map_image_warnings("1-0").unwrap(),
        ["mapNativeLayoutMismatch"]
    );
    // A playable/current map has no mismatched-layout annotation.
    assert!(r.map_image_warnings("3-79").unwrap().is_empty());
}

// Write synthetic query fields in the same segmented storage the native hook reads.
// Queries below must preserve the entire SAVE, including both banks and shared sectors.
fn clock_extra_byte(save: &mut Save, mut offset: usize, value: u8) {
    for id in 0..14 {
        let n = 0xff0 - save.layout.sizes[id];
        if offset < n {
            save.data[save.sections[id] + save.layout.sizes[id] + offset] = value;
            return;
        }
        offset -= n;
    }
    for sector in save.layout.extension_sectors.unwrap() {
        if offset < 0xff0 {
            save.data[sector * 4096 + offset] = value;
            return;
        }
        offset -= 0xff0;
    }
    panic!("synthetic clock offset exceeds extension storage");
}
fn clock_fields(save: &mut Save, fields: [u16; 4], forced: bool, speed: u16) {
    clock_extra_byte(save, 0x146, 0x20);
    clock_extra_byte(save, 0xe8, u8::from(forced) * 2);
    for (i, field) in fields.into_iter().chain([speed]).enumerate() {
        for (j, b) in field.to_le_bytes().into_iter().enumerate() {
            clock_extra_byte(save, 0x5de + i * 2 + j, b);
        }
    }
}
#[test]
fn saved_clock_preserves_bytes_and_distinguishes_rtc_invalid_and_simulated_time() {
    use crate::clock::ClockScenario;
    let mut r = adapter_rom(crate::mercury::PROFILE);
    let offset = r.profile.clock.unwrap().saved.unwrap().month_lengths;
    let data = std::sync::Arc::make_mut(&mut r.data);
    for (i, length) in [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        .into_iter()
        .enumerate()
    {
        put32(data, offset + i * 4, length);
    }
    let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let current = || ClockScenario {
        hour: None,
        weekday: None,
    };
    clock_fields(&mut save, [2026, 0xa05, 0x171e, 0x2d01], false, 2);
    let before = save.data.clone();
    let report = r.clock_query_with_save(Some(&save), current()).unwrap();
    assert_eq!(report.source, "save_virtual");
    assert_eq!(report.next_period_hour, Some(4));
    assert_eq!(
        report.seconds_until_next_period,
        Some(4 * 3600 + 29 * 60 + 15)
    );
    assert_eq!(report.weekday, Some(1));
    assert_eq!(report.saved.unwrap().speed, 2);
    assert_eq!(save.data, before);
    clock_fields(&mut save, [2026, 0xa05, 0x081e, 0], true, 3);
    let report = r.clock_query_with_save(Some(&save), current()).unwrap();
    assert_eq!(report.period, Some("night"));
    assert_eq!(report.next_period_hour, None);
    assert_eq!(report.saved.unwrap().speed, 1);
    let simulated = r
        .clock_query_with_save(
            Some(&save),
            ClockScenario {
                hour: Some(17),
                weekday: None,
            },
        )
        .unwrap();
    assert_eq!(simulated.source, "scenario");
    assert_eq!(simulated.period, Some("dusk"));
    assert!(!simulated.current_clock_verified);
    assert_eq!(simulated.forced_night, None);
    // A weekday-only scenario must not accidentally recover/inherit SAVE's hour.
    assert_eq!(
        r.clock_query_with_save(
            Some(&save),
            ClockScenario {
                hour: None,
                weekday: Some(2)
            }
        )
        .unwrap()
        .effective_hour,
        None
    );
    clock_extra_byte(&mut save, 0x146, 0);
    assert_eq!(
        r.clock_query_with_save(Some(&save), current())
            .unwrap()
            .issue,
        Some("hardware_rtc_unresolved")
    );
    for fields in [
        [1999, 0xa05, 0, 0],
        [3200, 0xa05, 0, 0],
        [2100, 0x21d, 0, 0],
        [2026, 0xa00, 0, 0],
        [2026, 0xa05, 0x1800, 0],
        [2026, 0xa05, 0, 7],
    ] {
        clock_fields(&mut save, fields, false, 1);
        let report = r.clock_query_with_save(Some(&save), current()).unwrap();
        assert_eq!(report.issue, Some("invalid_saved_clock"));
        assert_eq!(report.effective_hour, None);
    }
    clock_fields(&mut save, [2400, 0x21d, 0, 0], false, 1);
    assert!(
        r.clock_query_with_save(Some(&save), current())
            .unwrap()
            .current_clock_verified
    );
    // Use runtime month data, not a bundled Gregorian month catalog.
    std::sync::Arc::make_mut(&mut r.data)[offset + 9 * 4] = 4;
    clock_fields(&mut save, [2026, 0xa05, 0, 0], false, 1);
    assert_eq!(
        r.clock_query_with_save(Some(&save), current())
            .unwrap()
            .issue,
        Some("invalid_saved_clock")
    );
    for p in [
        profile::BW,
        profile::DP,
        profile::ROCKET,
        crate::ultimate::PROFILE,
    ] {
        let other = adapter_rom(p);
        assert_eq!(
            other
                .clock_query_with_save(Some(&save), current())
                .err()
                .unwrap()
                .code,
            "clock_save_layout"
        );
    }
}
#[test]
#[ignore = "requires exact Mercury 1.2 ROM/SAV and verify_mercury_clock.py parity vectors"]
fn local_mercury_clock_matches_native_restore_and_period_selection() {
    use crate::clock::ClockScenario;
    let r =
        Rom::open(std::fs::read(std::env::var("GEN3_ROM_MERCURY12").unwrap()).unwrap()).unwrap();
    let mut save = Save::open(
        std::fs::read(std::env::var("GEN3_SAVE_MERCURY12").unwrap()).unwrap(),
        r.profile.save,
    )
    .unwrap();
    let data: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_CLOCK_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(data["rom_md5"], r.profile.md5);
    let query = || ClockScenario {
        hour: None,
        weekday: None,
    };
    if let Some(expected) = data.get("save_sha256") {
        assert_eq!(
            expected,
            &serde_json::json!(crate::binary::sha256(&save.data)),
            "native vector belongs to a different SAV snapshot"
        );
    }
    let original = save.data.clone();
    let report = r.clock_query_with_save(Some(&save), query()).unwrap();
    let mut expected = data["current_save"]["clock"].clone();
    expected["speed"] = serde_json::json!(report.saved.as_ref().unwrap().speed);
    assert_eq!(serde_json::to_value(report.saved).unwrap(), expected);
    assert_eq!(
        serde_json::to_value(report.forced_night).unwrap(),
        data["current_save"]["forced_night"]
    );
    assert_eq!(save.data, original);
    for vector in data["restore_vectors"].as_array().unwrap() {
        let words: [u16; 4] = serde_json::from_value(vector["words"].clone()).unwrap();
        clock_fields(
            &mut save,
            words,
            vector["forced"].as_bool().unwrap(),
            vector["speed"].as_u64().unwrap() as u16,
        );
        let before = save.data.clone();
        let report = r.clock_query_with_save(Some(&save), query()).unwrap();
        let mut expected = vector["expected"].clone();
        expected["speed"] = vector["speed"].clone();
        assert_eq!(serde_json::to_value(report.saved).unwrap(), expected);
        assert_eq!(
            serde_json::to_value(report.period).unwrap(),
            vector["period"]
        );
        assert_eq!(save.data, before);
    }
    for words in data["invalid_vectors"].as_array().unwrap() {
        clock_fields(
            &mut save,
            serde_json::from_value(words.clone()).unwrap(),
            false,
            1,
        );
        assert_eq!(
            r.clock_query_with_save(Some(&save), query()).unwrap().issue,
            Some("invalid_saved_clock")
        );
    }
}

#[test]
fn receipt_queries_use_native_flags_not_bag_or_npc_visibility() {
    use crate::{
        acquisition::{AcquisitionIndex, Target, TargetKind},
        map_events::{ItemReward, MapEventReport, MapMarker},
        world::{Map, TrainerLocationIndex, World},
    };
    for profile in profile::PROFILES {
        let r = adapter_rom(profile);
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let range = profile.event_state.unwrap().flags[0];
        // Every adapter has a verified SB1 flag 1; the actual logical offset differs.
        let mut offset = range.offset;
        for section in 1..=4 {
            if offset < save.layout.sizes[section] {
                save.data[save.sections[section] + offset] |= 2;
                break;
            }
            offset -= save.layout.sizes[section];
        }
        let before = save.data.clone();
        let map = Map {
            id: "0-0".into(),
            group: 0,
            number: 0,
            name: "Synthetic".into(),
            region: 1,
            width: 4,
            height: 4,
            map_type: 1,
            header: 0,
            layout: 0,
            invalid_events: false,
            events: None,
            scripts: vec![],
            objects: vec![],
        };
        let markers = ["hidden", "pickup", "gift"]
            .into_iter()
            .map(|kind| MapMarker {
                id: kind.into(),
                kind,
                x: 1,
                y: 1,
                elevation: 0,
                local_id: Some(1),
                graphics_id: None,
                movement_type: None,
                underfoot: None,
                flag: Some(1),
                receipt_flag: (kind != "gift").then_some(1),
                offset: 0,
                script: None,
                stopped_at: vec![],
                pokemon: vec![],
                teaching: vec![],
                daycare: vec![],
                scripted_movements: vec![],
                rewards: vec![ItemReward {
                    item: 1,
                    quantity: Some(1),
                    offset: 0,
                    via: kind,
                    conditions: vec![],
                    receipt: None,
                }],
            })
            .collect();
        let mut index = AcquisitionIndex {
            wild_cache: std::cell::RefCell::default(),
            breeding_cache: Default::default(),
            world: World {
                maps: vec![map],
                map_events: vec![MapEventReport {
                    map_id: "0-0".into(),
                    markers,
                    unplaced_rewards: vec![],
                    unplaced_pokemon: vec![],
                    unplaced_teaching: vec![],
                    unplaced_daycare: vec![],
                    unplaced_movements: vec![],
                    stopped_at: vec![],
                }],
                encounters: vec![],
                trainers: vec![],
                trainer_locations: TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: vec![],
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let report = index
            .query(
                &r,
                Some(&save),
                Target {
                    kind: TargetKind::Item,
                    id: 1,
                },
            )
            .unwrap();
        let hidden = report.sources.iter().find(|s| s.kind == "hidden").unwrap();
        assert_eq!(
            (hidden.status, hidden.receipt_flag, hidden.repeatable),
            ("completed", Some(1), None)
        );
        let pickup = report.sources.iter().find(|s| s.kind == "pickup").unwrap();
        assert_eq!(
            pickup.status,
            if profile.event_state.unwrap().pickup_receipt {
                "completed"
            } else {
                "unknown"
            }
        );
        let gift = report.sources.iter().find(|s| s.kind == "gift").unwrap();
        assert_ne!(gift.status, "completed");
        assert_eq!(gift.receipt_flag, None);
        assert_eq!(pickup.repeatable, None);
        let plan = index
            .collection(
                &r,
                &save,
                crate::collection::CollectionRequest {
                    basis: crate::collection::CollectionBasis::Individuals,
                    families: true,
                    include_unknown_rewards: true,
                },
            )
            .unwrap();
        assert!(plan
            .regions
            .iter()
            .flat_map(|region| &region.tasks)
            .all(|task| {
                task.source
                    .as_ref()
                    .is_none_or(|s| !matches!(s.kind.as_str(), "hidden" | "pickup"))
            }));
        // Even a set visibility bit is insufficient for a compound/unverified pickup.
        index.world.map_events[0].markers[1].receipt_flag = None;
        let unverified = index
            .query(
                &r,
                Some(&save),
                Target {
                    kind: TargetKind::Item,
                    id: 1,
                },
            )
            .unwrap();
        assert_eq!(
            unverified
                .sources
                .iter()
                .find(|s| s.kind == "pickup")
                .unwrap()
                .status,
            "unknown"
        );
        // A receipt ID outside the native-verified persistence ranges stays unknown.
        index.world.map_events[0].markers[0].receipt_flag = Some(0xffff);
        let outside = index
            .query(
                &r,
                Some(&save),
                Target {
                    kind: TargetKind::Item,
                    id: 1,
                },
            )
            .unwrap();
        assert_eq!(
            outside
                .sources
                .iter()
                .find(|s| s.kind == "hidden")
                .unwrap()
                .status,
            "unknown"
        );
        // A native/unknown stop may alter later guards; do not turn incomplete traversal into a blocked claim.
        index.world.map_events[0].markers[2].stopped_at.push(0xdead);
        index.world.map_events[0].markers[2].rewards[0]
            .conditions
            .push(crate::map_events::EventCondition {
                kind: "flag",
                id: 2,
                value: 1,
                comparison: 1,
                taken: true,
            });
        let partial = index
            .query(
                &r,
                Some(&save),
                Target {
                    kind: TargetKind::Item,
                    id: 1,
                },
            )
            .unwrap();
        assert_eq!(
            partial
                .sources
                .iter()
                .find(|s| s.kind == "gift")
                .unwrap()
                .status,
            "unknown"
        );
        assert_eq!(save.data, before);
        let without_save = index
            .query(
                &r,
                None,
                Target {
                    kind: TargetKind::Item,
                    id: 1,
                },
            )
            .unwrap();
        assert!(without_save.sources.iter().all(|s| s.status == "unknown"));
    }
}

#[test]
fn pickup_receipts_require_complete_ordinary_scripts_per_adapter() {
    use crate::world::Map;
    let ordinary = [0x1a, 0, 0x80, 1, 0, 0x1a, 1, 0x80, 2, 0, 9, 1, 2];
    for profile in profile::PROFILES {
        let mut r = adapter_rom(profile);
        let ev = 0x25000;
        let objects = 0x25100;
        let script = 0x25300;
        {
            let b = std::sync::Arc::make_mut(&mut r.data);
            b[ev] = 1;
            put32(b, ev + 4, 0x08000000 + objects as u32);
            b[objects] = 1;
            put16(b, objects + 20, 1);
            put32(b, objects + 16, 0x08000000 + script as u32);
        }
        let map = Map {
            id: "0-0".into(),
            group: 0,
            number: 0,
            name: "Synthetic".into(),
            region: 0,
            width: 4,
            height: 4,
            map_type: 1,
            header: 0,
            layout: 0,
            invalid_events: false,
            events: Some(ev),
            scripts: vec![],
            objects: vec![],
        };
        // Same reward can exist in a compound script without owning its object's flag.
        let mut reassigned = vec![0x16, 0x0f, 0x80, 2, 0];
        reassigned.extend(ordinary);
        let mut native_prelude = vec![0x25, 0, 0];
        native_prelude.extend(ordinary);
        let mut extra_effect = ordinary[..12].to_vec();
        extra_effect.extend([0x29, 2, 0, 2]);
        let mut dynamic_quantity = ordinary.to_vec();
        dynamic_quantity[8..10].copy_from_slice(&0x8002u16.to_le_bytes());
        let mut gift = ordinary.to_vec();
        gift[11] = 0;
        for (code, receipt) in [
            (ordinary.to_vec(), Some(1)),
            (reassigned, None),
            (native_prelude, None),
            (extra_effect, None),
            (dynamic_quantity, None),
            (gift, None),
        ] {
            let b = std::sync::Arc::make_mut(&mut r.data);
            b[script..script + 64].fill(0);
            b[script..script + code.len()].copy_from_slice(&code);
            let report = r.map_events(&map).unwrap();
            assert_eq!(report.markers[0].receipt_flag, receipt, "{}", profile.id);
        }
        std::sync::Arc::make_mut(&mut r.data)[script..script + 13].copy_from_slice(&ordinary);
        let mut layout = r.profile.event_state.unwrap();
        layout.pickup_receipt = false;
        r.profile.event_state = Some(layout);
        assert_eq!(r.map_events(&map).unwrap().markers[0].receipt_flag, None);
        // Explicit constant assignments share the same verified native semantics.
        layout.pickup_receipt = true;
        r.profile.event_state = Some(layout);
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[script] = 0x16;
        b[script + 5] = 0x16;
        assert_eq!(r.map_events(&map).unwrap().markers[0].receipt_flag, Some(1));
        std::sync::Arc::make_mut(&mut r.data)[objects + 20..objects + 22].fill(0);
        assert_eq!(r.map_events(&map).unwrap().markers[0].receipt_flag, None);
    }
}

#[test]
#[ignore = "requires all five private ROM paths and GEN3_PICKUP_PROBES native vectors"]
fn local_pickup_receipts_match_native_protocol() {
    let vectors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_PICKUP_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        let proof = &vectors[key];
        assert_eq!(r.profile.md5, proof["md5"].as_str().unwrap());
        let world = r.world().unwrap();
        let mut actual = std::collections::BTreeMap::new();
        for report in &world.map_events {
            for marker in &report.markers {
                if marker.kind == "pickup" {
                    if let Some(flag) = marker.receipt_flag {
                        actual.insert((report.map_id.clone(), marker.offset), flag);
                    }
                }
            }
        }
        let expected: std::collections::BTreeMap<_, _> = proof["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                (
                    (
                        row["map_id"].as_str().unwrap().to_owned(),
                        row["marker_offset"].as_u64().unwrap() as usize,
                    ),
                    row["receipt_flag"].as_u64().unwrap() as u16,
                )
            })
            .collect();
        assert_eq!(actual, expected, "{key}");
        eprintln!(
            "{key}: {} ordinary receipts match native capacity branches",
            actual.len()
        );
    }
}

#[test]
fn gift_receipts_follow_success_branches_and_reject_ambiguous_flags() {
    // These are executable script fixtures, not copies of a game's NPC catalog.
    fn code(root: usize, flag: u16, result_guard: bool) -> Vec<u8> {
        let mut b = vec![0x2b, flag as u8, (flag >> 8) as u8, 6, 1];
        b.extend((0x08000000 + root as u32 + 64).to_le_bytes());
        b.extend([0x1a, 0, 0x80, 1, 0, 0x1a, 1, 0x80, 1, 0, 9, 0]);
        if result_guard {
            b.extend([0x21, 0x0d, 0x80, 0, 0, 6, 1]);
            b.extend((0x08000000 + root as u32 + 64).to_le_bytes());
        }
        b.extend([0x29, flag as u8, (flag >> 8) as u8, 2]);
        b.resize(65, 2);
        b
    }
    for profile in profile::PROFILES {
        let mut r = adapter_rom(profile);
        let root = 0x25000;
        let install = |r: &mut Rom, code: &[u8]| {
            let b = std::sync::Arc::make_mut(&mut r.data);
            b[root..root + 256].fill(2);
            b[root..root + code.len()].copy_from_slice(code);
        };
        let good = code(root, 1, true);
        install(&mut r, &good);
        let (rewards, stopped) = r.item_script(root).unwrap();
        assert!(stopped.is_empty());
        let proof = rewards[0].receipt.as_ref().unwrap();
        assert_eq!(
            (proof.flag, proof.root, proof.award_offset),
            (1, root, root + 19)
        );
        assert_eq!(proof.success_set_offsets, vec![root + 32]);
        // Shared calls retain the return stack and a copied result variable.
        let mut called = good[..19].to_vec();
        called.extend([4]);
        called.extend((0x08000000 + root as u32 + 80).to_le_bytes());
        called.extend([0x21, 7, 0x80, 0, 0, 6, 1]);
        called.extend((0x08000000 + root as u32 + 64).to_le_bytes());
        called.extend([0x29, 1, 0, 2]);
        called.resize(80, 2);
        called.extend([9, 0, 0x19, 7, 0x80, 0x0d, 0x80, 3]);
        install(&mut r, &called);
        assert_eq!(
            r.item_script(root).unwrap().0[0]
                .receipt
                .as_ref()
                .unwrap()
                .flag,
            1
        );
        // Direct additem has the same native boolean result contract.
        let mut direct = good[..19].to_vec();
        direct.extend([0x44, 1, 0, 1, 0, 0x21, 0x0d, 0x80, 0, 0, 6, 1]);
        direct.extend((0x08000000 + root as u32 + 64).to_le_bytes());
        direct.extend([0x29, 1, 0, 2]);
        direct.resize(65, 2);
        install(&mut r, &direct);
        assert!(r.item_script(root).unwrap().0[0].receipt.is_some());
        let mut cleared = good[..35].to_vec();
        cleared.extend([0x2a, 1, 0, 2]);
        cleared.resize(65, 2);
        let mut prelude = vec![0x25, 0, 0];
        prelude.extend(&good);
        let mut unguarded = good.clone();
        unguarded[..9].fill(0);
        let mut unknown = good.clone();
        unknown[35] = 0xff;
        let mut looped = good.clone();
        looped.truncate(35);
        looped.extend([5]);
        looped.extend((0x08000000 + root as u32 + 35).to_le_bytes());
        looped.resize(65, 2);
        let mut dynamic = good.clone();
        dynamic[17..19].copy_from_slice(&0x8002u16.to_le_bytes());
        for bad in [
            code(root, 1, false),
            cleared,
            prelude,
            unguarded,
            unknown,
            looped,
            dynamic,
        ] {
            install(&mut r, &bad);
            assert!(
                r.item_script(root)
                    .unwrap()
                    .0
                    .iter()
                    .all(|reward| reward.receipt.is_none()),
                "{}",
                profile.id
            );
        }
        // Two alternate awards cannot both claim a shared "done" flag.
        let mut alternate = vec![0x2b, 1, 0, 6, 1];
        alternate.extend((0x08000000 + root as u32 + 200).to_le_bytes());
        alternate.extend([0x21, 2, 0x80, 1, 0, 6, 1]);
        alternate.extend((0x08000000 + root as u32 + 80).to_le_bytes());
        for (start, item) in [(20, 1), (80, 2)] {
            alternate.resize(start, 0);
            alternate.extend([0x44, item, 0, 1, 0, 0x21, 0x0d, 0x80, 0, 0, 6, 1]);
            alternate.extend((0x08000000 + root as u32 + 200).to_le_bytes());
            alternate.extend([0x29, 1, 0, 2]);
        }
        alternate.resize(201, 2);
        install(&mut r, &alternate);
        assert!(r
            .item_script(root)
            .unwrap()
            .0
            .iter()
            .all(|r| r.receipt.is_none()));
        install(&mut r, &good);
        let mut layout = r.profile.event_state.unwrap();
        layout.gift_result = false;
        r.profile.event_state = Some(layout);
        assert!(r.item_script(root).unwrap().0[0].receipt.is_none());
    }
}

#[test]
fn gift_receipt_queries_and_plans_use_the_qualified_reward_not_visibility() {
    use crate::{
        acquisition::{AcquisitionIndex, Target, TargetKind},
        collection::{CollectionBasis, CollectionRequest},
        map_events::{EventCondition, ItemReward, MapEventReport, MapMarker, ReceiptEvidence},
    };
    for profile in profile::PROFILES {
        let r = adapter_rom(profile);
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let range = r.profile.event_state.unwrap().flags[0];
        let mut offset = range.offset;
        for section in 1..=4 {
            if offset < save.layout.sizes[section] {
                save.data[save.sections[section] + offset] |= 2;
                break;
            }
            offset -= save.layout.sizes[section];
        }
        let data = save.data.clone();
        let map = crate::world::Map {
            id: "0-0".into(),
            group: 0,
            number: 0,
            name: "Synthetic".into(),
            region: 1,
            width: 4,
            height: 4,
            map_type: 1,
            header: 0,
            layout: 0,
            invalid_events: false,
            events: None,
            scripts: vec![],
            objects: vec![],
        };
        let mut index = AcquisitionIndex {
            wild_cache: std::cell::RefCell::default(),
            breeding_cache: Default::default(),
            world: crate::world::World {
                maps: vec![map],
                map_events: vec![],
                encounters: vec![],
                trainers: vec![],
                trainer_locations: crate::world::TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: vec![],
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let reward = ItemReward {
            item: 1,
            quantity: Some(1),
            offset: 100,
            via: "gift",
            conditions: vec![EventCondition {
                kind: "flag",
                id: 1,
                value: 1,
                comparison: 1,
                taken: false,
            }],
            receipt: Some(ReceiptEvidence {
                flag: 1,
                root: 90,
                award_offset: 100,
                success_set_offsets: vec![110],
            }),
        };
        index.world.map_events = vec![MapEventReport {
            map_id: index.world.maps[0].id.clone(),
            markers: vec![MapMarker {
                id: "gift".into(),
                kind: "gift",
                x: 1,
                y: 1,
                elevation: 0,
                local_id: Some(1),
                graphics_id: None,
                movement_type: None,
                underfoot: None,
                flag: Some(2),
                receipt_flag: None,
                offset: 80,
                script: Some(90),
                pokemon: vec![],
                teaching: vec![],
                daycare: vec![],
                scripted_movements: vec![],
                rewards: vec![reward],
                stopped_at: vec![],
            }],
            unplaced_rewards: vec![],
            unplaced_pokemon: vec![],
            unplaced_teaching: vec![],
            unplaced_daycare: vec![],
            unplaced_movements: vec![],
            stopped_at: vec![],
        }];
        let target = Target {
            kind: TargetKind::Item,
            id: 1,
        };
        let query = index.query(&r, Some(&save), target.clone()).unwrap();
        let source = query.sources.iter().find(|s| s.kind == "gift").unwrap();
        assert_eq!((source.status, source.receipt_flag), ("completed", Some(1)));
        assert!(source.receipt.is_some());
        let plan = index
            .collection(
                &r,
                &save,
                CollectionRequest {
                    basis: CollectionBasis::Individuals,
                    families: true,
                    include_unknown_rewards: true,
                },
            )
            .unwrap();
        assert!(plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .all(|t| t.source.as_ref().is_none_or(|s| s.offset != 100)));
        index.world.map_events[0].markers[0].rewards[0].receipt = None;
        let query = index.query(&r, Some(&save), target).unwrap();
        assert_eq!(
            query
                .sources
                .iter()
                .find(|s| s.kind == "gift")
                .unwrap()
                .status,
            "unknown"
        );
        assert_eq!(save.data, data);
    }
}

#[test]
#[ignore = "requires five exact private ROMs and GEN3_NPC_PROBES native vectors"]
fn local_npc_receipts_match_native_control_flow() {
    let vectors: serde_json::Value =
        serde_json::from_slice(&std::fs::read(std::env::var("GEN3_NPC_PROBES").unwrap()).unwrap())
            .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        let proof = &vectors[key];
        assert_eq!(r.profile.md5, proof["md5"].as_str().unwrap());
        assert_eq!(proof["rom_memory_unchanged"], true);
        let world = r.world().unwrap();
        let mut actual = Vec::new();
        for report in &world.map_events {
            for marker in &report.markers {
                for reward in &marker.rewards {
                    if let Some(receipt) = &reward.receipt {
                        assert_eq!(marker.script, Some(receipt.root));
                        assert_eq!(reward.offset, receipt.award_offset);
                        assert!(!receipt.success_set_offsets.is_empty());
                        actual.push((
                            report.map_id.clone(),
                            marker.offset,
                            reward.offset,
                            receipt.flag,
                        ));
                    }
                }
            }
            assert!(
                report.unplaced_rewards.iter().all(|r| r.receipt.is_none()),
                "new unplaced proof needs a native vector"
            );
        }
        let mut expected = proof["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                (
                    row["map_id"].as_str().unwrap().to_owned(),
                    row["marker_offset"].as_u64().unwrap() as usize,
                    row["award_offset"].as_u64().unwrap() as usize,
                    row["receipt_flag"].as_u64().unwrap() as u16,
                )
            })
            .collect::<Vec<_>>();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected, "{key}");
        assert_eq!(proof["cases"].as_u64().unwrap() as usize, actual.len() * 4);
        eprintln!(
            "{key}: {} qualified NPC reward rows, {} native cases",
            actual.len(),
            actual.len() * 4
        );
    }
}

// Reader fixture for native zero-count records. The editor deliberately rejects
// creating these; only synthetic test bytes are changed here, never user saves.
fn fixture_zero_bag_quantity(save: &mut Save, pocket_id: &str, slot: usize) {
    let pocket = save
        .layout
        .pockets
        .iter()
        .find(|p| p.id == pocket_id)
        .unwrap();
    let spans: Vec<_> = match pocket.block {
        crate::save::PocketBlock::Main => (1..=4)
            .map(|id| (save.sections[id], save.layout.sizes[id]))
            .collect(),
        crate::save::PocketBlock::SectorExtensions => (0..14)
            .map(|id| {
                (
                    save.sections[id] + save.layout.sizes[id],
                    0xff0 - save.layout.sizes[id],
                )
            })
            .chain(
                save.layout
                    .extension_sectors
                    .unwrap()
                    .iter()
                    .map(|id| (id * 4096, 0xff0)),
            )
            .collect(),
    };
    let mut offset = pocket.offset + slot * 4 + 2;
    for (physical, length) in spans {
        if offset < length {
            assert!(offset + 2 <= length);
            let encoded = if pocket.encrypted {
                u32(&save.data, save.sections[0] + save.layout.key).unwrap() as u16
            } else {
                0
            };
            put16(&mut save.data, physical + offset, encoded);
            return;
        }
        offset -= length;
    }
    panic!("invalid test bag offset");
}

#[test]
fn resource_guards_follow_native_boolean_comparisons_aliases_and_noops() {
    const ROOT: usize = 0x27000;
    for p in profile::PROFILES {
        let mut r = adapter_rom(p);
        let rules = p.resource_checks.unwrap();
        {
            let b = std::sync::Arc::make_mut(&mut r.data);
            put32(
                b,
                rules.commands + 0x47 * 4,
                0x08000001 + rules.item_code as u32,
            );
            put32(
                b,
                rules.commands + 0x92 * 4,
                0x08000001 + rules.money_code as u32,
            );
        }
        for rhs in [0u16, 1, 2, 65535] {
            for comparison in 0..6u8 {
                // Check a quantity obtained from a script variable, copy RESULT,
                // and ignore a money check: the saved Boolean predicate must survive.
                let mut code = vec![
                    0x16,
                    5,
                    0x80,
                    1,
                    1,
                    0x47,
                    1,
                    0,
                    5,
                    0x80,
                    0x19,
                    4,
                    0x80,
                    0x0d,
                    0x80,
                    0x92,
                    0,
                    0,
                    0,
                    1,
                    1,
                    0x21,
                    4,
                    0x80,
                    rhs as u8,
                    (rhs >> 8) as u8,
                    6,
                    comparison,
                ];
                let target = ROOT + code.len() + 4 + 1;
                code.extend_from_slice(&(0x08000000 + target as u32).to_le_bytes());
                code.push(2);
                code.extend_from_slice(&[0x16, 0, 0x80, 2, 0, 0x16, 1, 0x80, 1, 0, 9, 0, 2]);
                let b = std::sync::Arc::make_mut(&mut r.data);
                b[ROOT..ROOT + code.len()].copy_from_slice(&code);
                let (rewards, stops) = r.item_script(ROOT).unwrap();
                assert!(stops.is_empty(), "{} {rhs} {comparison}", p.id);
                let predicate = |v: u16| match comparison {
                    0 => v < rhs,
                    1 => v == rhs,
                    2 => v > rhs,
                    3 => v <= rhs,
                    4 => v >= rhs,
                    _ => v != rhs,
                };
                if !predicate(0) && !predicate(1) {
                    assert!(rewards.is_empty());
                } else {
                    assert_eq!(rewards.len(), 1);
                    let guards = &rewards[0].conditions;
                    if predicate(0) == predicate(1) {
                        assert!(guards.is_empty());
                    } else {
                        assert_eq!(guards.len(), 1);
                        assert_eq!(guards[0].kind, "bag_item");
                        assert_eq!(guards[0].id, 1);
                        assert_eq!(guards[0].value, if rules.quantity_u8 { 1 } else { 257 });
                        assert_eq!(guards[0].taken, predicate(1));
                    }
                    assert!(
                        rewards[0].receipt.is_none(),
                        "New resource guards do not certify receipts"
                    );
                }
            }
        }
        // A full 32-bit money amount is not truncated to an event-variable word.
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[ROOT..ROOT + 25].copy_from_slice(&[
            0x92, 0x70, 0x11, 1, 0, 0, 0x21, 0x0d, 0x80, 0, 0, 6, 1, 0x18, 0x70, 2, 8, 0x16, 0,
            0x80, 2, 0, 9, 0, 2,
        ]);
        let guards = &r.item_script(ROOT).unwrap().0[0].conditions;
        assert_eq!(
            (guards[0].kind, guards[0].value, guards[0].taken),
            ("money", 70000, true)
        );
        // A resource mutation prevents using the initial SAV for a later check.
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[ROOT] = 0x90; // givemoney; same operand width as checkmoney
        b[ROOT + 6..ROOT + 24].copy_from_slice(&[
            0x92, 0x70, 0x11, 1, 0, 0, 0x21, 0x0d, 0x80, 0, 0, 6, 1, 0x25, 0x70, 2, 8, 2,
        ]);
        b[ROOT + 19..ROOT + 23].copy_from_slice(&0x08027024u32.to_le_bytes());
        b[ROOT + 23..ROOT + 36]
            .copy_from_slice(&[0x16, 0, 0x80, 2, 0, 0x16, 1, 0x80, 1, 0, 9, 0, 2]);
        b[ROOT + 36] = 2;
        assert_eq!(
            r.item_script(ROOT).unwrap().0[0].conditions[0].kind,
            "money_runtime"
        );
        // Width-decoded presentation commands have not been qualified as resource-pure.
        // A literal check remains visible but cannot use the initial wallet after one.
        let mut code = vec![
            0x6a, 0x92, 0x70, 0x11, 1, 0, 0, 0x21, 0x0d, 0x80, 0, 0, 6, 1,
        ];
        let target = ROOT + code.len() + 4 + 1;
        code.extend_from_slice(&(0x08000000 + target as u32).to_le_bytes());
        code.push(2);
        code.extend_from_slice(&[0x16, 0, 0x80, 2, 0, 0x16, 1, 0x80, 1, 0, 9, 0, 2]);
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[ROOT..ROOT + code.len()].copy_from_slice(&code);
        assert_eq!(
            r.item_script(ROOT).unwrap().0[0].conditions[0].kind,
            "money_runtime"
        );
        // Previously assigned variable operands and copied Boolean results cannot
        // cross this unqualified effect and become apparently verified guards.
        let code = [0x16, 5, 0x80, 3, 0, 0x6a, 0x47, 1, 0, 5, 0x80, 2];
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[ROOT..ROOT + code.len()].copy_from_slice(&code);
        assert_eq!(r.item_script(ROOT).unwrap().1, vec![ROOT + 6]);
        let mut code = vec![
            0x47, 1, 0, 3, 0, 0x19, 4, 0x80, 0x0d, 0x80, 0x6a, 0x21, 4, 0x80, 0, 0, 6, 1,
        ];
        let target = ROOT + code.len() + 4 + 1;
        code.extend_from_slice(&(0x08000000 + target as u32).to_le_bytes());
        code.push(2);
        code.extend_from_slice(&[0x16, 0, 0x80, 2, 0, 0x16, 1, 0x80, 1, 0, 9, 0, 2]);
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[ROOT..ROOT + code.len()].copy_from_slice(&code);
        let (rewards, _) = r.item_script(ROOT).unwrap();
        assert_eq!(rewards.len(), 1);
        assert!(rewards[0]
            .conditions
            .iter()
            .all(|c| !matches!(c.kind, "bag_item" | "bag_item_runtime")));
        // Unresolved quantities/native dispatch changes cannot become holdings guards.
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[ROOT..ROOT + 12].copy_from_slice(&[0x47, 1, 0, 5, 0x80, 0x16, 0, 0x80, 2, 0, 9, 0]);
        b[ROOT + 12] = 2;
        assert_eq!(r.item_script(ROOT).unwrap().1, vec![ROOT]);
        let b = std::sync::Arc::make_mut(&mut r.data);
        put32(
            b,
            rules.commands + 0x47 * 4,
            0x08000001 + rules.item_code as u32 + 4,
        );
        assert_eq!(
            r.script_resource_check(ROOT, Some, false, false)
                .unwrap_err()
                .code,
            "resource_dispatch"
        );
    }
}

#[test]
fn resource_conditions_query_item_links_without_mutating_or_claiming_receipt() {
    use crate::{
        acquisition::{AcquisitionIndex, Target, TargetKind},
        event_state::EventSnapshot,
    };
    for p in profile::PROFILES {
        let (mut r, map) = npc_trade_fixture(p);
        let rules = p.resource_checks.unwrap();
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[p.items.offset + p.items.stride + 26] = 1;
        put32(
            b,
            rules.commands + 0x47 * 4,
            0x08000001 + rules.item_code as u32,
        );
        b[0x26000..0x26013].copy_from_slice(&[
            0x47, 1, 0, 3, 0, 0x21, 0x0d, 0x80, 0, 0, 6, 1, 0x12, 0x60, 2, 8, 2, 0, 2,
        ]);
        b[0x26010] = 0x16;
        b[0x26010..0x2601d].copy_from_slice(&[0x16, 0, 0x80, 2, 0, 0x16, 1, 0x80, 1, 0, 9, 0, 2]);
        // Reward lies on the RESULT!=0 path (fallthrough).
        b[0x2600c..0x26010].copy_from_slice(&0x0802601eu32.to_le_bytes());
        b[0x2601e] = 2;
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let pocket = r
            .profile
            .save
            .pockets
            .iter()
            .find(|p| p.category == 1)
            .unwrap()
            .id;
        save.edit_bag(pocket, 0, 1, 1, &r, Policy::Free).unwrap();
        save.edit_bag(pocket, 1, 1, 2, &r, Policy::Free).unwrap();
        save.edit_bag("pc", 0, 1, 99, &r, Policy::Free).unwrap();
        let initial = save.data.clone();
        let state = EventSnapshot::new(&save, p.event_state.unwrap());
        assert_eq!(
            state.normal_bag_item(&r, 1),
            Some((if rules.alternate_bag { 3 } else { 1 }, true))
        );
        let index = AcquisitionIndex {
            wild_cache: std::cell::RefCell::default(),
            breeding_cache: Default::default(),
            world: crate::world::World {
                maps: vec![map.clone()],
                map_events: vec![r.map_events(&map).unwrap()],
                encounters: vec![],
                trainers: vec![],
                trainer_locations: crate::world::TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: vec![],
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let target = Target {
            kind: TargetKind::Item,
            id: 2,
        };
        let report = index.query(&r, Some(&save), target.clone()).unwrap();
        let source = &report.sources[0];
        assert!(source
            .related
            .iter()
            .any(|r| r.kind == TargetKind::Item && r.id == 1));
        let c = &source.conditions[0];
        if rules.alternate_bag {
            assert_eq!(c.actual, Some(3));
            assert_eq!(c.satisfied, None);
            assert_eq!(c.unresolved, Some("alternate_bag_unresolved"));
            assert_eq!(source.status, "unknown");
        } else {
            assert_eq!(c.actual, Some(1));
            assert_eq!(c.satisfied, Some(false));
            assert_eq!(source.status, "blocked");
        }
        assert_eq!(source.receipt_flag, None);
        assert_eq!(
            index.query(&r, None, target).unwrap().sources[0].conditions[0].actual,
            None
        );
        assert_eq!(save.data, initial);
        // A native zero check still needs a matching record, not just count>=0.
        save.edit_bag(pocket, 0, 1, 1, &r, Policy::Free).unwrap();
        fixture_zero_bag_quantity(&mut save, pocket, 0);
        save.edit_bag(pocket, 1, 0, 0, &r, Policy::Free).unwrap();
        assert_eq!(
            EventSnapshot::new(&save, p.event_state.unwrap()).normal_bag_item(&r, 1),
            Some((0, true))
        );
        save.edit_bag(pocket, 0, 0, 0, &r, Policy::Free).unwrap();
        // A malformed/free-edit entry in the wrong pocket must not count either.
        let wrong = p
            .save
            .pockets
            .iter()
            .find(|p| p.category != 0 && p.category != 1)
            .unwrap()
            .id;
        save.edit_bag(wrong, 0, 1, 99, &r, Policy::Free).unwrap();
        assert_eq!(
            EventSnapshot::new(&save, p.event_state.unwrap()).normal_bag_item(&r, 1),
            Some((0, false))
        );
    }
}

#[test]
fn resource_pocket_uses_native_sanitization_but_searches_original_item_id() {
    let mut r = adapter_rom(crate::ultimate::PROFILE);
    let rules = r.profile.resource_checks.unwrap();
    let b = std::sync::Arc::make_mut(&mut r.data);
    b[r.profile.items.offset + 26] = 2;
    b[r.profile.items.offset + 44 + 26] = 1;
    let address = (rules.item_sanitizer - 0x08000000) as usize;
    // Simulate a sanitized ID different from the requested record.
    b[address..address + 4].copy_from_slice(&[0, 0x20, 0x70, 0x47]);
    assert_eq!(r.resource_item_pocket(1).unwrap(), 2);
    let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let ordinary = r
        .profile
        .save
        .pockets
        .iter()
        .find(|p| p.category == 1)
        .unwrap()
        .id;
    let effective = r
        .profile
        .save
        .pockets
        .iter()
        .find(|p| p.category == 2)
        .unwrap()
        .id;
    save.edit_bag(ordinary, 0, 1, 10, &r, Policy::Free).unwrap();
    save.edit_bag(effective, 0, 1, 3, &r, Policy::Free).unwrap();
    let state = crate::event_state::EventSnapshot::new(&save, r.profile.event_state.unwrap());
    assert_eq!(state.normal_bag_item(&r, 1), Some((3, true)));
    // Category sanitization does not change the ID matched by the native bag loop.
    assert_eq!(state.normal_bag_item(&r, 2), Some((0, false)));
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_RESOURCE_PROBES independent native vectors"]
fn local_resource_guards_match_native_width_inventory_and_money() {
    use crate::event_state::EventSnapshot;
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_RESOURCE_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for name in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{name}")).unwrap()).unwrap())
                .unwrap();
        let report = &probes[name];
        assert_eq!(report["md5"], r.profile.md5);
        for (item, category) in report["categories"].as_array().unwrap().iter().enumerate() {
            assert_eq!(
                r.resource_item_pocket(item as u16).unwrap() as u64,
                category.as_u64().unwrap(),
                "{name} item {item}"
            );
        }
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let item = report["item"].as_u64().unwrap() as u16;
        let pocket = r
            .profile
            .save
            .pockets
            .iter()
            .find(|p| p.category == r.item(item).unwrap().pocket)
            .unwrap()
            .id;
        for vector in report["vectors"].as_array().unwrap() {
            if vector["kind"] == "bag_item" {
                for index in 0..3 {
                    save.edit_bag(pocket, index, 0, 0, &r, Policy::Free)
                        .unwrap();
                }
                for (index, quantity) in vector["quantities"].as_array().unwrap().iter().enumerate()
                {
                    save.edit_bag(
                        pocket,
                        index,
                        item,
                        (quantity.as_u64().unwrap() as u16).max(1),
                        &r,
                        Policy::Free,
                    )
                    .unwrap();
                    if quantity == &serde_json::json!(0) {
                        fixture_zero_bag_quantity(&mut save, pocket, index);
                    }
                }
                let before = save.data.clone();
                let state = EventSnapshot::new(&save, r.profile.event_state.unwrap());
                let (count, present) = state.normal_bag_item(&r, item).unwrap();
                let quantity = vector["requested"].as_u64().unwrap() as u16;
                let mut fixture = r.clone();
                let pc = 0x27000;
                let b = std::sync::Arc::make_mut(&mut fixture.data);
                b[pc..pc + 5].copy_from_slice(&[0x47, item as u8, (item >> 8) as u8, 5, 0x80]);
                let guard = fixture
                    .script_resource_check(
                        pc,
                        |v| if v == 0x8005 { Some(quantity) } else { Some(v) },
                        false,
                        false,
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(guard.value as u64, vector["value"].as_u64().unwrap());
                assert_eq!(
                    u8::from(present && count >= guard.value) as u64,
                    vector["result"].as_u64().unwrap(),
                    "{name}: {vector}"
                );
                assert_eq!(save.data, before);
            } else {
                // Synthetic encrypted wallet bytes; no external save is opened or written.
                let key = 0xdead4321;
                put32(&mut save.data, save.sections[0] + save.layout.key, key);
                let wallet = vector["wallet"].as_u64().unwrap() as u32;
                put32(
                    &mut save.data,
                    save.sections[1] + save.layout.money,
                    wallet ^ key,
                );
                let state = EventSnapshot::new(&save, r.profile.event_state.unwrap());
                assert_eq!(state.money(), Some(wallet));
                let required = vector["value"].as_u64().unwrap() as u32;
                let expected = if vector["ignore"] != 0 {
                    7
                } else {
                    u32::from(wallet >= required)
                };
                assert_eq!(expected as u64, vector["result"].as_u64().unwrap());
            }
        }
        // Bind independent native compare/copy/goto outcomes to Rust's
        // symbolic resource guards, rather than checking a copied Boolean formula.
        let mut fixture = r.clone();
        for vector in report["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["boolean"] == 1)
        {
            let rhs = vector["rhs"].as_u64().unwrap() as u16;
            let comparison = vector["comparison"].as_u64().unwrap() as u8;
            let zero = report["comparisons"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| {
                    v["boolean"] == 0
                        && v["rhs"] == vector["rhs"]
                        && v["comparison"] == vector["comparison"]
                })
                .unwrap();
            let on_zero = zero["taken"].as_bool().unwrap();
            let on_one = vector["taken"].as_bool().unwrap();
            let pc = 0x27000;
            let mut code = vec![
                0x92,
                1,
                0,
                0,
                0,
                0,
                0x19,
                4,
                0x80,
                0x0d,
                0x80,
                0x21,
                4,
                0x80,
                rhs as u8,
                (rhs >> 8) as u8,
                6,
                comparison,
            ];
            let target = pc + code.len() + 5;
            code.extend_from_slice(&(0x08000000 + target as u32).to_le_bytes());
            code.push(2);
            code.extend_from_slice(&[0x16, 0, 0x80, 2, 0, 0x16, 1, 0x80, 1, 0, 9, 0, 2]);
            let b = std::sync::Arc::make_mut(&mut fixture.data);
            b[pc..pc + code.len()].copy_from_slice(&code);
            let (rewards, stops) = fixture.item_script(pc).unwrap();
            assert!(stops.is_empty());
            if !on_zero && !on_one {
                assert!(rewards.is_empty());
            } else if on_zero == on_one {
                assert!(rewards[0].conditions.is_empty());
            } else {
                let c = &rewards[0].conditions[0];
                assert_eq!((c.kind, c.value, c.taken), ("money", 1, on_one));
            }
        }
        println!(
            "{name}: {} native holdings vectors",
            report["vectors"].as_array().unwrap().len()
        );
    }
}

#[test]
fn held_sources_follow_random_references_time_and_preserve_unreferenced_uncertainty() {
    use crate::{
        acquisition::{AcquisitionIndex, Target, TargetKind},
        world::{Encounter, TrainerLocationIndex, World},
    };
    for p in profile::PROFILES {
        let (mut rom, map) = npc_trade_fixture(p);
        rom.profile.wild_items = None; // Synthetic tables contain no native assignment code.
        let tail = if p.formats.species == crate::adapter::SpeciesFormat::Expanded36 {
            2
        } else {
            0
        };
        let b = std::sync::Arc::make_mut(&mut rom.data);
        put16(
            b,
            rom.profile.base_stats.offset + rom.profile.base_stats.stride + 12 + tail,
            1,
        );
        put16(
            b,
            rom.profile.base_stats.offset + 2 * rom.profile.base_stats.stride + 12 + tail,
            1,
        );
        let base = Encounter {
            selector: None,
            periods: vec!["base"],
            species: 1,
            map_id: map.id.clone(),
            map_name: map.name.clone(),
            region: map.region,
            method: "grass".into(),
            min_level: 10,
            max_level: 12,
            weight: Some(20),
            encounter_rate: Some(30),
            slot: Some(0),
            offset: 123,
            conditional: true,
        };
        let mut water = base.clone();
        water.method = "surf".into();
        water.offset = 124;
        let mut night = base.clone();
        night.species = 3;
        night.periods = vec!["night"];
        night.offset = 125;
        let mut fixed = base.clone();
        fixed.species = 2;
        fixed.method = "static".into();
        fixed.offset = 126;
        let index = AcquisitionIndex {
            wild_cache: Default::default(),
            breeding_cache: Default::default(),
            world: World {
                maps: vec![map],
                map_events: vec![],
                encounters: vec![base, water, night, fixed],
                trainers: vec![],
                trainer_locations: TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: (1..=3).map(|id| rom.species(id).unwrap()).collect(),
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let save = Save::open(save_bytes(&rom), rom.profile.save).unwrap();
        let before = save.data.clone();
        let data = rom.data.clone();
        let mut report = index
            .query(
                &rom,
                Some(&save),
                Target {
                    kind: TargetKind::Item,
                    id: 1,
                },
            )
            .unwrap();
        assert_eq!(report.sources.len(), 3);
        let land = &report.sources[0];
        assert_eq!(land.kind, "wild_held");
        assert_eq!(land.map_id.as_deref(), Some("0-0"));
        assert_eq!(land.encounter_method.as_deref(), Some("grass"));
        assert_eq!(land.encounter_percent, Some(20));
        assert_eq!((land.min_level, land.max_level), (Some(10), Some(12)));
        assert_eq!(land.held_percent, None);
        assert!(land.held_issue.is_some());
        assert_eq!(land.status, "unknown");
        assert_eq!(report.sources[2].kind, "wild_held_unreferenced");
        assert!(report.sources[2].map_id.is_none());
        assert_eq!(report.sources[2].repeatable, None);
        assert!(report.sources[2].partial);
        index.mark_period(&mut report.sources, Some("night"));
        assert_eq!(report.sources[0].in_scenario, Some(false));
        assert_eq!(report.sources[1].in_scenario, Some(true));
        assert!(index
            .query(
                &rom,
                None,
                Target {
                    kind: TargetKind::Item,
                    id: 0
                }
            )
            .unwrap()
            .sources
            .is_empty());
        assert_eq!(save.data, before);
        assert!(std::sync::Arc::ptr_eq(&rom.data, &data));
    }
}

#[test]
fn daycare_queries_preserve_guards_tiles_and_reject_changed_dispatch() {
    for profile in profile::PROFILES {
        let (mut r, map) = npc_trade_fixture(profile);
        let rules = profile.breeding.unwrap();
        r.profile.breeding = Some(rules);
        let b = std::sync::Arc::make_mut(&mut r.data);
        put32(
            b,
            rules.specials + rules.receive_special as usize * 4,
            0x08000001 + rules.receive_code as u32,
        );
        b[0x26000..0x2600e].copy_from_slice(&[
            0x2b,
            0x23,
            0x01,
            0x06,
            0x01,
            0x0a,
            0x60,
            0x02,
            0x08,
            0x02,
            0x25,
            rules.receive_special as u8,
            0,
            0x02,
        ]);
        let report = r.map_events(&map).unwrap();
        assert_eq!(report.markers.len(), 1);
        let marker = &report.markers[0];
        assert_eq!((marker.x, marker.y), (3, 2));
        assert_eq!(marker.daycare.len(), 1);
        assert_eq!(marker.kind, "npc");
        assert!(marker.rewards.is_empty() && marker.receipt_flag.is_none());
        let condition = &marker.daycare[0].conditions[0];
        assert_eq!(
            (condition.kind, condition.id, condition.taken),
            ("flag", 0x123, true)
        );
        assert!(report.unplaced_daycare.is_empty());
        let index = crate::acquisition::AcquisitionIndex {
            wild_cache: std::cell::RefCell::default(),
            breeding_cache: Default::default(),
            world: crate::world::World {
                maps: vec![map.clone()],
                map_events: vec![report],
                encounters: vec![],
                trainers: vec![],
                trainer_locations: crate::world::TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: vec![],
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let before = r.data.clone();
        let sources = index.daycare_sources(&r, None);
        assert_eq!((sources[0].x, sources[0].y), (Some(3), Some(2)));
        assert_eq!(sources[0].conditions[0].satisfied, None);
        assert_eq!(sources[0].status, "unknown");
        assert_eq!(*before, *r.data);
        put32(
            std::sync::Arc::make_mut(&mut r.data).as_mut_slice(),
            rules.specials + rules.receive_special as usize * 4,
            0x08026001,
        );
        let invalid = r.map_events(&map).unwrap();
        assert!(invalid.markers[0].daycare.is_empty());
        assert!(invalid.markers[0].stopped_at.contains(&0x2600a));
    }
}

#[test]
fn stored_breeding_queries_leave_save_and_parent_history_unchanged() {
    for p in profile::PROFILES {
        let mut r = adapter_rom(p);
        let mut rules = p.breeding.unwrap();
        rules.compatibility = 0x08021000;
        std::sync::Arc::make_mut(&mut r.data)[0x21000..0x21004]
            .copy_from_slice(&[0, 0x20, 0x70, 0x47]); // MOVS r0,#0; BX lr
        r.profile.breeding = Some(crate::breeding::BreedingRules {
            production: None,
            ..rules
        });
        let save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let request: crate::breeding::Request = serde_json::from_value(serde_json::json!({"parents":[
            {"kind":"stored","location":{"kind":"party","slot":0}},
            {"kind":"simulated","species":1,"gender":"female","held_item":0,"trainer_id":2}],"offspring_pid":24,"seed":42})).unwrap();
        let before = save.data.clone();
        let rom_before = r.data.clone();
        let preview = crate::breeding::preview(&r, Some(&save), &request).unwrap();
        assert!(preview.child.is_none());
        assert_eq!(preview.parents[0].pid, 12345);
        assert_eq!(save.data, before);
        assert_eq!(*r.data, *rom_before);
    }
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_PRODUCTION_PROBES independent mGBA evidence"]
fn local_breeding_production_matches_native_rolls_bag_and_steps() {
    use crate::breeding::{Gender, Parent, Request};
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_PRODUCTION_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let rom =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        let native = &probes[key];
        assert_eq!(native["md5"], rom.profile.md5);
        assert_eq!(native["engine"], "mGBA ARM7");
        let production = rom.profile.breeding.unwrap().production.unwrap();
        let (scale, divisor) =
            crate::breeding_production::roll_parameters(&rom, production).unwrap();
        let rolls = native["rolls"].as_array().unwrap();
        assert_eq!(rolls.len(), 65536);
        for (draw, roll) in rolls.iter().enumerate() {
            assert_eq!(
                draw as u32 * scale / divisor,
                roll.as_u64().unwrap() as u32,
                "{key}: {draw}"
            );
        }
        let rom_before = rom.data.clone();
        for row in native["rows"].as_array().unwrap() {
            let parents: [Vec<u8>; 2] = serde_json::from_value(row["parents"].clone()).unwrap();
            let mut bytes = save_bytes(&rom);
            let security_key = row["key"].as_u64().unwrap() as u32;
            for bank in 0..2 {
                let base = bank * 14 * 4096 + 10 * 4096;
                put32(&mut bytes, base + rom.profile.save.key, security_key);
                let checksum = if rom.profile.save.sector_checksum
                    == profile::SectorChecksum::NativeConstantOne
                {
                    1
                } else {
                    sector_checksum(&bytes[base..base + rom.profile.save.sizes[0]])
                };
                put16(&mut bytes, base + 0xff6, checksum);
            }
            let mut save = Save::open(bytes, rom.profile.save).unwrap();
            let item = native["modifier_item"].as_u64().map(|i| i as u16);
            let present = row["modifier_present"].as_bool().unwrap();
            if let Some(item) = item {
                let pocket = rom
                    .profile
                    .save
                    .pockets
                    .iter()
                    .find(|p| p.category == rom.item(item).unwrap().pocket)
                    .unwrap();
                save.edit_bag(
                    pocket.id,
                    0,
                    if present { item } else { 0 },
                    u16::from(present),
                    &rom,
                    Policy::Free,
                )
                .unwrap();
            }
            let before = save.data.clone();
            let mut request = Request {
                parents: [
                    Parent::Simulated {
                        species: 25,
                        gender: Gender::Female,
                        held_item: 0,
                        trainer_id: 1,
                    },
                    Parent::Simulated {
                        species: 25,
                        gender: Gender::Male,
                        held_item: 0,
                        trainer_id: 2,
                    },
                ],
                seed: row["seed"].as_u64().unwrap() as u32,
                offspring_pid: 24,
                production_item: None,
            };
            let expected = |result: crate::breeding_production::Preview| {
                assert_eq!(result.modifier_item, item);
                assert_eq!(result.modifier_present, item.map(|_| present));
                assert_eq!(
                    result.threshold as u64,
                    row["threshold"].as_u64().unwrap(),
                    "{key}"
                );
                assert_eq!(
                    result.numerator as u64,
                    row["numerator"].as_u64().unwrap(),
                    "{key}"
                );
                assert_eq!(result.scenario_roll as u64, row["roll"].as_u64().unwrap());
                assert_eq!(result.scenario_passed, row["passed"].as_bool().unwrap());
                assert_eq!(result.denominator, 65536);
                assert_eq!(result.interval_steps, 256);
            };
            expected(
                crate::breeding_production::preview(&rom, Some(&save), &request, &parents)
                    .unwrap()
                    .unwrap(),
            );
            if item.is_some() {
                request.production_item = Some(present);
                expected(
                    crate::breeding_production::preview(&rom, None, &request, &parents)
                        .unwrap()
                        .unwrap(),
                );
            }
            assert_eq!(save.data, before);
        }
        assert_eq!(*rom.data, *rom_before);
        eprintln!(
            "{key}: all 65536 native draws and {} keyed SAV/override scenarios",
            native["rows"].as_array().unwrap().len()
        );
    }
}

#[test]
fn collection_preparation_uses_directed_edges_current_individuals_and_keeps_saves_unchanged() {
    use crate::{
        acquisition::AcquisitionIndex,
        collection::{CollectionBasis, CollectionRequest},
        rom::Evolution,
    };
    for profile in profile::PROFILES {
        let (r, map) = npc_trade_fixture(profile);
        let report = r.map_events(&map).unwrap();
        let mut index = AcquisitionIndex {
            wild_cache: Default::default(),
            breeding_cache: Default::default(),
            world: crate::world::World {
                maps: vec![map],
                map_events: vec![report],
                encounters: vec![],
                trainers: vec![],
                trainer_locations: crate::world::TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: (1..=3).map(|id| r.valid_species(id).unwrap()).collect(),
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let edge = |target, condition, parameter, auxiliary, offset| Evolution {
            method: 4,
            condition,
            parameter,
            auxiliary,
            target,
            offset,
            requirements: vec![],
        };
        index
            .evolutions
            .insert(1, vec![edge(2, "level", 20, 0, 100)]);
        index
            .evolutions
            .insert(2, vec![edge(3, "item_hold_item", 1, 2, 200)]);
        let mut save = Save::open(save_bytes(&r), profile.save).unwrap();
        let request = || CollectionRequest {
            basis: CollectionBasis::Individuals,
            families: false,
            include_unknown_rewards: false,
        };
        let before = save.data.clone();
        let plan = index.collection(&r, &save, request()).unwrap();
        let task = plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .find(|t| t.target.kind == crate::acquisition::TargetKind::Species && t.target.id == 3)
            .unwrap();
        let p = task.preparation.as_ref().unwrap();
        assert_eq!(
            (p.origin, p.current_count, p.needs_hatching, p.truncated),
            (1, 1, false, false)
        );
        assert!(p.source.is_none());
        assert_eq!(
            p.steps
                .iter()
                .map(|s| (s.from, s.evolution.target))
                .collect::<Vec<_>>(),
            [(1, 2), (2, 3)]
        );
        assert_eq!(
            p.steps[1].related.iter().map(|t| t.id).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            task.source.as_ref().unwrap().status,
            "unknown",
            "possession must not certify evolution eligibility"
        );
        assert_eq!(save.data, before);
        // Historical Dex records cannot supply a presently usable parent.
        save.edit(
            party(),
            &PokemonPatch {
                species: Some(3),
                ..Default::default()
            },
            &r,
            Policy::Free,
        )
        .unwrap();
        if profile.save.dex.is_some() {
            save.edit_dex(1, true, true).unwrap();
            for (i, s) in index.species.iter_mut().enumerate() {
                s.dex_number = i as u16 + 1;
            }
            let plan = index
                .collection(
                    &r,
                    &save,
                    CollectionRequest {
                        basis: CollectionBasis::Dex,
                        ..request()
                    },
                )
                .unwrap();
            assert!(plan
                .regions
                .iter()
                .flat_map(|r| &r.tasks)
                .filter_map(|t| t.preparation.as_ref())
                .all(|p| p.current_count == 0 || p.origin == 3));
        }
        let before = save.data.clone();
        let plan = index.collection(&r, &save, request()).unwrap();
        let first = plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .find(|t| t.target.kind == crate::acquisition::TargetKind::Species && t.target.id == 1)
            .unwrap();
        assert!(
            first.preparation.is_none(),
            "owned final stage cannot be treated as a reverse-evolution parent"
        );
        let middle = plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .find(|t| t.target.kind == crate::acquisition::TargetKind::Species && t.target.id == 2)
            .unwrap();
        assert!(
            middle.preparation.is_none(),
            "an unreferenced origin is not an obtainable source"
        );
        assert_eq!(save.data, before);
        // Cycles terminate; eggs are excluded from the actual-individual inventory.
        index
            .evolutions
            .insert(3, vec![edge(1, "unknown", 0, 0, 300)]);
        save.edit(
            party(),
            &PokemonPatch {
                egg: Some(true),
                ..Default::default()
            },
            &r,
            Policy::Free,
        )
        .unwrap();
        let before = save.data.clone();
        let plan = index.collection(&r, &save, request()).unwrap();
        assert!(plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .filter_map(|t| t.preparation.as_ref())
            .all(|p| p.current_count == 0));
        assert_eq!(save.data, before);
    }
}

#[test]
#[ignore = "requires all five exact private ROMs; creates only synthetic readonly SAV fixtures"]
fn local_collection_preparation_all_fingerprints() {
    use crate::{
        acquisition::AcquisitionIndex,
        collection::{CollectionBasis, CollectionRequest},
    };
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        let index = AcquisitionIndex::build(&r).unwrap();
        let save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let before = save.data.clone();
        let rom_before = r.data.clone();
        let plan = index
            .collection(
                &r,
                &save,
                CollectionRequest {
                    basis: CollectionBasis::Individuals,
                    families: false,
                    include_unknown_rewards: false,
                },
            )
            .unwrap();
        let current = save.all(&r).unwrap();
        let mut chains = 0;
        let mut possessed = 0;
        for task in plan.regions.iter().flat_map(|r| &r.tasks) {
            if let Some(p) = &task.preparation {
                chains += 1;
                possessed += usize::from(p.current_count > 0);
                let mut from = p.origin;
                assert!(
                    (!p.steps.is_empty() || p.breeding.is_some())
                        && p.steps.len() <= 8
                        && p.partial
                );
                let mut visited = std::collections::BTreeSet::from([from]);
                for step in &p.steps {
                    assert_eq!(step.from, from);
                    let raw = r.evolutions(from).unwrap();
                    assert!(
                        raw.iter().any(|e| serde_json::to_value(e).unwrap()
                            == serde_json::to_value(&step.evolution).unwrap()),
                        "{key}: evolution must match current ROM"
                    );
                    assert_eq!(
                        serde_json::to_value(&step.related).unwrap(),
                        serde_json::to_value(crate::acquisition::evolution_targets(
                            &step.evolution
                        ))
                        .unwrap()
                    );
                    from = step.evolution.target;
                    assert!(
                        visited.insert(from),
                        "cycle must not become a proposed route"
                    );
                }
                assert_eq!(from, task.target.id);
                assert_eq!(
                    p.current_count,
                    current
                        .iter()
                        .filter(|v| v.pokemon.species == p.origin
                            && !v.pokemon.egg
                            && v.pokemon.checksum_ok)
                        .count()
                );
                if let Some(source) = &p.source {
                    let id = source.map_id.as_ref().unwrap();
                    assert!(plan.entrances.iter().any(|e| &e.map_id == id));
                    if p.breeding.is_some() {
                        assert!(index
                            .daycare_sources(&r, Some(&save))
                            .iter()
                            .any(|s| s.offset == source.offset && s.map_id == source.map_id));
                    } else {
                        assert!(index
                            .query(
                                &r,
                                Some(&save),
                                crate::acquisition::Target {
                                    kind: crate::acquisition::TargetKind::Species,
                                    id: p.origin
                                }
                            )
                            .unwrap()
                            .sources
                            .iter()
                            .any(|s| s.offset == source.offset
                                && s.map_id == source.map_id
                                && s.kind == source.kind));
                    }
                }
            }
        }
        assert!(
            chains > 0 && possessed > 0,
            "{key}: exercise real ROM edges and current origin"
        );
        assert_eq!(save.data, before);
        assert_eq!(*r.data, *rom_before);
        eprintln!("{key}: {chains} directed preparation chains ({possessed} from current individuals), {} goals; ROM/SAV unchanged",plan.missing_count);
    }
}

fn synthetic_daycare_main(save: &mut Save, main: &[u8]) {
    let mut cursor = 0;
    for id in 1..=4 {
        let section = save.sections[id];
        let length = save.layout.sizes[id];
        save.data[section..section + length].copy_from_slice(&main[cursor..cursor + length]);
        let checksum = if save.layout.sector_checksum == profile::SectorChecksum::NativeConstantOne
        {
            1
        } else {
            sector_checksum(&save.data[section..section + length])
        };
        put16(&mut save.data, section + 0xff6, checksum);
        cursor += length;
    }
    assert_eq!(cursor, main.len());
}

#[test]
fn saved_daycare_is_optional_and_deposited_parent_requests_are_bounded() {
    let mut r = rom();
    let save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let before = save.data.clone();
    assert!(crate::daycare_state::snapshot(&r, &save).unwrap().is_none());
    r.profile.breeding = Some(crate::breeding::EMERALD);
    assert_eq!(
        crate::daycare_state::parent_raw(&r, &save, 2)
            .unwrap_err()
            .code,
        "daycare_parent_slot"
    );
    let mut request:crate::breeding::Request=serde_json::from_value(serde_json::json!({"parents":[{"kind":"deposited","slot":0},{"kind":"deposited","slot":0}],"offspring_pid":24})).unwrap();
    assert_eq!(
        crate::breeding::preview(&r, Some(&save), &request)
            .unwrap_err()
            .code,
        "breeding_parent"
    );
    request.parents[1] = crate::breeding::Parent::Simulated {
        species: 1,
        gender: crate::breeding::Gender::Male,
        held_item: 0,
        trainer_id: 1,
    };
    assert_eq!(
        crate::breeding::preview(&r, None, &request)
            .unwrap_err()
            .code,
        "save_required"
    );
    assert!(serde_json::from_value::<crate::breeding::Request>(serde_json::json!({"parents":[{"kind":"deposited","slot":null},{"kind":"deposited","slot":1}],"offspring_pid":24})).is_err());
    assert_eq!(save.data, before);
}

#[test]
#[ignore = "requires five exact private ROMs and GEN3_DAYCARE_STATE_PROBES mGBA native vectors"]
fn local_saved_daycare_matches_native_state_and_deposited_parent_records() {
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_DAYCARE_STATE_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        let native = &probes[key];
        assert_eq!(native["engine"], "mGBA ARM7");
        assert_eq!(native["md5"], r.profile.md5);
        let breeding = r.profile.breeding.unwrap();
        let rules = breeding.saved.unwrap();
        let rom_before = r.data.clone();
        for row in native["rows"].as_array().unwrap() {
            let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
            let mut main = save.logical(1..=4);
            let raws: Vec<Vec<u8>> = serde_json::from_value(row["parents"].clone()).unwrap();
            let steps = row["steps"].as_u64().unwrap() as u32;
            let pending = row["pending"].as_u64().unwrap() as u32;
            for (slot, raw) in raws.iter().enumerate() {
                let at = breeding.daycare + slot * breeding.parent_stride;
                main[at..at + 80].copy_from_slice(raw);
                put32(&mut main, at + rules.parent_steps, steps);
            }
            if breeding.pending_width == 2 {
                put16(
                    &mut main,
                    breeding.daycare + breeding.pending_pid,
                    pending as u16,
                );
            } else {
                put32(&mut main, breeding.daycare + breeding.pending_pid, pending);
            }
            if let Some(flag) = row["flag"].as_u64() {
                let id = flag as u16;
                let range = r
                    .profile
                    .event_state
                    .unwrap()
                    .flags
                    .iter()
                    .find(|v| id >= v.first && id - v.first < v.count)
                    .unwrap();
                assert!(matches!(range.block, crate::event_state::EventBlock::Main));
                let bit = (id - range.first) as usize;
                main[range.offset + bit / 8] |=
                    u8::from(row["flag_set"].as_bool().unwrap()) << (bit % 8);
            }
            synthetic_daycare_main(&mut save, &main);
            save.validate(&r).unwrap();
            let before = save.data.clone();
            let state = crate::daycare_state::snapshot(&r, &save).unwrap().unwrap();
            assert_eq!(
                state.native_service_state,
                Some(row["native_state"].as_u64().unwrap() as u32)
            );
            assert_eq!(state.egg_available, row["available"] == 1);
            assert_eq!(state.legacy_pending_value, pending);
            assert_eq!(
                serde_json::to_value(state.compatibility).unwrap(),
                row["compatibility"]
            );
            for (slot, parent) in state.parents.iter().enumerate() {
                assert_eq!(parent.present, row["presence"][slot] == 1);
                assert!(parent.issue.is_none());
                assert_eq!(parent.accumulated_steps, steps);
                if parent.present {
                    assert_eq!(
                        crate::daycare_state::parent_raw(&r, &save, slot).unwrap(),
                        raws[slot]
                    );
                    assert_eq!(
                        serde_json::to_value(parent.pokemon.as_ref().unwrap()).unwrap(),
                        serde_json::to_value(pokemon::decode(&raws[slot], &r).unwrap()).unwrap()
                    );
                } else {
                    assert!(parent.pokemon.is_none());
                }
            }
            if state.status == "two_parents" && pending == 0 {
                let phase = native["phases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| p["steps"] == row["steps"])
                    .unwrap();
                assert_eq!(
                    state.next_check_steps,
                    Some(phase["next_check"].as_u64().unwrap() as u16)
                );
            } else {
                assert!(state.next_check_steps.is_none());
            }
            assert_eq!(save.data, before);
        }
        // The original deposited boxed records are used for preview, not a
        // reconstruction from decoded fields; this preserves PID/history/moves.
        let row = native["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| {
                row["presence"] == serde_json::json!([1, 1])
                    && row["pending"] == 0
                    && row["flag_set"] == false
            })
            .unwrap();
        let raws: Vec<Vec<u8>> = serde_json::from_value(row["parents"].clone()).unwrap();
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let mut main = save.logical(1..=4);
        for (i, raw) in raws.iter().enumerate() {
            main[breeding.daycare + i * breeding.parent_stride
                ..breeding.daycare + i * breeding.parent_stride + 80]
                .copy_from_slice(raw);
        }
        synthetic_daycare_main(&mut save, &main);
        let before = save.data.clone();
        let request = crate::breeding::Request {
            parents: [
                crate::breeding::Parent::Deposited { slot: 0 },
                crate::breeding::Parent::Deposited { slot: 1 },
            ],
            seed: 42,
            offspring_pid: 24,
            production_item: None,
        };
        let preview = crate::breeding::preview(&r, Some(&save), &request).unwrap();
        assert!(preview.child.as_ref().unwrap().egg);
        assert_eq!(
            preview.parents[0].pid,
            pokemon::decode(&raws[0], &r).unwrap().pid
        );
        assert_eq!(save.data, before);
        // An occupied corrupt record is never sent through the native repair path.
        if r.profile.save.pokemon_codec.plain_substructures() {
            // These engines ignore the legacy individual checksum. Corrupt the
            // actual species field, rather than inventing a checksum requirement.
            put16(&mut main, breeding.daycare + 32, u16::MAX);
        } else {
            main[breeding.daycare + 28] ^= 1;
        }
        synthetic_daycare_main(&mut save, &main);
        let before = save.data.clone();
        let state = crate::daycare_state::snapshot(&r, &save).unwrap().unwrap();
        assert_eq!(state.status, "unknown");
        assert!(state.parents[0].issue.is_some());
        assert!(state.native_service_state.is_none());
        assert!(state.compatibility.is_none());
        assert!(crate::daycare_state::parent_raw(&r, &save, 0).is_err());
        assert_eq!(save.data, before);
        assert_eq!(*r.data, *rom_before);
        eprintln!("{key}: {} native saved-state cases, 8 phases, deposited preview and corrupt-record safety passed",native["rows"].as_array().unwrap().len());
    }
}

#[test]
fn breeding_collection_has_no_unverified_fallback_or_fabricated_parents() {
    let r = rom();
    let save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let before = save.data.clone();
    assert!(crate::breeding_collection::suggestions(&r, &save)
        .unwrap()
        .is_none());
    assert_eq!(
        crate::breeding_collection::select(&r, Some(&save), &[vec![], vec![]], 42, 24)
            .unwrap_err()
            .code,
        "breeding_unverified"
    );
    assert_eq!(save.data, before);
}

#[test]
#[ignore = "requires five exact private ROMs and GEN3_BREEDING_SELECTION_PROBES independent native vectors"]
fn local_breeding_collection_matches_native_selection_and_current_parent_plans() {
    use crate::{
        acquisition::AcquisitionIndex,
        collection::{CollectionBasis, CollectionRequest},
    };
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_BREEDING_SELECTION_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        assert_eq!(probes[key]["md5"], r.profile.md5);
        assert_eq!(probes[key]["engine"], "mGBA ARM7");
        let original = r.data.clone();
        for row in probes[key]["rows"].as_array().unwrap() {
            let parents: [Vec<u8>; 2] = serde_json::from_value(row["parents"].clone()).unwrap();
            let (compat, child) = crate::breeding_collection::select(
                &r,
                None,
                &parents,
                row["seed"].as_u64().unwrap() as u32,
                row["offspring_pid"].as_u64().unwrap() as u32,
            )
            .unwrap();
            assert_eq!(
                serde_json::json!([compat, child]),
                serde_json::json!([row["compatibility"], row["species"]]),
                "{key}"
            );
        }
        let index = AcquisitionIndex::build(&r).unwrap();
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let row = probes[key]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["species"].as_u64().is_some_and(|s| s != 1 && s != 25))
            .unwrap();
        let raws: [Vec<u8>; 2] = serde_json::from_value(row["parents"].clone()).unwrap();
        let child = row["species"].as_u64().unwrap() as u16;
        for (i, raw) in raws.iter().enumerate() {
            save.insert(loc(i), raw, &r).unwrap();
        }
        save.validate(&r).unwrap();
        let before = save.data.clone();
        let request = || CollectionRequest {
            basis: CollectionBasis::Individuals,
            families: false,
            include_unknown_rewards: false,
        };
        let plan = index.collection(&r, &save, request()).unwrap();
        let coverage = plan.breeding_coverage.as_ref().unwrap();
        assert_eq!(
            (
                coverage.parent_count,
                coverage.checked_pairs,
                coverage.total_pairs,
                coverage.failed_pairs
            ),
            (3, 3, 3, 0)
        );
        let task = plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .find(|t| {
                t.target.kind == crate::acquisition::TargetKind::Species && t.target.id == child
            })
            .unwrap();
        let prep = task.preparation.as_ref().unwrap();
        assert_eq!(prep.origin, child);
        assert!(prep.breeding.is_some() && prep.needs_hatching && prep.steps.is_empty());
        let route = prep.breeding.as_ref().unwrap();
        let native = crate::breeding::preview(
            &r,
            Some(&save),
            &crate::breeding::Request {
                parents: [
                    crate::breeding::Parent::Stored {
                        location: route.parents[0].location,
                    },
                    crate::breeding::Parent::Stored {
                        location: route.parents[1].location,
                    },
                ],
                seed: route.seed,
                offspring_pid: route.offspring_pid,
                production_item: None,
            },
        )
        .unwrap();
        assert_eq!(native.child.unwrap().species, child);
        for p in &route.parents {
            assert_eq!(
                save.pokemon(p.location, &r).unwrap().unwrap().species,
                p.species
            );
        }
        assert_eq!(save.data, before);
        let again = index.collection(&r, &save, request()).unwrap();
        assert_eq!(
            serde_json::to_value(&again).unwrap(),
            serde_json::to_value(&plan).unwrap(),
            "cached native suggestions must remain ROM/SAV bound"
        );
        save.remove(loc(0), &r).unwrap();
        save.remove(loc(1), &r).unwrap();
        let after = save.data.clone();
        let without = index.collection(&r, &save, request()).unwrap();
        assert_eq!(without.breeding_coverage.as_ref().unwrap().parent_count, 1);
        assert!(without
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .filter_map(|t| t.preparation.as_ref())
            .all(|p| p.breeding.is_none()));
        assert_eq!(save.data, after);
        assert_eq!(*r.data, *original);
        eprintln!("{key}: 42 native compatibility scenarios and {} accepted selection checkpoints; existing-parent plan/receipt, cache invalidation and ROM/SAV preservation passed", probes[key]["rows"].as_array().unwrap().iter().filter(|row| !row["species"].is_null()).count());
    }
}

#[test]
fn event_dependency_queries_follow_guarded_writers_and_nonreward_tiles() {
    use crate::event_dependencies::{Index, Kind, Request};
    let (mut r, mut map) = npc_trade_fixture(profile::BW);
    let rules = r.profile.event_state.unwrap().effects.unwrap();
    let b = std::sync::Arc::make_mut(&mut r.data);
    for (op, code) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
        .into_iter()
        .zip(rules.handlers)
    {
        put32(
            b,
            rules.commands + op as usize * 4,
            0x08000001 + code as u32,
        );
    }
    // Flag 11 is needed before the NPC can set flag 12; alternate clears it.
    let root = 0x26000;
    let branch = 0x26100;
    b[root..root + 3].copy_from_slice(&[0x2b, 11, 0]);
    b[root + 3] = 6;
    b[root + 4] = 1;
    put32(b, root + 5, 0x08000000 + branch as u32);
    b[root + 9..root + 13].copy_from_slice(&[0x2a, 12, 0, 2]);
    b[branch..branch + 4].copy_from_slice(&[0x29, 12, 0, 2]);
    // A non-reward coordinate event has no reward marker but must remain positioned.
    b[0x25002] = 1;
    put32(b, 0x2500c, 0x08025200);
    put16(b, 0x25200, 1);
    put16(b, 0x25202, 2);
    put32(b, 0x2520c, 0x08026200);
    b[0x26200..0x26204].copy_from_slice(&[0x29, 11, 0, 2]);
    // Unreferenced bytes must never become an obtainable event.
    b[0x26300..0x26304].copy_from_slice(&[0x29, 12, 0, 2]);
    map.scripts.push(0x26200);
    let original = r.data.clone();
    let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let saved = save.data.clone();
    let index = Index::build(&r, std::slice::from_ref(&map)).unwrap();
    let request = |id| Request {
        kind: Kind::Flag,
        id,
        value: 1,
        comparison: 1,
        taken: true,
        expected_rom_md5: r.profile.md5.into(),
        offset: 0,
    };
    let report = index.query(&r, None, request(12)).unwrap();
    assert_eq!(report.writers.len(), 1);
    assert_eq!(report.writers[0].effect.offset, branch);
    assert_eq!(report.writers[0].reference.kind, "npc");
    assert!(report.writers[0]
        .conditions
        .iter()
        .any(|c| c.condition.id == 11 && c.satisfied.is_none()));
    let with_save = index.query(&r, Some(&save), request(12)).unwrap();
    assert_eq!(with_save.condition.satisfied, Some(false));
    assert!(with_save.writers[0]
        .conditions
        .iter()
        .any(|c| c.condition.id == 11 && c.satisfied == Some(false)));
    // SAV state is checked per query, not retained in the ROM-bound index.
    let layout = r.profile.event_state.unwrap();
    let range = &layout.flags[0];
    let logical = range.offset + 11 / 8;
    let per = 3968;
    let absolute = save.sections[1 + logical / per] + logical % per;
    save.data[absolute] |= 1 << (11 % 8);
    let changed = index.query(&r, Some(&save), request(12)).unwrap();
    assert!(changed.writers[0]
        .conditions
        .iter()
        .any(|c| c.condition.id == 11 && c.satisfied == Some(true)));
    let before = save.data.clone();
    let trigger = index.query(&r, Some(&save), request(11)).unwrap();
    assert_eq!(trigger.writers.len(), 1);
    assert_eq!(trigger.writers[0].reference.kind, "trigger");
    assert_eq!(
        (
            trigger.writers[0].reference.x,
            trigger.writers[0].reference.y
        ),
        (Some(1), Some(2))
    );
    assert!(trigger.partial && trigger.writers[0].reference.entry_unresolved);
    assert_eq!(save.data, before);
    assert_eq!(r.data, original);
    assert_ne!(save.data, saved); // Only this explicit synthetic state change above.
    let invalid = Request {
        kind: Kind::Variable,
        id: 0x8000,
        value: 1,
        comparison: 1,
        taken: true,
        expected_rom_md5: r.profile.md5.into(),
        offset: 0,
    };
    assert_eq!(
        index.query(&r, None, invalid).err().unwrap().code,
        "event_dependency_condition"
    );
    let mut other = r.clone();
    std::sync::Arc::make_mut(&mut other.data)[0] ^= 1;
    assert_eq!(
        index.query(&other, None, request(12)).err().unwrap().code,
        "rom_mismatch"
    );
    let mut other_profile = r.clone();
    other_profile.profile.md5 = profile::DP.md5;
    assert_eq!(
        index
            .query(&other_profile, None, request(12))
            .err()
            .unwrap()
            .code,
        "rom_mismatch"
    );
}

#[test]
fn event_effects_keep_unknown_values_and_native_copy_subtract_semantics() {
    let (mut r, _) = npc_trade_fixture(profile::BW);
    let rules = r.profile.event_state.unwrap().effects.unwrap();
    let b = std::sync::Arc::make_mut(&mut r.data);
    for (op, code) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
        .into_iter()
        .zip(rules.handlers)
    {
        put32(
            b,
            rules.commands + op as usize * 4,
            0x08000001 + code as u32,
        );
    }
    let start = 0x26000;
    b[start..start + 16].copy_from_slice(&[
        0x16, 0, 0x40, 10, 0, 0x16, 1, 0x40, 3, 0, 0x18, 0, 0x40, 1, 0x40, 2,
    ]);
    let report = r.event_effect_script(start).unwrap();
    assert!(report
        .effects
        .iter()
        .any(|e| e.operation == "subtract" && e.value == Some(7)));
    // An unresolved native call destroys local value knowledge, not potential writes.
    let b = std::sync::Arc::make_mut(&mut r.data);
    b[start + 10..start + 19].copy_from_slice(&[0x25, 1, 0, 0x17, 0, 0x40, 1, 0, 2]);
    let report = r.event_effect_script(start).unwrap();
    assert!(report
        .effects
        .iter()
        .any(|e| e.operation == "add" && e.value.is_none()));
    assert!(!report.complete && !report.stopped_at.is_empty());
    let b = std::sync::Arc::make_mut(&mut r.data);
    b[start..start + 10].copy_from_slice(&[0x19, 0, 0x40, 5, 0, 0x29, 12, 0, 2, 2]);
    let report = r.event_effect_script(start).unwrap();
    assert!(report.effects.is_empty());
    assert!(!report.complete);
    // Changed native dispatch is not silently parsed with another engine's rule.
    put32(
        std::sync::Arc::make_mut(&mut r.data).as_mut_slice(),
        rules.commands + 0x19 * 4,
        0x08026001,
    );
    assert_eq!(
        r.event_effect_script(start).err().unwrap().code,
        "event_dependency_dispatch"
    );
}

#[test]
#[ignore = "requires five private exact ROMs and GEN3_EVENT_EFFECT_PROBES independent mGBA evidence"]
fn local_event_effects_match_native_commands_and_referenced_dependency_queries() {
    use crate::event_dependencies::{Kind, Request};
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_EVENT_EFFECT_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    let mut app = crate::app::App::default();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let data = std::fs::read(&path).unwrap();
        let r = Rom::open(data.clone()).unwrap();
        assert_eq!(probes[key]["md5"], r.profile.md5);
        let rules = r.profile.event_state.unwrap().effects.unwrap();
        for (op, code) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
            .into_iter()
            .zip(rules.handlers)
        {
            assert_eq!(probes[key]["handlers"][op.to_string()], 0x08000000 + code);
        }
        for row in probes[key]["rows"].as_array().unwrap() {
            // Synthetic reader fixture only; native oracle used the same operands
            // in RAM. No modified ROM is written/exported or used by the app.
            let mut fixture = r.clone();
            let mut script = Vec::new();
            let op = row["opcode"].as_u64().unwrap() as u8;
            let dst = row["destination"].as_u64().unwrap() as u16;
            let mut assignment = |id: u16, value: u16| {
                script.push(0x16);
                script.extend(id.to_le_bytes());
                script.extend(value.to_le_bytes());
            };
            if op < 0x29 {
                assignment(dst, row["before"].as_u64().unwrap() as u16);
                let operand = row["operand"].as_u64().unwrap() as u16;
                if matches!(op, 0x18..=0x1a) && operand >= 0x4000 {
                    assignment(operand, row["source_value"].as_u64().unwrap() as u16);
                }
            }
            let start = fixture.data.len();
            let effect_offset = start + script.len();
            script.push(op);
            script.extend(dst.to_le_bytes());
            if op < 0x29 {
                script.extend((row["operand"].as_u64().unwrap() as u16).to_le_bytes());
            }
            script.push(2);
            std::sync::Arc::make_mut(&mut fixture.data).extend(script);
            let report = fixture.event_effect_script(start).unwrap();
            assert_eq!(
                report
                    .effects
                    .iter()
                    .find(|e| e.offset == effect_offset)
                    .unwrap()
                    .value,
                Some(row["result"].as_u64().unwrap() as u16),
                "{key}: {row}"
            );
        }
        let maps = r.maps().unwrap();
        let seed = maps
            .iter()
            .flat_map(|m| &m.scripts)
            .find_map(|root| {
                r.event_effect_script(*root)
                    .ok()?
                    .effects
                    .into_iter()
                    .next()
            })
            .unwrap();
        app.session = Some(Session::new(r.clone()));
        let output = app.dispatch(crate::app::Request {
            command: "event_dependencies".into(),
            payload: serde_json::json!({"kind": seed.kind, "id":seed.id, "value":seed.value.unwrap_or(1), "comparison":1,"taken":true,"expected_rom_md5":r.profile.md5}),
        }).unwrap();
        assert_eq!(output["rom_md5"], r.profile.md5);
        assert!(std::sync::Arc::ptr_eq(
            &app.event_dependency_cache.as_ref().unwrap().0,
            &r.data
        ));
        let index = &app.event_dependency_cache.as_ref().unwrap().1;
        assert_eq!(index.coverage.failed_scripts, 0, "{key}");
        let mut observed = 0;
        for map in &maps {
            for &root in map.scripts.iter().take(32) {
                let effects = r.event_effect_script(root).unwrap();
                for effect in effects.effects.iter().take(3) {
                    let report = index
                        .query(
                            &r,
                            None,
                            Request {
                                kind: if effect.kind == "flag" {
                                    Kind::Flag
                                } else {
                                    Kind::Variable
                                },
                                id: effect.id,
                                value: effect.value.unwrap_or(1) as u32,
                                comparison: 1,
                                taken: true,
                                expected_rom_md5: r.profile.md5.into(),
                                offset: 0,
                            },
                        )
                        .unwrap();
                    assert!(report.total_matches > 0, "{key}: {root:x}");
                    assert!(report.condition.actual.is_none());
                    assert!(report
                        .writers
                        .iter()
                        .all(|w| maps.iter().any(|m| m.id == w.reference.map_id)));
                    observed += 1;
                }
            }
        }
        assert!(observed > 0);
        assert_eq!(r.data.as_ref(), &data);
        assert_eq!(std::fs::read(path).unwrap(), data);
        println!(
            "{key}: {} native command cases; {observed} referenced effect queries; {}/{} roots",
            probes[key]["rows"].as_array().unwrap().len(),
            index.coverage.checked_scripts,
            index.coverage.total_scripts
        );
    }
}

#[test]
fn collection_export_traces_current_prerequisites_without_save_or_rom_writes() {
    use crate::{
        acquisition::AcquisitionIndex,
        app::{App, Request},
    };
    let (mut r, map) = npc_trade_fixture(profile::BW);
    let rules = r.profile.event_state.unwrap().effects.unwrap();
    let b = std::sync::Arc::make_mut(&mut r.data);
    for (op, code) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
        .into_iter()
        .zip(rules.handlers)
    {
        put32(
            b,
            rules.commands + op as usize * 4,
            0x08000001 + code as u32,
        );
    }
    // NPC needs flag 11; the map-level script can set it. Gift receipt unknown.
    b[0x26000..0x26003].copy_from_slice(&[0x2b, 11, 0]);
    b[0x26003] = 6;
    b[0x26004] = 1;
    put32(b, 0x26005, 0x08026100);
    b[0x26009] = 2;
    b[0x26100..0x2610d].copy_from_slice(&[0x16, 0, 0x80, 1, 0, 0x16, 1, 0x80, 1, 0, 9, 0, 2]);
    b[0x26200..0x26205].copy_from_slice(&[0x2b, 12, 0, 6, 1]);
    put32(b, 0x26205, 0x08026400);
    b[0x26209] = 2;
    b[0x26400..0x26404].copy_from_slice(&[0x29, 11, 0, 2]);
    b[0x26300..0x26304].copy_from_slice(&[0x29, 12, 0, 2]);
    let mut map = map;
    map.scripts.push(0x26200);
    map.scripts.push(0x26300);
    let report = r.map_events(&map).unwrap();
    let bytes = save_bytes(&r);
    let original = r.data.clone();
    let md5 = r.profile.md5;
    let mut session = Session::new(r);
    session.load(bytes.clone(), None).unwrap();
    let index = AcquisitionIndex {
        wild_cache: Default::default(),
        breeding_cache: Default::default(),
        world: crate::world::World {
            maps: vec![map],
            map_events: vec![report],
            encounters: vec![],
            trainers: vec![],
            trainer_locations: crate::world::TrainerLocationIndex {
                locations: vec![],
                unresolved_maps: vec![],
            },
            map_groups: &[],
        },
        species: vec![],
        evolutions: Default::default(),
        learnsets: Default::default(),
    };
    let mut app = App {
        acquisition_cache: Some((session.rom.data.clone(), index)),
        session: Some(session),
        ..Default::default()
    };
    let payload = serde_json::json!({"expected_rom_md5":md5,"query":{"basis":"individuals","families":true,"include_unknown_rewards":true}});
    let result = app
        .dispatch(Request {
            command: "collection_export".into(),
            payload: payload.clone(),
        })
        .unwrap();
    let prerequisites = &result["prerequisites"]["reports"];
    assert!(prerequisites
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["condition"]["condition"]["id"] == 11
            && r["writers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|w| w["reference"]["kind"] == "map_script")));
    assert!(result["prerequisites"]["partial"].as_bool().unwrap());
    let routes = result["prerequisites"]["routes"].as_array().unwrap();
    let route = routes
        .iter()
        .find(|route| {
            prerequisites[route["report_index"].as_u64().unwrap() as usize]["condition"]
                ["condition"]["id"]
                == 11
        })
        .unwrap();
    assert!(!route["goals"].as_array().unwrap().is_empty());
    assert!(prerequisites
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["condition"]["condition"]["id"] == 12));
    for goal in route["goals"].as_array().unwrap() {
        let task = &result["regions"][goal[0].as_u64().unwrap() as usize]["tasks"]
            [goal[1].as_u64().unwrap() as usize];
        assert!(task["source"]["conditions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["condition"]["id"] == 11));
    }
    assert_eq!(
        app.session.as_ref().unwrap().save.as_ref().unwrap().data,
        bytes
    );
    assert_eq!(app.session.as_ref().unwrap().rom.data, original);
    let cache = app.event_dependency_cache.as_ref().unwrap().0.clone();
    let repeated = app
        .dispatch(Request {
            command: "collection_export".into(),
            payload,
        })
        .unwrap();
    assert_eq!(result, repeated);
    let preview = app.dispatch(Request {
        command: "collection_prerequisites".into(),
        payload: serde_json::json!({"expected_rom_md5":md5,"query":{"basis":"individuals","families":true,"include_unknown_rewards":true}}),
    }).unwrap();
    assert_eq!(preview, result);
    let stale_preview = app.dispatch(Request {
        command: "collection_prerequisites".into(),
        payload: serde_json::json!({"expected_rom_md5":profile::ROCKET.md5,"query":{"basis":"individuals","families":true,"include_unknown_rewards":true}}),
    }).unwrap_err();
    assert_eq!(stale_preview.code, "rom_mismatch");
    assert!(std::sync::Arc::ptr_eq(
        &cache,
        &app.event_dependency_cache.as_ref().unwrap().0
    ));
    let stale=app.dispatch(Request { command:"event_dependencies".into(),payload:serde_json::json!({"kind":"flag","id":11,"value":1,"comparison":1,"taken":true,"expected_rom_md5":profile::ROCKET.md5}) }).unwrap_err();
    assert_eq!(stale.code, "rom_mismatch");
    let invalid=app.dispatch(Request { command:"event_dependencies".into(),payload:serde_json::json!({"kind":"flag","id":null,"value":1,"comparison":1,"taken":true,"expected_rom_md5":md5}) }).unwrap_err();
    assert_eq!(invalid.code, "json");
    assert_eq!(
        app.session.as_ref().unwrap().save.as_ref().unwrap().data,
        bytes
    );
    // Generated fixture only: an already-met condition must not suggest its
    // writers' prerequisites as additional work. No production flag-edit API.
    let session = app.session.as_mut().unwrap();
    let range = session
        .rom
        .profile
        .event_state
        .unwrap()
        .flags
        .iter()
        .find(|r| 11 >= r.first && 11 - r.first < r.count)
        .unwrap();
    assert!(matches!(range.block, crate::event_state::EventBlock::Main));
    let offset = range.offset + (11 - range.first) as usize / 8;
    let save = session.save.as_mut().unwrap();
    save.data[save.sections[1 + offset / 0xf80] + offset % 0xf80] |= 1 << ((11 - range.first) % 8);
    let satisfied_bytes = save.data.clone();
    let satisfied = app.dispatch(Request {
        command: "collection_prerequisites".into(),
        payload: serde_json::json!({"expected_rom_md5":md5,"query":{"basis":"individuals","families":true,"include_unknown_rewards":true}}),
    }).unwrap();
    let satisfied_reports = satisfied["prerequisites"]["reports"].as_array().unwrap();
    assert!(satisfied_reports
        .iter()
        .any(|r| r["condition"]["condition"]["id"] == 11 && r["condition"]["satisfied"] == true));
    assert!(!satisfied_reports
        .iter()
        .any(|r| r["condition"]["condition"]["id"] == 12));
    assert_eq!(
        app.session.as_ref().unwrap().save.as_ref().unwrap().data,
        satisfied_bytes
    );
    assert_eq!(app.session.as_ref().unwrap().rom.data, original);
}

#[test]
fn event_clue_search_preserves_references_guards_and_saved_snapshot_boundaries() {
    use crate::event_dependencies::{Index, SearchRequest};
    for profile in profile::PROFILES {
        let (mut rom, mut map) = npc_trade_fixture(profile);
        let rules = rom.profile.event_state.unwrap().effects.unwrap();
        let text = Codec::new().encode("HELLO", 6).unwrap();
        let only = Codec::new().encode("ONLY", 5).unwrap();
        let b = std::sync::Arc::make_mut(&mut rom.data);
        for (op, code) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
            .into_iter()
            .zip(rules.handlers)
        {
            put32(
                b,
                rules.commands + op as usize * 4,
                0x08000001 + code as u32,
            );
        }
        b[0x29000..0x29006].copy_from_slice(&text);
        b[0x29020..0x29025].copy_from_slice(&only);
        b[0x26000..0x26002].copy_from_slice(&[0x0f, 0]);
        put32(b, 0x26002, 0x08029000);
        b[0x26006..0x2600b].copy_from_slice(&[0x2b, 11, 0, 6, 1]);
        put32(b, 0x2600b, 0x08026100);
        b[0x2600f..0x26013].copy_from_slice(&[0x2a, 12, 0, 2]);
        for i in 0..66 {
            b[0x26100 + i * 3] = 0x29;
            put16(b, 0x26101 + i * 3, 12 + i as u16);
        }
        b[0x26100 + 66 * 3] = 2;
        b[0x26200..0x26202].copy_from_slice(&[0x0f, 0]);
        put32(b, 0x26202, 0x08029020);
        b[0x26206] = 2;
        // Seventy actors share a root, yet retain separate map positions/IDs.
        b[0x25000] = 70;
        put32(b, 0x25004, 0x08028000);
        for i in 0..70 {
            let at = 0x28000 + i * 24;
            b[at] = i as u8 + 1;
            put16(b, at + 4, if i == 69 { 99 } else { 1 });
            put16(b, at + 6, 2);
            put16(b, at + 20, 20);
            put32(b, at + 16, 0x08026000);
        }
        // A valid-looking but unreferenced root must not become a clue.
        b[0x26300..0x26307].copy_from_slice(&[0x0f, 0, 0, 0x90, 2, 8, 2]);
        b[0x26400..0x26402].copy_from_slice(&[0x0f, 0]);
        put32(b, 0x26402, 0x08029020);
        b[0x26406] = 2;
        map.scripts = vec![0x26000, 0x26200, 0x26400];
        let original = rom.data.clone();
        let mut save = Save::open(save_bytes(&rom), rom.profile.save).unwrap();
        let before = save.data.clone();
        let index = Index::build(&rom, std::slice::from_ref(&map)).unwrap();
        let request = |search: &str, offset| SearchRequest {
            expected_rom_md5: rom.profile.md5.into(),
            search: search.into(),
            map_id: None,
            offset,
            selected_id: None,
        };
        let first = index.search(&rom, None, request(" hello ", 0)).unwrap();
        assert_eq!(first.total_matches, 70);
        assert_eq!(first.entries.len(), 32);
        assert_eq!(first.next_offset, Some(32));
        assert!(first.entries[0].effects_truncated);
        assert!(first.entries[0]
            .effects
            .iter()
            .all(|e| e.observed.is_none()));
        assert_eq!(first.entries[0].text[0].text, "HELLO");
        assert_eq!(first.entries[0].reference.x, Some(1));
        let final_page = index
            .search(&rom, Some(&save), request("HELLO", 64))
            .unwrap();
        assert_eq!(final_page.entries.len(), 6);
        assert!(final_page.next_offset.is_none());
        let outside = final_page.entries.last().unwrap();
        assert_eq!(outside.reference.x, None);
        assert_eq!(outside.reference.y, None);
        assert!(outside.reference.entry_unresolved);
        let selected_id = outside.id.clone();
        let mut selected_request = request("HELLO", 0);
        selected_request.selected_id = Some(selected_id.clone());
        let selection = index.search(&rom, Some(&save), selected_request).unwrap();
        let selected = selection.selected.unwrap();
        assert_eq!(selected.id, selected_id);
        assert_eq!(selected.visibility[0].satisfied, Some(true));
        let effect = selected
            .effects
            .iter()
            .find(|e| e.effect.value == Some(1))
            .unwrap();
        assert_eq!(effect.observed, Some(false));
        assert!(effect
            .conditions
            .iter()
            .any(|c| c.condition.id == 11 && c.satisfied == Some(false)));
        assert!(!serde_json::to_value(&selected)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("status"));
        let text_only = index.search(&rom, None, request("ONLY", 0)).unwrap();
        assert_eq!(text_only.total_matches, 2);
        assert_ne!(text_only.entries[0].id, text_only.entries[1].id);
        assert_eq!(
            text_only.entries[0].reference.offset,
            text_only.entries[1].reference.offset
        );
        assert_ne!(
            text_only.entries[0].reference.root,
            text_only.entries[1].reference.root
        );
        assert!(text_only.entries[0].effects.is_empty());
        assert_eq!(text_only.entries[0].reference.kind, "map_script");
        assert_eq!(
            index
                .search(&rom, None, request("synthetic TRADE room", 0))
                .unwrap()
                .total_matches,
            72
        );
        let mut filtered = request("", 0);
        filtered.map_id = Some("missing-map".into());
        assert_eq!(index.search(&rom, None, filtered).unwrap().total_matches, 0);
        assert_eq!(save.data, before);
        assert_eq!(rom.data, original);
        // Explicitly change only a synthetic snapshot, and prove per-query invalidation.
        let layout = rom.profile.event_state.unwrap();
        let range = layout.flags[0];
        let logical = range.offset + 11 / 8;
        let absolute = save.sections[1 + logical / 3968] + logical % 3968;
        save.data[absolute] |= 1 << (11 % 8);
        let changed = index
            .search(&rom, Some(&save), request("HELLO", 0))
            .unwrap();
        assert!(changed.entries[0]
            .effects
            .iter()
            .any(|e| e
                .conditions
                .iter()
                .any(|c| c.condition.id == 11 && c.satisfied == Some(true))));
        let mut oversized = request("", 0);
        oversized.search = "a".repeat(513);
        assert_eq!(
            index.search(&rom, None, oversized).err().unwrap().code,
            "event_search_arguments"
        );
        assert_eq!(
            index
                .search(&rom, None, request("HELLO", 71))
                .err()
                .unwrap()
                .code,
            "event_search_offset"
        );
        let mut stale = request("", 0);
        stale.expected_rom_md5 = "wrong".into();
        assert_eq!(
            index.search(&rom, None, stale).err().unwrap().code,
            "rom_mismatch"
        );
        let mut other = rom.clone();
        std::sync::Arc::make_mut(&mut other.data)[0] ^= 1;
        assert_eq!(
            index
                .search(&other, None, request("", 0))
                .err()
                .unwrap()
                .code,
            "rom_mismatch"
        );
        // Read-only command: fresh SAV overlay, unchanged bytes, reject stale input.
        let saved_snapshot = save.data.clone();
        let mut session = Session::new(rom.clone());
        session.save = Some(save);
        let mut app = crate::app::App {
            session: Some(session),
            event_dependency_cache: Some((rom.data.clone(), index)),
            ..Default::default()
        };
        let dispatch = |payload| crate::app::Request {
            command: "event_search".into(),
            payload,
        };
        let payload = serde_json::json!({"expected_rom_md5":rom.profile.md5,"search":"HELLO"});
        let response = app.dispatch(dispatch(payload.clone())).unwrap();
        assert_eq!(response["total_matches"], 70);
        assert_eq!(app.dispatch(dispatch(payload)).unwrap(), response);
        for payload in [
            serde_json::Value::Null,
            serde_json::json!({"expected_rom_md5":"stale"}),
            serde_json::json!({"expected_rom_md5":rom.profile.md5,"offset":null}),
            serde_json::json!({"expected_rom_md5":rom.profile.md5,"write":true}),
        ] {
            assert!(app.dispatch(dispatch(payload)).is_err());
        }
        assert_eq!(
            app.session.as_ref().unwrap().save.as_ref().unwrap().data,
            saved_snapshot
        );
        assert_eq!(app.session.as_ref().unwrap().rom.data, original);
    }
}

#[test]
#[ignore = "requires all five exact private ROMs and GEN3_SAVE_MERCURY12"]
fn local_event_clue_search_cross_rom_queries() {
    use crate::app::{App, Request};
    let mut app = App::default();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let original = std::fs::read(&path).unwrap();
        let rom = Rom::open(original.clone()).unwrap();
        let maps = rom.maps().unwrap();
        app.session = Some(Session::new(rom.clone()));
        let query = |search: &str, offset, selected: Option<&str>| Request {
            command: "event_search".into(),
            payload: serde_json::json!({"expected_rom_md5":rom.profile.md5,"search":search,"offset":offset,"selected_id":selected}),
        };
        let first = app.dispatch(query("", 0, None)).unwrap();
        assert_eq!(first["rom_md5"], rom.profile.md5);
        assert!(first["total_matches"].as_u64().unwrap() > 0);
        assert!(std::sync::Arc::ptr_eq(
            &app.event_dependency_cache.as_ref().unwrap().0,
            &rom.data
        ));
        let total = first["total_matches"].as_u64().unwrap() as usize;
        let mut checked = 0;
        let mut ids = std::collections::BTreeSet::new();
        // Every search row is checked against actual map/event bytes and current
        // ROM strings. This is reference evidence, not gameplay accessibility.
        for offset in (0..total).step_by(32) {
            let page = app.dispatch(query("", offset, None)).unwrap();
            for entry in page["entries"].as_array().unwrap() {
                assert!(ids.insert(entry["id"].as_str().unwrap().to_string()));
                let reference = &entry["reference"];
                let map = maps
                    .iter()
                    .find(|m| m.id == reference["map_id"].as_str().unwrap())
                    .unwrap();
                assert_eq!(reference["map_name"], map.name);
                let root = reference["root"].as_u64().unwrap() as usize;
                let at = reference["offset"].as_u64().unwrap() as usize;
                let script_off = match reference["kind"].as_str().unwrap() {
                    "npc" => Some(16),
                    "trigger" => Some(12),
                    "sign" => Some(8),
                    "map_script" => None,
                    _ => panic!(),
                };
                if let Some(script_off) = script_off {
                    assert_eq!(pointer(&rom.data, at + script_off).unwrap(), root);
                } else {
                    assert!(map.scripts.contains(&root));
                }
                if let (Some(x), Some(y)) = (reference["x"].as_i64(), reference["y"].as_i64()) {
                    assert!(x >= 0 && y >= 0 && x < (map.width as i64) && y < (map.height as i64));
                }
                for text in entry["text"].as_array().unwrap() {
                    assert_eq!(
                        text["text"],
                        rom.cstring(text["offset"].as_u64().unwrap() as usize)
                    );
                }
                assert!(entry["effects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|e| e["observed"].is_null()));
                assert!(entry.get("status").is_none());
                checked += 1;
            }
        }
        assert_eq!(checked, total);
        let sample = first["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| !e["text"].as_array().unwrap().is_empty())
            .unwrap();
        let phrase = sample["text"][0]["text"]
            .as_str()
            .unwrap()
            .chars()
            .take(64)
            .collect::<String>();
        let found = app
            .dispatch(query(&phrase, 0, sample["id"].as_str()))
            .unwrap();
        assert_eq!(found["selected"]["id"], sample["id"]);
        if key == "MERCURY12" {
            let sav_path = std::env::var("GEN3_SAVE_MERCURY12").unwrap();
            let before = std::fs::read(&sav_path).unwrap();
            app.session
                .as_mut()
                .unwrap()
                .load(before.clone(), None)
                .unwrap();
            let current = app
                .dispatch(query(&phrase, 0, sample["id"].as_str()))
                .unwrap();
            assert_eq!(current["selected"]["text"], found["selected"]["text"]);
            assert_eq!(
                app.session.as_ref().unwrap().save.as_ref().unwrap().data,
                before
            );
            assert_eq!(std::fs::read(sav_path).unwrap(), before);
        }
        assert_eq!(rom.data.as_ref(), &original);
        assert_eq!(std::fs::read(path).unwrap(), original);
        println!("{key}: {total} searchable references, every map/root/text checked; search/detail/cache preservation passed");
    }
}

#[test]
fn trainer_references_keep_guarded_roots_actors_and_unresolved_access_separate() {
    use crate::event_dependencies::{Index, TrainerRequest};
    for profile in profile::PROFILES {
        let (mut r, mut map) = npc_trade_fixture(profile);
        let rules = r.profile.event_state.unwrap().effects.unwrap();
        let b = std::sync::Arc::make_mut(&mut r.data);
        for (op, code) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
            .into_iter()
            .zip(rules.handlers)
        {
            put32(
                b,
                rules.commands + op as usize * 4,
                0x08000001 + code as u32,
            );
        }
        b[0x26000..0x26005].copy_from_slice(&[0x2b, 11, 0, 6, 1]);
        put32(b, 0x26005, 0x08026100);
        b[0x26009] = 0x5c;
        b[0x2600a] = 3;
        put16(b, 0x2600b, 2);
        b[0x26013] = 2;
        b[0x26100] = 0x5c;
        b[0x26101] = 3;
        put16(b, 0x26102, 1);
        b[0x2610a] = 2;
        b[0x25000] = 2;
        put16(b, 0x25114, 20);
        b[0x25118] = 2;
        put16(b, 0x2511c, 2);
        put16(b, 0x2511e, 3);
        put32(b, 0x25128, 0x08026000);
        b[0x25002] = 1;
        put32(b, 0x2500c, 0x08025200);
        put16(b, 0x25200, 1);
        put16(b, 0x25202, 2);
        put32(b, 0x2520c, 0x08026200);
        b[0x26200] = 0x5c;
        b[0x26201] = 3;
        put16(b, 0x26202, 1);
        b[0x2620a] = 2;
        // Map-level trainer setup, width depends on native verified engine format.
        b[0x26400] = 0x5c;
        b[0x26401] = 10;
        put16(b, 0x26402, 1);
        b[0x26400 + rules.battle_lengths[10] as usize] = 2;
        map.scripts = vec![0x26000, 0x26200, 0x26400];
        let original = r.data.clone();
        let save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let saved = save.data.clone();
        let index = Index::build(&r, std::slice::from_ref(&map)).unwrap();
        let request = |id| TrainerRequest {
            expected_rom_md5: r.profile.md5.into(),
            trainer_id: id,
            offset: 0,
        };
        let report = index.trainer(&r, None, request(1)).unwrap();
        assert_eq!(report.references.len(), 4);
        assert_eq!(
            report
                .references
                .iter()
                .filter(|row| row.reference.kind == "npc")
                .count(),
            2
        );
        assert!(report
            .references
            .iter()
            .all(|row| row.reference.entry_unresolved));
        assert!(report
            .references
            .iter()
            .any(|row| row.reference.kind == "trigger"
                && row.reference.x == Some(1)
                && row.reference.y == Some(2)));
        assert!(report
            .references
            .iter()
            .any(|row| row.reference.kind == "map_script" && row.reference.x.is_none()));
        let npc = report
            .references
            .iter()
            .find(|row| row.reference.kind == "npc")
            .unwrap();
        assert!(npc
            .conditions
            .iter()
            .any(|c| c.condition.id == 11 && c.satisfied.is_none()));
        let current = index.trainer(&r, Some(&save), request(1)).unwrap();
        assert!(current.references.iter().any(|row| row
            .conditions
            .iter()
            .any(|c| c.condition.id == 11 && c.satisfied == Some(false))));
        assert!(current.references.iter().any(|row| row
            .visibility
            .iter()
            .any(|c| c.condition.id == 20 && c.satisfied == Some(true))));
        let alternate = index.trainer(&r, Some(&save), request(2)).unwrap();
        assert_eq!(alternate.references.len(), 2);
        assert!(alternate
            .references
            .iter()
            .all(|row| row
                .conditions
                .iter()
                .any(|c| c.condition.id == 11 && c.satisfied == Some(true))));
        let serialized = serde_json::to_value(&current).unwrap();
        assert!(serialized["references"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row.get("status").is_none() && row.get("completed").is_none()));
        assert_eq!(save.data, saved);
        assert_eq!(r.data, original);
        let mut app = crate::app::App {
            session: Some(Session::new(r.clone())),
            event_dependency_cache: Some((r.data.clone(), index)),
            ..Default::default()
        };
        let output = app
            .dispatch(crate::app::Request {
                command: "trainer_references".into(),
                payload: serde_json::json!({"expected_rom_md5":r.profile.md5,"trainer_id":1}),
            })
            .unwrap();
        assert_eq!(output["total_matches"], 4);
        for payload in [
            serde_json::json!({"expected_rom_md5":"wrong","trainer_id":1}),
            serde_json::json!({"expected_rom_md5":r.profile.md5,"trainer_id":null}),
            serde_json::json!({"expected_rom_md5":r.profile.md5,"trainer_id":1,"offset":9}),
        ] {
            assert!(app
                .dispatch(crate::app::Request {
                    command: "trainer_references".into(),
                    payload
                })
                .is_err());
        }
        // A changed native dispatch is not parsed with the former parameter layout.
        put32(
            std::sync::Arc::make_mut(&mut r.data).as_mut_slice(),
            rules.commands + 0x5c * 4,
            0x08012345,
        );
        assert_eq!(
            r.trainer_battle_length(0x26100).err().unwrap().code,
            "trainer_script_dispatch"
        );
    }
}

#[test]
#[ignore = "requires five exact private ROMs and GEN3_TRAINER_SCRIPT_PROBES independent native vectors"]
fn local_trainer_reference_formats_match_native_boundaries_and_current_roots() {
    use crate::event_dependencies::{Index, TrainerRequest};
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_TRAINER_SCRIPT_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    let mut app = crate::app::App::default();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let data = std::fs::read(&path).unwrap();
        let r = Rom::open(data.clone()).unwrap();
        let rules = r.profile.event_state.unwrap().effects.unwrap();
        assert_eq!(probes[key]["md5"], r.profile.md5);
        assert_eq!(probes[key]["handler"], rules.battle_handler);
        for row in probes[key]["rows"].as_array().unwrap() {
            let typ = row["type"].as_u64().unwrap() as usize;
            let mut fixture = r.clone();
            let at = fixture.data.len();
            let mut script = vec![0; 33];
            script[0] = 0x5c;
            script[1] = typ as u8;
            put16(
                &mut script,
                2,
                row["input_trainer"].as_u64().unwrap() as u16,
            );
            if rules.battle_lengths[typ] > 0 {
                assert_eq!(row["operand_length"], rules.battle_lengths[typ]);
                script[rules.battle_lengths[typ] as usize] = 2;
            }
            std::sync::Arc::make_mut(&mut fixture.data).extend(script);
            if rules.battle_lengths[typ] > 0 {
                assert_eq!(
                    fixture.trainer_battle_length(at).unwrap(),
                    rules.battle_lengths[typ] as usize
                );
            } else {
                assert!(fixture.trainer_battle_length(at).is_err());
            }
            if matches!(rules.battle_roles[typ], "primary" | "setup") {
                assert_eq!(row["opponent_a"], row["input_trainer"]);
            } else if rules.battle_roles[typ] == "secondary_setup" {
                assert_eq!(row["opponent_b"], row["input_trainer"]);
            }
        }
        let maps = r.maps().unwrap();
        let index = Index::build(&r, &maps).unwrap();
        let mut references = 0;
        let mut positioned = 0;
        for id in 1..r.profile.trainers.count as u16 {
            let mut offset = 0;
            loop {
                let report = index
                    .trainer(
                        &r,
                        None,
                        TrainerRequest {
                            expected_rom_md5: r.profile.md5.into(),
                            trainer_id: id,
                            offset,
                        },
                    )
                    .unwrap();
                for row in &report.references {
                    assert_eq!(r.data[row.battle.offset], 0x5c);
                    assert_eq!(u16(&r.data, row.battle.offset + 2).unwrap(), id);
                    assert!(maps.iter().any(|m| m.id == row.reference.map_id));
                    assert!(row.conditions.iter().all(|c| c.actual.is_none()));
                    assert!(row.visibility.iter().all(|c| c.actual.is_none()));
                    references += 1;
                    positioned += usize::from(row.reference.x.is_some());
                }
                if let Some(next) = report.next_offset {
                    offset = next;
                } else {
                    break;
                }
            }
        }
        app.session = Some(Session::new(r.clone()));
        let reply = app
            .dispatch(crate::app::Request {
                command: "trainer_references".into(),
                payload: serde_json::json!({"expected_rom_md5":r.profile.md5,"trainer_id":1}),
            })
            .unwrap();
        assert_eq!(reply["rom_md5"], r.profile.md5);
        assert!(std::sync::Arc::ptr_eq(
            &app.event_dependency_cache.as_ref().unwrap().0,
            &r.data
        ));
        assert_eq!(r.data.as_ref(), &data);
        assert_eq!(std::fs::read(path).unwrap(), data);
        println!("{key}: {} native scenarios; {references} guarded literal-record references, {positioned} positioned; ROM preserved", probes[key]["rows"].as_array().unwrap().len());
    }
}

#[test]
fn hardware_clock_snapshots_are_not_live_time_and_remain_rom_scoped() {
    use crate::{
        clock::ClockScenario,
        hardware_clock::{Input, Request, Time},
    };
    for p in [
        profile::BW,
        profile::DP,
        profile::ROCKET,
        crate::ultimate::PROFILE,
    ] {
        let r = adapter_rom(p);
        let original = r.data.clone();
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let at = save.sections[0];
        put16(&mut save.data, at + 0x98, (-1_i16) as u16);
        save.data[at + 0x9a..at + 0x9d].copy_from_slice(&[23, 59, 58]);
        put16(&mut save.data, at + 0xa0, 2000);
        save.data[at + 0xa2..at + 0xa5].copy_from_slice(&[12, 30, 40]);
        let before = save.data.clone();
        let report = r
            .clock_query_with_save(
                Some(&save),
                ClockScenario {
                    hour: None,
                    weekday: None,
                },
            )
            .unwrap();
        assert_eq!(report.rom_md5, p.md5);
        assert_eq!(report.issue, Some("hardware_rtc_unresolved"));
        assert_eq!(report.source, "unresolved");
        assert_eq!(report.effective_hour, None);
        assert_eq!(report.weekday, None);
        assert_eq!(report.period, None);
        assert!(!report.current_clock_verified);
        let hw = report.hardware.unwrap();
        assert_eq!(
            hw.offset,
            Time {
                days: -1,
                hour: 23,
                minute: 59,
                second: 58
            }
        );
        assert_eq!(
            hw.last_update,
            Time {
                days: 2000,
                hour: 12,
                minute: 30,
                second: 40
            }
        );
        assert!(hw.offset_valid && hw.last_update_valid);
        let input = Input {
            year: 2026,
            month: 10,
            day: 6,
            hour: 12,
            minute: 30,
            second: 40,
        };
        assert_eq!(
            r.hardware_clock_preview(
                Some(&save),
                Request {
                    expected_rom_md5: "stale".into(),
                    rtc: input,
                    offset: None
                }
            )
            .err()
            .unwrap()
            .code,
            "rom_mismatch"
        );
        assert_eq!(
            r.hardware_clock_preview(
                None,
                Request {
                    expected_rom_md5: p.md5.into(),
                    rtc: input,
                    offset: None
                }
            )
            .err()
            .unwrap()
            .code,
            "clock_save_required"
        );
        assert!(r
            .hardware_clock_preview(
                Some(&save),
                Request {
                    expected_rom_md5: p.md5.into(),
                    rtc: Input { month: 0, ..input },
                    offset: None
                }
            )
            .is_err());
        save.data[at + 0x9a] = 255;
        assert!(
            !r.hardware_clock_snapshot(Some(&save))
                .unwrap()
                .unwrap()
                .offset_valid
        );
        assert_eq!(
            r.hardware_clock_preview(
                Some(&save),
                Request {
                    expected_rom_md5: p.md5.into(),
                    rtc: input,
                    offset: None
                }
            )
            .err()
            .unwrap()
            .code,
            "clock_saved_offset"
        );
        save.data[at + 0x9a] = 23;
        assert_eq!(save.data, before);
        assert_eq!(r.data, original);
        let mut app = crate::app::App {
            session: Some(Session::new(r)),
            ..Default::default()
        };
        app.session.as_mut().unwrap().save = Some(save);
        for payload in [
            serde_json::Value::Null,
            serde_json::json!({"expected_rom_md5":p.md5,"rtc":null}),
            serde_json::json!({"expected_rom_md5":p.md5,"rtc":input,"offset":{"days":null,"hour":0,"minute":0,"second":0}}),
            serde_json::json!({"expected_rom_md5":p.md5,"rtc":input,"write":true}),
        ] {
            assert!(app
                .dispatch(crate::app::Request {
                    command: "clock_rtc_preview".into(),
                    payload
                })
                .is_err());
        }
        assert_eq!(
            app.session.as_ref().unwrap().save.as_ref().unwrap().data,
            before
        );
    }
    let r = adapter_rom(crate::mercury::PROFILE);
    assert!(r
        .hardware_clock_preview(
            None,
            Request {
                expected_rom_md5: r.profile.md5.into(),
                rtc: Input {
                    year: 2026,
                    month: 10,
                    day: 6,
                    hour: 12,
                    minute: 0,
                    second: 0
                },
                offset: Some(Time {
                    days: 0,
                    hour: 0,
                    minute: 0,
                    second: 0
                })
            }
        )
        .is_err());
}

#[test]
#[ignore = "requires four exact private ROMs and GEN3_HARDWARE_CLOCK_PROBES independent native vectors"]
fn local_hardware_clock_matches_native_validation_offsets_and_borrowing() {
    use crate::hardware_clock::{Input, Request, Time};
    let native: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_HARDWARE_CLOCK_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    let mut app = crate::app::App::default();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let r = Rom::open(bytes.clone()).unwrap();
        let rules = r.profile.hardware_clock.unwrap();
        assert_eq!(native[key]["md5"], r.profile.md5);
        assert_eq!(native[key]["validation"], rules.validate);
        assert_eq!(native[key]["difference"], rules.difference);
        assert_eq!(native[key]["getter"], rules.getter);
        let mut saved = Save::open(save_bytes(&r), r.profile.save).unwrap();
        for row in native[key]["rows"].as_array().unwrap() {
            let v = row["date"].as_array().unwrap();
            let n = |i: usize| v[i].as_u64().unwrap() as u8;
            let rtc = Input {
                year: v[0].as_u64().unwrap() as u16,
                month: n(1),
                day: n(2),
                hour: n(3),
                minute: n(4),
                second: n(5),
            };
            let off = row["offset"].as_array().unwrap();
            let i = |idx: usize| off[idx].as_i64().unwrap() as i8;
            let offset = Time {
                days: off[0].as_i64().unwrap() as i16,
                hour: i(1),
                minute: i(2),
                second: i(3),
            };
            let projected = r
                .hardware_clock_preview(
                    None,
                    Request {
                        expected_rom_md5: r.profile.md5.into(),
                        rtc,
                        offset: Some(offset),
                    },
                )
                .unwrap();
            assert_eq!(
                serde_json::to_value(projected.local_time).unwrap(),
                row["result"],
                "{key} {row}"
            );
            assert_eq!(projected.weekday, None);
            assert_eq!(projected.period, None);
            assert!(!projected.current_clock_verified);
            let at = saved.sections[0] + rules.save_offset;
            put16(&mut saved.data, at, offset.days as u16);
            saved.data[at + 2..at + 5].copy_from_slice(&[
                offset.hour as u8,
                offset.minute as u8,
                offset.second as u8,
            ]);
            let before = saved.data.clone();
            let actual = r
                .hardware_clock_preview(
                    Some(&saved),
                    Request {
                        expected_rom_md5: r.profile.md5.into(),
                        rtc,
                        offset: None,
                    },
                )
                .unwrap();
            assert_eq!(actual.offset_source, "save");
            assert_eq!(actual.local_time, projected.local_time);
            assert_eq!(saved.data, before);
        }
        let rtc = Input {
            year: 2026,
            month: 10,
            day: 6,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let offset = Some(Time {
            days: 0,
            hour: 0,
            minute: 0,
            second: 0,
        });
        for (year, month, day) in [(2001, 2, 29), (2026, 2, 30), (2026, 4, 31)] {
            assert_eq!(
                r.hardware_clock_preview(
                    None,
                    Request {
                        expected_rom_md5: r.profile.md5.into(),
                        rtc: Input {
                            year,
                            month,
                            day,
                            ..rtc
                        },
                        offset
                    }
                )
                .err()
                .unwrap()
                .code,
                "clock_rtc_invalid"
            );
        }
        app.session = Some(Session::new(r.clone()));
        app.session.as_mut().unwrap().save = Some(saved.clone());
        let payload = serde_json::json!({"expected_rom_md5":r.profile.md5,"rtc":rtc});
        let query = |payload| crate::app::Request {
            command: "clock_rtc_preview".into(),
            payload,
        };
        let first = app.dispatch(query(payload.clone())).unwrap();
        let at = app
            .session
            .as_ref()
            .unwrap()
            .save
            .as_ref()
            .unwrap()
            .sections[0]
            + rules.save_offset;
        app.session.as_mut().unwrap().save.as_mut().unwrap().data[at + 2] = 1;
        let second = app.dispatch(query(payload)).unwrap();
        assert_ne!(first["local_time"], second["local_time"]);
        assert_eq!(second["rom_md5"], r.profile.md5);
        assert_eq!(std::fs::read(path).unwrap(), bytes);
        assert_eq!(r.data.as_ref(), &bytes);
        println!("{key}: {} native RTC projections, SAV-offset parity, invalid calendar and fresh snapshot passed",native[key]["rows"].as_array().unwrap().len());
    }
}

#[test]
fn training_queries_reject_stale_rom_and_malformed_individuals() {
    use crate::app::{App, Request};
    let mut app = App {
        session: Some(Session::new(rom())),
        ..Default::default()
    };
    assert_eq!(
        app.dispatch(Request {
            command: "training_catalog".into(),
            payload: serde_json::json!({"expected_rom_md5":"stale"})
        })
        .unwrap_err()
        .code,
        "rom_mismatch"
    );
    let valid = serde_json::json!({"expected_rom_md5":profile::BW.md5,"item":63,"individual":{"kind":"simulated","species":1,"level":5,"evs":[0,0,0,0,0,0],"friendship":70}});
    for bad in [
        serde_json::json!([null, 0, 0, 0, 0, 0]),
        serde_json::json!([256, 0, 0, 0, 0, 0]),
        serde_json::json!([-1, 0, 0, 0, 0, 0]),
        serde_json::json!([0, 0]),
        serde_json::json!([0.5, 0, 0, 0, 0, 0]),
    ] {
        let mut p = valid.clone();
        p["individual"]["evs"] = bad;
        assert_eq!(
            app.dispatch(Request {
                command: "training_preview".into(),
                payload: p
            })
            .unwrap_err()
            .code,
            "json"
        );
    }
    let mut p = valid;
    p["unexpected"] = serde_json::json!(true);
    assert_eq!(
        app.dispatch(Request {
            command: "training_preview".into(),
            payload: p
        })
        .unwrap_err()
        .code,
        "json"
    );
}

#[test]
#[ignore = "requires five exact private ROMs and GEN3_TRAINING_PROBES mGBA vectors"]
fn local_training_classification_and_effect_match_native_all_profiles() {
    use crate::training::{Individual, Request};
    let vectors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_TRAINING_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for (key, profile) in [
        ("BW", profile::BW),
        ("DP", profile::DP),
        ("ROCKET", profile::ROCKET),
        ("ULTIMATE", crate::ultimate::PROFILE),
        ("MERCURY12", crate::mercury::PROFILE),
    ] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let rom = Rom::open(std::fs::read(&path).unwrap()).unwrap();
        let hash = crate::binary::sha256(&rom.data);
        assert_eq!(vectors[key]["md5"], rom.profile.md5);
        let catalog = rom.training_catalog().unwrap();
        assert_eq!(catalog.rom_md5, profile.md5);
        let rows = vectors[key]["categories"].as_array().unwrap();
        let expected: Vec<_> = rows
            .iter()
            .filter(|r| (12..=17).contains(&r["category"].as_u64().unwrap()))
            .collect();
        assert_eq!(catalog.offers.len(), expected.len());
        for offer in &catalog.offers {
            let row = expected.iter().find(|r| r["item"] == offer.item).unwrap();
            assert_eq!(row["category"], offer.native_category);
            assert_eq!(row["handler"], offer.handler);
            assert_eq!(
                offer.stat,
                match offer.native_category {
                    13 => 0,
                    12 => 1,
                    17 => 2,
                    16 => 3,
                    14 => 4,
                    15 => 5,
                    _ => unreachable!(),
                }
            );
        }
        let mut save = Save::open(save_bytes(&rom), profile.save).unwrap();
        let first = save.sections[1];
        save.data[first + save.layout.party_count] = 1;
        let at = first + save.layout.party;
        for row in vectors[key]["rows"].as_array().unwrap() {
            let raw: Vec<u8> = serde_json::from_value(row["before"].clone()).unwrap();
            save.data[at..at + 100].copy_from_slice(&raw);
            let old = save.data.clone();
            let item = row["item"].as_u64().unwrap() as u16;
            let actual = rom
                .training_preview(
                    Some(&save),
                    Request {
                        expected_rom_md5: profile.md5.into(),
                        item,
                        rng_seed: None,
                        individual: Individual::Stored { location: party() },
                    },
                )
                .unwrap();
            let expected: Vec<u8> = serde_json::from_value(row["after"].clone()).unwrap();
            assert_eq!(actual.raw, expected, "{key} item {item}");
            assert_eq!(actual.native_no_effect, row["no_effect"].as_bool().unwrap());
            assert_eq!(actual.changed, raw != expected);
            assert_eq!(actual.before.pid, actual.after.pid);
            assert_eq!(actual.before.ot_id, actual.after.ot_id);
            assert_eq!(save.data, old);
        }
        let bad = rom
            .training_preview(
                None,
                Request {
                    expected_rom_md5: profile.md5.into(),
                    item: 0,
                    rng_seed: None,
                    individual: Individual::Simulated {
                        species: 25,
                        level: 5,
                        evs: [0; 6],
                        friendship: 70,
                        ivs: None,
                        nature_override: None,
                        ability_slot: None,
                    },
                },
            )
            .unwrap_err();
        assert_eq!(bad.code, "training_item_unverified");
        assert_eq!(crate::binary::sha256(&rom.data), hash);
        assert_eq!(crate::binary::sha256(&std::fs::read(path).unwrap()), hash);
        println!(
            "{key}: {} EV offers, {} native application vectors, original ROM/SAV preserved",
            catalog.offers.len(),
            vectors[key]["rows"].as_array().unwrap().len()
        );
    }
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_TRAINING_NATURE_PROBES independent mGBA vectors"]
fn local_training_nature_items_match_native_and_existing_save_editor() {
    use crate::training::{Individual, Request};
    let vectors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_TRAINING_NATURE_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let rom = Rom::open(std::fs::read(&path).unwrap()).unwrap();
        let hash = crate::binary::sha256(&rom.data);
        let catalog = rom.training_catalog().unwrap();
        if key != "ROCKET" {
            assert!(
                catalog.nature_items.is_empty(),
                "unverified handlers must not leak between profiles"
            );
            continue;
        }
        assert_eq!(vectors["md5"], rom.profile.md5);
        assert_eq!(
            catalog.nature_items.len(),
            vectors["offers"].as_array().unwrap().len()
        );
        for offer in &catalog.nature_items {
            let expected = vectors["offers"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["item"] == offer.item)
                .unwrap();
            assert_eq!(expected["nature"], offer.nature);
            assert_eq!(offer.handler, 0x08136ddd);
        }
        let index = crate::acquisition::AcquisitionIndex::build(&rom).unwrap();
        let mut sources = 0;
        let mut positioned = 0;
        for offer in &catalog.nature_items {
            let report = index
                .query(
                    &rom,
                    None,
                    crate::acquisition::Target {
                        kind: crate::acquisition::TargetKind::Item,
                        id: offer.item,
                    },
                )
                .unwrap();
            assert_eq!(report.target.id, offer.item);
            for source in &report.sources {
                sources += 1;
                if let Some(map) = &source.map_id {
                    let navigation = rom.map_navigation(map).unwrap();
                    assert_eq!(navigation.map_id, *map);
                    positioned += 1;
                }
            }
        }
        println!("Rocket mint acquisition: {sources} parsed sources, {positioned} map navigation references (access remains qualified)");
        let mut save = Save::open(save_bytes(&rom), rom.profile.save).unwrap();
        let first = save.sections[1];
        save.data[first + save.layout.party_count] = 1;
        let at = first + save.layout.party;
        let mut edited = 0;
        for row in vectors["rows"].as_array().unwrap() {
            let raw: Vec<u8> = serde_json::from_value(row["before"].clone()).unwrap();
            let expected: Vec<u8> = serde_json::from_value(row["after"].clone()).unwrap();
            let item = row["item"].as_u64().unwrap() as u16;
            let nature = row["nature"].as_u64().unwrap() as u8;
            let no_effect = row["no_effect"].as_bool().unwrap();
            save.data[at..at + 100].copy_from_slice(&raw);
            let original = save.data.clone();
            let preview = rom
                .training_preview(
                    Some(&save),
                    Request {
                        expected_rom_md5: rom.profile.md5.into(),
                        item,
                        rng_seed: None,
                        individual: Individual::Stored { location: party() },
                    },
                )
                .unwrap();
            assert_eq!(
                preview.raw, expected,
                "item {item}, PID {}",
                preview.before.pid
            );
            assert_eq!(preview.native_no_effect, no_effect);
            assert_eq!(preview.effect_scope, "nature_persistent_stage");
            for i in 0..6 {
                assert_eq!(
                    preview.party_stats_before[i],
                    crate::binary::u16(&raw, 88 + i * 2).unwrap()
                );
                assert_eq!(
                    preview.party_stats_after[i],
                    crate::binary::u16(&expected, 88 + i * 2).unwrap()
                );
            }
            assert_eq!(preview.before.pid, preview.after.pid);
            assert_eq!(preview.before.ot_id, preview.after.ot_id);
            assert_eq!(preview.before.evs, preview.after.evs);
            assert_eq!(preview.before.ivs, preview.after.ivs);
            assert_eq!(preview.before.ball, preview.after.ball);
            assert_eq!(save.data, original);
            if !no_effect {
                let patch = PokemonPatch {
                    nature_override: Some(nature),
                    ..Default::default()
                };
                let (party, _) = pokemon::edit(&raw, &patch, &rom, Policy::Standard).unwrap();
                let (boxed, _) = pokemon::edit(&raw[..80], &patch, &rom, Policy::Standard).unwrap();
                assert_eq!(party, expected, "party editor item {item}");
                assert_eq!(boxed, expected[..80], "box editor item {item}");
                edited += 1;
            } else {
                assert_eq!(expected, raw);
            }
        }
        assert_eq!(crate::binary::sha256(&rom.data), hash);
        assert_eq!(crate::binary::sha256(&std::fs::read(path).unwrap()), hash);
        println!("Rocket: {} mint scenarios; {edited} native-equivalent party and boxed nature edits; original ROM/SAV unchanged", vectors["rows"].as_array().unwrap().len());
    }
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_TRAINING_ABILITY_PROBES independent mGBA vectors"]
fn local_training_ability_items_match_native_and_keep_profiles_isolated() {
    use crate::training::{ability_decision, AbilityExecution, Individual, Request};
    use armv4t_emu::Memory;
    let vectors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_TRAINING_ABILITY_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let rom = Rom::open(std::fs::read(&path).unwrap()).unwrap();
        let hash = crate::binary::sha256(&rom.data);
        let catalog = rom.training_catalog().unwrap();
        assert_eq!(catalog.offers.len(), 12);
        assert_eq!(
            catalog.nature_items.len(),
            if key == "ROCKET" { 21 } else { 0 }
        );
        if !matches!(key, "ROCKET" | "MERCURY12" | "ULTIMATE") {
            assert!(
                catalog.ability_items.is_empty(),
                "unverified handlers must not cross profiles"
            );
            continue;
        }
        let evidence = &vectors[key];
        assert_eq!(evidence["md5"], rom.profile.md5);
        assert_eq!(
            catalog.ability_items.len(),
            evidence["offers"].as_array().unwrap().len()
        );
        let b = rom.profile.breeding.unwrap();
        let mut sources = 0;
        let mut positioned = 0;
        let index = crate::acquisition::AcquisitionIndex::build(&rom).unwrap();
        for offer in &catalog.ability_items {
            assert!(evidence["offers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["item"] == offer.item && v["handler"] == offer.handler));
            let rule = rom
                .profile
                .training
                .unwrap()
                .abilities
                .iter()
                .find(|r| r.handler == offer.handler)
                .unwrap();
            assert_eq!(
                offer.random_pid,
                matches!(rule.execution, AbilityExecution::Pid { .. })
            );
            let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
            for guard in evidence["guards"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v["item"] == offer.item)
            {
                let raw: Vec<u8> = serde_json::from_value(guard["before"].clone()).unwrap();
                for (i, byte) in raw.iter().enumerate() {
                    ram.w8(b.party + i as u32, *byte);
                }
                ram.w32(b.rng, 42);
                let decision = ability_decision(&mut ram, rule, offer.item).unwrap();
                assert_eq!(decision.accepted, guard["accepted"].as_bool().unwrap());
                assert_eq!(decision.target, guard["target"].as_u64().unwrap() as u32);
                assert_eq!(
                    (0..100).map(|i| ram.r8(b.party + i)).collect::<Vec<_>>(),
                    raw
                );
            }
            let report = index
                .query(
                    &rom,
                    None,
                    crate::acquisition::Target {
                        kind: crate::acquisition::TargetKind::Item,
                        id: offer.item,
                    },
                )
                .unwrap();
            for source in &report.sources {
                sources += 1;
                if let Some(map) = &source.map_id {
                    assert_eq!(rom.map_navigation(map).unwrap().map_id, *map);
                    positioned += 1;
                }
            }
        }
        let mut save = Save::open(save_bytes(&rom), rom.profile.save).unwrap();
        let first = save.sections[1];
        save.data[first + save.layout.party_count] = 1;
        let at = first + save.layout.party;
        let mut editor_matches = 0;
        let mut changed_pids = 0;
        let mut identity_differences = 0;
        for row in evidence["rows"].as_array().unwrap() {
            let raw: Vec<u8> = serde_json::from_value(row["before"].clone()).unwrap();
            let expected: Vec<u8> = serde_json::from_value(row["after"].clone()).unwrap();
            save.data[at..at + 100].copy_from_slice(&raw);
            let old = save.data.clone();
            let item = row["item"].as_u64().unwrap() as u16;
            let seed = catalog
                .ability_items
                .iter()
                .find(|o| o.item == item && o.requires_rng_seed)
                .map(|_| row["seed"].as_u64().unwrap() as u32);
            let actual = rom
                .training_preview(
                    Some(&save),
                    Request {
                        expected_rom_md5: rom.profile.md5.into(),
                        item,
                        rng_seed: seed,
                        individual: Individual::Stored { location: party() },
                    },
                )
                .unwrap();
            assert_eq!(
                actual.raw, expected,
                "{key} item {item}, PID {}",
                actual.before.pid
            );
            assert_eq!(actual.native_no_effect, !row["accepted"].as_bool().unwrap());
            assert_eq!(actual.changed, raw != expected);
            assert_eq!(actual.effect_scope, "ability_persistent_stage");
            assert_eq!(actual.rng_seed, seed);
            if seed.is_some() {
                assert_eq!(
                    actual.rng_after,
                    Some(row["rng_after"].as_u64().unwrap() as u32)
                );
            }
            assert_eq!(actual.before.ot_id, actual.after.ot_id);
            assert_eq!(actual.before.ivs, actual.after.ivs);
            assert_eq!(actual.before.evs, actual.after.evs);
            assert_eq!(actual.before.moves, actual.after.moves);
            assert_eq!(save.data, old);
            changed_pids += usize::from(actual.before.pid != actual.after.pid);
            identity_differences += usize::from(
                actual.before.nature != actual.after.nature
                    || actual.before.gender != actual.after.gender
                    || actual.before.shiny != actual.after.shiny,
            );
            if !actual.native_no_effect {
                let native_target = if key == "ROCKET" {
                    rom.species(actual.before.species).unwrap().abilities
                        [row["target"].as_u64().unwrap() as usize]
                } else if key == "ULTIMATE" {
                    let target = row["target"].as_u64().unwrap() as usize;
                    rom.species(actual.before.species).unwrap().abilities
                        [if target >= 2 { 2 } else { target }]
                } else {
                    row["target"].as_u64().unwrap() as u16
                };
                assert_eq!(actual.ability_target, Some(native_target));
                if key == "ROCKET" {
                    let patch = PokemonPatch {
                        ability_slot: Some(row["target"].as_u64().unwrap() as u8),
                        ..Default::default()
                    };
                    assert_eq!(
                        pokemon::edit(&raw, &patch, &rom, Policy::Standard)
                            .unwrap()
                            .0,
                        expected
                    );
                    assert_eq!(
                        pokemon::edit(&raw[..80], &patch, &rom, Policy::Standard)
                            .unwrap()
                            .0,
                        expected[..80]
                    );
                    editor_matches += 1;
                }
                if key == "ULTIMATE" && actual.after.ability_id != 0 {
                    let patch = PokemonPatch {
                        ability_slot: Some(actual.after.ability_slot),
                        ..Default::default()
                    };
                    assert_eq!(
                        pokemon::edit(&raw[..80], &patch, &rom, Policy::Standard)
                            .unwrap()
                            .0,
                        expected[..80]
                    );
                    editor_matches += 1;
                }
            } else {
                assert!(actual.ability_target.is_none());
            }
        }
        let item = catalog.ability_items[0].item;
        for seeded in catalog.ability_items.iter().filter(|o| o.requires_rng_seed) {
            assert_eq!(
                rom.training_preview(
                    Some(&save),
                    Request {
                        expected_rom_md5: rom.profile.md5.into(),
                        item: seeded.item,
                        rng_seed: None,
                        individual: Individual::Stored { location: party() },
                    }
                )
                .unwrap_err()
                .code,
                "training_seed_required"
            );
        }
        let invalid = serde_json::json!({"expected_rom_md5":rom.profile.md5,"item":item,"rng_seed":42,
            "individual":{"kind":"simulated","species":1,"level":5,"evs":[0,0,0,0,0,0],"friendship":70,"ability_slot":4}});
        assert_eq!(
            rom.training_preview(None, serde_json::from_value(invalid).unwrap())
                .unwrap_err()
                .code,
            "range"
        );
        assert_eq!(crate::binary::sha256(&rom.data), hash);
        assert_eq!(crate::binary::sha256(&std::fs::read(path).unwrap()), hash);
        println!("{key}: {} native guards; {} persistent scenarios; {editor_matches} exact existing editor matches (Rocket party/box; Ultimate box only); {changed_pids} changed PIDs; {identity_differences} nature/gender/shiny differences; {sources} parsed item sources, {positioned} map refs; original ROM/SAV unchanged", evidence["guards"].as_array().unwrap().len(), evidence["rows"].as_array().unwrap().len());
    }
}

#[test]
fn training_requests_require_typed_rng_and_ability_scenarios() {
    let base = serde_json::json!({"expected_rom_md5":"fixture","item":1,
        "individual":{"kind":"simulated","species":1,"level":5,"evs":[0,0,0,0,0,0],"friendship":70}});
    let parsed: crate::training::Request = serde_json::from_value(base.clone()).unwrap();
    assert!(parsed.rng_seed.is_none());
    for seed in [
        serde_json::json!(-1),
        serde_json::json!(4294967296u64),
        serde_json::json!(0.5),
        serde_json::json!("42"),
        serde_json::json!(true),
    ] {
        let mut invalid = base.clone();
        invalid["rng_seed"] = seed;
        assert!(serde_json::from_value::<crate::training::Request>(invalid).is_err());
    }
    for seed in [0u32, u32::MAX] {
        let mut valid = base.clone();
        valid["rng_seed"] = serde_json::json!(seed);
        assert_eq!(
            serde_json::from_value::<crate::training::Request>(valid)
                .unwrap()
                .rng_seed,
            Some(seed)
        );
    }
    for ability in [
        serde_json::json!(-1),
        serde_json::json!(256),
        serde_json::json!("1"),
        serde_json::json!(0.5),
    ] {
        let mut invalid = base.clone();
        invalid["individual"]["ability_slot"] = ability;
        assert!(serde_json::from_value::<crate::training::Request>(invalid).is_err());
    }
}

#[test]
fn crown_services_are_read_only_and_reject_stale_profiles() {
    use crate::app::{App, Request};
    let mut app = App {
        session: Some(Session::new(rom())),
        ..Default::default()
    };
    assert_eq!(
        app.dispatch(Request {
            command: "training_services".into(),
            payload: serde_json::json!({"expected_rom_md5": profile::ROCKET.md5})
        })
        .unwrap_err()
        .code,
        "rom_mismatch"
    );
    let result = app
        .dispatch(Request {
            command: "training_services".into(),
            payload: serde_json::json!({"expected_rom_md5": profile::BW.md5}),
        })
        .unwrap();
    assert_eq!(result["services"], serde_json::json!([]));
    assert_eq!(result["partial"], true);
    assert!(app.session.as_ref().unwrap().save.is_none());
}

#[test]
#[ignore = "requires five exact private ROMs and GEN3_TRAINING_CROWN_PROBES independent mGBA vectors"]
fn local_crown_services_match_native_and_isolate_save_progress() {
    use armv4t_emu::Memory;
    let vectors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_TRAINING_CROWN_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let rom = Rom::open(std::fs::read(&path).unwrap()).unwrap();
        let digest = sha256(&rom.data);
        let report = rom.training_services(None).unwrap();
        assert!(report.partial);
        assert_eq!(report.rom_md5, rom.profile.md5);
        if key == "MERCURY12" {
            assert_eq!(report.services.len(), 1);
            assert_eq!(report.services[0].kind, "base_iv_training");
            continue;
        }
        if key != "ULTIMATE" {
            assert!(
                report.services.is_empty(),
                "unverified crown services must not cross profiles"
            );
            continue;
        }
        assert_eq!(vectors["rom_md5"], rom.profile.md5);
        assert_eq!(report.services.len(), 1);
        let service = &report.services[0];
        assert_eq!(service.minimum_level, 100);
        assert_eq!(service.choices.len(), 7);
        assert_eq!(service.locations.len(), 1);
        let source = &service.locations[0];
        assert_eq!(
            (&*source.map_id, source.x, source.y, source.local_id),
            ("35-28", 3, 39, Some(19))
        );
        assert!(service.text[0].contains("满级"));
        assert!(service.text[3].contains("放入电脑"));
        for choice in &service.choices {
            assert_eq!(choice.quantity, 1);
            assert_eq!(choice.credit.as_ref().unwrap().satisfied, None);
            assert_eq!(choice.item_requirement.satisfied, None);
            assert_eq!(choice.item, if choice.menu_index == 0 { 687 } else { 688 });
            assert_eq!(
                choice.credit.as_ref().unwrap().condition.id,
                if choice.menu_index == 0 {
                    0x40fb
                } else {
                    0x40fc
                }
            );
        }
        // SAVE overlays follow the actual certificate variables, including a
        // logical-sector boundary; the ROM-only catalogue remains unchanged.
        let before_report = serde_json::to_value(&report).unwrap();
        let mut save = Save::open(save_bytes(&rom), rom.profile.save).unwrap();
        for amount in [0, 1, u16::MAX] {
            let range = rom.profile.event_state.unwrap().variables[0];
            for id in [0x40fb, 0x40fc] {
                let mut offset = range.offset + (id - range.first) as usize * 2;
                for section in 1..=4 {
                    if offset < save.layout.sizes[section] {
                        put16(&mut save.data, save.sections[section] + offset, amount);
                        break;
                    }
                    offset -= save.layout.sizes[section];
                }
            }
            let saved = save.data.clone();
            let overlay = rom.training_services(Some(&save)).unwrap();
            for choice in &overlay.services[0].choices {
                assert_eq!(choice.credit.as_ref().unwrap().actual, Some(amount as u32));
                assert_eq!(choice.credit.as_ref().unwrap().satisfied, Some(amount != 0));
            }
            assert_eq!(save.data, saved);
        }
        assert_eq!(
            serde_json::to_value(rom.training_services(None).unwrap()).unwrap(),
            before_report
        );
        let crate::training_services::ServiceRules::Flags(rules) = service.evidence else {
            panic!("wrong service mechanism");
        };
        let party = rom.profile.breeding.unwrap().party;
        let raw =
            pokemon::to_party(&pokemon::create(&rom, 25, 1, "", 100, 42).unwrap(), &rom).unwrap();
        let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
        for row in vectors["levels"].as_array().unwrap() {
            let slot = row["slot"].as_u64().unwrap() as u32;
            let level = row["level"].as_u64().unwrap() as u8;
            ram.w8(party + slot * 100 + 84, level);
            ram.w16(rules.selected_individual, slot as u16);
            ram.w16(rules.selected_individual + 2, 84);
            ram.call(rules.level_reader, [0; 4], [0; 2], 8192).unwrap();
            assert_eq!(
                ram.r16(rules.selected_individual + 2) as u64,
                row["actual"].as_u64().unwrap()
            );
        }
        assert_eq!(vectors["rows"].as_array().unwrap().len(), 10_752);
        for row in vectors["rows"].as_array().unwrap() {
            let slot = row["slot"].as_u64().unwrap() as u32;
            let header = row["header"].as_u64().unwrap() as u8;
            let menu = row["menu_index"].as_u64().unwrap() as u8;
            let mut expected = raw.repeat(6);
            expected[slot as usize * 100 + 30] = header;
            for (i, byte) in expected.iter().enumerate() {
                ram.w8(party + i as u32, *byte);
            }
            ram.w16(rules.selected_individual, slot as u16);
            ram.w16(
                rules.selected_individual + 2,
                if menu == 0 { 126 } else { 1 },
            );
            if menu != 0 {
                ram.w8(rules.selected_choice, menu);
                ram.call(rules.selection_mask, [0; 4], [0; 2], 8192)
                    .unwrap();
            }
            assert_eq!(
                ram.r16(rules.selected_individual + 2) as u64,
                row["mask"].as_u64().unwrap()
            );
            ram.call(rules.mark, [0; 4], [0; 2], 8192).unwrap();
            expected[slot as usize * 100 + 30] = row["current"].as_u64().unwrap() as u8;
            let actual: Vec<u8> = (0..600).map(|i| ram.r8(party + i)).collect();
            assert_eq!(actual, expected, "unrelated party data changed");
            assert_eq!(
                ram.r8(rules.selected_individual + 4) as u64,
                row["previous"].as_u64().unwrap()
            );
            assert_eq!(
                ram.r8(rules.selected_individual + 6) as u64,
                row["current"].as_u64().unwrap()
            );
        }
        assert_eq!(sha256(&std::fs::read(path).unwrap()), digest);
        eprintln!("{key}: 48 native readers, 10,752 native crown mutations, 7 positioned offers and certificate overlays verified");
    }
}

#[test]
#[ignore = "requires exact private Mercury 1.2 ROM and GEN3_MERCURY_CROWN_PROBES independent mGBA vectors"]
fn local_mercury_crown_services_match_native_fields_and_payment_anomaly() {
    use armv4t_emu::Memory;
    let r =
        Rom::open(std::fs::read(std::env::var("GEN3_ROM_MERCURY12").unwrap()).unwrap()).unwrap();
    let vectors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_MERCURY_CROWN_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    let digest = sha256(&r.data);
    let report = r.training_services(None).unwrap();
    assert_eq!(report.services.len(), 1);
    let service = &report.services[0];
    assert_eq!(service.kind, "base_iv_training");
    assert_eq!(service.minimum_level, 50);
    assert_eq!(service.choices.len(), 7);
    assert_eq!(
        (
            &*service.locations[0].map_id,
            service.locations[0].x,
            service.locations[0].y,
            service.locations[0].local_id
        ),
        ("46-0", 14, 10, Some(20))
    );
    assert_eq!(service.conditions[0].condition.id, 0xb15);
    assert_eq!(service.conditions[0].satisfied, None);
    for choice in &service.choices {
        assert!(choice.credit.is_none());
        assert_eq!(choice.item, if choice.menu_index == 0 { 640 } else { 639 });
        assert_eq!(choice.payment.item, 640);
        assert_eq!(choice.payment.before_stat_selection, choice.menu_index != 0);
        assert!(!choice.payment.result_checked);
        assert_eq!(
            choice.stat,
            (choice.menu_index != 0).then(|| choice.menu_index as usize - 1)
        );
    }
    let crate::training_services::ServiceRules::BaseIvs(rules) = service.evidence else {
        panic!("wrong mechanism");
    };
    let party = r.profile.breeding.unwrap().party;
    let mut ram = crate::native_trainer::Sandbox::new(&r.data);
    assert_eq!(vectors["rom_md5"], r.profile.md5);
    assert_eq!(vectors["levels"].as_array().unwrap().len(), 216);
    assert_eq!(vectors["rows"].as_array().unwrap().len(), 1512);
    let array = |row: &serde_json::Value, name: &str| {
        row[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect::<Vec<_>>()
    };
    for row in vectors["levels"].as_array().unwrap() {
        let slot = row["slot"].as_u64().unwrap() as u32;
        let raw = array(row, "before").repeat(6);
        for (i, byte) in raw.iter().enumerate() {
            ram.w8(party + i as u32, *byte);
        }
        ram.w16(rules.selected_individual, slot as u16);
        ram.call(rules.level_reader, [0; 4], [0; 2], 100_000)
            .unwrap();
        assert_eq!(
            ram.r16(rules.selected_individual + 4) as u64,
            row["actual"].as_u64().unwrap()
        );
        assert_eq!((0..600).map(|i| ram.r8(party + i)).collect::<Vec<_>>(), raw);
        for (branch, check_at) in rules.level_checks.iter().enumerate() {
            for (op, at) in [(0x21u32, *check_at), (0x06, *check_at + 5)] {
                let entry = crate::binary::u32(&r.data, 0x15f9b4 + op as usize * 4).unwrap() & !1;
                ram.w32(0x02010008, 0x08000000 + at as u32 + 1);
                ram.call(entry, [0x02010000, 0, 0, 0], [0; 2], 100_000)
                    .unwrap();
            }
            assert_eq!(
                ram.r32(0x02010008) == 0x08000000 + *check_at as u32 + 11,
                row["accepted"][branch].as_bool().unwrap()
            );
        }
    }
    for row in vectors["rows"].as_array().unwrap() {
        let slot = row["slot"].as_u64().unwrap() as u32;
        let mut raw = array(row, "before").repeat(6);
        for (i, byte) in raw.iter().enumerate() {
            ram.w8(party + i as u32, *byte);
        }
        ram.w16(rules.selected_individual - 2, 0);
        ram.w16(rules.selected_individual, slot as u16);
        ram.w16(
            rules.selected_individual + 2,
            row["selector"].as_u64().unwrap() as u16,
        );
        ram.w16(rules.selected_individual + 4, 31);
        ram.call(rules.setter, [0; 4], [0; 2], 100_000).unwrap();
        raw[slot as usize * 100..slot as usize * 100 + 100].copy_from_slice(&array(row, "after"));
        assert_eq!((0..600).map(|i| ram.r8(party + i)).collect::<Vec<_>>(), raw);
    }

    for slot in vectors["invalid_slots"].as_array().unwrap() {
        let before: Vec<u8> = (0..600).map(|i| ram.r8(party + i)).collect();
        ram.w16(rules.selected_individual, slot.as_u64().unwrap() as u16);
        ram.w16(rules.selected_individual + 2, 6);
        ram.w16(rules.selected_individual + 4, 31);
        ram.call(rules.setter, [0; 4], [0; 2], 100_000).unwrap();
        assert_eq!(
            (0..600).map(|i| ram.r8(party + i)).collect::<Vec<_>>(),
            before
        );
        ram.call(rules.level_reader, [0; 4], [0; 2], 100_000)
            .unwrap();
        assert_eq!(ram.r16(rules.selected_individual + 4), 0);
    }
    let mut ram = crate::native_trainer::Sandbox::new(&r.data);
    ram.w32(0x03005008, 0x02020000);
    ram.w32(0x0300500c, 0x02030000);
    let result = ram
        .call(0x0806e454, [0x800d, 0, 0, 0], [0; 2], 100_000)
        .unwrap();
    assert_eq!(vectors["payments"].as_array().unwrap().len(), 32);
    for row in vectors["payments"].as_array().unwrap() {
        let silver = row["silver"].as_u64().unwrap() as u16;
        let gold = row["gold"].as_u64().unwrap() as u16;
        for i in 0..40 {
            ram.w8(0x02009000 + i, 0);
        }
        ram.w16(0x02009000, 639);
        ram.w16(0x02009002, silver);
        ram.w16(0x02009004, 640);
        ram.w16(0x02009006, gold);
        ram.w32(0x0203988c, 0x02009000);
        ram.w16(0x02039890, 10);
        let branch = usize::from(row["branch"] == "silver");
        for (op, at, expected) in [
            (0x47u32, rules.item_checks[branch], "check_result"),
            (0x45, rules.payments[branch], "remove_result"),
        ] {
            let entry = crate::binary::u32(&r.data, 0x15f9b4 + op as usize * 4).unwrap() & !1;
            ram.w32(0x02010008, 0x08000000 + at as u32 + 1);
            ram.w16(result, 7);
            ram.call(entry, [0x02010000, 0, 0, 0], [0; 2], 100_000)
                .unwrap();
            assert_eq!(ram.r16(result) as u64, row[expected].as_u64().unwrap());
        }
        let mut actual = std::collections::BTreeMap::new();
        for i in 0..10 {
            let item = ram.r16(0x02009000 + i * 4);
            if item != 0 {
                actual.insert(item.to_string(), ram.r16(0x02009002 + i * 4));
            }
        }
        assert_eq!(serde_json::to_value(actual).unwrap(), row["bag"]);
    }
    // Current SAVE unlock overlays are separate from the ROM-only reference.
    let base_report = serde_json::to_value(&report).unwrap();
    let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let range = r
        .profile
        .event_state
        .unwrap()
        .flags
        .iter()
        .find(|v| 0xb15 >= v.first && 0xb15 - v.first < v.count)
        .unwrap();
    assert!(matches!(
        range.block,
        crate::event_state::EventBlock::Extensions
    ));
    let bit = (0xb15 - range.first) as usize;
    let logical = range.offset + bit / 8;
    assert!(logical < 0xff0 - save.layout.sizes[0]);
    let absolute = save.sections[0] + save.layout.sizes[0] + logical;
    for set in [false, true] {
        save.data[absolute] =
            (save.data[absolute] & !(1 << (bit % 8))) | (u8::from(set) << (bit % 8));
        let before = save.data.clone();
        assert_eq!(
            r.training_services(Some(&save)).unwrap().services[0].conditions[0].satisfied,
            Some(set)
        );
        assert_eq!(save.data, before);
    }

    let pocket = r
        .profile
        .save
        .pockets
        .iter()
        .find(|p| p.category == 1)
        .unwrap()
        .id;
    for amount in [0, 1, 2] {
        save.edit_bag(
            pocket,
            0,
            if amount == 0 { 0 } else { 639 },
            amount,
            &r,
            Policy::Standard,
        )
        .unwrap();
        let before = save.data.clone();
        let overlay = r.training_services(Some(&save)).unwrap();
        assert_eq!(
            overlay.services[0].choices[1].item_requirement.satisfied,
            Some(amount >= 1)
        );
        assert_eq!(save.data, before);
    }
    assert_eq!(
        serde_json::to_value(r.training_services(None).unwrap()).unwrap(),
        base_report
    );
    assert_eq!(sha256(&r.data), digest);
}

#[test]
fn service_previews_reject_unknown_sources_and_stale_identity_without_writes() {
    use crate::app::{App, Request};
    let rom = rom();
    let mut app = App {
        session: Some(session()),
        ..Default::default()
    };
    let saved = app
        .session
        .as_ref()
        .unwrap()
        .save
        .as_ref()
        .unwrap()
        .data
        .clone();
    let request = |fingerprint: &str| serde_json::json!({"expected_rom_md5":fingerprint,"service_root":1,"choice_index":0,"individual":{"kind":"stored","location":{"kind":"party","slot":0}}});
    assert_eq!(
        app.dispatch(Request {
            command: "training_service_preview".into(),
            payload: request("stale")
        })
        .err()
        .unwrap()
        .code,
        "rom_mismatch"
    );
    assert_eq!(
        app.dispatch(Request {
            command: "training_service_preview".into(),
            payload: request(rom.profile.md5)
        })
        .err()
        .unwrap()
        .code,
        "training_service_unverified"
    );
    assert!(serde_json::from_value::<crate::training_service_preview::Request>(serde_json::json!({"expected_rom_md5":rom.profile.md5,"service_root":null,"choice_index":0,"individual":{"kind":"stored","location":{"kind":"party","slot":0}}})).is_err());
    assert_eq!(
        app.session.as_ref().unwrap().save.as_ref().unwrap().data,
        saved
    );
}

#[test]
#[ignore = "requires five exact ROMs, GEN3_MERCURY_CROWN_PROBES and GEN3_TRAINING_CROWN_PROBES independent native vectors"]
fn local_service_previews_match_native_and_keep_reference_save_read_only() {
    use crate::training::Individual;
    use crate::training_service_preview::Request;
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let rom =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        let hash = sha256(&rom.data);
        let mut save = Save::open(save_bytes(&rom), rom.profile.save).unwrap();
        let report = rom.training_services(None).unwrap();
        if !matches!(key, "ULTIMATE" | "MERCURY12") {
            assert!(report.services.is_empty());
            assert_eq!(
                rom.training_service_preview(
                    None,
                    Request {
                        expected_rom_md5: rom.profile.md5.into(),
                        service_root: 1,
                        choice_index: 0,
                        individual: Individual::Stored { location: party() }
                    }
                )
                .err()
                .unwrap()
                .code,
                "training_service_unverified"
            );
            continue;
        }
        let service = &report.services[0];
        let root = match service.evidence {
            crate::training_services::ServiceRules::Flags(r) => r.root,
            crate::training_services::ServiceRules::BaseIvs(r) => r.root,
        };
        let at = save.sections[1] + save.layout.party;
        if key == "MERCURY12" {
            let vectors: serde_json::Value = serde_json::from_slice(
                &std::fs::read(std::env::var("GEN3_MERCURY_CROWN_PROBES").unwrap()).unwrap(),
            )
            .unwrap();
            let mut count = 0;
            for row in vectors["rows"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["slot"] == 0)
            {
                let before: Vec<u8> = serde_json::from_value(row["before"].clone()).unwrap();
                let native: Vec<u8> = serde_json::from_value(row["after"].clone()).unwrap();
                save.data[at..at + 100].copy_from_slice(&before);
                let old = save.data.clone();
                let selector = row["selector"].as_u64().unwrap() as u8;
                let choice_index = if selector == 6 { 0 } else { selector + 1 };
                let preview = rom
                    .training_service_preview(
                        Some(&save),
                        Request {
                            expected_rom_md5: rom.profile.md5.into(),
                            service_root: root,
                            choice_index,
                            individual: Individual::Stored { location: party() },
                        },
                    )
                    .unwrap();
                let level_ok = before[84] >= 50;
                assert_eq!(preview.level_satisfied, level_ok);
                assert_eq!(preview.raw, if level_ok { native } else { before });
                assert_eq!(preview.stat_refresh, "immediate");
                assert_eq!(preview.before.pid, preview.after.pid);
                assert_eq!(preview.before.ot_id, preview.after.ot_id);
                assert_eq!(save.data, old);
                count += 1;
            }
            assert_eq!(count, 252);
        } else {
            let raw = pokemon::to_party(&pokemon::create(&rom, 25, 1, "", 100, 42).unwrap(), &rom)
                .unwrap();
            for level in [99, 100] {
                for header in [0, 1, 0x56, 0xff] {
                    for choice in &service.choices {
                        let mut before = raw.clone();
                        before[30] = header;
                        before[84] = level;
                        save.data[at..at + 100].copy_from_slice(&before);
                        let old = save.data.clone();
                        let preview = rom
                            .training_service_preview(
                                Some(&save),
                                Request {
                                    expected_rom_md5: rom.profile.md5.into(),
                                    service_root: root,
                                    choice_index: choice.menu_index,
                                    individual: Individual::Stored { location: party() },
                                },
                            )
                            .unwrap();
                        let mut expected = before;
                        if level >= 100 {
                            expected[30] |= choice.mask;
                        }
                        assert_eq!(preview.raw, expected);
                        assert_eq!(preview.stat_refresh, "deferred");
                        assert_eq!(preview.before.ivs, preview.after.ivs);
                        assert_eq!(preview.party_stats_before, preview.party_stats_after);
                        assert_eq!(save.data, old);
                    }
                }
            }
        }
        // ROM-only simulations and boxed individuals share preparation semantics.
        for level in [service.minimum_level as u8 - 1, service.minimum_level as u8] {
            let preview = rom
                .training_service_preview(
                    None,
                    Request {
                        expected_rom_md5: rom.profile.md5.into(),
                        service_root: root,
                        choice_index: 0,
                        individual: Individual::Simulated {
                            species: 25,
                            level,
                            evs: [0; 6],
                            friendship: 70,
                            ivs: Some([13; 6]),
                            nature_override: None,
                            ability_slot: None,
                        },
                    },
                )
                .unwrap();
            assert_eq!(preview.before.ivs, [13; 6]);
            assert!(!preview.withdrawal_required);
            assert_eq!(
                preview.level_satisfied,
                level >= service.minimum_level as u8
            );
            assert_eq!(
                preview.known_requirements_met,
                if level < service.minimum_level as u8 {
                    Some(false)
                } else {
                    None
                }
            );
        }
        let boxed = pokemon::create(&rom, 25, 1, "", service.minimum_level as u8, 42).unwrap();
        let loc = Location::Box {
            box_index: 0,
            slot: 0,
        };
        save.insert(loc, &boxed, &rom).unwrap();
        let old = save.data.clone();
        let preview = rom
            .training_service_preview(
                Some(&save),
                Request {
                    expected_rom_md5: rom.profile.md5.into(),
                    service_root: root,
                    choice_index: 0,
                    individual: Individual::Stored { location: loc },
                },
            )
            .unwrap();
        assert_eq!(preview.party_state, "boxed_full_hp_scenario");
        assert!(preview.withdrawal_required);
        assert_eq!(save.data, old);
        assert_eq!(
            rom.training_service_preview(
                Some(&save),
                Request {
                    expected_rom_md5: rom.profile.md5.into(),
                    service_root: root,
                    choice_index: 7,
                    individual: Individual::Stored { location: loc }
                }
            )
            .err()
            .unwrap()
            .code,
            "training_service_choice"
        );
        assert_eq!(sha256(&rom.data), hash);
    }
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_TRAINING_SERVICE_MENU_PROBES independent native evidence"]
fn local_training_service_menus_match_native_input_decisions() {
    use armv4t_emu::Memory;
    let vectors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_TRAINING_SERVICE_MENU_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let rom =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        let digest = sha256(&rom.data);
        let report = rom.training_services(None).unwrap();
        if !matches!(key, "ULTIMATE" | "MERCURY12") {
            assert!(report.services.is_empty());
            continue;
        }
        let v = &vectors[key];
        assert_eq!(v["md5"], rom.profile.md5);
        let rule = &v["rules"];
        let number = |name: &str| rule[name].as_u64().unwrap() as u32;
        let menus = &report.services[0].menus;
        assert_eq!(menus.len(), if key == "MERCURY12" { 2 } else { 1 });
        for entry in v["entries"].as_array().unwrap() {
            let at = entry["command"].as_u64().unwrap() as usize;
            let menu = menus.iter().find(|m| m.command == at).unwrap();
            let params: Vec<u32> = entry["params"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u32)
                .collect();
            assert_eq!(menu.cancel_with_b, params[3] & number("ignore_b_mask") == 0);
            assert_eq!(menu.payment_precedes_menu, menu.stage == "stat_choice");
            assert_eq!(menu.single_stat_only, menu.stage == "stat_choice");
            let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
            ram.w32(0x02010008, 0x08000000 + at as u32 + 1);
            let entry = u32(&rom.data, number("commands") as usize + 0x6f * 4).unwrap() & !1;
            let regs = ram
                .observe(
                    entry,
                    [0x02010000, 0, 0, 0],
                    [0; 2],
                    10_000,
                    number("menu_entry"),
                )
                .unwrap();
            assert_eq!(regs[..4], params);
        }
        assert_eq!(
            v["rows"].as_array().unwrap().len(),
            if key == "MERCURY12" { 32 } else { 28 }
        );
        for row in v["rows"].as_array().unwrap() {
            let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
            let flags = row["params"][3].as_u64().unwrap() as u32;
            let id = row["params"][2].as_u64().unwrap() as u32;
            let count = row["count"].as_u64().unwrap() as u32;
            ram.call(
                number("create"),
                [flags & number("ignore_b_mask"), count, 0, id],
                [0; 2],
                100_000,
            )
            .unwrap();
            assert_eq!(ram.r8(number("tasks") + 4), 1);
            assert_eq!(
                ram.r16(number("tasks") + 16),
                (flags & number("ignore_b_mask")) as u16
            );
            for _ in 0..row["startup_ticks"].as_u64().unwrap() {
                ram.call(number("callback"), [0; 4], [0; 2], 100_000)
                    .unwrap();
            }
            assert_eq!(ram.r8(number("delay")), 0);
            ram.w8(number("cursor") + 2, row["cursor"].as_u64().unwrap() as u8);
            ram.w8(number("cursor") + 11, 1);
            ram.w16(number("keys"), row["keys"].as_u64().unwrap() as u16);
            ram.w16(number("result"), 255);
            let regs = ram
                .observe(
                    number("callback"),
                    [0; 4],
                    [0; 2],
                    100_000,
                    row["stop"].as_u64().unwrap() as u32,
                )
                .unwrap();
            if row["kind"] == "selected" {
                assert_eq!(regs[1], row["cursor"].as_u64().unwrap() as u32);
            }
            assert_eq!(ram.r16(number("result")), 255);
            if row["kind"] == "cancel" {
                assert_eq!(row["cancel_result"], 127);
            }
        }
        for path in v["cancel_paths"].as_array().unwrap() {
            let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
            ram.w16(number("result"), 127);
            for step in path["trace"].as_array().unwrap() {
                let at = step["command"].as_u64().unwrap() as usize;
                let op = rom.data[at];
                assert!(matches!(op, 0x21 | 0x05 | 0x06));
                let native =
                    u32(&rom.data, number("commands") as usize + op as usize * 4).unwrap() & !1;
                ram.w32(0x02010008, 0x08000000 + at as u32 + 1);
                ram.call(native, [0x02010000, 0, 0, 0], [0; 2], 100_000)
                    .unwrap();
                assert_eq!(
                    ram.r32(0x02010008),
                    0x08000000 + step["next"].as_u64().unwrap() as u32
                );
            }
            assert_eq!(rom.data[path["dialogue"].as_u64().unwrap() as usize], 0x0f);
        }
        assert_eq!(v["transforms"].as_array().unwrap().len(), 256);
        for row in v["transforms"].as_array().unwrap() {
            let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
            let flags = row["flags"].as_u64().unwrap() as u32;
            ram.w32(0x03007e00 + number("transform_local"), flags);
            let regs = ram
                .observe(
                    number("transform_start"),
                    [0; 4],
                    [
                        if number("transform_local") == 0 {
                            flags
                        } else {
                            0
                        },
                        0,
                    ],
                    1000,
                    number("transform_stop"),
                )
                .unwrap();
            assert_eq!(
                regs[number("transform_reg") as usize],
                row["actual"].as_u64().unwrap() as u32
            );
            assert_eq!(
                row["actual"].as_u64().unwrap() as u32,
                flags & number("ignore_b_mask")
            );
        }
        assert_eq!(sha256(&rom.data), digest);
    }
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_TRAINING_PARTY_SELECTION_PROBES independent native evidence"]
fn local_training_party_selection_matches_native_and_keeps_individuals_unchanged() {
    use armv4t_emu::Memory;
    let vectors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_TRAINING_PARTY_SELECTION_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    let mut total = 0;
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let rom =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        let digest = sha256(&rom.data);
        let report = rom.training_services(None).unwrap();
        if !matches!(key, "ULTIMATE" | "MERCURY12") {
            assert!(report.services.is_empty());
            continue;
        }
        let v = &vectors[key];
        assert_eq!(v["md5"], rom.profile.md5);
        let rules = &v["rules"];
        let number = |name: &str| rules[name].as_u64().unwrap() as u32;
        let selection = &report.services[0].selection;
        assert_eq!(selection.scope, "party");
        assert!(selection.cancel_with_b);
        assert!(!selection.rejects_fainted_during_selection);
        assert!(!selection.rejects_egg_during_selection);
        assert_eq!(
            selection.cancel_value,
            if key == "MERCURY12" { 7 } else { 255 }
        );
        for entry in v["entries"].as_array().unwrap() {
            let at = entry["command"].as_u64().unwrap() as usize;
            assert!(selection.evidence.commands.contains(&at));
            assert_eq!(
                selection.evidence.special_id as u64,
                entry["special"].as_u64().unwrap()
            );
            assert_eq!(
                selection.evidence.native as u64,
                entry["native"].as_u64().unwrap()
            );
            let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
            ram.w32(0x02010008, 0x08000000 + at as u32 + 1);
            let handler = u32(&rom.data, number("commands") as usize + 0x25 * 4).unwrap() & !1;
            ram.observe(
                handler,
                [0x02010000, 0, 0, 0],
                [0; 2],
                100_000,
                number("entry"),
            )
            .unwrap();
            // Full fade/task creation and overworld cleanup are independently
            // observed by mGBA. The stricter core runner checks the native Init
            // argument prefix only, with explicit zeroed engine context.
            let regs = ram
                .observe(
                    number("init_prefix"),
                    [number("init_argument"), 0, 0, 0],
                    [0; 2],
                    100_000,
                    number("init"),
                )
                .unwrap();
            assert_eq!(regs[..4], [3, 0, 11, 0]);
            for (i, value) in entry["callbacks"].as_array().unwrap().iter().enumerate() {
                assert_eq!(
                    ram.r32(regs[13] + i as u32 * 4),
                    value.as_u64().unwrap() as u32
                );
            }
        }
        assert_eq!(v["rows"].as_array().unwrap().len(), 144);
        for row in v["rows"].as_array().unwrap() {
            let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
            let raw: Vec<u8> = row["raw"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u8)
                .collect();
            assert_eq!(raw.len(), 100);
            let b = rom.profile.breeding.unwrap();
            ram.w32(b.save_pointers[0], 0x02030000);
            ram.w32(b.save_pointers[1], 0x02034000);
            for (i, value) in raw.iter().cycle().take(600).enumerate() {
                ram.w8(b.party + i as u32, *value);
            }
            ram.w8(b.party_count, 6);
            for (i, value) in [3, row["slot"].as_u64().unwrap() as u8, 0, 11]
                .iter()
                .enumerate()
            {
                ram.w8(number("menu") + 8 + i as u32, *value);
            }
            ram.w16(number("selected"), 88);
            ram.w16(number("keys"), row["keys"].as_u64().unwrap() as u16);
            ram.observe(
                number("input"),
                [0; 4],
                [0; 2],
                100_000,
                row["stop"].as_u64().unwrap() as u32,
            )
            .unwrap();
            assert_eq!(
                ram.r16(number("selected")) as u64,
                row["selection"].as_u64().unwrap()
            );
            assert_eq!(
                ram.r8(number("menu") + 9) as u64,
                row["menu_slot"].as_u64().unwrap()
            );
            if let Some(buffer) = rules["buffer"].as_u64() {
                if row["decision"] != "waiting" {
                    ram.observe(
                        buffer as u32,
                        [0; 4],
                        [0; 2],
                        100_000,
                        number("buffer_stop"),
                    )
                    .unwrap();
                }
            }
            assert_eq!(
                ram.r16(number("selected")) as u64,
                row["result"].as_u64().unwrap()
            );
            for (i, value) in raw.iter().cycle().take(600).enumerate() {
                assert_eq!(ram.r8(b.party + i as u32), *value);
            }
            total += 1;
        }
        for row in v["normalized"].as_array().unwrap() {
            let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
            ram.w8(number("menu") + 9, row["slot"].as_u64().unwrap() as u8);
            ram.observe(
                number("buffer"),
                [0; 4],
                [0; 2],
                100_000,
                number("buffer_stop"),
            )
            .unwrap();
            assert_eq!(
                ram.r16(number("selected")) as u64,
                row["result"].as_u64().unwrap()
            );
        }
        for path in v["paths"].as_array().unwrap() {
            let mut ram = crate::native_trainer::Sandbox::new(&rom.data);
            ram.w16(number("selected"), path["value"].as_u64().unwrap() as u16);
            for step in path["trace"].as_array().unwrap() {
                let at = step["command"].as_u64().unwrap() as usize;
                let native = u32(
                    &rom.data,
                    number("commands") as usize + rom.data[at] as usize * 4,
                )
                .unwrap()
                    & !1;
                ram.w32(0x02010008, 0x08000000 + at as u32 + 1);
                ram.call(native, [0x02010000, 0, 0, 0], [0; 2], 100_000)
                    .unwrap();
                assert_eq!(
                    ram.r32(0x02010008),
                    0x08000000 + step["next"].as_u64().unwrap() as u32
                );
            }
        }
        assert_eq!(sha256(&rom.data), digest);
    }
    assert_eq!(total, 288);
}

#[test]
#[ignore = "requires five local exact ROMs; synthetic persistent-state SAV scenarios"]
fn local_collection_prerequisite_routes_all_profiles() {
    use crate::{
        acquisition::{AcquisitionIndex, Target, TargetKind},
        collection::{CollectionBasis, CollectionPlan, CollectionRegion, CollectionTask},
        event_dependencies::Index,
    };
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let original = std::fs::read(&path).unwrap();
        let rom = Rom::open(original.clone()).unwrap();
        let acquisition = AcquisitionIndex::build(&rom).unwrap();
        let save = Save::open(save_bytes(&rom), rom.profile.save).unwrap();
        let before = save.data.clone();
        let ids: std::collections::BTreeSet<_> = acquisition
            .world
            .map_events
            .iter()
            .flat_map(|map| {
                map.markers
                    .iter()
                    .flat_map(|m| &m.rewards)
                    .chain(&map.unplaced_rewards)
            })
            .filter(|reward| {
                reward
                    .conditions
                    .iter()
                    .any(|c| matches!(c.kind, "flag" | "variable"))
            })
            .map(|reward| reward.item)
            .collect();
        let mut tasks = Vec::new();
        for id in ids {
            let target = Target {
                kind: TargetKind::Item,
                id,
            };
            let source = acquisition
                .query(&rom, Some(&save), target.clone())
                .unwrap()
                .sources
                .into_iter()
                .find(|s| {
                    s.map_id.is_some()
                        && s.conditions
                            .iter()
                            .any(|c| matches!(c.condition.kind, "flag" | "variable"))
                });
            if let Some(source) = source {
                tasks.push(CollectionTask {
                    target,
                    family: vec![],
                    existing_family_members: vec![],
                    source: Some(source),
                    alternatives: 1,
                    preparation: None,
                });
            }
            if tasks.len() == 3 {
                break;
            }
        }
        assert!(!tasks.is_empty(), "{key}: no guarded runtime source");
        let mut plan = CollectionPlan {
            dex_status: None,
            entrance_coverage: None,
            entrance_diagnostics: vec![],
            prerequisites: None,
            clock: None,
            rom_md5: rom.profile.md5,
            basis: CollectionBasis::Individuals,
            families: true,
            owned_count: 0,
            missing_count: 0,
            regions: vec![CollectionRegion {
                region: None,
                tasks,
            }],
            entrances: vec![],
            breeding_coverage: None,
            partial: true,
        };
        let index = Index::build(&rom, &acquisition.world.maps).unwrap();
        let graph = index
            .navigation_graph(&rom, &acquisition.world.maps, Some(&save))
            .unwrap();
        let goal_maps = plan
            .regions
            .iter()
            .flat_map(|r| &r.tasks)
            .filter_map(|t| t.source.as_ref().and_then(|s| s.map_id.clone()))
            .collect::<std::collections::BTreeSet<_>>();
        plan.entrances = crate::navigation::suggestions(&acquisition.world.maps, &graph, goal_maps);
        for entry in &plan.entrances {
            let navigation = index
                .map_navigation(&rom, &acquisition.world.maps, &entry.map_id, Some(&save))
                .unwrap();
            assert_eq!(
                serde_json::to_value(&entry.chains).unwrap(),
                serde_json::to_value(&navigation.approaches).unwrap(),
                "{key}: planner/map parity"
            );
        }
        let bundle = index
            .trace_plan(&rom, &save, &acquisition.world.maps, &plan)
            .unwrap();
        assert!(!bundle.routes.is_empty(), "{key}: no prerequisite routes");
        for route in &bundle.routes {
            let report = &bundle.reports[route.report_index];
            assert!(!route.goals.is_empty());
            assert!(route
                .goals
                .iter()
                .all(|g| g[0] == 0 && g[1] < plan.regions[0].tasks.len()));
            for candidate in &route.candidates {
                let writer = &report.writers[candidate.writer_index];
                for required in &candidate.requires {
                    let guard = &bundle.reports[*required].condition;
                    assert!(writer
                        .conditions
                        .iter()
                        .any(|c| c.satisfied != Some(true) && c.condition == guard.condition));
                }
                let entry = bundle
                    .entrances
                    .iter()
                    .find(|e| e.map_id == report.writers[candidate.writer_index].reference.map_id)
                    .unwrap();
                assert_eq!(
                    candidate.entry_requires.len(),
                    entry.chains.len() + entry.unresolved_incoming.len()
                );
                for condition in candidate
                    .entry_requires
                    .iter()
                    .flatten()
                    .map(|i| &bundle.reports[*i].condition.condition)
                {
                    assert!(entry
                        .chains
                        .iter()
                        .flatten()
                        .chain(&entry.unresolved_incoming)
                        .filter_map(|e| e.script.as_ref())
                        .flat_map(|s| &s.checks)
                        .any(|g| g.satisfied != Some(true) && &g.condition == condition));
                }
                for untraced in &candidate.untraced_conditions {
                    assert!(untraced.satisfied != Some(true));
                    assert!(!bundle
                        .reports
                        .iter()
                        .any(|r| r.condition.condition == untraced.condition));
                }
            }
        }
        assert_eq!(save.data, before);
        assert_eq!(rom.data.as_ref(), &original);
        assert_eq!(std::fs::read(&path).unwrap(), original);
        println!("{key}: {} runtime goals, {} condition routes, {} candidate events; truncation={} (synthetic SAV, no accessibility claim)",plan.regions[0].tasks.len(),bundle.routes.len(),bundle.routes.iter().map(|r|r.candidates.len()).sum::<usize>(),bundle.truncated);
    }
}

#[test]
fn native_pocket_categories_isolate_stored_items_and_standard_edits() {
    use crate::event_state::EventSnapshot;
    for p in profile::PROFILES {
        let mut r = adapter_rom(p);
        let (berry_category, key_category) = if p.id == profile::ROCKET.id {
            (5, 8)
        } else if p.id == crate::mercury::PROFILE.id {
            (5, 2)
        } else {
            (4, 5)
        };
        // Synthetic native ID sanitizer is identity; no real ROM is patched.
        let sanitizer = r.profile.resource_checks.unwrap().item_sanitizer as usize - 0x08000000;
        let table = r.profile.items;
        let b = std::sync::Arc::make_mut(&mut r.data);
        put16(b, sanitizer, 0x4770);
        for (id, category) in [(2, berry_category), (3, key_category)] {
            b[table.offset + id * table.stride + 26] = category;
        }
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        for (pocket, other, id, category, amount) in [
            ("berries", "key_items", 2, berry_category, 7),
            ("key_items", "berries", 3, key_category, 1),
        ] {
            assert_eq!(
                r.profile
                    .save
                    .pockets
                    .iter()
                    .find(|s| s.id == pocket)
                    .unwrap()
                    .category,
                category,
                "{} {pocket}",
                p.id
            );
            save.edit_bag(pocket, 0, id, amount, &r, Policy::Standard)
                .unwrap();
            let before = save.data.clone();
            assert_eq!(
                save.edit_bag(other, 0, id, 1, &r, Policy::Standard)
                    .unwrap_err()
                    .code,
                "item_pocket"
            );
            assert_eq!(save.data, before);
            save.edit_bag(other, 1, id, 99, &r, Policy::Free).unwrap();
            save.edit_bag("pc", 0, id, 999, &r, Policy::Standard)
                .unwrap();
            let state = EventSnapshot::new(&save, r.profile.event_state.unwrap());
            assert_eq!(state.normal_bag_item(&r, id), Some((amount as u32, true)));
            save.edit_bag(pocket, 0, 0, 0, &r, Policy::Standard)
                .unwrap();
            let state = EventSnapshot::new(&save, r.profile.event_state.unwrap());
            assert_eq!(state.normal_bag_item(&r, id), Some((0, false)));
        }
    }
}

#[test]
#[ignore = "requires all five exact ROMs and GEN3_INVENTORY_CATEGORY_PROBES booted native vectors"]
fn local_inventory_categories_match_booted_native_pockets_and_checks() {
    use crate::event_state::EventSnapshot;
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_INVENTORY_CATEGORY_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let r =
            Rom::open(std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap())
                .unwrap();
        assert_eq!(probes[key]["md5"], r.profile.md5);
        for pocket in probes[key]["pockets"].as_array().unwrap() {
            let id = pocket["id"].as_str().unwrap();
            let item = pocket["item"].as_u64().unwrap() as u16;
            let layout = r.profile.save.pockets.iter().find(|s| s.id == id).unwrap();
            assert_eq!(
                layout.category as u64,
                pocket["category"].as_u64().unwrap(),
                "{key} {id}"
            );
            assert_eq!(layout.count as u64, pocket["capacity"].as_u64().unwrap());
            assert_eq!(
                r.resource_item_pocket(item).unwrap() as u64,
                pocket["category"].as_u64().unwrap()
            );
            for vector in pocket["vectors"].as_array().unwrap() {
                let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
                for (slot, quantity) in vector["quantities"].as_array().unwrap().iter().enumerate()
                {
                    let quantity = quantity.as_u64().unwrap() as u16;
                    save.edit_bag(id, slot, item, quantity.max(1), &r, Policy::Free)
                        .unwrap();
                    if quantity == 0 {
                        fixture_zero_bag_quantity(&mut save, id, slot);
                    }
                }
                let before = save.data.clone();
                let state = EventSnapshot::new(&save, r.profile.event_state.unwrap());
                let (quantity, present) = state.normal_bag_item(&r, item).unwrap();
                assert_eq!(
                    u8::from(present && quantity >= vector["value"].as_u64().unwrap() as u32)
                        as u64,
                    vector["result"].as_u64().unwrap(),
                    "{key} {id}: {vector}"
                );
                assert_eq!(save.data, before);
            }
        }
    }
}

fn player_condition_fixture(p: profile::Profile) -> Rom {
    let mut r = adapter_rom(p);
    let rules = r.profile.event_state.unwrap().effects.unwrap();
    let b = std::sync::Arc::make_mut(&mut r.data);
    for (op, handler) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
        .into_iter()
        .zip(rules.handlers)
    {
        put32(
            b,
            rules.commands + op as usize * 4,
            0x08000001 + handler as u32,
        );
    }
    let (handler, sb2, result) = rules.player_gender;
    put32(b, rules.commands + 0xa0 * 4, 0x08000001 + handler as u32);
    for (i, value) in [0x4903, 0x4804, 0x6800, 0x7a00, 0x8008, 0x2000, 0x4770]
        .into_iter()
        .enumerate()
    {
        put16(b, handler + i * 2, value);
    }
    put32(b, handler + 0x10, result);
    put32(b, handler + 0x14, sb2);
    r
}

fn player_condition_branch(
    r: &mut Rom,
    rhs: u16,
    reversed: bool,
    selector: u8,
    branch: u8,
    copy: u8,
) -> crate::event_dependencies::ScriptEffects {
    let root = 0x26000;
    let target = 0x26100;
    let mut script = vec![0x16, 1, 0x80];
    script.extend(rhs.to_le_bytes());
    script.extend([0xa0, copy, 0, 0x80, 0x0d, 0x80]);
    if reversed {
        script.extend([0x22, 1, 0x80, 0, 0x80]);
    } else {
        script.extend([0x21, 0, 0x80]);
        script.extend(rhs.to_le_bytes());
    }
    script.extend([branch, selector]);
    script.extend((0x08000000u32 + target as u32).to_le_bytes());
    script.extend([0x29, 12, 0, 2]);
    let b = std::sync::Arc::make_mut(&mut r.data);
    b[root..root + script.len()].copy_from_slice(&script);
    b[target..target + 4].copy_from_slice(&[0x29, 13, 0, if branch == 7 { 3 } else { 2 }]);
    r.event_effect_script(root).unwrap()
}

#[test]
fn player_conditions_preserve_raw_gender_aliases_comparisons_and_unknown_boundaries() {
    use crate::acquisition::check;
    use crate::event_state::EventSnapshot;
    for p in profile::PROFILES {
        let mut r = player_condition_fixture(p);
        assert_eq!(r.player_gender_offset(), Some(8));
        for rhs in [0, 1, 2, 255, 65535] {
            for reversed in [false, true] {
                for selector in 0..6 {
                    for branch in [6, 7] {
                        for copy in [0x19, 0x1a] {
                            let walk = player_condition_branch(
                                &mut r, rhs, reversed, selector, branch, copy,
                            );
                            assert!(walk.complete && walk.stopped_at.is_empty());
                            let guard =
                                &walk.effects.iter().find(|e| e.id == 13).unwrap().conditions;
                            assert_eq!(guard.len(), 1);
                            assert_eq!(guard[0].kind, "player_gender");
                            assert_eq!(guard[0].value, u32::from(rhs));
                            assert!(check(None, Some(&r), &guard[0]).satisfied.is_none());
                            for gender in [0, 1, 2, 255] {
                                let mut trainer = vec![0; 16];
                                trainer[8] = gender;
                                let snapshot = EventSnapshot::fixture(
                                    vec![],
                                    trainer,
                                    vec![],
                                    p.event_state.unwrap(),
                                );
                                let actual = check(Some(&snapshot), Some(&r), &guard[0]);
                                let (a, b) = if reversed {
                                    (rhs, u16::from(gender))
                                } else {
                                    (u16::from(gender), rhs)
                                };
                                let expected = [a < b, a == b, a > b, a <= b, a >= b, a != b]
                                    [selector as usize];
                                assert_eq!(actual.actual, Some(u32::from(gender)));
                                assert_eq!(
                                    actual.satisfied,
                                    Some(expected),
                                    "{} {rhs} {selector} {reversed}",
                                    p.id
                                );
                            }
                        }
                    }
                }
            }
        }
        // Native gender reading does not overwrite a previous cached comparison.
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[0x26000..0x2600b].copy_from_slice(&[0x2b, 11, 0, 0xa0, 6, 1, 0, 0x61, 2, 8, 2]);
        let walk = r.event_effect_script(0x26000).unwrap();
        assert!(walk.complete);
        assert!(walk
            .effects
            .iter()
            .find(|e| e.id == 13)
            .unwrap()
            .conditions
            .iter()
            .any(|c| c.kind == "flag" && c.id == 11));
        // An overwritten alias must not retain the earlier player condition.
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[0x26000..0x26016].copy_from_slice(&[
            0xa0, 0x16, 0x0d, 0x80, 1, 0, 0x21, 0x0d, 0x80, 1, 0, 6, 1, 0, 0x61, 2, 8, 0x29, 12, 0,
            2, 2,
        ]);
        let walk = r.event_effect_script(0x26000).unwrap();
        assert!(walk
            .effects
            .iter()
            .all(|e| e.conditions.iter().all(|c| c.kind != "player_gender")));
        // Changed executable/literals cannot silently inherit another ROM's rule.
        let h = p.event_state.unwrap().effects.unwrap().player_gender.0;
        for offset in [0, 6, 12, 16, 20] {
            let b = std::sync::Arc::make_mut(&mut r.data);
            b[h + offset] ^= 1;
            assert!(r.player_gender_offset().is_none());
            let walk = player_condition_branch(&mut r, 1, false, 1, 6, 0x19);
            assert!(!walk.complete && walk.stopped_at.contains(&0x26005));
            assert!(walk
                .effects
                .iter()
                .all(|e| e.conditions.iter().all(|c| c.kind != "player_gender")));
            let b = std::sync::Arc::make_mut(&mut r.data);
            b[h + offset] ^= 1;
        }
    }
}

#[test]
#[ignore = "requires five exact private ROMs and GEN3_PLAYER_CONDITION_PROBES native vectors"]
fn local_player_conditions_match_complete_native_reads_copies_and_branches() {
    use crate::{acquisition::check, event_state::EventSnapshot};
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_PLAYER_CONDITION_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let rom = Rom::open(bytes.clone()).unwrap();
        let native = &probes[key];
        assert_eq!(native["md5"].as_str().unwrap(), rom.profile.md5);
        assert_eq!(native["sha256"].as_str().unwrap(), sha256(&bytes));
        assert_eq!(rom.player_gender_offset(), Some(8));
        let (handler, sb2, result) = rom
            .profile
            .event_state
            .unwrap()
            .effects
            .unwrap()
            .player_gender;
        assert_eq!(
            native["handlers"]["160"].as_u64().unwrap() as usize,
            handler
        );
        assert_eq!(native["save_pointer"].as_u64().unwrap(), u64::from(sb2));
        assert_eq!(
            native["result_pointer"].as_u64().unwrap(),
            u64::from(result)
        );
        assert_eq!(native["reads"].as_array().unwrap().len(), 256);
        for row in native["reads"].as_array().unwrap() {
            let mut trainer = vec![0; 16];
            trainer[8] = row["gender"].as_u64().unwrap() as u8;
            let snapshot =
                EventSnapshot::fixture(vec![], trainer, vec![], rom.profile.event_state.unwrap());
            assert_eq!(
                snapshot.player_gender(&rom).unwrap(),
                row["result"].as_u64().unwrap() as u32
            );
        }
        assert_eq!(native["copies"].as_array().unwrap().len(), 8);
        assert_eq!(native["branches"].as_array().unwrap().len(), 480);
        let mut fixture = player_condition_fixture(rom.profile);
        for row in native["branches"].as_array().unwrap() {
            let walk = player_condition_branch(
                &mut fixture,
                row["rhs"].as_u64().unwrap() as u16,
                row["reversed"].as_bool().unwrap(),
                row["selector"].as_u64().unwrap() as u8,
                row["opcode"].as_u64().unwrap() as u8,
                0x19,
            );
            let guard = &walk.effects.iter().find(|e| e.id == 13).unwrap().conditions[0];
            let mut trainer = vec![0; 16];
            trainer[8] = row["gender"].as_u64().unwrap() as u8;
            let snapshot =
                EventSnapshot::fixture(vec![], trainer, vec![], rom.profile.event_state.unwrap());
            assert_eq!(
                check(Some(&snapshot), Some(&rom), guard).satisfied,
                row["selected"].as_bool(),
                "{key} {row}"
            );
        }
        assert_eq!(bytes, std::fs::read(path).unwrap());
    }
}

fn install_buffer_fixture(r: &mut Rom) {
    let rules = r.profile.event_state.unwrap().effects.unwrap();
    let item_name = r.codec.encode("ITEM", 14).unwrap();
    let b = std::sync::Arc::make_mut(&mut r.data);
    for (op, handler) in [0x7d, 0x80, 0x82, 0x83, 0x85]
        .into_iter()
        .zip(rules.buffer_handlers)
    {
        put32(
            b,
            rules.commands + op as usize * 4,
            0x08000001 + handler as u32,
        );
        put16(b, handler, if op == 0x83 { 0xb530 } else { 0xb510 });
        put16(b, handler + 2, 0x6881);
        let literal = match op {
            0x7d | 0x82 => 0x38,
            0x80 => 0x30,
            0x83 => 0x40,
            _ => 0x24,
        };
        put32(b, handler + literal, 0x08027100);
        if matches!(op, 0x7d | 0x82) {
            put16(b, handler + 0x22, if op == 0x7d { 0x210b } else { 0x210d });
            put32(b, handler + 0x3c, 0x08027200);
        }
    }
    for (i, pointer) in rules.buffer_destinations.into_iter().enumerate() {
        put32(b, 0x27100 + i * 4, pointer);
    }
    b[0x27200..0x27400].fill(0xff);
    let item = r.profile.items.offset + r.profile.items.stride;
    b[item..item + 14].copy_from_slice(&item_name);
    if let Some(compare) = rules.buffer_item_dynamic_compare {
        put16(b, compare, 0x28af);
    }
}

#[test]
fn text_buffers_keep_reward_parameters_player_checks_and_guarded_map_routes() {
    use crate::{
        acquisition::{AcquisitionIndex, Target, TargetKind},
        event_dependencies::Index,
    };
    for p in profile::PROFILES {
        let (mut r, map) = npc_trade_fixture(p);
        // Install only verified engine identity/instruction facts in synthetic ROM.
        let engine = player_condition_fixture(p);
        let rules = r.profile.event_state.unwrap().effects.unwrap();
        let b = std::sync::Arc::make_mut(&mut r.data);
        for (op, h) in [0x16, 0x17, 0x18, 0x19, 0x1a, 0x29, 0x2a, 0x0f]
            .into_iter()
            .zip(rules.handlers)
        {
            put32(b, rules.commands + op as usize * 4, 0x08000001 + h as u32);
        }
        let (h, _, _) = rules.player_gender;
        b[h..h + 24].copy_from_slice(&engine.data[h..h + 24]);
        put32(b, rules.commands + 0xa0 * 4, 0x08000001 + h as u32);
        let warp = rules.warp_handlers[2];
        put32(b, rules.commands + 0x3b * 4, 0x08000001 + warp as u32);
        install_buffer_fixture(&mut r);
        let mut target_map = map.clone();
        target_map.id = "0-1".into();
        target_map.number = 1;
        target_map.width = 10;
        target_map.events = None;
        target_map.scripts.clear();
        let maps = vec![map.clone(), target_map];
        for op in [0x7d, 0x80, 0x82, 0x83, 0x85] {
            let mut script = vec![
                0x16, 0, 0x80, 1, 0, 0x16, 1, 0x80, 7, 0, 0xa0, 0x21, 0x0d, 0x80, 1, 0, op, 2,
            ];
            if op == 0x85 {
                script.extend(0x08027200u32.to_le_bytes());
            } else {
                script.extend([0, 0x80]);
            }
            script.extend([6, 1]);
            script.extend(0x08026100u32.to_le_bytes());
            script.push(2);
            let b = std::sync::Arc::make_mut(&mut r.data);
            b[0x26000..0x26000 + script.len()].copy_from_slice(&script);
            b[0x26100..0x2610e]
                .copy_from_slice(&[0x44, 0, 0x80, 1, 0x80, 0x3b, 0, 1, 255, 1, 0x80, 0, 0x80, 2]);
            let events = r.map_events(&map).unwrap();
            let reward = events
                .markers
                .iter()
                .flat_map(|m| &m.rewards)
                .find(|r| r.offset == 0x26100)
                .unwrap_or_else(|| {
                    panic!(
                        "{} {op:x} buffer={} {}",
                        p.id,
                        r.script_buffer_preserves(0x26010, |_| Some(1)),
                        serde_json::to_string(&events).unwrap()
                    )
                });
            assert_eq!(
                (reward.item, reward.quantity),
                (1, Some(7)),
                "{} {op:x}",
                p.id
            );
            assert!(reward
                .conditions
                .iter()
                .any(|c| c.kind == "player_gender" && c.value == 1));
            let index = Index::build(&r, &maps).unwrap();
            let report = index.map_navigation(&r, &maps, "0-1", None).unwrap();
            assert_eq!(report.incoming.len(), 1);
            let edge = &report.incoming[0];
            assert_eq!((edge.target_x, edge.target_y), (Some(7), Some(1)));
            assert!(edge
                .script
                .as_ref()
                .unwrap()
                .checks
                .iter()
                .any(|c| c.condition.kind == "player_gender" && c.satisfied.is_none()));
            let acquisition = AcquisitionIndex {
                wild_cache: Default::default(),
                breeding_cache: Default::default(),
                world: crate::world::World {
                    maps: maps.clone(),
                    map_events: vec![events],
                    encounters: vec![],
                    trainers: vec![],
                    trainer_locations: crate::world::TrainerLocationIndex {
                        locations: vec![],
                        unresolved_maps: vec![],
                    },
                    map_groups: &[],
                },
                species: vec![r.valid_species(1).unwrap()],
                evolutions: Default::default(),
                learnsets: Default::default(),
            };
            let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
            for gender in [0, 1] {
                save.edit_trainer(
                    &crate::save::TrainerPatch {
                        gender: Some(gender),
                        ..Default::default()
                    },
                    &r,
                )
                .unwrap();
                let before = save.data.clone();
                let query = acquisition
                    .query(
                        &r,
                        Some(&save),
                        Target {
                            kind: TargetKind::Item,
                            id: 1,
                        },
                    )
                    .unwrap();
                let source = query.sources.iter().find(|s| s.offset == 0x26100).unwrap();
                assert_eq!(source.quantity, Some(7));
                assert!(source.conditions.iter().any(
                    |c| c.condition.kind == "player_gender" && c.satisfied == Some(gender == 1)
                ));
                let navigation = index.map_navigation(&r, &maps, "0-1", Some(&save)).unwrap();
                assert!(navigation.incoming[0]
                    .script
                    .as_ref()
                    .unwrap()
                    .checks
                    .iter()
                    .any(
                        |c| c.condition.kind == "player_gender" && c.satisfied == Some(gender == 1)
                    ));
                assert_eq!(before, save.data);
            }
            // Slot overflow cannot inherit this engine summary or leak old parameters.
            let b = std::sync::Arc::make_mut(&mut r.data);
            b[0x26011] = 3;
            assert!(!r.script_buffer_preserves(0x26010, |_| Some(1)));
            assert!(r.item_script(0x26000).unwrap().1.contains(&0x26010));
        }
        let b = std::sync::Arc::make_mut(&mut r.data);
        b[0x26000..0x26004].copy_from_slice(&[0x7d, 0, 0, 0x80]);
        assert!(!r.script_buffer_preserves(0x26000, |_| None));
        assert!(!r.script_buffer_preserves(0x26000, |_| Some(u16::MAX)));
        let b = std::sync::Arc::make_mut(&mut r.data);
        put32(b, 0x27100, 0x02030000);
        assert!(!r.script_buffer_preserves(0x26000, |_| Some(1)));
        let b = std::sync::Arc::make_mut(&mut r.data);
        put32(b, 0x27100, rules.buffer_destinations[0]);
        b[0x26000..0x26006].copy_from_slice(&[0x85, 0, 0, 0x73, 2, 8]);
        b[0x27300..0x2730e].fill(0);
        assert!(!r.script_buffer_preserves(0x26000, |_| Some(1)));
    }
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_SCRIPT_BUFFER_PROBES native RAM/output evidence"]
fn local_script_buffers_match_complete_native_state_preservation() {
    let probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_SCRIPT_BUFFER_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let path = std::env::var(format!("GEN3_ROM_{key}")).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let rom = Rom::open(bytes.clone()).unwrap();
        let proof = &probes[key];
        assert_eq!(proof["md5"].as_str().unwrap(), rom.profile.md5);
        assert_eq!(proof["sha256"].as_str().unwrap(), sha256(&bytes));
        let rules = rom.profile.event_state.unwrap().effects.unwrap();
        for (op, h) in [0x7d, 0x80, 0x82, 0x83, 0x85]
            .into_iter()
            .zip(rules.buffer_handlers)
        {
            assert_eq!(
                proof["handlers"][op.to_string()].as_u64().unwrap() as usize,
                h
            );
        }
        for (i, d) in rules.buffer_destinations.into_iter().enumerate() {
            assert_eq!(proof["destinations"][i].as_u64().unwrap() as u32, d);
        }
        let mut fixture = rom.clone();
        for row in proof["rows"].as_array().unwrap() {
            let op = row["opcode"].as_u64().unwrap() as u8;
            let value = row["value"].as_u64().unwrap() as u16;
            let slot = row["slot"].as_u64().unwrap() as u8;
            let operand = row["operand"].as_u64().unwrap() as u16;
            // Disposable decoding context, not a referenced-script/access assertion.
            let b = std::sync::Arc::make_mut(&mut fixture.data);
            b[0x26000..0x26002].copy_from_slice(&[op, slot]);
            if op == 0x85 {
                put32(
                    b,
                    0x26002,
                    0x08000000 + row["source"].as_u64().unwrap() as u32,
                );
            } else {
                put16(b, 0x26002, operand);
            }
            let classified = fixture.script_buffer_preserves(0x26000, |v| {
                if v == 0x8000 {
                    Some(value)
                } else if v < 0x4000 {
                    Some(v)
                } else {
                    None
                }
            });
            assert_eq!(
                classified,
                row["context_dependent"].as_bool() != Some(true),
                "{key} {row}"
            );
        }
        assert!(proof["rows"].as_array().unwrap().len() > 500);
        assert_eq!(bytes, std::fs::read(path).unwrap());
    }
}

#[test]
fn native_read_only_dex_banks_preserve_flags_initialization_and_edit_boundaries() {
    for profile in profile::PROFILES {
        let r = adapter_rom(profile);
        let mut save = Save::open(save_bytes(&r), profile.save).unwrap();
        if let Some(banks) = profile.save.dex_read {
            let base = save.sections[1];
            for bank in banks {
                for i in 0..(bank.count as usize).div_ceil(8) {
                    save.data[base + bank.seen + i] = 0xa5;
                    save.data[base + bank.owned + i] = 0x5a;
                }
                if let Some((off, value)) = bank.initialization {
                    put16(&mut save.data, base + off, value);
                }
            }
            let before = save.data.clone();
            let flags = save.dex().unwrap();
            assert_eq!(flags.len(), 1027);
            for bank in banks {
                for i in 0..bank.count {
                    let f = &flags[(bank.first + i - 1) as usize];
                    assert_eq!(f.number, bank.first + i);
                    assert_eq!(f.seen, 0xa5 & (1 << (i % 8)) != 0);
                    assert_eq!(f.owned, 0x5a & (1 << (i % 8)) != 0);
                }
            }
            assert!(save.dex_read_status().unwrap().unwrap().read_only);
            for n in [1, 905, 906, 1027] {
                assert_eq!(
                    save.edit_dex(n, true, true).unwrap_err().code,
                    "unsupported_feature"
                );
                assert_eq!(save.data, before);
            }
            let (off, _) = banks[1].initialization.unwrap();
            put16(&mut save.data, base + off, 0);
            let before = save.data.clone();
            let flags = save.dex().unwrap();
            assert!(flags[905..].iter().all(|f| !f.seen && !f.owned));
            assert!(flags[..905].iter().any(|f| f.seen));
            let status = save.dex_read_status().unwrap().unwrap();
            assert_eq!(
                (
                    status.uninitialized_ranges[0].first,
                    status.uninitialized_ranges[0].count
                ),
                (906, 122)
            );
            assert_eq!(
                save.data, before,
                "lazy initialization is projected, never written"
            );
            let sum = sector_checksum(&save.data[base..base + profile.save.sizes[1]]);
            put16(&mut save.data, base + 0xff6, sum);
            let original = save.data.clone();
            let mut session = Session::new(r);
            session.load(original.clone(), None).unwrap();
            for policy in [Policy::Standard, Policy::Free] {
                assert_eq!(
                    session
                        .apply(
                            Action::Dex {
                                number: 906,
                                seen: true,
                                owned: true
                            },
                            policy
                        )
                        .unwrap_err()
                        .code,
                    "unsupported_feature"
                );
                assert_eq!(session.save_ref().unwrap().data, original);
                assert!(!session.snapshot().unwrap().can_undo);
            }
            assert_eq!(session.snapshot().unwrap().dex.len(), 1027);
        } else {
            let before = save.data.clone();
            let status = save.dex_read_status().unwrap().unwrap();
            assert!(!status.read_only);
            assert_eq!(save.dex().unwrap().len(), status.count);
            assert_eq!(save.data, before);
            save.edit_dex(1, true, true).unwrap();
            assert!(
                save.dex().unwrap()[0].owned,
                "legacy editing stays unchanged"
            );
        }
    }
}

#[test]
#[ignore = "requires GEN3_ROM_MERCURY12, GEN3_SAVE_MERCURY12 and GEN3_MERCURY_DEX_PROBES"]
fn local_mercury_dex_matches_native_split_banks_and_booted_save() {
    use sha2::{Digest, Sha256};
    let proof: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_MERCURY_DEX_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    let rom_bytes = std::fs::read(std::env::var("GEN3_ROM_MERCURY12").unwrap()).unwrap();
    let r = Rom::open(rom_bytes.clone()).unwrap();
    assert_eq!(r.profile.md5, proof["md5"]);
    assert_eq!(
        format!("{:x}", Sha256::digest(&rom_bytes)),
        proof["rom_sha256"]
    );
    let banks = r.profile.save.dex_read.unwrap();
    let mut total = 0;
    for pattern in [0u8, 255, 165, 90] {
        for initialized in [true, false] {
            let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
            let base = save.sections[1];
            for (i, bank) in banks.iter().enumerate() {
                assert_eq!(
                    (bank.first, bank.count, bank.seen, bank.owned),
                    (
                        proof["banks"][i]["first"].as_u64().unwrap() as u16,
                        proof["banks"][i]["count"].as_u64().unwrap() as u16,
                        proof["banks"][i]["seen"].as_u64().unwrap() as usize,
                        proof["banks"][i]["owned"].as_u64().unwrap() as usize
                    )
                );
                let length = (bank.count as usize).div_ceil(8);
                save.data[base + bank.seen..base + bank.seen + length].fill(pattern);
                save.data[base + bank.owned..base + bank.owned + length].fill(pattern ^ 255);
                if let Some((off, marker)) = bank.initialization {
                    put16(
                        &mut save.data,
                        base + off,
                        if initialized { marker } else { marker ^ 1 },
                    );
                }
            }
            let before = save.data.clone();
            let flags = save.dex().unwrap();
            for row in proof["rows"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v["pattern"] == pattern && v["initialized"] == initialized)
            {
                let f = &flags[row["number"].as_u64().unwrap() as usize - 1];
                assert_eq!(
                    serde_json::to_value(f).unwrap(),
                    serde_json::json!({"number":row["number"],"seen":row["seen"],"owned":row["owned"]})
                );
                total += 1;
            }
            assert_eq!(save.data, before);
        }
    }
    assert_eq!(total, 8 * 1027);
    let original = std::fs::read(std::env::var("GEN3_SAVE_MERCURY12").unwrap()).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&original)),
        proof["save_sha256"]
    );
    let save = Save::open(original.clone(), r.profile.save).unwrap();
    assert_eq!(
        serde_json::to_value(save.dex().unwrap()).unwrap(),
        proof["real"]
    );
    assert_eq!(save.data, original);
}

#[test]
fn read_only_dex_planning_distinguishes_history_from_current_individuals() {
    use crate::{
        acquisition::AcquisitionIndex,
        collection::{CollectionBasis, CollectionRequest},
        world::{TrainerLocationIndex, World},
    };
    let r = adapter_rom(crate::mercury::PROFILE);
    let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    let base = save.sections[1];
    let bank = r.profile.save.dex_read.unwrap()[1];
    let (offset, marker) = bank.initialization.unwrap();
    put16(&mut save.data, base + offset, marker);
    save.data[base + bank.owned] = 1;
    let mut species = (1..=3)
        .map(|id| r.valid_species(id).unwrap())
        .collect::<Vec<_>>();
    for (entry, n) in species.iter_mut().zip([1, 906, 1027]) {
        entry.dex_number = n;
    }
    let index = AcquisitionIndex {
        wild_cache: Default::default(),
        breeding_cache: Default::default(),
        world: World {
            maps: vec![],
            map_events: vec![],
            encounters: vec![],
            trainers: vec![],
            trainer_locations: TrainerLocationIndex {
                locations: vec![],
                unresolved_maps: vec![],
            },
            map_groups: &[],
        },
        species,
        evolutions: Default::default(),
        learnsets: Default::default(),
    };
    let request = |basis| CollectionRequest {
        basis,
        families: false,
        include_unknown_rewards: false,
    };
    let original = save.data.clone();
    let plan = index
        .collection(&r, &save, request(CollectionBasis::Dex))
        .unwrap();
    let goals = plan
        .regions
        .iter()
        .flat_map(|region| &region.tasks)
        .map(|task| task.target.id)
        .collect::<Vec<_>>();
    assert_eq!(goals, [1, 3]);
    assert_eq!(plan.owned_count, 1);
    assert!(plan.dex_status.unwrap().read_only);
    let plan = index
        .collection(&r, &save, request(CollectionBasis::Individuals))
        .unwrap();
    let goals = plan
        .regions
        .iter()
        .flat_map(|region| &region.tasks)
        .map(|task| task.target.id)
        .collect::<Vec<_>>();
    assert_eq!(goals, [2, 3]);
    assert!(plan.dex_status.is_none());
    assert_eq!(save.data, original);
    // Missing initialization projects no extended history, independently of
    // unchanged nonzero raw bytes. It does not invent currently usable parents.
    put16(&mut save.data, base + offset, 0);
    let original = save.data.clone();
    let plan = index
        .collection(&r, &save, request(CollectionBasis::Dex))
        .unwrap();
    assert_eq!(plan.owned_count, 0);
    assert_eq!(plan.missing_count, 3);
    assert_eq!(plan.dex_status.unwrap().uninitialized_ranges.len(), 1);
    assert_eq!(save.data, original);
}

/// Set one synthetic legacy record across rotated logical save sectors.
fn set_legacy_dex_record(save: &mut Save, number: u16, state: u8) {
    let layout = save.layout.dex.unwrap();
    let index = number as usize - 1 + layout.bit_bias as usize;
    let mask = 1 << (index % 8);
    let fields = [
        (layout.main_block, layout.owned),
        (layout.main_block, layout.seen),
    ]
    .into_iter()
    .chain(layout.seen_mirrors.iter().map(|at| (true, *at)));
    for (bit, (main, at)) in fields.enumerate() {
        let mut offset = at + index / 8;
        let mut section = usize::from(main);
        while offset >= save.layout.sizes[section] {
            offset -= save.layout.sizes[section];
            section += 1;
            assert!(section <= 4);
        }
        let byte = &mut save.data[save.sections[section] + offset];
        *byte = (*byte & !mask) | if state & (1 << bit) != 0 { mask } else { 0 };
    }
}

#[test]
fn legacy_dex_native_checks_preserve_raw_records_and_explicit_editing() {
    for profile in profile::PROFILES {
        let Some(layout) = profile.save.dex else {
            continue;
        };
        let r = adapter_rom(profile);
        for number in [1, 7, 8, 9, layout.count] {
            for state in 0..1u8 << (2 + layout.seen_mirrors.len()) {
                let mut save = Save::open(save_bytes(&r), profile.save).unwrap();
                set_legacy_dex_record(&mut save, number, state);
                let raw_owned = state & 1 != 0;
                let raw_seen = state & 2 != 0;
                let seen =
                    raw_seen && (0..layout.seen_mirrors.len()).all(|i| state & (1 << (i + 2)) != 0);
                let owned = raw_owned && (!layout.owned_requires_seen || seen);
                let before = save.data.clone();
                let flags = save.dex().unwrap();
                let flag = &flags[number as usize - 1];
                assert_eq!(
                    (flag.seen, flag.owned),
                    (seen, owned),
                    "{} {number} {state}",
                    profile.id
                );
                let status = save.dex_read_status().unwrap().unwrap();
                assert_eq!(
                    status.inconsistent_numbers,
                    if (seen, owned) != (raw_seen, raw_owned) {
                        vec![number]
                    } else {
                        vec![]
                    }
                );
                assert_eq!(
                    save.data, before,
                    "query must not perform native invalidation"
                );
                save.edit_dex(number, true, true).unwrap();
                let flag = &save.dex().unwrap()[number as usize - 1];
                assert!(flag.seen && flag.owned);
                assert!(save
                    .dex_read_status()
                    .unwrap()
                    .unwrap()
                    .inconsistent_numbers
                    .is_empty());
                let after = save.data.clone();
                let reopened = Save::open(after.clone(), profile.save).unwrap();
                assert!(reopened.dex().unwrap()[number as usize - 1].owned);
                assert_eq!(reopened.data, after);
            }
        }
    }
}

#[test]
#[ignore = "requires four exact local ROMs and GEN3_DEX_FLAG_PROBES"]
fn local_legacy_dex_matches_complete_native_getters() {
    use sha2::{Digest, Sha256};
    let proof: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_DEX_FLAG_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    let mut total = 0;
    for key in ["BW", "DP", "ROCKET", "ULTIMATE"] {
        let bytes = std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap();
        let r = Rom::open(bytes.clone()).unwrap();
        let p = &proof[key];
        assert_eq!(p["md5"], r.profile.md5);
        assert_eq!(p["rom_sha256"], format!("{:x}", Sha256::digest(&bytes)));
        let layout = r.profile.save.dex.unwrap();
        assert_eq!(p["count"], layout.count);
        assert_eq!(p["bit_bias"], layout.bit_bias);
        assert_eq!(p["main_block"], layout.main_block);
        assert_eq!(p["seen"], layout.seen);
        assert_eq!(p["owned"], layout.owned);
        assert_eq!(
            p["mirrors"],
            serde_json::to_value(layout.seen_mirrors).unwrap()
        );
        assert_eq!(p["owned_requires_seen"], layout.owned_requires_seen);
        let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
        let rows = p["rows"].as_array().unwrap();
        assert_eq!(
            rows.len(),
            layout.count as usize * (1 << (2 + layout.seen_mirrors.len())) * 2
        );
        for row in rows {
            let number = row["number"].as_u64().unwrap() as u16;
            set_legacy_dex_record(&mut save, number, row["state"].as_u64().unwrap() as u8);
            let before = save.data.clone();
            let flags = save.dex().unwrap();
            let flag = &flags[number as usize - 1];
            assert_eq!(flag.seen, row["seen"].as_bool().unwrap(), "{key} {row}");
            assert_eq!(flag.owned, row["owned"].as_bool().unwrap(), "{key} {row}");
            assert_eq!(save.data, before);
            total += 1;
        }
        assert_eq!(
            bytes,
            std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap()
        );
    }
    assert_eq!(total, 41504);
}

#[test]
fn invalid_native_dex_records_remain_missing_in_collection() {
    use crate::{
        acquisition::AcquisitionIndex,
        collection::{CollectionBasis, CollectionRequest},
        world::{TrainerLocationIndex, World},
    };
    let r = adapter_rom(profile::BW);
    let mut save = Save::open(save_bytes(&r), r.profile.save).unwrap();
    // Both raw caught bits are set; only #2 has consistent seen mirrors.
    set_legacy_dex_record(&mut save, 1, 0b0111);
    set_legacy_dex_record(&mut save, 2, 0b1111);
    let mut species = (1..=2)
        .map(|id| r.valid_species(id).unwrap())
        .collect::<Vec<_>>();
    for (n, entry) in species.iter_mut().enumerate() {
        entry.dex_number = n as u16 + 1;
    }
    let index = AcquisitionIndex {
        wild_cache: Default::default(),
        breeding_cache: Default::default(),
        world: World {
            maps: vec![],
            map_events: vec![],
            encounters: vec![],
            trainers: vec![],
            trainer_locations: TrainerLocationIndex {
                locations: vec![],
                unresolved_maps: vec![],
            },
            map_groups: &[],
        },
        species,
        evolutions: Default::default(),
        learnsets: Default::default(),
    };
    let before = save.data.clone();
    let plan = index
        .collection(
            &r,
            &save,
            CollectionRequest {
                basis: CollectionBasis::Dex,
                families: false,
                include_unknown_rewards: false,
            },
        )
        .unwrap();
    assert_eq!((plan.owned_count, plan.missing_count), (1, 1));
    assert_eq!(plan.dex_status.unwrap().inconsistent_numbers, [1]);
    assert_eq!(
        plan.regions
            .iter()
            .flat_map(|r| &r.tasks)
            .map(|t| t.target.id)
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(save.data, before);
}

#[test]
fn scripted_actor_references_keep_initial_tiles_and_unverified_execution_boundaries() {
    use crate::{
        acquisition::{AcquisitionIndex, Target, TargetKind},
        world::{Map, TrainerLocationIndex, World},
    };
    for profile in profile::PROFILES {
        let mut r = adapter_rom(profile);
        let rules = profile.event_state.unwrap().effects.unwrap();
        let b = std::sync::Arc::make_mut(&mut r.data);
        for (i, handler) in rules.movement_handlers.iter().enumerate() {
            put32(
                b,
                rules.commands + (0x4f + i) * 4,
                0x08000001 + *handler as u32,
            );
        }
        b[0x24000] = 1;
        put32(b, 0x24004, 0x08024100);
        b[0x24100] = 7;
        put16(b, 0x24104, 2);
        put16(b, 0x24106, 3);
        put32(b, 0x24110, 0x08024200);
        // The direct reference retains the resolved low-byte actor identity.
        // Waiting reuses live engine state, never an inferred current position.
        let script = [
            0x16, 0x00, 0x80, 0x07, 0x01, 0x4f, 0x00, 0x80, 0x00, 0x45, 0x02, 0x08, 0x51, 0, 0,
            0x44, 1, 0, 2, 0, 0x02,
        ];
        b[0x24200..0x24200 + script.len()].copy_from_slice(&script);
        b[0x24400..0x2440f].copy_from_slice(&[
            0x50, 8, 0, 0x00, 0x45, 0x02, 0x08, 9, 4, 0x52, 0, 0, 9, 4, 0x02,
        ]);
        b[0x24500] = 0xfe;
        let map = Map {
            id: "2-4".into(),
            group: 2,
            number: 4,
            name: "Initial tiles".into(),
            region: 1,
            width: 10,
            height: 10,
            map_type: 1,
            header: 0,
            layout: 0,
            invalid_events: false,
            events: Some(0x24000),
            scripts: vec![0x24200, 0x24400],
            objects: vec![],
        };
        let before = r.data.clone();
        let report = r.map_events(&map).unwrap();
        let marker = &report.markers[0];
        assert_eq!((marker.x, marker.y, marker.local_id), (2, 3, Some(7)));
        assert_eq!(marker.scripted_movements.len(), 2);
        assert_eq!(
            (
                marker.scripted_movements[0].kind,
                marker.scripted_movements[0].local_id,
                marker.scripted_movements[0].movement_script
            ),
            ("apply", Some(7), Some(0x24500))
        );
        assert!(marker.scripted_movements[1].reuses_last_actor);
        assert_eq!(marker.scripted_movements[1].local_id, None);
        assert_eq!(report.unplaced_movements.len(), 2);
        assert!(report
            .unplaced_movements
            .iter()
            .all(|m| m.map_id.as_deref() == Some("9-4")));
        assert!(marker.stopped_at.contains(&0x24205) && marker.stopped_at.contains(&0x2420c));
        assert!(marker.rewards.iter().all(|r| r.receipt.is_none()));
        assert!(
            r.script_warp_instruction(0x24400, |_| Some(8))
                .unwrap()
                .is_none(),
            "actor-map operands are not passages"
        );
        let index = AcquisitionIndex {
            wild_cache: Default::default(),
            breeding_cache: Default::default(),
            world: World {
                maps: vec![map],
                map_events: vec![report],
                encounters: vec![],
                trainers: vec![],
                trainer_locations: TrainerLocationIndex {
                    locations: vec![],
                    unresolved_maps: vec![],
                },
                map_groups: &[],
            },
            species: vec![],
            evolutions: Default::default(),
            learnsets: Default::default(),
        };
        let sources = index
            .query(
                &r,
                None,
                Target {
                    kind: TargetKind::Item,
                    id: 1,
                },
            )
            .unwrap()
            .sources;
        assert_eq!(sources.len(), 1);
        assert_eq!(
            (
                sources[0].map_id.as_deref(),
                sources[0].x,
                sources[0].y,
                sources[0].status
            ),
            (Some("2-4"), Some(2), Some(3), "unknown")
        );
        assert!(sources[0].partial);
        assert_eq!(r.data, before);
        let b = std::sync::Arc::make_mut(&mut r.data);
        put32(b, rules.commands + 0x4f * 4, 0x08000001);
        assert!(r
            .script_movement_instruction(0x24205, |_| Some(7))
            .is_none());
    }
}

#[test]
#[ignore = "requires five exact ROMs and GEN3_MOVEMENT_PROBES"]
fn local_script_movement_operands_match_native_dispatch_and_lifecycle() {
    use sha2::{Digest, Sha256};
    let proofs: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("GEN3_MOVEMENT_PROBES").unwrap()).unwrap(),
    )
    .unwrap();
    let mut cases = 0;
    for key in ["BW", "DP", "ROCKET", "ULTIMATE", "MERCURY12"] {
        let bytes = std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap();
        let mut r = Rom::open(bytes.clone()).unwrap();
        let p = &proofs[key];
        assert_eq!(p["md5"], r.profile.md5);
        assert_eq!(p["rom_sha256"], format!("{:x}", Sha256::digest(&bytes)));
        let rules = r.profile.event_state.unwrap().effects.unwrap();
        assert_eq!(
            p["handlers"],
            serde_json::to_value(rules.movement_handlers).unwrap()
        );
        let rows = p["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 128);
        for row in rows {
            let operand = row["operand"].as_u64().unwrap() as u16;
            let value = row["resolved"].as_u64().unwrap() as u16;
            let parts = row["map_id"]
                .as_str()
                .unwrap()
                .split('-')
                .map(|v| v.parse::<u8>().unwrap())
                .collect::<Vec<_>>();
            for name in ["apply_opcode", "wait_opcode"] {
                let op = row[name].as_u64().unwrap() as u8;
                let wait = op >= 0x51;
                let mut code = vec![op];
                code.extend(operand.to_le_bytes());
                if !wait {
                    code.extend(0x02018000u32.to_le_bytes());
                }
                if op == 0x50 || op == 0x52 {
                    code.extend(&parts);
                }
                let b = std::sync::Arc::make_mut(&mut r.data);
                b[0x26000..0x26000 + code.len()].copy_from_slice(&code);
                let trace = r
                    .script_movement_instruction(0x26000, |_| Some(value))
                    .unwrap();
                assert_eq!(trace.kind, if wait { "wait" } else { "apply" });
                assert_eq!(trace.reuses_last_actor, wait && value == 0);
                assert_eq!(
                    trace.local_id,
                    if wait && value == 0 {
                        None
                    } else {
                        Some(value as u8)
                    }
                );
                assert_eq!(
                    trace.map_id,
                    if op == 0x50 || op == 0x52 {
                        Some(row["map_id"].as_str().unwrap().into())
                    } else {
                        None
                    }
                );
                assert!(
                    trace.movement_script.is_none(),
                    "RAM-only movement bodies are not ROM offsets"
                );
            }
            assert_eq!(row["finished"], 1);
            cases += 1;
        }
        assert_eq!(
            bytes,
            std::fs::read(std::env::var(format!("GEN3_ROM_{key}")).unwrap()).unwrap()
        );
    }
    assert_eq!(cases, 640);
}
