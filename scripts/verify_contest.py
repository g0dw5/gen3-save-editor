"""Verify contest feeding against exact native ROMs using synthetic individuals.

Requires GEN3_ROM_BW/DP/ROCKET, GEN3_CONTEST_PROBES and Unicorn. Never opens a save.
"""

import os, sys, struct, json, hashlib, heapq
from pathlib import Path

sys.path.insert(0, "scripts")
from verify_rocket_battle_forms import Native, pack, unpack
from unicorn import UC_HOOK_CODE
from unicorn.arm_const import UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_PC, UC_ARM_REG_LR


def verify_blender(rom):
    n = Native(rom)
    # Only unrelated presentation calls and explicit test scene inputs are hooked.
    state = {"master": False, "berries": []}

    def hook(cpu, address, size, data):
        if address == 0x809D790:
            cpu.reg_write(UC_ARM_REG_R0, 0 if state["master"] else 1)
        elif address == 0x8082FDC:
            state["berries"].append(cpu.reg_read(UC_ARM_REG_R1) - 133)
        cpu.reg_write(UC_ARM_REG_PC, cpu.reg_read(UC_ARM_REG_LR))

    for address in [0x809D790, 0x8082FDC, 0x807F738, 0x80832BC]:
        n.cpu.hook_add(UC_HOOK_CODE, hook, begin=address, end=address)
    berries = [
        list(rom[0x58A670 + i * 28 + 21 : 0x58A670 + i * 28 + 27]) for i in range(43)
    ]
    opponents = list(rom[0x339CA0:0x339CBE])
    masters = list(rom[0x339CBE:0x339CC3])
    count = 0
    for berry in range(43):
        for players in (2, 3, 4):
            for master in ([False, True] if players == 2 else [False]):
                state.update(master=master, berries=[])
                n.half(struct.unpack_from("<I", rom, 0x80744)[0], players - 1)
                entry = (
                    struct.pack("<H", berry + 133)
                    + b"TEST\xff\0\0"
                    + bytes(berries[berry])
                    + b"\0"
                )
                assert len(entry) == 16
                n.write(0x2023000, entry)
                n.call(0x80674, berry + 133, players, 0x2023000)
                ix = (
                    min(range(5), key=lambda i: berries[berry][i]) + 5
                    if berry == 42
                    else berry if berry < 5 else berry % 5 + 5
                )
                expected = (
                    [masters[ix % 5] - (5 if 30 <= berry < 35 else 0)]
                    if master
                    else opponents[ix * 3 : ix * 3 + players - 1]
                )
                assert state["berries"] == expected, (
                    berry,
                    players,
                    master,
                    state,
                    expected,
                )
                ids = [berry] + expected
                for i, id in enumerate(ids):
                    n.write(
                        0x2023000 + i * 16,
                        struct.pack("<H", id + 133)
                        + b"TEST\xff\0\0"
                        + bytes(berries[id])
                        + b"\0",
                    )
                for rpm in (0, 10000, 14000, 16633, 18000):
                    n.call(0x81BE0, 0x2023000, 0x2023100, players, 0x2023200, rpm)
                    block = list(n.read(0x2023100, 8))
                    total = [sum(berries[id][j] for id in ids) for j in range(6)]
                    raw = [total[i] - total[(i + 1) % 5] for i in range(5)]
                    neg = sum(v < 0 for v in raw)
                    values = [
                        min(255, (max(0, v - neg) * (100 + rpm // 333) + 50) // 100)
                        for v in raw
                    ]
                    feel = max(0, total[5] // players - players)
                    if block[0] == 12:
                        assert sorted(block[1:6]) == [0, 0, 2, 2, 2]
                    else:
                        assert block[1:6] == values, (ids, rpm, block, values)
                    assert block[6] == feel
                    if (
                        rpm == 16633
                        and block[2]
                        and not any(block[j] for j in (1, 3, 4, 5))
                    ):
                        assert (feel == 23 and block[2] <= 28) or (
                            feel >= 20 and block[2] <= 14
                        ), (ids, block)
                    count += 1
    # Verify native speed transitions around both branches and through the relaxed
    # search domain; audio and background shaking are skipped, arithmetic is native.
    global_ptr = struct.unpack_from("<I", rom, 0x813D4)[0]
    n.word(global_ptr, 0x2024000)
    best = struct.unpack_from("<I", rom, 0x81390)[0]
    good = struct.unpack_from("<I", rom, 0x813A0)[0]
    for players in (2, 3, 4):
        n.write(0x202407C, bytes([players]))
        for speed in (128, 256, 1498, 1499, 1500, 1501, 2000, 3000, 4000, 4999):
            for command in (best, good):
                n.half(0x202404C, speed)
                n.call(0x81370, command)
                actual = struct.unpack("<h", n.read(0x202404C, 2))[0]
                increment = (
                    (384 if speed < 1500 else 128) // players
                    if command == best
                    else 256 // players if speed < 1500 else 0
                )
                assert actual == speed + increment, (players, speed, command, actual)
    # Minimum progress needed to reach each speed, allowing free slowdown at any
    # time and perfect hits without timing restrictions. More permissive than play.
    bounds = []
    for players in (2, 3, 4):
        dist = [10**9] * 5001
        dist[128] = 0
        queue = [(0, 128)]
        while queue:
            progress, speed = heapq.heappop(queue)
            if progress != dist[speed] or progress >= 1000:
                continue
            fast = speed + (384 if speed < 1500 else 128) // players
            ok = speed + 256 // players if speed < 1500 else speed
            for next_speed, cost in [
                (fast, fast // 55),
                (ok, ok // 70),
                (max(128, speed - 1), 0),
            ]:
                assert next_speed < len(dist), "expand upper-bound search domain"
                new = progress + cost
                if new < dist[next_speed]:
                    dist[next_speed] = new
                    heapq.heappush(queue, (new, next_speed))
        peak = max(s for s, p in enumerate(dist) if p < 1000) + players * (
            128 // players
        )
        bounds.append(360000 * peak // 65536)
    assert max(bounds) < 18000, bounds
    print(
        "NPC selectors/blends:",
        count,
        "native probes; relaxed RPM bounds:",
        bounds,
        flush=True,
    )
    return {"blend_probes": count, "rpm_bounds": bounds}


def verify_beauty_only(rom):
    """A lower bound and a native 227-feel witness for PID nature 9.

    The bound is scoped to ordinary solo NPC recipes, not link/event berries.
    Two-player positive inputs cannot be consecutive: A uses new presses, the NPC
    latches each visit, and their windows are opposite (including its one-frame
    delay). Allow arbitrary favorable timing subject only to this weaker bound.
    """
    dist = [10**9] * (5000 * 6)
    dist[128 * 6] = 0
    queue = [(0, 128, 0)]
    peak = 0
    while queue:
        progress, speed, timer = heapq.heappop(queue)
        if progress != dist[speed * 6 + timer] or progress >= 1000:
            continue
        nt = (timer + 1) % 6
        ns = speed - (1 if nt == 0 and speed > 128 else 0)
        for value, divisor in [
            (ns, 0),
            (ns + (384 if ns < 1500 else 128) // 2, 55),
            (ns + 128 if ns < 1500 else ns, 70),
        ]:
            cost = value // divisor if divisor else 0
            total = progress + cost
            if total >= 1000:
                peak = max(peak, value)
                continue
            tt = (nt + 1) % 6 if divisor else nt
            vv = value - (1 if divisor and tt == 0 and value > 128 else 0)
            if total < dist[vv * 6 + tt]:
                dist[vv * 6 + tt] = total
                heapq.heappush(queue, (total, vv, tt))
    rpm_bound = peak * 360000 // 65536
    assert rpm_bound == 16633 and rpm_bound < 16650
    # At this generous bound, the best pure-dry recipe gives <=28 per 23 feel.
    # All other pure-dry NPC recipes give <=14, with feel >=20 (also granting the
    # 3/4-player recipes the two-player ceiling). No other flavors can decrease.
    # Overestimating cheap blocks as 14 still proves feel <227 insufficient:
    best = [-1] * 255
    best[0] = 0
    for feel in range(255):
        if best[feel] < 0:
            continue
        for cost, gain in [(23, 28), (20, 14)]:
            if feel + cost < 255:
                best[feel + cost] = max(best[feel + cost], best[feel] + gain)
    assert max(best[:227]) < 255
    n = Native(rom)
    n.word(0x203BC90, 0x2020000)
    raw = bytearray(100)
    pid = 0xB4F35640
    pid = pid - pid % 25 + 9
    struct.pack_into("<II", raw, 0, pid, 0x12345678)
    raw[18] = 2
    raw[19] = 2
    c = bytearray(48)
    struct.pack_into("<H", c, 0, 328)
    raw = pack(raw, c)
    n.write(0x2021000, raw)
    proof = []
    for berry in [11] * 9 + [6]:
        ids = [berry, 1]  # verified ordinary NPC selector, not the Blender Master
        for i, id in enumerate(ids):
            values = rom[0x58A670 + id * 28 + 21 : 0x58A670 + id * 28 + 27]
            n.write(
                0x2023000 + i * 16,
                struct.pack("<H", id + 133) + b"TEST\xff\0\0" + values + b"\0",
            )
        n.call(0x81BE0, 0x2023000, 0x2021100, 2, 0x2023200, 14000)
        block = list(n.read(0x2021100, 8))
        assert block[1:6] == [0, 27 if berry == 11 else 13, 0, 0, 0]
        n.half(0x203BC9E, 0)  # nature 9 is neutral to the only present flavor (dry)
        n.call(0x167054, 0x2021100, 0x2021000)
        after = list(unpack(n.read(0x2021000, 100))[30:36])
        proof.append(dict(berry_index=berry, block=block, after=after))
    assert proof[-1]["after"] == [0, 255, 0, 0, 0, 227]
    print(
        "Beauty-only witness: 255 Beauty / 227 feel; lower bound:",
        rpm_bound,
        flush=True,
    )
    return {"two_player_rpm_bound": rpm_bound, "feed_sequence": proof}


profiles = [
    ("BW", os.environ["GEN3_ROM_BW"], 0x167054, 0x203BC90, 0x203BC9E, 0x5B25A0),
    ("DP", os.environ["GEN3_ROM_DP"], 0x167054, 0x203BC90, 0x203BC9E, 0x5B25A0),
    ("ROCKET", os.environ["GEN3_ROM_ROCKET"], 0x1B31DC, 0x203CC20, 0x203CC2E, 0xC7C598),
]
expected_md5 = dict(
    BW="0d9b129f7dd76895f79bb47ad7dec2fe",
    DP="cb2940215f4dafb1bef133c3af379f44",
    ROCKET="59c658a1081f542086de1060bb65f0b3",
)

reports = []
for key, path, fun, info, gain, table in profiles:
    r = Path(path).read_bytes()
    assert hashlib.md5(r).hexdigest() == expected_md5[key]
    n = Native(r)
    n.word(info, 0x2020000)
    cases = []
    for nature in range(25):
        pref = struct.unpack_from("<5b", r, table + nature * 5)
        for current in ([0] * 6, [0, 250, 254, 10, 0, 254], [255] * 6):
            for block in (
                [10, 25, 35, 45, 55, 20],
                [0, 0, 0, 20, 20, 30],
                [255, 255, 255, 255, 255, 255],
            ):
                raw = bytearray(100)
                pid = 0xB4F35647
                pid = pid - pid % 25 + nature
                struct.pack_into("<II", raw, 0, pid, 0x12345678)
                if key == "ROCKET":
                    raw[18] = 0x12
                else:
                    raw[18] = 2
                    raw[19] = 2
                c = bytearray(48)
                struct.pack_into("<H", c, 0, 1)
                c[30:36] = bytes(current)
                if key == "ROCKET":
                    struct.pack_into("<H", c, 9, (26 << 5) | 1)
                raw = pack(raw, c)
                n.write(0x2021000, raw)
                n.write(0x2021100, bytes([1, *block, 0]))
                g = sum(a * b for a, b in zip(pref, block))
                n.half(gain, g & 65535)
                n.call(fun, 0x2021100, 0x2021000)
                after = n.read(0x2021000, 100)
                out = list(unpack(after)[30:36])
                expected = current.copy()
                if current[5] < 255:
                    for i in range(5):
                        expected[i] = min(
                            255,
                            current[i]
                            + block[i]
                            + (
                                ((block[i] + 5) // 10) * pref[i]
                                if g and pref[i] == (1 if g > 0 else -1)
                                else 0
                            ),
                        )
                    expected[5] = min(255, current[5] + block[5])
                assert out == expected, (key, nature, current, block, out, expected)
                # No unrelated decrypted bytes or headers change (except checksum).
                ac = unpack(after)
                assert ac[:30] == c[:30] and ac[36:] == c[36:]
                assert (
                    after[:28] == raw[:28]
                    and after[30:32] == raw[30:32]
                    and after[80:] == raw[80:]
                )
                cases.append(
                    dict(nature=nature, before=current, block=block, after=out)
                )
    print(
        key, len(cases), "native feed and byte-preservation probes passed", flush=True
    )
    blender = verify_blender(r) if key != "ROCKET" else None
    beauty = verify_beauty_only(r) if key != "ROCKET" else None
    reports.append(
        dict(
            profile=key,
            md5=hashlib.md5(r).hexdigest(),
            cases=cases,
            blender=blender,
            beauty=beauty,
        )
    )
Path(os.environ["GEN3_CONTEST_PROBES"]).write_text(json.dumps(reports))
