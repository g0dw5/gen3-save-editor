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

pub(super) fn recipe(md5: &str) -> Recipe {
    let scope = if md5 == crate::profile::ROCKET.md5 {
        text(
            "西班牙火箭队 2.1 汉化版 · CodeBreaker · 四行整组启用",
            "Team Rocket 2.1 Chinese · CodeBreaker · enable all four lines",
        )
    } else if md5 == super::ULTIMATE_MD5 {
        text(
            "究极绿宝石 5.5 · CodeBreaker · 四行整组启用",
            "Ultimate Emerald 5.5 · CodeBreaker · enable all four lines",
        )
    } else {
        text(
            "漆黑的魅影 5.0EX · CodeBreaker · 四行整组启用",
            "Dark Phantom 5.0EX · CodeBreaker · enable all four lines",
        )
    };
    Recipe {
        id: ALWAYS_PROTECTED,
        category: "battle",
        parameters: None,
        title: text("己方持续守住", "Persistent player-side Protect"),
        summary: text(
            "战斗中让己方在场宝可梦持续获得游戏原生的守住标记，不占用出招回合。",
            "Give active player-side battlers the game's native Protect flag throughout battle without using a move turn.",
        ),
        scope,
        steps: vec![
            text("战斗前正常保存并备份存档；在模拟器中选 CodeBreaker，完整启用本条目的四行代码。", "Save in-game and back up the save; select CodeBreaker in the emulator and enable all four lines."),
            text("在普通单人对战中保持启用。己方两侧的在场宝可梦每帧补上原生守住标记；退出战斗后停用整组。", "Keep the group enabled during a normal solo battle. The native Protect bit is refreshed for both player-side battler slots; disable the group after battle."),
        ],
        limitations: vec![
            text("只阻挡本作本来受守住影响的招式；混乱自伤、穿透守住的招式、天气和其他间接伤害仍按 ROM 原生规则结算。对手可能因守住状态改变选招。", "Only moves natively affected by Protect are blocked. Confusion self-damage, Protect-piercing moves, weather and other indirect damage retain the ROM's own behavior. The opponent may change move choice in response to Protect."),
            text("代码只给场上己方位置生效，不改变存档中的宝可梦；双打、联机／伙伴战和战斗设施的完整流程尚未验证。不同模拟器对 CodeBreaker 的即时写入时机可能不同。", "Only active player-side slots are affected; no Pokémon save record is changed. Complete doubles, link/partner and facility flows are unverified. Emulator timing for live CodeBreaker writes can vary."),
        ],
        verification: vec![
            text("mGBA 原生战斗状态中，四款精确 MD5 ROM 的单打对照均出现关闭时受伤、启用后不受同次可守住攻击伤害；字节 0x02→0x03 证明其他战斗标记保留。战斗外主回调为零时不写入。", "In native mGBA single-battle states for all four exact-MD5 ROMs, an unpatched player took damage while the enabled group blocked the same Protect-affected attack. A 0x02→0x03 flag check confirmed preservation of other battle flags. The group did not write while the battle callback was null."),
        ],
        formats: vec![Format::Codebreaker],
    }
}
