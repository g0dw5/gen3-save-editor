//! Route the native field SELECT handler to the original Pokémon PC menu.
//! Keep its facility guards and registered-item data; never force a UI callback
//! from a frame-independent RAM write. See docs/research/cheat-verification.md.
use super::{text, Format, Recipe, RomHalfword, PORTABLE_PC};

pub(super) const DARK_PHANTOM: [RomHalfword; 7] = patches(0x1ad568, 0x1ad5d8, 0x082736b3, 0x311300);
pub(super) const ULTIMATE: [RomHalfword; 7] = patches(0x1ad568, 0x1ad5d8, 0x082736b3, 0x1fff100);
pub(super) const ROCKET: [RomHalfword; 7] = patches(0x1f9bd4, 0x1f9c44, 0x082e04f7, 0x1f00100);

const fn patches(
    compare: u32,
    literal: u32,
    original_script: u32,
    script: u32,
) -> [RomHalfword; 7] {
    let address = 0x08000000 + script;
    [
        // cmp r0,r0 selects the existing script branch, without changing the
        // registered item or bypassing the preceding native eligibility guards.
        RomHalfword {
            offset: compare,
            before: 0x2800,
            after: 0x4280,
        },
        RomHalfword {
            offset: literal,
            before: original_script as u16,
            after: address as u16,
        },
        RomHalfword {
            offset: literal + 2,
            before: (original_script >> 16) as u16,
            after: (address >> 16) as u16,
        },
        // lockall; special 0x3f (ShowPokemonStorageSystemPC); waitstate;
        // releaseall; end; nop padding. Exit resumes this script naturally.
        RomHalfword {
            offset: script,
            before: 0xffff,
            after: 0x256a,
        },
        RomHalfword {
            offset: script + 2,
            before: 0xffff,
            after: 0x003f,
        },
        RomHalfword {
            offset: script + 4,
            before: 0xffff,
            after: 0x6c27,
        },
        RomHalfword {
            offset: script + 6,
            before: 0xffff,
            after: 0x0002,
        },
    ]
}

pub(super) fn recipe() -> Recipe {
    Recipe {
        id: PORTABLE_PC,
        category: "storage",
        parameters: None,
        title: text("随身电脑（SELECT）", "Portable Pokémon PC (SELECT)"),
        summary: text("在可以自由行走时按 SELECT，原地打开本作的宝可梦电脑菜单，可取出、存放、整理宝可梦和整理携带道具。", "Press SELECT while freely walking to open this ROM's Pokémon PC menu in place: withdraw, deposit, move Pokémon and manage held items."),
        scope: text("GameShark Advance V1/V2 · 无需主码 · 7 行整组启用", "GameShark Advance V1/V2 · No master code · Enable all 7 lines together"),
        steps: vec![
            text("先正常保存并备份电池存档，在战斗、对话、动画和保存过程之外添加整组代码。", "Save normally and back up the battery save. Add the full set outside battles, dialogue, animations and saving."),
            text("选择 GameShark Advance V1/V2，启用全部 7 行，重启并从游戏内存档继续。", "Choose GameShark Advance V1/V2, enable all 7 lines, restart and continue from the in-game save."),
            text("在可自由行走的地图按 SELECT，在电脑菜单选择本作的整理宝可梦选项。用 B 或退出选项正常退出，回到原位置。", "Press SELECT in a freely walkable map, then choose the ROM's move/organize Pokémon option. Exit normally with B or the exit option to return to the same place."),
            text("退出电脑后再停用整组并重启，恢复原来登记道具的 SELECT 快捷键。", "Exit the PC before disabling the full set and restarting. SELECT then returns to the original registered-item shortcut."),
        ],
        limitations: vec![
            text("启用期间临时占用 SELECT，原登记道具不会被清空。战斗、对话和其他菜单中不提供强行打开功能；保留原生联机房及部分对战设施的禁用判断。", "Temporarily replaces SELECT; the registered item is not cleared. Does not force-open storage during battles, dialogue or other menus. Native Union Room and certain battle-facility restrictions remain."),
            text("这是省去往返电脑的便利修改，会改变远离电脑时可换队的游戏规则；不是该 ROM 正常就有的随身功能。取出、存放、移动及道具操作仍由原生电脑处理。", "This convenience cheat changes when teams can be reorganized away from a PC; it is not a normal portable feature of the ROM. Withdrawal, deposit, movement and item handling remain native."),
            text("电脑菜单内不要只关部分代码。正常整理后保存会保留变更，停用代码不会撤销存取。只支持本页完整 MD5；手机模拟器尚未实测。", "Do not disable individual lines inside the PC. Normal saves retain storage changes; disabling does not undo transfers. Requires this exact MD5; mobile emulators are untested."),
        ],
        verification: vec![
            text("逐 ROM 使用最终 7 行代码在 mGBA 验证原地打开、存入、盒内移动、正常保存、全新模拟器核心读档及取回；个体记录一致，并按各 ROM 的原生规则检查完整性。另核对整组启停恢复、16 项原生设施限制及退出后行走。", "Per-ROM mGBA tests of the final 7-line set cover opening in place, deposit, box-slot movement, normal saving, a fresh-core battery reload and withdrawal with identical records and integrity checks appropriate to each native ROM. Also checks full-set restoration, 16 native facility cases and walking after exit."),
            text("沿用本作原生个体与电脑读写，不直接覆盖队伍、盒子、PID 或存档校验字段。未宣称覆盖全部剧情、设施或长期手机游玩。", "Uses the ROM's own Pokémon/storage routines rather than overwriting party, boxes, PID or save checksums. This is not exhaustive story/facility or long-term mobile coverage."),
        ],
        formats: vec![Format::GamesharkV1V2],
    }
}
