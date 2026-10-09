//! Route the native field SELECT handler to the original Pokémon PC menu.
//! Keep its facility guards and registered-item data; never force a UI callback
//! from a frame-independent RAM write. See docs/research/cheat-verification.md.
use super::{text, Format, Recipe, RomHalfword, PORTABLE_PC};

pub(super) const DARK_PHANTOM: [RomHalfword; 7] = patches(0x1ad568, 0x1ad5d8, 0x082736b3, 0x311300);
pub(super) const ULTIMATE: [RomHalfword; 7] = patches(0x1ad568, 0x1ad5d8, 0x082736b3, 0x1fff100);
pub(super) const ROCKET: [RomHalfword; 7] = patches(0x1f9bd4, 0x1f9c44, 0x082e04f7, 0x1f00100);
// Mercury's live SELECT dispatcher replaces the original FireRed function.
// Preserve its eligibility checks and multi-registration records; route only
// its fallback branch to the PC special actually registered in this ROM.
pub(super) const MERCURY: [RomHalfword; 7] = {
    let mut p = patches(0x1d522ce, 0x1d523ac, 0x081a77a0, 0x13fd200);
    p[0].before = 0x2e00;
    p[0].after = 0x42b6;
    p[4].after = 0x003c;
    p
};

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
        summary: text("自由行走时按 SELECT，打开宝可梦电脑。", "Press SELECT while walking to open Pokémon storage."),
        scope: text("GameShark Advance V1/V2 · 无需主码 · 7 行整组启用", "GameShark Advance V1/V2 · No master code · Enable all 7 lines together"),
        steps: vec![
text("整组启用，重启并从游戏内存档继续。", "Enable the full set; restart and continue from the in-game save."),
text("按 SELECT 打开电脑，正常退出后再停用。", "Press SELECT to open storage. Exit normally before disabling.")
],
        limitations: vec![
text("占用登记道具的 SELECT 快捷键；战斗、对话中不能打开。", "Replaces the registered-item SELECT shortcut; unavailable during battles or dialogue."),
text("存取操作随正常保存保留；停用不会撤销。", "Storage changes persist when saved; disabling does not undo them.")
],
        formats: vec![Format::GamesharkV1V2],
    }
}
