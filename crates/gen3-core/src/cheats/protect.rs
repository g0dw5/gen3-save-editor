//! Transient native Protect flag for the player's active battlers.
//!
//! CodeBreaker OR preserves every other bit in the shared ProtectStruct word.
//! The repeated condition keeps the code inert when no battle callback is set.
use super::{err, text, Format, Recipe, Result, ALWAYS_PROTECTED};

struct Binding {
    battle_callback: u32,
    protect_structs: u32,
    stride: u32,
}

fn binding(md5: &str) -> Option<Binding> {
    if md5 == crate::mercury::PROFILE.md5 {
        return Some(Binding {
            battle_callback: 0x03004f84,
            protect_structs: 0x02023e8c,
            stride: 16,
        });
    }
    if md5 == crate::profile::ROCKET.md5 {
        return Some(Binding {
            battle_callback: 0x0300_51b4,
            protect_structs: 0x0202_4f6c,
            stride: 20,
        });
    }
    if md5 == super::ULTIMATE_MD5 || md5 == crate::profile::BW.md5 || md5 == crate::profile::DP.md5
    {
        return Some(Binding {
            battle_callback: 0x0300_5d04,
            protect_structs: 0x0202_433c,
            stride: 16,
        });
    }
    None
}

pub(super) fn supported(md5: &str) -> bool {
    binding(md5).is_some()
}

pub(super) fn lines(md5: &str) -> Result<Vec<String>> {
    let b = binding(md5).ok_or_else(|| err("unsupported_feature", "no protect binding"))?;
    let mut lines = Vec::with_capacity(4);
    for battler in [0, 2] {
        // CB_IF_NE: the battle-main callback is null outside battle.
        lines.push(format!("{:08X} 0000", 0xa000_0000 | b.battle_callback));
        // CB_OR_2: turn on only Protect's low bit, preserving other turn flags.
        lines.push(format!(
            "{:08X} 0001",
            0x2000_0000 | (b.protect_structs + battler * b.stride)
        ));
    }
    Ok(lines)
}

pub(super) fn recipe(_md5: &str) -> Recipe {
    Recipe {
        id: ALWAYS_PROTECTED,
        category: "battle",
        parameters: None,
        title: text("己方持续守住", "Persistent player-side Protect"),
        summary: text("己方在场宝可梦持续获得守住效果，不占回合。", "Active player Pokémon stay protected without using a turn."),
        scope: text("整组启用", "Enable the complete set"),
        steps: vec![
text("按 CodeBreaker 整组启用。", "Enable the full set as CodeBreaker."),
text("战斗结束后停用。", "Disable after battle.")
],
        limitations: vec![
text("穿透守住的招式、天气、混乱自伤和其他间接伤害仍生效。", "Protect-piercing moves, weather, confusion self-damage and other indirect damage still apply."),
text("联机、伙伴战及完整设施流程未验证。", "Link, partner and complete facility runs are untested.")
],
        formats: vec![Format::Codebreaker],
    }
}
