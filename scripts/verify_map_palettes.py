"""Check shared map palettes against the exact ROMs' native palette loader.

Requires Unicorn, Pillow, GEN3_ROM_BW / GEN3_ROM_DP and GEN3_DEV_BIN.
Checks every map's palette and representative static map PNGs. No saves are used.
This does not emulate animated tiles, weather, NPCs or the GBA display pipeline.
"""
import base64
import io
import json
import os
import secrets
import struct
import subprocess
import time
import urllib.request
from pathlib import Path

from PIL import Image
from unicorn import Uc, UC_ARCH_ARM, UC_MODE_THUMB, UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_SP, UC_ARM_REG_LR, UC_ARM_REG_PC, UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2
from verify_pokemon_sprites import decompress


class NativePalette:
    def __init__(self, rom):
        self.cpu = cpu = Uc(UC_ARCH_ARM, UC_MODE_THUMB)
        cpu.mem_map(0x3000000, 0x8000)
        cpu.mem_map(0x8000000, 0x2000000)
        cpu.mem_write(0x8000000, rom)
        cpu.hook_add(UC_HOOK_CODE, self.load, begin=0x80a1938, end=0x80a1938)
        # All supported map tilesets must use the ordinary palette path.
        cpu.hook_add(UC_HOOK_CODE, self.compressed, begin=0x80a18f4, end=0x80a18f4)

    def compressed(self, *_):
        raise AssertionError('unexpected compressed map palette')

    def load(self, cpu, *_):
        src = cpu.reg_read(UC_ARM_REG_R0)
        dest = cpu.reg_read(UC_ARM_REG_R1) * 2
        size = cpu.reg_read(UC_ARM_REG_R2)
        self.palette[dest:dest + size] = cpu.mem_read(src, size)
        cpu.reg_write(UC_ARM_REG_PC, cpu.reg_read(UC_ARM_REG_LR))

    def render(self, layout):
        self.palette = bytearray(512)
        self.cpu.reg_write(UC_ARM_REG_SP, 0x3007000)
        self.cpu.reg_write(UC_ARM_REG_LR, 0x8000001)
        self.cpu.reg_write(UC_ARM_REG_R0, 0x8000000 + layout)
        self.cpu.emu_start(0x8088dd5, 0x8000000, count=10000)
        assert self.cpu.reg_read(UC_ARM_REG_PC) == 0x8000000
        return bytes(self.palette)


def pointer(rom, offset):
    return struct.unpack_from('<I', rom, offset)[0] - 0x8000000


def render(rom, m, palette):
    layout = m['layout']
    sets = [pointer(rom, layout + offset) for offset in [16, 20]]
    tiles = []
    for s in sets:
        p = pointer(rom, s + 4)
        data = decompress(rom, p + 0x8000000) if rom[s] else rom[p:p + 16384]
        tiles.append(data.ljust(16384, b'\0'))
    graphics = b''.join(tiles)
    colors = [tuple(((v >> shift & 31) << 3) | ((v >> shift & 31) >> 2)
                    for shift in [0, 5, 10]) for v in struct.unpack('<256H', palette)]
    image = Image.new('RGBA', (m['width'] * 16, m['height'] * 16))
    block = pointer(rom, layout + 12)
    cache = {}
    for y in range(m['height']):
        for x in range(m['width']):
            metatile = struct.unpack_from('<H', rom, block + 2 * (y * m['width'] + x))[0] & 1023
            offset = pointer(rom, sets[metatile // 512] + 12) + metatile % 512 * 16
            for part, entry in enumerate(struct.unpack_from('<8H', rom, offset)):
                key = (entry, part >= 4)
                if key not in cache:
                    tile, bank = entry & 1023, entry >> 12
                    patch = Image.new('RGBA', (8, 8))
                    for ty in range(8):
                        for tx in range(8):
                            value = graphics[tile * 32 + ty * 4 + tx // 2]
                            index = value >> (4 * (tx % 2)) & 15
                            patch.putpixel((tx, ty), colors[bank * 16 + index] + (0 if part >= 4 and index == 0 else 255,))
                    if entry & 0x400:
                        patch = patch.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
                    if entry & 0x800:
                        patch = patch.transpose(Image.Transpose.FLIP_TOP_BOTTOM)
                    cache[key] = patch
                image.alpha_composite(cache[key], (x * 16 + part % 2 * 8, y * 16 + part % 4 // 2 * 8))
    return image


def main():
    token = secrets.token_hex(32)
    bridge = subprocess.Popen([os.environ.get('GEN3_DEV_BIN', 'gen3-dev')],
        env=dict(os.environ, GEN3_DEV_TOKEN=token), stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    def api(command, payload):
        request = urllib.request.Request('http://127.0.0.1:8766/api',
            json.dumps({'command': command, 'payload': payload}).encode(),
            {'Content-Type': 'application/json', 'X-Gen3-Token': token})
        result = json.load(urllib.request.urlopen(request, timeout=60))
        assert result['ok'], result
        return result['data']
    try:
        for _ in range(100):
            assert bridge.poll() is None, bridge.stderr.read().decode()
            try:
                api('profiles', {})
                break
            except OSError:
                time.sleep(.05)
        else:
            raise AssertionError('development bridge did not start')
        for key in ['GEN3_ROM_BW', 'GEN3_ROM_DP']:
            path = os.environ[key]
            api('open_rom', {'path': path})
            rom = Path(path).read_bytes()
            native = NativePalette(rom)
            maps = api('world', {})['maps']
            checked = 0
            for m in maps:
                palette = native.render(m['layout'])
                primary, secondary = [pointer(rom, m['layout'] + o) for o in [16, 20]]
                pp, sp = pointer(rom, primary + 8), pointer(rom, secondary + 8)
                expected = b'\0\0' + rom[pp + 2:pp + 192] + rom[sp + 192:sp + 416] + bytes(96)
                assert palette == expected, (key, m['id'])
                if m['id'] in ['35-24', '26-1', '0-0', '0-16', '24-0', '34-0', '35-46', '37-38']:
                    reference = render(rom, m, palette)
                    url = api('map_image', {'id': m['id']})['url']
                    actual = Image.open(io.BytesIO(base64.b64decode(url.split(',')[1]))).convert('RGBA')
                    assert actual.size == reference.size and actual.tobytes() == reference.tobytes(), (key, m['id'])
                    checked += 1
            print(f'{key}: {len(maps)} native palettes and {checked} static PNGs matched', flush=True)
    finally:
        bridge.terminate()
        bridge.wait(timeout=10)


if __name__ == '__main__':
    main()
