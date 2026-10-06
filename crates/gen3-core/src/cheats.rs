//! Read-only, exact-ROM cheat recipes. No save, emulator or file writes.
//!
//! Bindings contain verified engine instructions, never extracted game catalogs.
//! Cheat state is deliberately separate from an open editor Profile/session.
use crate::{binary, err, profile::PROFILES, Result};
use serde::{Deserialize, Serialize};
mod emergency;
mod parameters;
mod protect;
mod storage;
pub use parameters::{Options, Parameters};
use parameters::{ENCOUNTER, SHINY, TELEPORT};

pub const ULTIMATE_MD5: &str = "17ce9785b33319b3dbda9a5d37c57ec1";
pub const NO_PEEK: &str = "disable-input-peeking";
pub const FIX_ACCURACY: &str = "fix-difficulty-accuracy";
pub const NO_ENCOUNTERS: &str = "disable-walking-encounters";
pub const GUARANTEED_CATCH: &str = "guaranteed-wild-catch";
pub const FAST_HATCH: &str = "faster-egg-hatching";
pub const DAYCARE_EGG: &str = "guaranteed-compatible-daycare-egg";
pub const PORTABLE_PC: &str = "portable-pokemon-storage";
pub const EMERGENCY_HEAL: &str = "emergency-battle-heal";
pub const ALWAYS_PROTECTED: &str = "persistent-player-protect";

#[derive(Clone, Copy, Serialize)]
pub struct Text {
    pub zh: &'static str,
    pub en: &'static str,
}
const fn text(zh: &'static str, en: &'static str) -> Text {
    Text { zh, en }
}
#[derive(Clone, Serialize)]
pub struct CheatRom {
    md5: String,
    label: String,
    editor_supported: bool,
    #[serde(skip)]
    options: Options,
}
#[derive(Serialize)]
pub struct Catalog {
    pub rom: CheatRom,
    pub entries: Vec<Recipe>,
    pub options: Options,
}
#[derive(Serialize)]
pub struct Recipe {
    pub id: &'static str,
    pub category: &'static str,
    pub parameters: Option<&'static str>,
    pub title: Text,
    pub summary: Text,
    pub scope: Text,
    pub steps: Vec<Text>,
    pub limitations: Vec<Text>,
    pub formats: Vec<Format>,
}
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    GamesharkV1V2,
    Codebreaker,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateRequest {
    pub expected_rom_md5: String,
    pub cheat_id: String,
    pub format: Format,
    #[serde(default)]
    pub parameters: Option<Parameters>,
}
#[derive(Debug, Serialize)]
pub struct Code {
    pub rom_md5: String,
    pub cheat_id: String,
    pub format: Format,
    pub lines: Vec<String>,
    /// Same protocol, alternate whitespace for VBA-M's format detection.
    pub compact_lines: Vec<String>,
    pub parameters: Option<Parameters>,
}

#[derive(Clone, Copy)]
struct RomHalfword {
    offset: u32,
    before: u16,
    after: u16,
}
const NO_PEEK_PATCHES: [RomHalfword; 2] = [
    RomHalfword {
        offset: 0x1d492ce,
        before: 0xd915,
        after: 0xe015,
    },
    RomHalfword {
        offset: 0x1d4ec46,
        before: 0xd820,
        after: 0xe020,
    },
];
// The original checks the target's side with opposite branch conditions:
// Lunatic boosts attacks targeting the opponent, and Casual boosts attacks
// targeting the player. Reverse only those two conditions.
const FIX_ACCURACY_PATCHES: [RomHalfword; 2] = [
    RomHalfword {
        offset: 0x1d48cbc,
        before: 0xd505,
        after: 0xd405,
    },
    RomHalfword {
        offset: 0x1d48ccc,
        before: 0xd40a,
        after: 0xd50a,
    },
];

/// Semantics and bindings are separate: similar engines still require an explicit
/// binding and independent verification for each exact fingerprint.
struct Binding {
    id: &'static str,
    patches: &'static [RomHalfword],
}
const DARK_PHANTOM: &[Binding] = &[
    Binding {
        id: DAYCARE_EGG,
        // The native charm adds 20 even to incompatible pairs. Retain zero
        // compatibility while this recipe is active, then force positive pairs.
        patches: &[
            RomHalfword {
                offset: 0x70b2c,
                before: 0x4284,
                after: 0x2c00,
            },
            RomHalfword {
                offset: 0x310ebc,
                before: 0x3414,
                after: 0x46c0,
            },
        ],
    },
    Binding {
        id: PORTABLE_PC,
        patches: &storage::DARK_PHANTOM,
    },
    Binding {
        id: NO_ENCOUNTERS,
        patches: &[RomHalfword {
            offset: 0xb52a2,
            before: 0xd100,
            after: 0x46c0,
        }],
    },
    Binding {
        id: GUARANTEED_CATCH,
        patches: &[RomHalfword {
            offset: 0x56566,
            before: 0xd92f,
            after: 0x46c0,
        }],
    },
    Binding {
        id: FAST_HATCH,
        patches: &[RomHalfword {
            offset: 0x70b46,
            before: 0xd13b,
            after: 0x46c0,
        }],
    },
];
const MERCURY: &[Binding] = &[
    Binding {
        id: PORTABLE_PC,
        patches: &storage::MERCURY,
    },
    Binding {
        id: NO_ENCOUNTERS,
        patches: &[RomHalfword {
            offset: 0x1d69e72,
            before: 0xd1f3,
            after: 0xe7f3,
        }],
    },
    Binding {
        id: GUARANTEED_CATCH,
        patches: &[RomHalfword {
            offset: 0x1d0e9e6,
            before: 0xd92e,
            after: 0x46c0,
        }],
    },
    Binding {
        id: FAST_HATCH,
        patches: &[RomHalfword {
            offset: 0x46346,
            before: 0xd12f,
            after: 0x46c0,
        }],
    },
    Binding {
        id: DAYCARE_EGG,
        patches: &[RomHalfword {
            offset: 0x4632c,
            before: 0x4284,
            after: 0x2c00,
        }],
    },
];
const ROCKET: &[Binding] = &[
    Binding {
        id: PORTABLE_PC,
        patches: &storage::ROCKET,
    },
    Binding {
        id: NO_ENCOUNTERS,
        patches: &[RomHalfword {
            offset: 0xec7ec,
            before: 0xd100,
            after: 0x46c0,
        }],
    },
    Binding {
        id: GUARANTEED_CATCH,
        patches: &[RomHalfword {
            offset: 0x80bb4,
            before: 0xd948,
            after: 0x46c0,
        }],
    },
    Binding {
        id: FAST_HATCH,
        patches: &[RomHalfword {
            offset: 0x9eb36,
            before: 0xd13b,
            after: 0x46c0,
        }],
    },
    Binding {
        id: DAYCARE_EGG,
        patches: &[RomHalfword {
            offset: 0x9eb1c,
            before: 0x4284,
            after: 0x2c00,
        }],
    },
];
fn bindings(md5: &str) -> &'static [Binding] {
    match md5 {
        ULTIMATE_MD5 => &[
            Binding {
                id: DAYCARE_EGG,
                patches: &[RomHalfword {
                    offset: 0x70b2c,
                    before: 0x4284,
                    after: 0x2c00,
                }],
            },
            Binding {
                id: NO_PEEK,
                patches: &NO_PEEK_PATCHES,
            },
            Binding {
                id: FIX_ACCURACY,
                patches: &FIX_ACCURACY_PATCHES,
            },
            Binding {
                id: PORTABLE_PC,
                patches: &storage::ULTIMATE,
            },
            Binding {
                id: NO_ENCOUNTERS,
                patches: &[RomHalfword {
                    offset: 0xb52a2,
                    before: 0xd100,
                    after: 0x46c0,
                }],
            },
            Binding {
                id: GUARANTEED_CATCH,
                patches: &[RomHalfword {
                    offset: 0x56566,
                    before: 0xd92f,
                    after: 0x46c0,
                }],
            },
            Binding {
                id: FAST_HATCH,
                patches: &[RomHalfword {
                    offset: 0x70b46,
                    before: 0xd13b,
                    after: 0x46c0,
                }],
            },
        ],
        m if m == crate::profile::BW.md5 || m == crate::profile::DP.md5 => DARK_PHANTOM,
        m if m == crate::profile::ROCKET.md5 => ROCKET,
        m if m == crate::mercury::PROFILE.md5 => MERCURY,
        _ => &[],
    }
}

impl CheatRom {
    pub fn open(data: &[u8]) -> Result<Self> {
        let md5 = binary::hash(data);
        let mut rom = Self::identify(&md5, data.len())?;
        if parameters::supported(&rom.md5) {
            for p in parameters::encounter_for(&md5, 1, 1)
                .into_iter()
                .chain(parameters::shiny_for(&md5))
                .chain(parameters::teleport_for(&md5, 0, 0, 0))
            {
                if binary::u16(data, p.offset as usize)? != p.before {
                    return Err(err("cheat_original_bytes", p.offset));
                }
            }
            rom.options = Options::read(&crate::rom::Rom::open(data.to_vec())?)?;
        }
        for binding in bindings(&md5) {
            // Document and assert every original native instruction, in addition to the hash.
            for p in binding.patches {
                if binary::u16(data, p.offset as usize)? != p.before {
                    return Err(err("cheat_original_bytes", p.offset));
                }
            }
        }
        if emergency::supported(&md5) {
            emergency::validate(data, &md5)?;
        }
        Ok(rom)
    }
    fn identify(md5: &str, size: usize) -> Result<Self> {
        if let Some(p) = PROFILES.iter().find(|p| p.md5 == md5 && p.size == size) {
            return Ok(Self {
                md5: md5.into(),
                label: p.label.into(),
                editor_supported: true,
                options: Options::default(),
            });
        }
        Err(err("unsupported_rom", format!("MD5 {md5}; {size} bytes")))
    }
    pub fn catalog(&self) -> Catalog {
        let mut entries: Vec<_> = bindings(&self.md5)
            .iter()
            .map(|binding| recipe(binding.id))
            .collect();
        if parameters::supported(&self.md5) {
            entries.extend([ENCOUNTER, SHINY, TELEPORT].map(parameter_recipe));
        }
        if emergency::supported(&self.md5) {
            entries.push(emergency::recipe(&self.md5));
        }
        if protect::supported(&self.md5) {
            entries.push(protect::recipe(&self.md5));
        }
        Catalog {
            rom: self.clone(),
            entries,
            options: self.options.clone(),
        }
    }
    pub fn matches(&self, md5: &str) -> bool {
        self.md5 == md5
    }
    pub fn generate(&self, request: &GenerateRequest) -> Result<Code> {
        if !self.matches(&request.expected_rom_md5) {
            return Err(err(
                "cheat_context_mismatch",
                "cheat ROM changed; reopen its catalog",
            ));
        }
        if request.cheat_id == ALWAYS_PROTECTED {
            if request.format != Format::Codebreaker {
                return Err(err(
                    "cheat_format",
                    "persistent Protect requires CodeBreaker",
                ));
            }
            if request.parameters.is_some() {
                return Err(err("cheat_parameters", "this recipe has no parameters"));
            }
            let lines = protect::lines(&self.md5)?;
            return Ok(Code {
                rom_md5: self.md5.clone(),
                cheat_id: request.cheat_id.clone(),
                format: request.format,
                compact_lines: lines.clone(),
                lines,
                parameters: None,
            });
        }
        if request.format != Format::GamesharkV1V2 {
            return Err(err(
                "cheat_format",
                "this recipe requires GameShark Advance V1/V2",
            ));
        }
        let patches = if request.cheat_id == EMERGENCY_HEAL && emergency::supported(&self.md5) {
            if request.parameters.is_some() {
                return Err(err("cheat_parameters", "this recipe has no parameters"));
            }
            emergency::patches(&self.md5).expect("supported binding")
        } else if [ENCOUNTER, SHINY, TELEPORT].contains(&request.cheat_id.as_str()) {
            parameters::generate(self, request)?
        } else {
            if request.parameters.is_some() {
                return Err(err("cheat_parameters", "this recipe has no parameters"));
            }
            let binding = bindings(&self.md5)
                .iter()
                .find(|binding| binding.id == request.cheat_id)
                .ok_or_else(|| {
                    err(
                        "unsupported_feature",
                        "no verified recipe for this ROM and cheat",
                    )
                })?;
            binding.patches.to_vec()
        };
        let lines = patches
            .iter()
            .map(|p| encode_rom_halfword(p.offset, p.after))
            .collect::<Result<Vec<_>>>()?;
        let compact_lines = lines.iter().map(|s| s.replace(' ', "")).collect();
        Ok(Code {
            rom_md5: self.md5.clone(),
            cheat_id: request.cheat_id.clone(),
            format: request.format,
            lines,
            compact_lines,
            parameters: request.parameters.clone(),
        })
    }
}

/// GameShark Advance V1/V2 encrypted ROM halfword replacement.
/// ROM addresses use the physical 0x08000000 window, including its upper 16 MiB.
/// Deliberately not an arbitrary public write-code generator.
fn encode_rom_halfword(offset: u32, value: u16) -> Result<String> {
    if offset >= 0x02000000 || offset & 1 != 0 {
        return Err(err("range", "ROM halfword offset"));
    }
    let mut a = 0x60000000 | ((0x08000000 + offset) >> 1);
    let mut b = u32::from(value);
    let mut sum = 0u32;
    let k = [0x09f4fbbdu32, 0x9681884a, 0x352027e9, 0xf3dee5a7];
    for _ in 0..32 {
        sum = sum.wrapping_add(0x9e3779b9);
        a = a.wrapping_add(
            (b.wrapping_shl(4).wrapping_add(k[0]))
                ^ b.wrapping_add(sum)
                ^ ((b >> 5).wrapping_add(k[1])),
        );
        b = b.wrapping_add(
            (a.wrapping_shl(4).wrapping_add(k[2]))
                ^ a.wrapping_add(sum)
                ^ ((a >> 5).wrapping_add(k[3])),
        );
    }
    Ok(format!("{a:08X} {b:08X}"))
}

fn recipe(id: &'static str) -> Recipe {
    if id == PORTABLE_PC {
        return storage::recipe();
    }
    if id == NO_PEEK {
        return no_peek();
    }
    if id == FIX_ACCURACY {
        return fix_accuracy();
    }
    let (category, title, summary, limitation) = match id {
NO_ENCOUNTERS => ("encounters", text("暂停走路遇敌", "Pause walking encounters"), text("走路、骑车和冲浪时不遇敌。", "No encounters while walking, cycling or surfing."), text("钓鱼、碎岩、甜甜香气及剧情战斗不受影响。", "Fishing, Rock Smash, Sweet Scent and scripted battles remain.")),
GUARANTEED_CATCH => ("catching", text("野生投球必定捕获", "Guaranteed wild capture"), text("野生战斗中投球必定成功，保留实际球种。", "Catch eligible wild Pokémon with the ball you actually use."), text("仍消耗球；保留本作的捕捉限制。", "Balls are consumed; the game’s capture restrictions remain.")),
FAST_HATCH => ("breeding", text("加快同行蛋孵化", "Faster party-egg hatching"), text("行走时更快减少孵化周期，再正常孵化。", "Walking reduces egg cycles faster; hatching proceeds normally."), text("只影响同行的有效蛋，盒子中的蛋不变。", "Only valid party eggs are affected; boxed eggs stay unchanged.")),
DAYCARE_EGG => ("breeding", text("兼容寄养组合必定产蛋", "Guaranteed compatible daycare egg"), text("相容的两只宝可梦在正常检查点必定产蛋。", "Compatible parents produce an egg at the normal checkpoint."), text("仍需行走并领取已有的蛋；不改变遗传，也不让不相容组合产蛋。", "Walk and collect any pending egg first. Inheritance and incompatible pairs stay unchanged.")),
        _ => unreachable!("only verified bindings construct recipes"),
    };
    Recipe {
        id,
        category,
        parameters: None,
        title,
        summary,
        scope: text(
            "GameShark Advance V1/V2 · 无需主码",
            "GameShark Advance V1/V2 · no master code",
        ),
        steps: vec![
            text(
                "整组启用，重启并从游戏内存档继续。",
                "Enable the full set; restart and continue from the in-game save.",
            ),
            text(
                "完成后整组停用并重启。",
                "Disable the full set and restart when finished.",
            ),
        ],
        limitations: vec![limitation],
        formats: vec![Format::GamesharkV1V2],
    }
}

fn parameter_recipe(id: &'static str) -> Recipe {
    let (title, summary, kind, limitation) = match id {
        ENCOUNTER => (text("指定遇怪与等级", "Choose wild Pokémon and level"), text("选择目标，下一场普通野生遭遇生效。可与闪光叠加。", "Choose a target for the next ordinary wild encounter. Can be combined with shiny encounters."), Some("encounter"), text("不主动触发战斗；先关闭暂停遇敌。独立定点、游走、赠送、蛋及训练家不受影响。", "Does not start a battle; disable paused encounters first. Separate static, roamer, gift, egg and trainer paths remain.")),
        SHINY => (text("普通野生必定闪光", "Shiny ordinary wild encounters"), text("新遇到的普通野生宝可梦变为闪光。", "New ordinary wild Pokémon are shiny."), None, text("整组复制，不能截取前几行；不影响已有个体、赠送、蛋及训练家。", "Copy every line. Existing Pokémon, gifts, eggs and trainers are unaffected.")),
        TELEPORT => (text("传送到指定地图", "Teleport to a map"), text("选择区域、地图和入口，进入房门时传送。", "Choose a region, map and entrance, then enter a door to teleport."), Some("teleport"), text("到达后立即停用，确认能行走和离开后再保存。不会解锁剧情；入场脚本仍会执行。道路直接连接不触发。", "Disable immediately on arrival. Check movement and exits before saving. Does not unlock story access; arrival scripts still run. Continuous route connections do not trigger it.")),
        _ => unreachable!(),
    };
    let mut result = recipe(NO_ENCOUNTERS);
    result.id = id;
    result.category = if id == TELEPORT {
        "travel"
    } else {
        "encounters"
    };
    result.title = title;
    result.summary = summary;
    result.parameters = kind;
    result.limitations = vec![limitation];
    if id == TELEPORT {
        result.steps = vec![
            text(
                "门外整组启用，进入房门。",
                "Enable outside a door, then enter.",
            ),
            text(
                "到达后立即整组停用。",
                "Disable the full set immediately on arrival.",
            ),
        ];
    }
    result
}

fn no_peek() -> Recipe {
    Recipe {
        id: NO_PEEK,
        category: "battle",
        parameters: None,
        title: text("关闭 AI 窥屏 · 全模式", "Disable AI input peeking · all modes"),
        summary: text("全部难度与设施关闭本轮输入读取，保留原生预测与评分。", "Disable current-turn input peeking in every mode; retain native prediction and scoring."),
        scope: text("对战 AI · 两条代码须一起启用 · 无需主码", "Battle AI · enable both lines together · no master code required"),
        steps: vec![
text("整组启用，重启后进入新战斗。", "Enable the full set; restart and enter a new battle."),
text("停用整组并重启即可恢复。", "Disable the full set and restart to restore the original behavior.")
],
        limitations: vec![
text("会一并关闭窥屏分支关联的主动换人；原有预测、评分和换人仍保留。", "Also disables proactive switching tied to the peeking branch. Ordinary prediction, scoring and switching remain."),
text("不改变难度、努力值、命中加成或 PP 规则。", "Difficulty, EVs, accuracy bonuses and PP rules stay unchanged.")
],
        formats: vec![Format::GamesharkV1V2],
    }
}

fn fix_accuracy() -> Recipe {
    Recipe {
        id: FIX_ACCURACY,
        category: "battle",
        parameters: None,
        title: text("修正养生／疯子命中加成方向", "Correct Casual/Lunatic accuracy bonus direction"),
        summary: text("养生改为玩家加命中，疯子改为对手加命中；加成仍是相对 +20%。", "Casual boosts player accuracy; Lunatic boosts the opponent's. The relative +20% bonus remains."),
        scope: text("究极绿宝石 5.5 · GameShark Advance V1/V2 · 两行整组启用", "Ultimate Emerald 5.5 · GameShark Advance V1/V2 · enable both lines together"),
        steps: vec![
text("整组启用，重启后进入新战斗。", "Enable the full set; restart and enter a new battle."),
text("停用整组并重启即可恢复。", "Disable the full set and restart to restore the original behavior.")
],
        limitations: vec![
text("只调整加成阵营；疯子仍仅在训练家战生效，其他难度规则不变。", "Only changes the bonus side; Lunatic retains its trainer-battle gate. Other difficulty rules remain.")
],
        formats: vec![Format::GamesharkV1V2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ultimate() -> CheatRom {
        CheatRom::identify(ULTIMATE_MD5, 32 * 1024 * 1024).unwrap()
    }
    fn request() -> GenerateRequest {
        GenerateRequest {
            expected_rom_md5: ULTIMATE_MD5.into(),
            cheat_id: NO_PEEK.into(),
            format: Format::GamesharkV1V2,
            parameters: None,
        }
    }
    #[test]
    fn verified_encoder_vectors_and_boundaries() {
        let code = ultimate().generate(&request()).unwrap();
        assert_eq!(code.lines, ["270AABF9 EF4D3B91", "05EFAF30 6F13BEC4"]);
        assert_eq!(code.compact_lines, ["270AABF9EF4D3B91", "05EFAF306F13BEC4"]);
        let mut accuracy = request();
        accuracy.cheat_id = FIX_ACCURACY.into();
        let code = ultimate().generate(&accuracy).unwrap();
        assert_eq!(code.lines, ["63C417D3 41A376D9", "ADA6DF4E C0A3F156"]);
        assert!(encode_rom_halfword(1, 0).is_err());
        assert!(encode_rom_halfword(0x2000000, 0).is_err());
        assert!(encode_rom_halfword(0x1fffffe, 0xffff).is_ok());
    }
    #[test]
    fn emergency_recovery_is_exact_rom_only_and_has_no_patch_collisions() {
        let mut request = request();
        request.cheat_id = EMERGENCY_HEAL.into();
        for profile in PROFILES.into_iter().filter(|p| emergency::supported(p.md5)) {
            let rom = CheatRom::identify(profile.md5, profile.size).unwrap();
            request.expected_rom_md5 = profile.md5.into();
            let code = rom.generate(&request).unwrap();
            let patches = emergency::patches(profile.md5).unwrap();
            assert_eq!(code.lines.len(), patches.len());
            let (callback, original, cave, blank) = if profile.md5 == crate::mercury::PROFILE.md5 {
                (0x12424, 0x03004f84u32, 0x13fd400, 0xffff)
            } else if profile.md5 == crate::profile::ROCKET.md5 {
                (0x4ee70, 0x030051b4, 0x1fff200, 0xffff)
            } else {
                (
                    0x39f30,
                    0x03005d04,
                    0x1fff200,
                    if profile.md5 == ULTIMATE_MD5 {
                        0xffff
                    } else {
                        0
                    },
                )
            };
            let target = 0x08000000 + cave;
            for (i, (off, value)) in [
                (callback, target as u16),
                (callback + 2, (target >> 16) as u16),
                (cave, (target + 5) as u16),
                (cave + 2, ((target + 5) >> 16) as u16),
            ]
            .into_iter()
            .enumerate()
            {
                assert_eq!(code.lines[i], encode_rom_halfword(off, value).unwrap());
            }
            let mut offsets = std::collections::HashSet::new();
            for patch in &patches {
                assert!(offsets.insert(patch.offset));
                assert_ne!(patch.before, patch.after);
                let expected = if patch.offset == callback {
                    original as u16
                } else if patch.offset == callback + 2 {
                    (original >> 16) as u16
                } else {
                    blank
                };
                assert_eq!(patch.before, expected);
            }
            for binding in bindings(profile.md5) {
                for patch in binding.patches {
                    assert!(!offsets.contains(&patch.offset));
                }
            }
            for patch in parameters::encounter_for(profile.md5, 1, 1)
                .into_iter()
                .chain(parameters::shiny_for(profile.md5))
                .chain(parameters::teleport_for(profile.md5, 0, 0, 0))
            {
                assert!(!offsets.contains(&patch.offset));
            }
            assert!(rom
                .catalog()
                .entries
                .iter()
                .any(|entry| entry.id == EMERGENCY_HEAL));
            let invalid = GenerateRequest {
                expected_rom_md5: profile.md5.into(),
                cheat_id: EMERGENCY_HEAL.into(),
                format: Format::GamesharkV1V2,
                parameters: Some(Parameters::Encounter {
                    species: 1,
                    level: 5,
                }),
            };
            assert_eq!(rom.generate(&invalid).unwrap_err().code, "cheat_parameters");
        }
    }
    #[test]
    fn exact_identity_and_cross_rom_isolation() {
        assert!(CheatRom::open(&[0; 192]).is_err());
        assert!(CheatRom::identify(ULTIMATE_MD5, 1024).is_err());
        for p in PROFILES
            .into_iter()
            .filter(|p| p.md5 != ULTIMATE_MD5 && !bindings(p.md5).is_empty())
        {
            let rom = CheatRom::identify(p.md5, p.size).unwrap();
            let displayed = serde_json::to_value(rom.catalog()).unwrap();
            assert!(displayed["entries"]
                .as_array()
                .unwrap()
                .iter()
                .all(|entry| {
                    entry.get("verification").is_none()
                        && entry.get("steps").is_some()
                        && entry.get("limitations").is_some()
                }));
            assert_eq!(rom.catalog().entries.len(), 10);
            let mut request = request();
            assert_eq!(
                rom.generate(&request).unwrap_err().code,
                "cheat_context_mismatch"
            );
            request.expected_rom_md5 = p.md5.into();
            assert_eq!(
                rom.generate(&request).unwrap_err().code,
                "unsupported_feature"
            );
        }
        let mut request = request();
        request.cheat_id = PORTABLE_PC.into();
        assert_eq!(ultimate().generate(&request).unwrap().lines.len(), 7);
        request.cheat_id = "unknown".into();
        assert!(ultimate().generate(&request).is_err());
    }
    #[test]
    fn rejects_wrong_format_and_unimplemented_parameters() {
        for value in [
            serde_json::json!({"format":"not_a_format"}),
            serde_json::json!({"format":null}),
            serde_json::json!({"format":"gameshark_v1_v2","parameters":{"species":185}}),
        ] {
            let mut v = serde_json::to_value(
                serde_json::json!({"expected_rom_md5":ULTIMATE_MD5,"cheat_id":NO_PEEK}),
            )
            .unwrap();
            v.as_object_mut()
                .unwrap()
                .extend(value.as_object().unwrap().clone());
            assert!(serde_json::from_value::<GenerateRequest>(v).is_err());
        }
        let mut request = request();
        request.format = Format::Codebreaker;
        assert_eq!(
            ultimate().generate(&request).unwrap_err().code,
            "cheat_format"
        );
        request.cheat_id = ALWAYS_PROTECTED.into();
        request.format = Format::GamesharkV1V2;
        assert_eq!(
            ultimate().generate(&request).unwrap_err().code,
            "cheat_format"
        );
    }
    #[test]
    fn native_protect_is_exact_rom_and_keeps_other_bits() {
        for profile in PROFILES.into_iter().filter(|p| protect::supported(p.md5)) {
            let rom = CheatRom::identify(profile.md5, profile.size).unwrap();
            let request = GenerateRequest {
                expected_rom_md5: profile.md5.into(),
                cheat_id: ALWAYS_PROTECTED.into(),
                format: Format::Codebreaker,
                parameters: None,
            };
            let code = rom.generate(&request).unwrap();
            let (callback, protect, stride) = if profile.md5 == crate::mercury::PROFILE.md5 {
                (0x03004f84u32, 0x02023e8cu32, 16)
            } else if profile.md5 == crate::profile::ROCKET.md5 {
                (0x030051b4u32, 0x02024f6cu32, 20)
            } else {
                (0x03005d04u32, 0x0202433cu32, 16)
            };
            assert_eq!(
                code.lines,
                vec![
                    format!("{:08X} 0000", 0xa0000000 | callback),
                    format!("{:08X} 0001", 0x20000000 | protect),
                    format!("{:08X} 0000", 0xa0000000 | callback),
                    format!("{:08X} 0001", 0x20000000 | (protect + 2 * stride)),
                ]
            );
            assert_eq!(code.compact_lines, code.lines);
            assert!(rom
                .catalog()
                .entries
                .iter()
                .any(|e| e.id == ALWAYS_PROTECTED && e.formats == vec![Format::Codebreaker]));
            let mut wrong = request;
            wrong.parameters = Some(Parameters::Encounter {
                species: 1,
                level: 5,
            });
            assert_eq!(rom.generate(&wrong).unwrap_err().code, "cheat_parameters");
        }
    }
    #[test]
    fn common_recipes_remain_scoped_and_do_not_overlap() {
        for p in PROFILES {
            let rom = CheatRom::identify(p.md5, p.size).unwrap();
            let mut ids = std::collections::HashSet::new();
            let mut offsets = std::collections::HashSet::new();
            for binding in bindings(p.md5) {
                assert!(ids.insert(binding.id));
                for patch in binding.patches {
                    assert!(offsets.insert(patch.offset));
                    assert_ne!(patch.before, patch.after);
                }
                let recipe = recipe(binding.id);
                assert!(!recipe.title.zh.is_empty() && !recipe.title.en.is_empty());
                let mut request = GenerateRequest {
                    expected_rom_md5: p.md5.into(),
                    cheat_id: binding.id.into(),
                    format: Format::GamesharkV1V2,
                    parameters: None,
                };
                let code = rom.generate(&request).unwrap();
                assert_eq!(code.rom_md5, p.md5);
                assert_eq!(code.lines.len(), binding.patches.len());
                assert_eq!(code.lines, rom.generate(&request).unwrap().lines);
                request.expected_rom_md5 = if p.md5 == ULTIMATE_MD5 {
                    crate::profile::BW.md5
                } else {
                    ULTIMATE_MD5
                }
                .into();
                assert_eq!(
                    rom.generate(&request).unwrap_err().code,
                    "cheat_context_mismatch"
                );
            }
            let daycare = rom.generate(&GenerateRequest {
                expected_rom_md5: p.md5.into(),
                cheat_id: DAYCARE_EGG.into(),
                format: Format::GamesharkV1V2,
                parameters: None,
            });
            assert!(daycare.is_ok());
            if matches!(p.md5, m if m == crate::profile::BW.md5 || m == crate::profile::DP.md5) {
                assert!(bindings(p.md5)
                    .iter()
                    .find(|b| b.id == DAYCARE_EGG)
                    .unwrap()
                    .patches
                    .iter()
                    .any(|p| p.offset == 0x310ebc && p.before == 0x3414 && p.after == 0x46c0));
            }
        }
    }
    #[test]
    fn parameter_validation_and_patch_isolation() {
        use parameters::{Landing, MapChoice, SpeciesChoice};
        for p in PROFILES
            .into_iter()
            .filter(|p| parameters::supported(p.md5))
        {
            let mut rom = CheatRom::identify(p.md5, p.size).unwrap();
            rom.options = Options {
                species: vec![SpeciesChoice {
                    id: 400,
                    name: "Fixture".into(),
                }],
                maps: vec![MapChoice {
                    id: "1-2".into(),
                    group: 1,
                    number: 2,
                    name: "Fixture".into(),
                    region: 1,
                    code: "01 02".into(),
                    landings: vec![Landing { id: 0, x: 2, y: 3 }],
                }],
            };
            let mut r = GenerateRequest {
                expected_rom_md5: p.md5.into(),
                cheat_id: ENCOUNTER.into(),
                format: Format::GamesharkV1V2,
                parameters: None,
            };
            assert!(rom.generate(&r).is_err());
            for (species, level) in [(400, 0), (400, 101), (0, 5), (65535, 5)] {
                r.parameters = Some(Parameters::Encounter { species, level });
                assert!(rom.generate(&r).is_err());
            }
            r.parameters = Some(Parameters::Encounter {
                species: 400,
                level: 100,
            });
            assert_eq!(
                rom.generate(&r).unwrap().lines.len(),
                if p.md5 == ULTIMATE_MD5 {
                    8
                } else if p.md5 == crate::mercury::PROFILE.md5 {
                    16
                } else {
                    4
                }
            );
            r.cheat_id = TELEPORT.into();
            assert!(rom.generate(&r).is_err());
            for (map_id, warp_id, valid) in [("1-2", 0, true), ("1-2", 1, false), ("2-1", 0, false)]
            {
                r.parameters = Some(Parameters::Teleport {
                    map_id: map_id.into(),
                    warp_id,
                });
                assert_eq!(rom.generate(&r).is_ok(), valid);
            }
            r.cheat_id = SHINY.into();
            assert!(rom.generate(&r).is_err());
            r.parameters = None;
            assert_eq!(
                rom.generate(&r).unwrap().lines.len(),
                if p.md5 == ULTIMATE_MD5 {
                    28
                } else if p.md5 == crate::mercury::PROFILE.md5 {
                    129
                } else {
                    86
                }
            );
            let mut offsets = std::collections::HashSet::new();
            for patch in bindings(p.md5)
                .iter()
                .flat_map(|b| b.patches.iter().copied())
                .chain(parameters::encounter_for(p.md5, 400, 100))
                .chain(parameters::shiny_for(p.md5))
                .chain(parameters::teleport_for(p.md5, 1, 2, 0))
            {
                assert!(offsets.insert(patch.offset));
                assert!(patch.offset < p.size as u32);
            }
        }
        for value in [
            serde_json::json!({"kind":"encounter","species":null,"level":5}),
            serde_json::json!({"kind":"encounter","species":1,"level":1.5}),
            serde_json::json!({"kind":"encounter","species":1,"level":256}),
            serde_json::json!({"kind":"teleport","map_id":"1-2","warp_id":0,"x":1}),
        ] {
            assert!(serde_json::from_value::<Parameters>(value).is_err());
        }
    }

    #[test]
    fn api_never_generates_without_a_loaded_identity() {
        let mut app = crate::app::App::default();
        let result = app.dispatch(crate::app::Request {
            command: "cheat_code".into(),
            payload: serde_json::json!({"expected_rom_md5": ULTIMATE_MD5, "cheat_id": NO_PEEK, "format": "gameshark_v1_v2"}),
        });
        assert_eq!(result.unwrap_err().code, "cheat_context_mismatch");
        assert!(app.session.is_none());
    }

    #[test]
    #[ignore = "requires five private ROMs via GEN3_ROM_ULTIMATE/BW/DP/ROCKET/MERCURY12"]
    fn local_cheat_catalog_cross_rom_regression() {
        use crate::app::{App, Request};
        use serde_json::{json, Value};
        let dispatch = |app: &mut App, command: &str, payload: Value| {
            app.dispatch(Request {
                command: command.into(),
                payload,
            })
        };
        let mut app = App::default();
        let ue_path = std::env::var("GEN3_ROM_ULTIMATE").expect("GEN3_ROM_ULTIMATE");
        let original = std::fs::read(&ue_path).unwrap();
        dispatch(&mut app, "open_rom", json!({"path": ue_path})).unwrap();
        let catalog = dispatch(
            &mut app,
            "cheats",
            json!({"expected_rom_md5": ULTIMATE_MD5}),
        )
        .unwrap();
        assert_eq!(catalog["entries"].as_array().unwrap().len(), 12);
        assert!(catalog["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["id"] == EMERGENCY_HEAL));
        assert_eq!(catalog["rom"]["editor_supported"], true);
        assert!(app.session.is_some());
        let request = json!({"expected_rom_md5": ULTIMATE_MD5, "cheat_id": NO_PEEK, "format": "gameshark_v1_v2"});
        assert_eq!(
            dispatch(&mut app, "cheat_code", request.clone()).unwrap()["lines"],
            json!(["270AABF9 EF4D3B91", "05EFAF30 6F13BEC4"])
        );
        assert_eq!(
            dispatch(
                &mut app,
                "cheat_code",
                json!({"expected_rom_md5": ULTIMATE_MD5,
                "cheat_id": FIX_ACCURACY, "format": "gameshark_v1_v2"})
            )
            .unwrap()["lines"],
            json!(["63C417D3 41A376D9", "ADA6DF4E C0A3F156"])
        );
        let mut modified = original.clone();
        modified[0x1d492cf] = 0xe0;
        assert_eq!(
            CheatRom::open(&modified).err().unwrap().code,
            "unsupported_rom"
        );
        for (env, profile) in [
            ("GEN3_ROM_BW", crate::profile::BW),
            ("GEN3_ROM_DP", crate::profile::DP),
            ("GEN3_ROM_ROCKET", crate::profile::ROCKET),
            ("GEN3_ROM_MERCURY12", crate::mercury::PROFILE),
        ] {
            let path = std::env::var(env).expect(env);
            let source = std::fs::read(&path).unwrap();
            dispatch(&mut app, "open_rom", json!({"path": path})).unwrap();
            let baseline = app.session.as_ref().unwrap().rom.data.clone();
            let catalog =
                dispatch(&mut app, "cheats", json!({"expected_rom_md5": profile.md5})).unwrap();
            assert_eq!(catalog["entries"].as_array().unwrap().len(), 10);
            assert_eq!(catalog["rom"]["md5"], profile.md5);
            for entry in catalog["entries"].as_array().unwrap() {
                let code = dispatch(
                    &mut app,
                    "cheat_code",
                    json!({
                        "expected_rom_md5": profile.md5,
                        "cheat_id": entry["id"],
                        "format": entry["formats"][0],
                        "parameters": match entry["parameters"].as_str() {
                            Some("encounter") => json!({"kind":"encounter", "species":catalog["options"]["species"][0]["id"], "level":5}),
                            Some("teleport") => {
                                let m = catalog["options"]["maps"].as_array().unwrap().iter().find(|m| !m["landings"].as_array().unwrap().is_empty()).unwrap();
                                json!({"kind":"teleport", "map_id":m["id"], "warp_id":m["landings"][0]["id"]})
                            },
                            _ => serde_json::Value::Null,
                        }
                    }),
                )
                .unwrap();
                assert_eq!(code["rom_md5"], profile.md5);
                assert!(!code["lines"].as_array().unwrap().is_empty());
            }
            assert!(std::sync::Arc::ptr_eq(
                &baseline,
                &app.session.as_ref().unwrap().rom.data
            ));
            assert!(app.session.as_ref().unwrap().save.is_none());
            assert_eq!(
                dispatch(&mut app, "cheat_code", request.clone())
                    .unwrap_err()
                    .code,
                "cheat_context_mismatch"
            );
            let mut cross = request.clone();
            cross["expected_rom_md5"] = json!(profile.md5);
            assert_eq!(
                dispatch(&mut app, "cheat_code", cross).unwrap_err().code,
                "unsupported_feature"
            );
            assert_eq!(source, std::fs::read(path).unwrap());
        }
        assert_eq!(original, std::fs::read(ue_path).unwrap());
    }
}
