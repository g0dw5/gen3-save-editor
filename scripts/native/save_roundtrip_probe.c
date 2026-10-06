/* Full-frame SAV round trips. ROM and battery inputs are read-only; the only
 * game-state control is ordinary key input. No cheats, RAM writes, CPU function
 * calls or savestate loading. Explicit exports create new private test files. */
#include <mgba/flags.h>
#include <mgba/core/core.h>
#include <mgba/core/log.h>
#include <stdio.h>
#include <stdlib.h>

static struct mCore *core;
static color_t pixels[240 * 160];
static void quiet(struct mLogger *log, int category, enum mLogLevel level,
                  const char *format, va_list args) {
    (void)log; (void)category; (void)level; (void)format; (void)args;
}
static struct mLogger logger = {.log = quiet};

int start(const char *path) {
    if (core) return 0;
    mLogSetDefaultLogger(&logger);
    core = mCoreFind(path);
    if (!core) return 0;
    if (!core->init(core)) { free(core); core = NULL; return 0; }
    mCoreInitConfig(core, "gen3-save-roundtrip-probe");
    mCoreConfigSetDefaultValue(&core->config, "idleOptimization", "remove");
    mCoreLoadConfig(core);
    core->setVideoBuffer(core, pixels, 240);
    if (!mCoreLoadFile(core, path)) { core->deinit(core); core = NULL; return 0; }
    core->reset(core);
    return 1;
}
void finish(void) { if (core) { core->deinit(core); core = NULL; } }
void reset(void) { if (core) core->reset(core); }
int load_battery(const char *path) {
    if (!core) return 0;
    FILE *file = fopen(path, "rb");
    if (!file) return 0;
    if (fseek(file, 0, SEEK_END)) { fclose(file); return 0; }
    long size = ftell(file);
    rewind(file);
    if (size != 0x20000) { fclose(file); return 0; }
    void *bytes = malloc((size_t)size);
    int ok = bytes && fread(bytes, 1, (size_t)size, file) == (size_t)size;
    fclose(file);
    if (ok) ok = core->savedataRestore(core, bytes, (size_t)size, false);
    free(bytes);
    return ok;
}
int export_battery(const char *path) {
    if (!core) return 0;
    void *bytes = NULL;
    size_t size = core->savedataClone(core, &bytes);
    FILE *file = size ? fopen(path, "wbx") : NULL;
    int ok = file && fwrite(bytes, 1, size, file) == size;
    if (file && fclose(file)) ok = 0;
    free(bytes);
    return ok;
}
void frames(unsigned count, unsigned keys) {
    core->setKeys(core, keys);
    for (unsigned i = 0; i < count; ++i) core->runFrame(core);
    core->setKeys(core, 0);
}
void readbytes(unsigned address, unsigned char *bytes, unsigned length) {
    for (unsigned i = 0; i < length; ++i) bytes[i] = core->busRead8(core, address + i);
}
int screenshot(const char *path) {
    /* Renderer metadata in the high pixel byte is not PNG opacity. */
    unsigned char rgba[240 * 160 * 4];
    for (unsigned i = 0; i < 240 * 160; ++i) {
        rgba[i * 4] = pixels[i] & 255;
        rgba[i * 4 + 1] = (pixels[i] >> 8) & 255;
        rgba[i * 4 + 2] = (pixels[i] >> 16) & 255;
        rgba[i * 4 + 3] = 255;
    }
    FILE *file = fopen(path, "wbx");
    if (!file) return 0;
    int ok = fwrite(rgba, 1, sizeof(rgba), file) == sizeof(rgba);
    fclose(file);
    return ok;
}
