//! Read-only, exact-ROM cheat recipes. No save, emulator or file writes.
//!
//! Bindings contain verified engine instructions, never extracted game catalogs.
//! A cheat-only identity is deliberately separate from a full editor Profile.
use crate::{binary, err, profile::PROFILES, Result};
use serde::{Deserialize, Serialize};

pub const ULTIMATE_MD5: &str = "17ce9785b33319b3dbda9a5d37c57ec1";
pub const NO_PEEK: &str = "disable-input-peeking";

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
}
#[derive(Serialize)]
pub struct Catalog {
    pub rom: CheatRom,
    pub entries: Vec<Recipe>,
}
#[derive(Serialize)]
pub struct Recipe {
    pub id: &'static str,
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
}
#[derive(Debug, Serialize)]
pub struct Code {
    pub rom_md5: String,
    pub cheat_id: String,
    pub format: Format,
    pub lines: Vec<String>,
    /// Same protocol, alternate whitespace for VBA-M's format detection.
    pub compact_lines: Vec<String>,
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

impl CheatRom {
    pub fn open(data: &[u8]) -> Result<Self> {
        let md5 = binary::hash(data);
        let rom = Self::identify(&md5, data.len())?;
        if md5 == ULTIMATE_MD5 {
            // A second assertion documents exactly which native instructions this recipe replaces.
            for p in NO_PEEK_PATCHES {
                if binary::u16(data, p.offset as usize)? != p.before {
                    return Err(err("cheat_original_bytes", p.offset));
                }
            }
        }
        Ok(rom)
    }
    fn identify(md5: &str, size: usize) -> Result<Self> {
        if md5 == ULTIMATE_MD5 && size == 32 * 1024 * 1024 {
            return Ok(Self {
                md5: md5.into(),
                label: "究极绿宝石 5.5 · Ultimate Emerald 5.5".into(),
                editor_supported: false,
            });
        }
        if let Some(p) = PROFILES.iter().find(|p| p.md5 == md5 && p.size == size) {
            return Ok(Self {
                md5: md5.into(),
                label: p.label.into(),
                editor_supported: true,
            });
        }
        Err(err("unsupported_rom", format!("MD5 {md5}; {size} bytes")))
    }
    pub fn catalog(&self) -> Catalog {
        Catalog {
            rom: self.clone(),
            entries: if self.md5 == ULTIMATE_MD5 {
                vec![no_peek()]
            } else {
                vec![]
            },
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
        if self.md5 != ULTIMATE_MD5 || request.cheat_id != NO_PEEK {
            return Err(err(
                "unsupported_feature",
                "no verified recipe for this ROM and cheat",
            ));
        }
        let lines = NO_PEEK_PATCHES
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

fn no_peek() -> Recipe {
    Recipe {
        id: NO_PEEK,
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
        for p in PROFILES {
            let rom = CheatRom::identify(p.md5, p.size).unwrap();
            assert!(rom.catalog().entries.is_empty());
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
        assert_eq!(catalog["entries"].as_array().unwrap().len(), 1);
        assert_eq!(catalog["rom"]["editor_supported"], false);
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
            assert_eq!(catalog["entries"], json!([]));
            assert_eq!(catalog["rom"]["md5"], profile.md5);
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
