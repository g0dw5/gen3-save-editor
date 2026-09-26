/* Opt-in native fixture runner for verify_common_cheats_mgba.py.
 * Loads a ROM read-only; no save-file API is exposed. RAM fixtures are disposable.
 */
#include <mgba/core/core.h>
#include <mgba/core/cheats.h>
#include <mgba/core/log.h>
#include <mgba/internal/gba/cheats.h>
#include <mgba/internal/arm/arm.h>
#include <mgba/internal/arm/isa-inlines.h>
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static struct mCore *core;
static struct mCheatDevice *device;
static struct mCheatSet *sets[256];
static unsigned count;
static color_t pixels[240 * 160];
static void quiet(struct mLogger *log, int category, enum mLogLevel level,
                  const char *format, va_list args) {
    (void)log; (void)category; (void)level; (void)format; (void)args;
}
static struct mLogger logger = {.log = quiet};

int start(const char *path) {
    mLogSetDefaultLogger(&logger);
    core = mCoreFind(path);
    if (!core || !core->init(core)) return 0;
    mCoreInitConfig(core, "gen3-common-cheat-probe");
    core->setVideoBuffer(core, pixels, 240);
    if (!mCoreLoadFile(core, path)) return 0;
    core->reset(core);
    device = core->cheatDevice(core);
    count = 0;
    return 1;
}
void finish(void) { core->deinit(core); core = NULL; }
void reset(void) { core->reset(core); }
void clearcodes(void) {
    /* mCheatDeviceClear only frees sets; explicitly restore ROM patches first. */
    for (unsigned i = 0; i < count; ++i) {
        sets[i]->enabled = false;
        mCheatRefresh(device, sets[i]);
    }
    mCheatDeviceClear(device);
    count = 0;
}
unsigned readmem(unsigned address, unsigned width) {
    return width == 1 ? core->busRead8(core, address) :
           width == 2 ? core->busRead16(core, address) : core->busRead32(core, address);
}
void writemem(unsigned address, unsigned value, unsigned width) {
    if (width == 1) core->busWrite8(core, address, value);
    else if (width == 2) core->busWrite16(core, address, value);
    else core->busWrite32(core, address, value);
}
void readbytes(unsigned address, unsigned char *bytes, unsigned length) {
    for (unsigned i = 0; i < length; ++i) bytes[i] = core->busRead8(core, address + i);
}
void writebytes(unsigned address, const unsigned char *bytes, unsigned length) {
    for (unsigned i = 0; i < length; ++i) core->busWrite8(core, address + i, bytes[i]);
}
int addcode(const char *line) {
    assert(count < 256);
    struct mCheatSet *set = device->createSet(device, "fixture");
    if (!mCheatAddLine(set, line, GBA_CHEAT_GAMESHARK)) return -1;
    mCheatAddSet(device, set);
    set->enabled = true;
    mCheatRefresh(device, set);
    assert(mCheatPatchListSize(&set->romPatches) == 1);
    sets[count] = set;
    return count++;
}
/* Import exactly as a user would: one multiline emulator cheat set. */
int addgroup(const char *lines) {
    assert(count < 256);
    struct mCheatSet *set = device->createSet(device, "multiline fixture");
    char *copy = strdup(lines), *cursor = NULL;
    unsigned expected = 0;
    for (char *line = strtok_r(copy, "\n", &cursor); line; line = strtok_r(NULL, "\n", &cursor)) {
        if (!mCheatAddLine(set, line, GBA_CHEAT_GAMESHARK)) { free(copy); return -1; }
        ++expected;
    }
    free(copy);
    mCheatAddSet(device, set);
    set->enabled = true;
    mCheatRefresh(device, set);
    assert(mCheatPatchListSize(&set->romPatches) == expected);
    sets[count] = set;
    return count++;
}
/* CodeBreaker RAM operations are processed every emulator frame, not installed
 * as ROM patches. Keep this separate from the GameShark ROM-patch assertion. */
int addgroup_codebreaker(const char *lines) {
    assert(count < 256);
    struct mCheatSet *set = device->createSet(device, "CodeBreaker fixture");
    char *copy = strdup(lines), *cursor = NULL;
    for (char *line = strtok_r(copy, "\n", &cursor); line; line = strtok_r(NULL, "\n", &cursor)) {
        if (!mCheatAddLine(set, line, GBA_CHEAT_CODEBREAKER)) { free(copy); return -1; }
    }
    free(copy);
    mCheatAddSet(device, set);
    set->enabled = true;
    mCheatRefresh(device, set);
    assert(mCheatPatchListSize(&set->romPatches) == 0);
    sets[count] = set;
    return count++;
}
void togglecode(unsigned index, int enabled) {
    assert(index < count);
    sets[index]->enabled = enabled;
    mCheatRefresh(device, sets[index]);
}
/* Preserve caller CPU state while executing the complete native Thumb routine.
 * ROM hooks can use ARM interworking. No native subroutine is stubbed.
 */
int callfunc(unsigned address, unsigned r0, unsigned r1, unsigned r2, unsigned r3) {
    struct ARMCore *cpu = core->cpu, saved = *cpu;
    unsigned ime = readmem(0x04000208, 2);
    writemem(0x04000208, 0, 2);
    _ARMSetMode(cpu, MODE_THUMB);
    cpu->cpsr.i = 1;
    cpu->gprs[0] = r0; cpu->gprs[1] = r1; cpu->gprs[2] = r2; cpu->gprs[3] = r3;
    cpu->gprs[13] = 0x03007a00;
    cpu->gprs[14] = 0x03007b01;
    cpu->gprs[15] = address;
    ThumbWritePC(cpu);
    unsigned steps = 0;
    while (cpu->gprs[15] != 0x03007b02 && steps++ < 20000000) core->step(core);
    int result = cpu->gprs[0];
    if (cpu->gprs[15] != 0x03007b02) {
        fprintf(stderr, "unfinished native call %08x at %08x\n", address, cpu->gprs[15]);
        result = -1;
    }
    *cpu = saved;
    writemem(0x04000208, ime, 2);
    return result;
}
