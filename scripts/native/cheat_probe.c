/* Optional mGBA integration probe; never loads a save or writes a ROM file.
 * Build through verify_cheats_mgba.py against a separately installed mGBA.
 */
#include <mgba/core/core.h>
#include <mgba/core/cheats.h>
#include <mgba/core/log.h>
#include <mgba/internal/gba/cheats.h>
#include <mgba/internal/arm/arm.h>
#include <mgba/internal/arm/isa-inlines.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

static void quiet(struct mLogger *log, int category, enum mLogLevel level,
                  const char *format, va_list args) {
    (void)log; (void)category; (void)level; (void)format; (void)args;
}
static struct mLogger logger = {.log = quiet};
static color_t pixels[240 * 160];
static struct mCheatSet *add(struct mCheatDevice *device, const char *line, int type) {
    struct mCheatSet *set = device->createSet(device, "probe");
    assert(mCheatAddLine(set, line, type));
    mCheatAddSet(device, set);
    set->enabled = true;
    mCheatRefresh(device, set);
    return set;
}
static unsigned step_species(struct mCore *core, unsigned sp) {
    struct ARMCore *cpu = core->cpu;
    _ARMSetMode(cpu, MODE_THUMB);
    cpu->cpsr.i = 1;
    cpu->gprs[1] = 0;
    cpu->gprs[6] = 25;
    cpu->gprs[13] = sp;
    cpu->gprs[15] = 0x08067b86;
    ThumbWritePC(cpu);
    core->step(core);
    return cpu->gprs[1];
}
int main(int argc, char **argv) {
    assert(argc >= 3);
    mLogSetDefaultLogger(&logger);
    struct mCore *core = mCoreFind(argv[1]);
    assert(core && core->init(core));
    mCoreInitConfig(core, "gen3-cheat-probe");
    core->setVideoBuffer(core, pixels, 240);
    assert(mCoreLoadFile(core, argv[1]));
    core->reset(core);
    struct mCheatDevice *device = core->cheatDevice(core);
    if (!strcmp(argv[2], "ultimate")) {
        assert(argc == 5);
        unsigned char *original = malloc(0x2000000);
        assert(original);
        for (unsigned i = 0; i < 0x2000000; ++i) original[i] = core->rawRead8(core, 0x8000000 + i, -1);
        struct mCheatSet *a = add(device, argv[3], GBA_CHEAT_GAMESHARK);
        struct mCheatSet *b = add(device, argv[4], GBA_CHEAT_GAMESHARK);
        assert(mCheatPatchListSize(&a->romPatches) == 1 && mCheatPatchListSize(&b->romPatches) == 1);
        unsigned changed = 0;
        for (unsigned i = 0; i < 0x2000000; ++i) {
            unsigned now = core->rawRead8(core, 0x8000000 + i, -1);
            if (now != original[i]) {
                assert((i == 0x1d492cf || i == 0x1d4ec47) && now == 0xe0);
                ++changed;
            }
        }
        assert(changed == 2);
        for (int cycle = 0; cycle < 24; ++cycle) {
            a->enabled = b->enabled = cycle % 2;
            mCheatRefresh(device, a); mCheatRefresh(device, b);
            core->reset(core);
            assert(core->busRead16(core, 0x09d492ce) == (cycle % 2 ? 0xe015 : 0xd915));
            assert(core->busRead16(core, 0x09d4ec46) == (cycle % 2 ? 0xe020 : 0xd820));
        }
        free(original);
        puts("{\"changed_rom_bytes\":2,\"toggle_reset_checks\":24}");
    } else {
        assert(!strcmp(argv[2], "old-sudowoodo"));
        assert(core->busRead16(core, 0x08067b86) == 0x1c31);
        core->busWrite32(core, 0x03007e28, 0);
        core->busWrite32(core, 0x03007e20, 123);
        struct mCheatSet *cb = add(device, "83007E28 00B9", GBA_CHEAT_CODEBREAKER);
        assert(mCheatListSize(&cb->list) == 1);
        struct mCheat *write = mCheatListGetPointer(&cb->list, 0);
        assert(write->address == 0x03007e28 && write->width == 2 && write->operand == 185);
        assert(core->busRead16(core, 0x03007e28) == 185);
        assert(step_species(core, 0x03007d28) == 25); /* CB alone does not replace the instruction. */
        struct mCheatSet *gs = add(device, "0146DCEA 3E32A31D", GBA_CHEAT_GAMESHARK);
        assert(core->busRead16(core, 0x08067b86) == 0x9940);
        assert(step_species(core, 0x03007d28) == 185);
        assert(step_species(core, 0x03007d20) == 123);
        cb->enabled = false;
        mCheatRefresh(device, cb);
        core->busWrite32(core, 0x03007e28, 77);
        assert(step_species(core, 0x03007d28) == 77); /* Patch alone reads existing stack data. */
        gs->enabled = false;
        mCheatRefresh(device, gs);
        assert(core->busRead16(core, 0x08067b86) == 0x1c31);
        puts("{\"cb_address\":\"03007E28\",\"cb_width\":2,\"cb_value\":185,\"gs_patch\":\"08067B86\",\"before\":\"1C31\",\"after\":\"9940\",\"cb_only_r1\":25,\"both_matching_sp_r1\":185,\"both_other_sp_r1\":123,\"gs_only_existing_stack_r1\":77}");
    }
    core->deinit(core);
    return 0;
}
