/* Exact-ROM Ultimate Emerald 5.5 hook, installed in emulator ROM memory only.
 * All addresses below were verified against MD5 17ce9785b33319b3dbda9a5d37c57ec1.
 * The wrapper must call the original battle-main callback on every frame. */
#include <stdint.h>
#define REG8(a) (*(volatile uint8_t *)(uintptr_t)(a))
#define REG16(a) (*(volatile uint16_t *)(uintptr_t)(a))
#define REG32(a) (*(volatile uint32_t *)(uintptr_t)(a))
#define PARTY_FLAGS 0x02022FECu
#define MAIN_HELD 0x030022ECu
#define MAIN_NEW 0x030022EEu
#define BATTLE_MON 0x02024084u
#define BATTLE_COUNT 0x0202406Cu
#define HEALTHBOX_IDS 0x03005D70u
#define PARTY_INDEXES 0x0202406Eu
#define PLAYER_PARTY 0x020244ECu
#define UPDATE_HEALTHBOX 0x08074861u
#define BATTLE_FN 0x03005D04u
#define HEAL_PARTY 0x080F9181u
#define CALC_PP 0x0806B961u
#define COMBO 0x0304u
#define COMMAND_PHASE 0x0803BE75u
#define UNSUPPORTED_BATTLE_FLAGS 0x004000c2u /* link, multi, Safari, in-game partner */
__attribute__((section(".payload"),used,noinline)) void emergency(void) {
    if ((REG16(MAIN_HELD) & COMBO) == COMBO &&
        (REG16(MAIN_NEW) & COMBO) &&
        !(REG32(PARTY_FLAGS) & UNSUPPORTED_BATTLE_FLAGS) &&
        REG32(BATTLE_FN) == COMMAND_PHASE) {
        unsigned safe = 1;
        for (unsigned b = 0; b < REG8(BATTLE_COUNT) && b < 4; b += 2) {
            volatile uint8_t *mon = (volatile uint8_t *)(uintptr_t)(BATTLE_MON + b * 0x58u);
            if (*(volatile uint16_t *)(mon + 0) == 0 ||
                *(volatile uint16_t *)(mon + 0x28) == 0) continue;
            unsigned slot = REG16(PARTY_INDEXES + b * 2u);
            if (slot >= 6 || *(volatile uint16_t *)(mon + 0x2c) !=
                REG16(PLAYER_PARTY + slot * 100u + 88u)) safe = 0;
        }
        if (safe) {
            /* HealPlayerParty updates party HP, PP and persistent status natively. */
            ((void (*)(void))HEAL_PARTY)();
            for (unsigned b = 0; b < REG8(BATTLE_COUNT) && b < 4; b += 2) {
                volatile uint8_t *mon = (volatile uint8_t *)(uintptr_t)(BATTLE_MON + b * 0x58u);
                if (*(volatile uint16_t *)(mon + 0) == 0 ||
                    *(volatile uint16_t *)(mon + 0x28) == 0) continue;
                *(volatile uint16_t *)(mon + 0x28) = *(volatile uint16_t *)(mon + 0x2c);
                *(volatile uint32_t *)(mon + 0x4c) = 0;
                uint8_t bonuses = *(volatile uint8_t *)(mon + 0x3b);
                for (unsigned m = 0; m < 4; ++m) {
                    uint16_t move = *(volatile uint16_t *)(mon + 0x0c + m * 2);
                    if (move) mon[0x24 + m] = ((uint8_t (*)(uint16_t,uint8_t,uint8_t))CALC_PP)(move, bonuses, m);
                }
                /* Refresh the healthbox through the ROM's own UI routine. */
                unsigned slot = REG16(PARTY_INDEXES + b * 2u);
                unsigned sprite = *(volatile uint8_t *)(uintptr_t)(HEALTHBOX_IDS + b);
                if (slot < 6 && sprite < 64)
                    ((void (*)(uint8_t, void *, uint8_t))UPDATE_HEALTHBOX)(sprite,
                        (void *)(uintptr_t)(PLAYER_PARTY + slot * 100u), 0);
            }
        }
    }
    ((void (*)(void))REG32(BATTLE_FN))();
}
