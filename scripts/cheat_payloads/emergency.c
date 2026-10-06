/* Exact-ROM battle recovery hook, installed in emulator ROM memory only.
 * The selected addresses and layouts are verified per supported ROM.
 * The wrapper must call the original battle-main callback on every frame. */
#include <stdint.h>
#define REG8(a) (*(volatile uint8_t *)(uintptr_t)(a))
#define REG16(a) (*(volatile uint16_t *)(uintptr_t)(a))
#define REG32(a) (*(volatile uint32_t *)(uintptr_t)(a))
#ifdef GEN3_EMERGENCY_MERCURY
#define PARTY_FLAGS 0x02022B4Cu
#define MAIN_HELD 0x0300311Cu
#define MAIN_NEW 0x0300311Eu
#define BATTLE_MON 0x02023BE4u
#define BATTLE_COUNT 0x02023BCCu
#define HEALTHBOX_IDS 0x03004FF0u
#define PARTY_INDEXES 0x02023BCEu
#define PLAYER_PARTY 0x02024284u
#define UPDATE_HEALTHBOX 0x08049D99u
#define BATTLE_FN 0x03004F84u
#define HEAL_PARTY 0x080A0059u
#define GET_MON_DATA 0x0803FBE9u
#define COMMAND_PHASE 0x08014041u
#define MON_STRIDE 0x58u
#define MON_HP 0x28u
#define MON_MAX_HP 0x2Cu
#define MON_STATUS 0x4Cu
#define MON_PP 0x24u
#elif defined(GEN3_EMERGENCY_ROCKET)
#define PARTY_FLAGS 0x02024BB8u
#define MAIN_HELD 0x0300330Cu
#define MAIN_NEW 0x0300330Eu
#define BATTLE_MON 0x02024C50u
#define BATTLE_COUNT 0x02024C30u
#define HEALTHBOX_IDS 0x03005230u
#define PARTY_INDEXES 0x02024C3Au
#define PLAYER_PARTY 0x02025170u
#define UPDATE_HEALTHBOX 0x080A90B5u
#define BATTLE_FN 0x030051B4u
#define HEAL_PARTY 0x0813108Du
#define GET_MON_DATA 0x080976D1u
#define COMMAND_PHASE 0x08050D51u
#define MON_STRIDE 0x5Cu
#define MON_HP 0x2Au
#define MON_MAX_HP 0x2Eu
#define MON_STATUS 0x50u
#define MON_PP 0x25u
#else
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
#define COMMAND_PHASE 0x0803BE75u
#define MON_STRIDE 0x58u
#define MON_HP 0x28u
#define MON_MAX_HP 0x2Cu
#define MON_STATUS 0x4Cu
#define MON_PP 0x24u
#endif
#define COMBO 0x0304u
#define UNSUPPORTED_BATTLE_FLAGS 0x004000c2u /* link, multi, Safari, in-game partner */
__attribute__((section(".payload"),used,noinline)) void emergency(void) {
    if ((REG16(MAIN_HELD) & COMBO) == COMBO &&
        (REG16(MAIN_NEW) & COMBO) &&
        !(REG32(PARTY_FLAGS) & UNSUPPORTED_BATTLE_FLAGS) &&
        REG32(BATTLE_FN) == COMMAND_PHASE) {
        unsigned safe = 1;
        for (unsigned b = 0; b < REG8(BATTLE_COUNT) && b < 4; b += 2) {
            volatile uint8_t *mon = (volatile uint8_t *)(uintptr_t)(BATTLE_MON + b * MON_STRIDE);
            if (*(volatile uint16_t *)(mon + 0) == 0 ||
                *(volatile uint16_t *)(mon + MON_HP) == 0) continue;
            unsigned slot = REG16(PARTY_INDEXES + b * 2u);
            if (slot >= 6 || *(volatile uint16_t *)(mon + MON_MAX_HP) !=
                REG16(PLAYER_PARTY + slot * 100u + 88u)) safe = 0;
        }
        if (safe) {
            /* HealPlayerParty updates party HP, PP and persistent status natively. */
            ((void (*)(void))HEAL_PARTY)();
            for (unsigned b = 0; b < REG8(BATTLE_COUNT) && b < 4; b += 2) {
                volatile uint8_t *mon = (volatile uint8_t *)(uintptr_t)(BATTLE_MON + b * MON_STRIDE);
                if (*(volatile uint16_t *)(mon + 0) == 0 ||
                    *(volatile uint16_t *)(mon + MON_HP) == 0) continue;
                *(volatile uint16_t *)(mon + MON_HP) = *(volatile uint16_t *)(mon + MON_MAX_HP);
                *(volatile uint32_t *)(mon + MON_STATUS) = 0;
                unsigned slot = REG16(PARTY_INDEXES + b * 2u);
#if defined(GEN3_EMERGENCY_ROCKET) || defined(GEN3_EMERGENCY_MERCURY)
                for (unsigned m = 0; m < 4; ++m)
                    mon[MON_PP + m] = ((uint32_t (*)(void *, uint32_t, void *))GET_MON_DATA)(
                        (void *)(uintptr_t)(PLAYER_PARTY + slot * 100u), 17u + m, 0);
#else
                uint8_t bonuses = *(volatile uint8_t *)(mon + 0x3b);
                for (unsigned m = 0; m < 4; ++m) {
                    uint16_t move = *(volatile uint16_t *)(mon + 0x0c + m * 2);
                    if (move) mon[MON_PP + m] = ((uint8_t (*)(uint16_t,uint8_t,uint8_t))CALC_PP)(move, bonuses, m);
                }
#endif
                /* Refresh the healthbox through the ROM's own UI routine. */
                unsigned sprite = *(volatile uint8_t *)(uintptr_t)(HEALTHBOX_IDS + b);
                if (slot < 6 && sprite < 64)
                    ((void (*)(uint8_t, void *, uint8_t))UPDATE_HEALTHBOX)(sprite,
                        (void *)(uintptr_t)(PLAYER_PARTY + slot * 100u), 0);
            }
        }
    }
    ((void (*)(void))REG32(BATTLE_FN))();
}
