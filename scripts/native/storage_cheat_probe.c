/* Full-frame storage tests. Battery/state inputs are read into memory only;
 * the caller supplies disposable fixtures and owns exported test artifacts. */
#include "common_cheat_probe.c"

static void *read_fixture(const char *path, size_t *size) {
    FILE *file = fopen(path, "rb");
    if (!file) return NULL;
    fseek(file, 0, SEEK_END);
    long length = ftell(file);
    rewind(file);
    if (length <= 0) { fclose(file); return NULL; }
    void *data = malloc((size_t)length);
    if (!data || fread(data, 1, (size_t)length, file) != (size_t)length) {
        free(data); fclose(file); return NULL;
    }
    fclose(file);
    *size = (size_t)length;
    return data;
}
int load_battery(const char *path) {
    size_t size;
    void *data = read_fixture(path, &size);
    if (!data) return 0;
    int ok = core->savedataRestore(core, data, size, false);
    free(data);
    return ok;
}
int load_state(const char *path) {
    size_t size;
    void *data = read_fixture(path, &size);
    if (!data) return 0;
    int ok = size == core->stateSize(core) && core->loadState(core, data);
    free(data);
    return ok;
}
int export_battery(const char *path) {
    void *data = NULL;
    size_t size = core->savedataClone(core, &data);
    FILE *file = size ? fopen(path, "wb") : NULL;
    int ok = file && fwrite(data, 1, size, file) == size;
    if (file) fclose(file);
    free(data);
    return ok;
}
void frames(unsigned count, unsigned keys) {
    core->setKeys(core, keys);
    for (unsigned i = 0; i < count; ++i) core->runFrame(core);
    core->setKeys(core, 0);
}
void screenshot(const char *path) {
    /* mGBA's high pixel byte contains renderer flags, not PNG opacity.
     * Export opaque RGBA explicitly instead of treating the framebuffer as RGBA. */
    unsigned char rgba[240 * 160 * 4];
    for (unsigned i = 0; i < 240 * 160; ++i) {
        rgba[i * 4] = pixels[i] & 0xFF;
        rgba[i * 4 + 1] = (pixels[i] >> 8) & 0xFF;
        rgba[i * 4 + 2] = (pixels[i] >> 16) & 0xFF;
        rgba[i * 4 + 3] = 255;
    }
    FILE *file = fopen(path, "wb");
    assert(file);
    assert(fwrite(rgba, sizeof(rgba), 1, file) == 1);
    fclose(file);
}
