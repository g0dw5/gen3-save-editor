//! Exact-ROM, battle-command emergency recovery across supported games.
//!
//! The payload is authored Thumb code, not extracted ROM content. Its C source
//! and reproducible build/check script live in `scripts/cheat_payloads`.
use super::{binary, err, text, Format, Recipe, Result, RomHalfword, EMERGENCY_HEAL};

const CAVE_OFFSET: u32 = 0x1fff200;
const CAVE_TARGET: u32 = 0x09ff_f205;
const EMERALD_PAYLOAD: &[u8] = include_bytes!("emergency_ultimate.bin");
const ROCKET_PAYLOAD: &[u8] = include_bytes!("emergency_rocket.bin");

struct Binding {
    callback_literal: u32,
    callback_before: u32,
    cave_before: u32,
    payload: &'static [u8],
}

fn binding(md5: &str) -> Option<Binding> {
    let rocket = md5 == crate::profile::ROCKET.md5;
    if rocket {
        return Some(Binding {
            callback_literal: 0x4ee70,
            callback_before: 0x0300_51b4,
            cave_before: 0xffff_ffff,
            payload: ROCKET_PAYLOAD,
        });
    }
    if md5 == super::ULTIMATE_MD5 || md5 == crate::profile::BW.md5 || md5 == crate::profile::DP.md5
    {
        return Some(Binding {
            callback_literal: 0x39f30,
            callback_before: 0x0300_5d04,
            cave_before: if md5 == super::ULTIMATE_MD5 {
                0xffff_ffff
            } else {
                0
            },
            payload: EMERALD_PAYLOAD,
        });
    }
    None
}

pub(super) fn supported(md5: &str) -> bool {
    binding(md5).is_some()
}

pub(super) fn patches(md5: &str) -> Option<Vec<RomHalfword>> {
    let config = binding(md5)?;
    let mut result = Vec::with_capacity(4 + config.payload.len() / 2);
    for (offset, before, after) in [
        (
            config.callback_literal,
            config.callback_before,
            CAVE_OFFSET + 0x0800_0000,
        ),
        (CAVE_OFFSET, config.cave_before, CAVE_TARGET),
    ] {
        for i in 0..2 {
            result.push(RomHalfword {
                offset: offset + 2 * i,
                before: (before >> (i * 16)) as u16,
                after: (after >> (i * 16)) as u16,
            });
        }
    }
    for (i, word) in config.payload.chunks_exact(2).enumerate() {
        result.push(RomHalfword {
            offset: CAVE_OFFSET + 4 + (i as u32) * 2,
            before: (config.cave_before & 0xffff) as u16,
            after: u16::from_le_bytes([word[0], word[1]]),
        });
    }
    Some(result)
}

pub(super) fn validate(data: &[u8], md5: &str) -> Result<()> {
    let config = binding(md5).ok_or_else(|| err("unsupported_feature", "no emergency binding"))?;
    if config.payload.is_empty() || config.payload.len() % 2 != 0 {
        return Err(err("cheat_payload", "invalid emergency payload length"));
    }
    for patch in patches(md5).expect("validated binding") {
        if binary::u16(data, patch.offset as usize)? != patch.before {
            return Err(err("cheat_original_bytes", patch.offset));
        }
    }
    Ok(())
}

pub(super) fn recipe(md5: &str) -> Recipe {
    let rocket = md5 == crate::profile::ROCKET.md5;
    let scope = if rocket {
        text(
            "西班牙火箭队 2.1 汉化版 · GameShark Advance V1/V2 · 整组启用",
            "Team Rocket 2.1 Chinese · GameShark Advance V1/V2 · enable the complete set",
        )
    } else if md5 == super::ULTIMATE_MD5 {
        text(
            "究极绿宝石 5.5 · GameShark Advance V1/V2 · 整组启用",
            "Ultimate Emerald 5.5 · GameShark Advance V1/V2 · enable the complete set",
        )
    } else {
        text(
            "漆黑的魅影 5.0EX · GameShark Advance V1/V2 · 整组启用",
            "Dark Phantom 5.0EX · GameShark Advance V1/V2 · enable the complete set",
        )
    };
    Recipe {
        id: EMERGENCY_HEAL,
        category: "battle",
        parameters: None,
        title: text("战斗紧急整队恢复", "Emergency battle-party recovery"),
        summary: text(
            "在选择战斗指令时按 L＋R＋SELECT，立即复活并治疗全队，不消耗一个吃药回合。",
            "At the battle command menu, press L+R+SELECT to revive and heal the whole party without spending an item turn.",
        ),
        scope,
        steps: vec![
            text("先在战斗外正常保存，并备份电池存档；救急时暂停模拟器。", "Save in-game outside battle and back up the battery save; pause the emulator when rescue is needed."),
            text("将本条目的全部代码作为一组 GameShark Advance V1/V2 启用。该功能需要模拟器即时应用代码，不能重启后再回到同一场战斗。", "Enable all lines as one GameShark Advance V1/V2 set. This feature requires live cheat application; restarting cannot return to the same battle."),
            text("回到己方选择“战斗／背包／宝可梦／逃跑”或招式的界面，短按 L＋R＋SELECT。看到 HP 条更新后立即停用整组。", "At the player's action or move menu, briefly press L+R+SELECT. Disable the complete set after the HP bars update."),
            text("战斗结束后核对队伍，再正常保存。若模拟器不能即时停用补丁，先结束战斗，再重启并从游戏内存档确认。", "Check the party after battle, then save normally. If the emulator cannot remove live patches, finish the battle and restart from the in-game save to confirm."),
        ],
        limitations: vec![
            text("原生全队治疗会恢复 HP、PP 和持续性异常；在场己方的 HP、PP、持续性异常和血条也同步。混乱、替身、能力等级等临时战斗状态不清除。", "The ROM's native party heal restores HP, PP and persistent status. Active player battlers and their health bars are synchronized. Temporary battle effects such as confusion, Substitute and stat stages remain."),
            text("只在己方指令阶段触发。已进入全队败北结算时来不及救援；场上宝可梦若已昏厥，不强行恢复其战斗对象。联机、野生原野区、多人／剧情伙伴战斗，以及场上最大 HP 与队伍记录不一致的形态被保护条件排除。", "Triggers only during player command selection. It cannot reverse an already running whiteout; a fainted active battler is not forced back into the field. Link, Safari, multi/partner battles and active forms whose max HP differs from party data are guarded out."),
            text("仅适用于本页完整 MD5 对应的 ROM。需临时开关整组；不同模拟器对即时 GameShark 补丁的支持可能不同。手机模拟器和全部设施流程尚未实测。正常保存会保留恢复后的队伍状态。", "Only for this exact ROM MD5. Enable and disable the whole set temporarily. Live GameShark patching can differ by emulator. Mobile emulators and complete facility runs are untested. Normal saves retain the restored party state."),
        ],
        verification: vec![
            text("mGBA 完整对战状态：四款 ROM 的单打均从低血量和昏厥后备队员恢复，究极绿宝石另验证双打；在场 HP、PP、异常同步，对手记录保持原样。无组合键、联机／原野区／多人及伙伴标记、最大 HP 不符和停用状态均不触发；究绿另验证非指令阶段。", "mGBA full battle states: all four ROMs recover from low HP and fainted reserves in singles; Ultimate Emerald was additionally tested in doubles. Active HP, PP and status synchronize while opponent records remain unchanged. No combo, link/Safari/multi/partner flags, mismatched max HP and disabled codes do not trigger; the wrong-phase guard was separately tested in Ultimate Emerald."),
            text("只向模拟器内存写入经过完整指纹校验的指令与空白区域；停用时恢复原指令。源 ROM 文件和测试输入存档未改动。", "Only exact-fingerprint checked instructions and unused in-memory ROM space are patched; disabling restores original instructions. Source ROM and test input saves are unchanged."),
        ],
        formats: vec![Format::GamesharkV1V2],
    }
}
