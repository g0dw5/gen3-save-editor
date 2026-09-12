"""Compare rendered PNGs with each supported ROM's native front-picture loader.

Requires Unicorn, Pillow, GEN3_ROM_BW / GEN3_ROM_DP, and a dev-server build
selected by GEN3_DEV_BIN. Starts/stops its own authenticated localhost bridge.
No save is loaded or written. GEN3_SPRITE_PREVIEW optionally writes a contact sheet.
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

from PIL import Image, ImageDraw
from unicorn import Uc, UC_ARCH_ARM, UC_MODE_THUMB, UC_HOOK_CODE
from unicorn.arm_const import (
    UC_ARM_REG_SP, UC_ARM_REG_LR, UC_ARM_REG_PC, UC_ARM_REG_R0,
    UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3,
)


def decompress(rom, address):
    offset = address - 0x8000000
    assert rom[offset] == 0x10
    size = int.from_bytes(rom[offset + 1:offset + 4], 'little')
    output = bytearray()
    offset += 4
    while len(output) < size:
        flags = rom[offset]
        offset += 1
        for bit in range(7, -1, -1):
            if len(output) == size:
                break
            if flags & (1 << bit):
                a, b = rom[offset:offset + 2]
                offset += 2
                distance = ((a & 15) << 8 | b) + 1
                for _ in range((a >> 4) + 3):
                    if len(output) < size:
                        output.append(output[-distance])
            else:
                output.append(rom[offset])
                offset += 1
    return bytes(output)


class NativeRenderer:
    def __init__(self, rom):
        self.rom = rom
        self.cpu = cpu = Uc(UC_ARCH_ARM, UC_MODE_THUMB)
        for address, size in [(0x2000000, 0x40000), (0x3000000, 0x8000),
                              (0x8000000, 0x2000000)]:
            cpu.mem_map(address, size)
        cpu.mem_write(0x8000000, rom)
        cpu.hook_add(UC_HOOK_CODE, self.bios, begin=0x82e7084, end=0x82e7090)

    def bios(self, cpu, address, size, data):
        src, dst = cpu.reg_read(UC_ARM_REG_R0), cpu.reg_read(UC_ARM_REG_R1)
        if address == 0x82e7090:  # LZ77UnCompWram
            cpu.mem_write(dst, decompress(self.rom, src))
        elif address == 0x82e7084:  # CpuSet, used to select Deoxys frame 1
            control = cpu.reg_read(UC_ARM_REG_R2)
            assert not control & (1 << 24), 'unexpected fill operation'
            length = (control & 0x1fffff) * (4 if control & (1 << 26) else 2)
            cpu.mem_write(dst, bytes(cpu.mem_read(src, length)))
        else:
            raise AssertionError(hex(address))
        cpu.reg_write(UC_ARM_REG_PC, cpu.reg_read(UC_ARM_REG_LR))

    def render(self, species, pid, shiny):
        cpu = self.cpu
        cpu.mem_write(0x2020000, bytes(16384))
        cpu.reg_write(UC_ARM_REG_SP, 0x3007000)
        cpu.mem_write(0x3007000, struct.pack('<I', 1))  # isFrontPic
        for reg, val in [(UC_ARM_REG_LR, 0x8000001),
                         (UC_ARM_REG_R0, 0x830a18c + species * 8),
                         (UC_ARM_REG_R1, 0x2020000),
                         (UC_ARM_REG_R2, species), (UC_ARM_REG_R3, pid)]:
            cpu.reg_write(reg, val)
        cpu.emu_start(0x8034a41, 0x8000000, count=2_000_000)
        assert cpu.reg_read(UC_ARM_REG_PC) == 0x8000000
        tiles = bytes(cpu.mem_read(0x2020000, 2048))
        table = 0x304438 if shiny else 0x303678
        palette = decompress(self.rom, struct.unpack_from('<I', self.rom, table + species * 8)[0])
        colors = []
        for index, color in enumerate(struct.unpack('<16H', palette[:32])):
            rgb = [((color >> shift) & 31) for shift in (0, 5, 10)]
            colors.append(tuple((v << 3) | (v >> 2) for v in rgb) + (255 if index else 0,))
        image = Image.new('RGBA', (64, 64))
        for y in range(64):
            for x in range(64):
                byte = tiles[(y // 8 * 8 + x // 8) * 32 + y % 8 * 4 + x % 8 // 2]
                image.putpixel((x, y), colors[(byte >> ((x & 1) * 4)) & 15])
        return image


def main():
    token = secrets.token_hex(32)
    env = dict(os.environ, GEN3_DEV_TOKEN=token)
    bridge = subprocess.Popen([os.environ.get('GEN3_DEV_BIN', 'gen3-dev')], env=env,
                              stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    def api(command, payload):
        request = urllib.request.Request('http://127.0.0.1:8766/api',
            json.dumps({'command': command, 'payload': payload}).encode(),
            {'Content-Type': 'application/json', 'X-Gen3-Token': token})
        result = json.load(urllib.request.urlopen(request, timeout=30))
        assert result['ok'], result
        return result['data']
    previews = []
    try:
        for attempt in range(100):
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
            api('open_rom', {'path': path})  # Validates exact fingerprint.
            native = NativeRenderer(Path(path).read_bytes())
            # All 256 letter selectors, unused PID bits, spot extremes/random PIDs,
            # both palettes, normal species, Castform and the Deoxys frame.
            unown_pids = [(v & 3) | ((v & 12) << 6) | ((v & 48) << 12) | ((v & 192) << 18)
                         for v in range(256)]
            cases = [(201, pid) for pid in unown_pids + [0xffffffff, 0xfcfcfcfc]]
            cases += [(308, pid) for pid in [0, 0xffffffff, 0x88888888, 0x12345678,
                                             0xabcdef01, 0x0f0f0f0f, 0xf0f0f0f0]]
            cases += [(id, 0x12345678) for id in [1, 385, 410]]
            for species, pid in cases:
                for shiny in [False, True]:
                    url = api('sprite', {'id': species, 'pid': pid, 'shiny': shiny})['url']
                    image = Image.open(io.BytesIO(base64.b64decode(url.split(',')[1]))).convert('RGBA')
                    assert image.tobytes() == native.render(species, pid, shiny).tobytes(), (key, species, hex(pid), shiny)
                    if key == 'GEN3_ROM_BW' and not shiny and (species != 201 or pid in unown_pids[:28]):
                        previews.append((f'{species}/{pid:08X}', image.copy()))
            print(f'{key}: {len(cases) * 2} PNGs match native rendering pixel for pixel')
        if os.environ.get('GEN3_SPRITE_PREVIEW'):
            sheet = Image.new('RGB', (8 * 144, ((len(previews) + 7) // 8) * 154), '#edf0f4')
            draw = ImageDraw.Draw(sheet)
            for i, (label, image) in enumerate(previews):
                x, y = i % 8 * 144, i // 8 * 154
                image = image.resize((128, 128), Image.Resampling.NEAREST)
                sheet.paste(image, (x + 8, y), image)
                draw.text((x + 5, y + 130), label, fill='black')
            sheet.save(os.environ['GEN3_SPRITE_PREVIEW'])
    finally:
        bridge.terminate()
        bridge.wait(timeout=10)


if __name__ == '__main__':
    main()
