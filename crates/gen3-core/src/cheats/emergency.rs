//! Exact-ROM, battle-command emergency recovery for Ultimate Emerald 5.5.
//!
//! The payload is authored Thumb code, not extracted ROM content. Its C source
//! and reproducible build/check script live in `scripts/cheat_payloads`.
use super::{binary, err, text, Format, Recipe, Result, RomHalfword, EMERGENCY_HEAL};

const CALLBACK_LITERAL: u32 = 0x39f30;
const CALLBACK_LITERAL_BEFORE: u32 = 0x0300_5d04;
const CAVE_OFFSET: u32 = 0x1fff200;
const CAVE_TARGET: u32 = 0x09ff_f205;
const PAYLOAD: &[u8] = include_bytes!("emergency_ultimate.bin");

pub(super) fn patches() -> Vec<RomHalfword> {
    let mut result = Vec::with_capacity(4 + PAYLOAD.len() / 2);
    for (offset, before, after) in [
        (
            CALLBACK_LITERAL,
            CALLBACK_LITERAL_BEFORE,
            CAVE_OFFSET + 0x0800_0000,
        ),
        (CAVE_OFFSET, 0xffff_ffff, CAVE_TARGET),
    ] {
        for i in 0..2 {
            result.push(RomHalfword {
                offset: offset + 2 * i,
                before: (before >> (i * 16)) as u16,
                after: (after >> (i * 16)) as u16,
            });
        }
    }
    for (i, word) in PAYLOAD.chunks_exact(2).enumerate() {
        result.push(RomHalfword {
            offset: CAVE_OFFSET + 4 + (i as u32) * 2,
            before: 0xffff,
            after: u16::from_le_bytes([word[0], word[1]]),
        });
    }
    result
}

pub(super) fn validate(data: &[u8]) -> Result<()> {
    if PAYLOAD.is_empty() || PAYLOAD.len() % 2 != 0 {
        return Err(err("cheat_payload", "invalid emergency payload length"));
    }
    for patch in patches() {
        if binary::u16(data, patch.offset as usize)? != patch.before {
            return Err(err("cheat_original_bytes", patch.offset));
        }
    }
    Ok(())
}

pub(super) fn recipe() -> Recipe {
    Recipe {
        id: EMERGENCY_HEAL,
        category: "battle",
        parameters: None,
        title: text("战斗紧急整队恢复", "Emergency battle-party recovery"),
        summary: text(
            "在选择战斗指令时按 L＋R＋SELECT，立即复活并治疗全队，不消耗一个吃药回合。",
            "At the battle command menu, press L+R+SELECT to revive and heal the whole party without spending an item turn.",
        ),
        scope: text(
            "究极绿宝石 5.5 · GameShark Advance V1/V2 · 整组代码同时启用",
            "Ultimate Emerald 5.5 · GameShark Advance V1/V2 · enable the complete set",
        ),
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
            text("mGBA 0.10.5 完整对战状态：单打／双打均从低血量和多只昏厥后备队员恢复；在场 HP、PP、异常与血条同步，对手记录保持原样。无组合键、非指令阶段、联机／原野区／多人及伙伴标记、最大 HP 不符和停用状态均不触发。", "mGBA 0.10.5 full battle states: singles and doubles recover from low HP and multiple fainted reserves; active HP, PP, status and health bars synchronize while opponents remain unchanged. No combo, wrong phase, link/Safari/multi/partner flags, mismatched max HP and disabled codes do not trigger."),
            text("只向模拟器内存写入经过完整指纹校验的指令与空白区域；停用时恢复原指令。源 ROM 文件和测试输入存档未改动。", "Only exact-fingerprint checked instructions and unused in-memory ROM space are patched; disabling restores original instructions. Source ROM and test input saves are unchanged."),
        ],
        formats: vec![Format::GamesharkV1V2],
    }
}
