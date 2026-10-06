//! Exact-ROM, battle-command emergency recovery across supported games.
//!
//! The payload is authored Thumb code, not extracted ROM content. Its C source
//! and reproducible build/check script live in `scripts/cheat_payloads`.
use super::{binary, err, text, Format, Recipe, Result, RomHalfword, EMERGENCY_HEAL};

const CAVE_OFFSET: u32 = 0x1fff200;
const EMERALD_PAYLOAD: &[u8] = include_bytes!("emergency_ultimate.bin");
const ROCKET_PAYLOAD: &[u8] = include_bytes!("emergency_rocket.bin");
const MERCURY_PAYLOAD: &[u8] = include_bytes!("emergency_mercury.bin");

struct Binding {
    cave_offset: u32,
    callback_literal: u32,
    callback_before: u32,
    cave_before: u32,
    payload: &'static [u8],
}

fn binding(md5: &str) -> Option<Binding> {
    if md5 == crate::mercury::PROFILE.md5 {
        return Some(Binding {
            cave_offset: 0x13fd400,
            callback_literal: 0x12424,
            callback_before: 0x03004f84,
            cave_before: 0xffffffff,
            payload: MERCURY_PAYLOAD,
        });
    }
    let rocket = md5 == crate::profile::ROCKET.md5;
    if rocket {
        return Some(Binding {
            cave_offset: CAVE_OFFSET,
            callback_literal: 0x4ee70,
            callback_before: 0x0300_51b4,
            cave_before: 0xffff_ffff,
            payload: ROCKET_PAYLOAD,
        });
    }
    if md5 == super::ULTIMATE_MD5 || md5 == crate::profile::BW.md5 || md5 == crate::profile::DP.md5
    {
        return Some(Binding {
            cave_offset: CAVE_OFFSET,
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
            config.cave_offset + 0x0800_0000,
        ),
        (
            config.cave_offset,
            config.cave_before,
            config.cave_offset + 0x08000005,
        ),
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
            offset: config.cave_offset + 4 + (i as u32) * 2,
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

pub(super) fn recipe(_md5: &str) -> Recipe {
    Recipe {
        id: EMERGENCY_HEAL,
        category: "battle",
        parameters: None,
        title: text("战斗紧急整队恢复", "Emergency battle-party recovery"),
        summary: text("选择战斗指令时按 L＋R＋SELECT，复活并恢复全队 HP、PP 和持续异常。", "At the command menu, press L+R+SELECT to revive the party and restore HP, PP and persistent status."),
        scope: text("整组启用", "Enable the complete set"),
        steps: vec![
text("暂停模拟器，整组即时启用。", "Pause the emulator and enable the full set live."),
text("回到己方指令菜单，短按 L＋R＋SELECT；血条更新后停用。", "Return to the player command menu. Briefly press L+R+SELECT; disable after the HP bars update.")
],
        limitations: vec![
text("不消耗回合；混乱、替身和能力等级保留。全队败北结算后无法救援。", "Uses no turn. Confusion, Substitute and stat stages remain; cannot reverse an ongoing whiteout."),
text("不支持联机、原野区、伙伴战及最大 HP 已变化的战斗形态；模拟器须支持即时启停。", "Excludes link, Safari, partner battles and forms with changed max HP. Requires live cheat toggling.")
],
        formats: vec![Format::GamesharkV1V2],
    }
}
