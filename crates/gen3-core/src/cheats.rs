//! Read-only, exact-ROM cheat recipes. No save, emulator or file writes.
//!
//! Bindings contain verified engine instructions, never extracted game catalogs.
//! Cheat state is deliberately separate from an open editor Profile/session.
use crate::{binary, err, profile::PROFILES, Result};
use serde::{Deserialize, Serialize};
mod parameters;
mod storage;
pub use parameters::{Options, Parameters};
use parameters::{ENCOUNTER, SHINY, TELEPORT};

pub const ULTIMATE_MD5: &str = "17ce9785b33319b3dbda9a5d37c57ec1";
pub const NO_PEEK: &str = "disable-input-peeking";
pub const NO_ENCOUNTERS: &str = "disable-walking-encounters";
pub const GUARANTEED_CATCH: &str = "guaranteed-wild-catch";
pub const FAST_HATCH: &str = "faster-egg-hatching";
pub const DAYCARE_EGG: &str = "guaranteed-compatible-daycare-egg";
pub const PORTABLE_PC: &str = "portable-pokemon-storage";

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
    pub verification: Vec<Text>,
    pub formats: Vec<Format>,
}
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    GamesharkV1V2,
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

/// Semantics and bindings are separate: similar engines still require an explicit
/// binding and independent verification for each exact fingerprint.
struct Binding {
    id: &'static str,
    patches: &'static [RomHalfword],
}
const DARK_PHANTOM: &[Binding] = &[
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
                id: NO_PEEK,
                patches: &NO_PEEK_PATCHES,
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
        _ => &[],
    }
}

impl CheatRom {
    pub fn open(data: &[u8]) -> Result<Self> {
        let md5 = binary::hash(data);
        let mut rom = Self::identify(&md5, data.len())?;
        if parameters::supported(&rom.md5) {
            let rocket = md5 == crate::profile::ROCKET.md5;
            for p in parameters::encounter_for(&md5, 1, 1)
                .into_iter()
                .chain(parameters::shiny_for(&md5))
                .chain(parameters::teleport(rocket, 0, 0, 0))
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
        let patches = if [ENCOUNTER, SHINY, TELEPORT].contains(&request.cheat_id.as_str()) {
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
    let (category, title, summary, limitation, evidence) = match id {
        NO_ENCOUNTERS => (
            "encounters",
            text("暂停走路遇敌", "Pause walking encounters"),
            text("走路、骑车或冲浪移动时，跳过原生随机遇敌入口。适合赶路、捡道具。", "Skip the native random-encounter entry while walking, cycling or moving on water. Useful for travel and item collection."),
            text("也会挡住该入口触发的游走、群聚或设施随机战。不关闭 NPC、脚本定点、钓鱼、甜甜香气和碎岩的独立触发；已经开始的战斗不会结束。", "Also blocks roamers, outbreaks and facility random encounters using this entry. Separate NPC, scripted, fishing, Sweet Scent and Rock Smash triggers remain; an existing battle is not ended."),
            text("按本 ROM 的原生移动遇敌入口执行有／无补丁对照；开启后返回不遇敌，停用恢复原有入口。独立野生生成与训练家流程未被补丁覆盖。", "Patched/unpatched tests execute this ROM's native movement encounter entry. Enabling returns no encounter; disabling restores the entry. Separate wild constructors and trainer paths are not patched."),
        ),
        GUARANTEED_CATCH => (
            "catching",
            text("野生战斗投球必定捕获", "Guaranteed wild-battle capture"),
            text("在能正常投球的野生战斗中，让捕捉判定走游戏原有的成功分支。仍记录实际使用的球，不把球改成大师球。", "Use the game's native success path when a ball can normally be thrown in a wild battle. The actual ball used is recorded; it is not replaced with a Master Ball."),
            text("保留训练家挡球和教程分支，不解锁禁用背包、禁用捕捉或剧情限制。仍消耗正常投出的球。已捕获个体和图鉴登记会随正常保存留下，关闭代码不会撤销。", "Trainer ball-blocking and tutorial branches remain. Does not unlock banned bags, blocked captures or story restrictions. Balls are still consumed normally. Caught Pokémon and dex registrations persist when saved; disabling does not undo them."),
            text("mGBA 完整执行原生投球指令，对照不同随机种子、野生／训练家／教程分支及用球记录，按各 ROM 的原生规则核对个体。不是对所有剧情捕捉限制的穷举。", "mGBA executes the complete native ball-throw command with different seeds and wild/trainer/tutorial cases, checking success scripts, ball records and Pokémon using each ROM's native rules. Not exhaustive coverage of all story capture restrictions."),
        ),
        FAST_HATCH => (
            "breeding",
            text("加快同行蛋孵化", "Faster party-egg hatching"),
            text("每次有效行走步数检查都扣减蛋的孵化周期，不再等到原来的周期检查点；继续使用本作原有的特性加速和孵化流程。", "Reduce egg cycles at each eligible walking-step check instead of waiting for the usual cycle boundary. The ROM's ability bonuses and normal hatching flow remain."),
            text("只处理同行中有效的蛋，跳过坏蛋，盒子里的蛋不变。剩余周期仍需走完，降为 0 后下一次检查触发正常孵化，不是直接把蛋标记成已孵化。已经减少的周期不会因停用而回退。", "Only valid party eggs are processed; Bad Eggs and boxed eggs are untouched. Remaining cycles still count down, then the next check triggers normal hatching. This does not simply clear the egg flag. Progress already made is not rolled back when disabled."),
            text("mGBA 执行原生孵蛋入口，检查个体记录、周期边界、普通同行／坏蛋跳过、特性加速及关闭恢复；无关字段逐字节比较。", "mGBA native hatching-entry tests cover Pokémon records, cycle boundaries, non-egg/Bad Egg skips, ability bonuses and disable/restore. Unrelated fields are compared byte for byte."),
        ),
        DAYCARE_EGG => (
            "breeding",
            text("兼容寄养组合必定产蛋", "Guaranteed egg for compatible daycare parents"),
            text("在原有产蛋检查点，将相性大于 0 的组合改为必定产蛋。仍需要走到检查点、寄养两只宝可梦，且没有尚未领取的蛋。", "At the normal breeding checkpoint, make a positive-compatibility pair produce an egg. Still requires reaching the checkpoint, two daycare parents and no pending egg."),
            text("仅支持已核对的西班牙火箭队。不改变遗传、父母身份、蛋的生成方式或步数门槛；不兼容组合仍不产蛋。新产生的待领取蛋可随存档保留，停用不会删除。", "Only available for the verified Team Rocket ROM. Inheritance, parent identity, egg generation and the step threshold remain. Incompatible parents still produce no egg. A pending egg can persist in a save and is not removed by disabling."),
            text("mGBA 对照正常相性和圆形护身符、兼容／不兼容组合、空寄养位、已有待领取蛋与非检查点；父母加密记录保持不变。漆黑的魅影护符分支存在不同语义，因此未套用。", "mGBA compares compatibility with/without Oval Charm, compatible/incompatible pairs, empty slots, pending eggs and non-checkpoint steps. Parent records stay unchanged. Dark Phantom's charm branch has different semantics and is not given this recipe."),
        ),
        _ => unreachable!("only private verified bindings construct recipes"),
    };
    Recipe {
        id,
        category,
        parameters: None,
        title,
        summary,
        scope: text("GameShark Advance V1/V2 · 无需主码 · 复制完整代码组", "GameShark Advance V1/V2 · no master code · copy the complete set"),
        steps: vec![
            text("战斗、孵化动画和保存过程之外，正常保存并备份电池存档。", "Outside battles, hatching animations and saving, save in-game and back up the battery save."),
            text("在模拟器中按 GameShark Advance V1/V2 添加本条目的全部代码，作为同一组启用；不要选 CodeBreaker 或 Action Replay V3。", "Add all lines as one GameShark Advance V1/V2 set in the emulator. Do not select CodeBreaker or Action Replay V3."),
            text("重启游戏，从游戏内存档继续。投球功能请新开野生战斗；孵蛋和产蛋功能请继续行走。不要拿旧即时存档的结果判断启用状态。", "Restart and continue from the in-game save. Enter a new wild battle for capture; walk for egg features. Do not infer activation from an old save state."),
            text("停用整组后重启。指令补丁恢复不等于撤销已经捕获的宝可梦或已经发生的孵化／产蛋。", "Disable the full set and restart. Restoring instructions does not undo captures, hatching progress or eggs already produced."),
        ],
        limitations: vec![limitation,
            text("只支持本页完整 MD5 对应的原始 ROM。手机模拟器尚未实测，不保证仅凭相同代码格式名称就兼容。不要与其他修改同一功能的代码混用。", "Only the original ROM with this exact MD5 is supported. Mobile emulators are untested; matching format names do not prove compatibility. Do not combine with other cheats changing the same feature."),
        ],
        verification: vec![evidence,
            text("验证引擎：mGBA 0.10.5。逐条解析最终代码、比较 ROM 补丁差异，执行启用／停用／重置检查；源 ROM 和用户存档不写入。测试包含受控内存夹具，不代表全剧情或长期手机游玩的保证。", "Test engine: mGBA 0.10.5. Final codes are independently decoded, ROM patch differences compared and enable/disable/reset checked. Source ROMs and user saves are not written. Tests include controlled RAM fixtures, not a guarantee of full-story or long mobile play."),
            text("VBA-M 16 位无空格选项只是输入排版；这些新增条目未逐项在 VBA-M 实测。当前已验证范围以 mGBA 记录为准。", "The compact VBA-M option is only an input layout; these new recipes have not each been tested in VBA-M. Current verified coverage is the mGBA record."),
        ],
        formats: vec![Format::GamesharkV1V2],
    }
}

fn parameter_recipe(id: &'static str) -> Recipe {
    let (title, summary, kind, limitation) = match id {
        ENCOUNTER => (text("指定野生宝可梦与等级", "Choose wild Pokémon and level"),
            text("选择当前 ROM 的宝可梦和等级，在下一场普通野生遭遇中生成。可与闪光代码一起使用。", "Choose a Pokémon and level from this ROM for the next ordinary wild encounter. Can be combined with the shiny recipe."), Some("encounter"),
            text("改变普通野生生成入口，不覆盖单独生成的游走、定点、礼物、蛋和训练家。不会主动触发战斗；走路遇敌暂停时，请先关闭暂停。列表不提供临时战斗形态。", "Changes the ordinary wild constructor, not separate roamer, static, gift, egg or trainer constructors. Does not trigger a battle; disable paused walking encounters first. Temporary battle forms are excluded.")),
        SHINY => (text("普通野生遭遇必定闪光", "Shiny ordinary wild encounters"),
            text("仅在普通野生生成链中构造闪光 PID，继续执行本作原生性格、性别筛选和个体保存。", "Construct a shiny PID only in the ordinary wild generation chain, retaining native nature/gender selection and Pokémon record handling."), None,
            text("只影响新生成的普通野生个体，保留同步和迷人之躯；不把已有宝可梦、礼物、蛋或训练家的宝可梦变闪。完整代码组较长，必须一次性全部启用，不能只复制前几行。", "Only newly generated ordinary wild Pokémon; Synchronize and Cute Charm remain. Existing Pokémon, gifts, eggs and trainer Pokémon are not made shiny. Enable the entire long code set together, never just its first few lines.")),
        TELEPORT => (text("传送到指定地图", "Teleport to a chosen map"),
            text("按区域和地图选择目的地，查看十进制地图编号和十六进制组／图编码。下一次经原生传送入口切图时替换目的地。", "Choose a region and map, with decimal map IDs and hexadecimal group/map codes. Redirect the next transition using the native warp setter."), Some("teleport"),
            text("仅开放 ROM 中有入口引用、坐标有效的落点；这不证明当前剧情可达或已实机走遍。进图脚本仍会运行，可能触发剧情。不要在战斗、动画或保存时使用；到达后立即停用整组，确认可行走与出入后再保存。连续道路连接、部分动态返回入口不经此函数，不会触发。", "Only referenced, in-bounds ROM landings are offered; this is not proof of story reachability or gameplay testing of every map. Arrival scripts still run and may advance events. Use outside battles, animations and saving. Disable the whole set immediately after arrival; verify movement and exits before saving. Continuous map connections and some dynamic return warps bypass this setter.")),
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
    result.limitations[0] = limitation;
    result.steps[2] = if id == TELEPORT {
        text("备份后在门外启用，进入普通房门触发传送。到达后立即关闭整组；下次切图前确认代码已停用。若模拟器不能即时恢复 ROM 指令，请先保留原存档再重启验证。", "After backing up, enable outside a door and enter it. Disable the complete set immediately on arrival, before another transition. If the emulator cannot restore ROM instructions live, retain the original save and restart to verify.")
    } else {
        text("重启后进入新的普通野生战斗。指定遇怪与闪光可叠加；更换目标前先停用旧的整组代码。", "Restart and enter a new ordinary wild battle. Species/level and shiny recipes can be combined. Disable the previous complete set before changing the target.")
    };
    result.verification[0] = text("mGBA 原生函数夹具验证生成、个体记录与代码启停；地图落点来自当前 ROM 的门／洞口记录，未承诺所有地图剧情可达。", "mGBA native-function fixtures verify generation, Pokémon records and code toggling. Landings come from this ROM's warp records; not all maps are claimed story-reachable.");
    result
}

fn no_peek() -> Recipe {
    Recipe {
        id: NO_PEEK,
        category: "battle",
        parameters: None,
        title: text("关闭 AI 窥屏 · 全模式", "Disable AI input peeking · all modes"),
        summary: text("让 AI 走游戏已有的不窥屏分支，关闭读取本轮玩家指令的组合模式。包括挑战、疯子及特殊设施。", "Select the game's existing non-peeking path, disabling the combined mode that reads the player's current input. Covers Challenge, Lunatic and special facilities."),
        scope: text("对战 AI · 两条代码须一起启用 · 无需主码", "Battle AI · enable both lines together · no master code required"),
        steps: vec![
            text("先在战斗外正常保存游戏，并备份电池存档。", "Save in-game outside battle and back up the battery save."),
            text("选择 GameShark Advance V1/V2，将下面两条代码作为同一组一起启用。不是 CodeBreaker，也不是 Action Replay V3。", "Select GameShark Advance V1/V2 and enable both lines as one set. These are not CodeBreaker or Action Replay V3 codes."),
            text("重新启动游戏，从游戏内存档继续，再进入一场新战斗。不要用旧的战斗中即时存档判断是否生效。", "Restart the game, continue from the in-game save, then enter a new battle. Do not test using an old mid-battle save state."),
            text("停用时关闭整组两条代码并重新启动游戏。此窗口只提供代码，不会替模拟器启用，也不会改写 ROM 或存档文件。", "To stop, disable both lines and restart. This window only provides codes; it does not activate them in an emulator or write ROM/save files."),
        ],
        limitations: vec![
            text("这会连同该组合开关关联的部分主动换人逻辑一起关闭；保留不窥屏分支原有的预测、评分和换人。不是对整个 ROM 做了信息隔离。", "Also disables proactive-switch behavior associated with this combined flag. Existing prediction, scoring and switching on the non-peeking path remain. This is not a whole-ROM information-isolation guarantee."),
            text("不改难度本身：超额努力值、命中加成、保留 1 HP、不消耗 PP 等规则保持原样。保留原位置的随机数调用，不保证后续整场随机结果相同。", "Does not change difficulty bonuses such as excess EVs, accuracy, surviving at 1 HP or no PP consumption. Original RNG calls remain; later battle outcomes need not match."),
            text("Manic EMU、Delta 及其他手机模拟器尚未实测；它们对 GameShark 格式的支持不能仅凭名称推定。只适用于本页完整 MD5 对应的未改 ROM。", "Manic EMU, Delta and other mobile emulators are not tested. A format name alone does not establish compatibility. Only the unmodified ROM with this exact MD5 is supported."),
        ],
        verification: vec![
            text("mGBA 0.10.5：导入后完整 32 MiB 内存 ROM 比较，仅两字节变化；24 次启停／重置检查通过，源 ROM 文件未改变。", "mGBA 0.10.5: full 32 MiB in-memory ROM comparison found exactly two changed bytes; 24 toggle/reset checks passed and the source file was unchanged."),
            text("原生例程测试：38,912 组难度／设施／随机输入检查。完整核心对战夹具：挑战单打、疯子单打及双打共 311 次回合更新，窥屏标记均关闭；未开启代码的疯子对照为 63/129 次开启。", "Native routine tests: 38,912 difficulty/facility/random-input cases. Full-core battle fixtures: 311 turn updates across Challenge singles and Lunatic singles/doubles, all with peeking off; untreated Lunatic control enabled it in 63/129 updates."),
            text("出招对照夹具：同样输入，卡比兽第二回合用保护。原版隆隆石用诅咒强化；开启代码后用雷电拳，被保护挡住。各重复两次结果一致；不是每次保护都会诱使 AI 攻击。", "Controlled move example: Snorlax uses Protect on turn two with identical inputs. Original Graveler uses Curse; with the code it uses Thunder Punch, blocked by Protect. Each repeated twice consistently; this does not mean every Protect will draw an attack."),
            text("VBA-M 2.2.3：界面导入、启停内存读回通过。手工输入时每条用不带空格的 16 位格式；8+8 可能被自动识别成另一协议。", "VBA-M 2.2.3: GUI import and enable/disable memory readback passed. Enter each line as 16 characters without a space; 8+8 can be auto-detected as another protocol."),
            text("验证日期：2026-09-22。设施覆盖为原生分支测试；未覆盖全剧情、所有设施完整流程或长时间手机游玩。对战夹具为受控测试环境。", "Verified 2026-09-22. Facilities were covered by native branch tests, not full tours. Entire-story play and long mobile sessions are untested. Battle fixtures are controlled test environments."),
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
        assert!(encode_rom_halfword(1, 0).is_err());
        assert!(encode_rom_halfword(0x2000000, 0).is_err());
        assert!(encode_rom_halfword(0x1fffffe, 0xffff).is_ok());
    }
    #[test]
    fn exact_identity_and_cross_rom_isolation() {
        assert!(CheatRom::open(&[0; 192]).is_err());
        assert!(CheatRom::identify(ULTIMATE_MD5, 1024).is_err());
        for p in PROFILES.into_iter().filter(|p| p.md5 != ULTIMATE_MD5) {
            let rom = CheatRom::identify(p.md5, p.size).unwrap();
            assert_eq!(
                rom.catalog().entries.len(),
                if p.md5 == crate::profile::ROCKET.md5 {
                    8
                } else {
                    7
                }
            );
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
            serde_json::json!({"format":"codebreaker"}),
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
            assert_eq!(daycare.is_ok(), p.md5 == crate::profile::ROCKET.md5);
        }
    }
    #[test]
    fn parameter_validation_and_patch_isolation() {
        use parameters::{Landing, MapChoice, SpeciesChoice};
        for p in PROFILES {
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
                if p.md5 == ULTIMATE_MD5 { 8 } else { 4 }
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
                if p.md5 == ULTIMATE_MD5 { 28 } else { 86 }
            );
            let rocket = p.md5 == crate::profile::ROCKET.md5;
            let mut offsets = std::collections::HashSet::new();
            for patch in bindings(p.md5)
                .iter()
                .flat_map(|b| b.patches.iter().copied())
                .chain(parameters::encounter_for(p.md5, 400, 100))
                .chain(parameters::shiny_for(p.md5))
                .chain(parameters::teleport(rocket, 1, 2, 0))
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
    #[ignore = "requires private ROMs via GEN3_ROM_ULTIMATE/BW/DP/ROCKET"]
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
        let catalog = dispatch(&mut app, "open_cheat_rom", json!({"path": ue_path})).unwrap();
        assert_eq!(catalog["entries"].as_array().unwrap().len(), 8);
        assert_eq!(catalog["rom"]["editor_supported"], true);
        assert!(app.session.is_none());
        let request = json!({"expected_rom_md5": ULTIMATE_MD5, "cheat_id": NO_PEEK, "format": "gameshark_v1_v2"});
        assert_eq!(
            dispatch(&mut app, "cheat_code", request.clone()).unwrap()["lines"],
            json!(["270AABF9 EF4D3B91", "05EFAF30 6F13BEC4"])
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
        ] {
            let path = std::env::var(env).expect(env);
            let source = std::fs::read(&path).unwrap();
            dispatch(&mut app, "open_rom", json!({"path": path})).unwrap();
            let baseline = app.session.as_ref().unwrap().rom.data.clone();
            let catalog =
                dispatch(&mut app, "cheats", json!({"expected_rom_md5": profile.md5})).unwrap();
            assert_eq!(
                catalog["entries"].as_array().unwrap().len(),
                if profile.md5 == crate::profile::ROCKET.md5 {
                    8
                } else {
                    7
                }
            );
            assert_eq!(catalog["rom"]["md5"], profile.md5);
            for entry in catalog["entries"].as_array().unwrap() {
                let code = dispatch(
                    &mut app,
                    "cheat_code",
                    json!({
                        "expected_rom_md5": profile.md5,
                        "cheat_id": entry["id"],
                        "format": "gameshark_v1_v2",
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
            dispatch(&mut app, "open_cheat_rom", json!({"path": ue_path})).unwrap();
            dispatch(&mut app, "cheat_code", request.clone()).unwrap();
            assert!(std::sync::Arc::ptr_eq(
                &baseline,
                &app.session.as_ref().unwrap().rom.data
            ));
            assert!(app.session.as_ref().unwrap().save.is_none());
            dispatch(&mut app, "open_cheat_rom", json!({"path": path})).unwrap();
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
