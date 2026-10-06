# Native RTC scenarios / 原生 RTC 情景验证

The Game time reference tab is read-only. Mercury 1.2 retains its separate
saved virtual-clock rules. The other four fingerprints expose saved local-time
offsets and periodic-update checkpoints, plus explicit RTC-input scenarios.
They do not infer current RTC from the host clock or a battery-save trailer.

游戏时间页只读。水银 1.2 使用已验证的虚拟时钟；其余四份指纹展示保存的时间偏移、
周期更新检查点，允许输入 RTC 日期／时间，通过当前 ROM 的原生例程计算情景结果。
偏移与检查点不能恢复当前 RTC。结果不是模拟器实时状态，不修改时钟或存档。

## Implementation and evidence / 实现与证据

- Runtime `hardware_clock.rs` executes the loaded ROM calendar validation and
  time-difference routines in disposable RAM. Native six-byte time records use
  signed 16-bit days and signed 8-bit hour/minute/second, including day-counter
  wrap. No standard calendar/day-night formula replaces the loaded routines.
- BW/DP/Ultimate validation/difference/getter file offsets:
  `0x2F2FC` / `0x2F504` / `0x2F588`; Rocket:
  `0x441A0` / `0x443A8` / `0x4442C`. SAV block 2 offsets are
  `0x98` (local-time offset) and `0xA0` (periodic-update checkpoint).
  These addresses/layouts are adapter metadata, not extracted content.
- `scripts/verify_hardware_clock.py` independently executes exact ROM routines
  using mGBA: 864 projections per fingerprint, **3,456 total**, covering 24 hours,
  leap/year boundaries and six signed offsets. Another **24 complete getter
  error-clock cases** confirm the native dummy-clock path subtracts block 2's
  saved offset. This does not validate a real hardware RTC read.
- `local_hardware_clock_matches_native_validation_offsets_and_borrowing`
  compares all projection vectors, manual versus SAV offsets, native invalid
  calendar rejection and fresh SAV snapshots. Source ROM/SAV hashes and original
  in-memory bytes are preserved. Public fixtures reject stale fingerprints,
  malformed fields, foreign save layouts and invalid offsets.
- Mercury's existing independent clock oracle was regenerated against the
  current private SAV snapshot and now records ROM/SAV SHA-256. Its native
  restore, period and jump-menu regression passed; stale SAV vectors are rejected.

The upstream [RTC implementation](https://github.com/pret/pokeemerald/blob/master/src/rtc.c)
and [clock implementation](https://github.com/pret/pokeemerald/blob/master/src/clock.c)
were semantic guides; exact local ROM execution is the verification evidence.
Private vectors and real files are excluded from Git and packages.

## Limits / 边界

All five overall clock workflows remain **partial**. For the four hardware-clock
profiles, current RTC input, initialization state, hardware fault behavior,
weekday, complete calendar/day-night selection and event refresh are not fully
resolved. A saved checkpoint is not a receipt/completion flag. The interface
keeps missing values unknown and labels manual input as a simulated condition.
Mercury's bounded saved virtual-clock evidence does not establish all weekday
refresh or event activation. No new edited-SAV emulator load/save cycle is claimed.

五份 ROM 的完整时钟工作流均为部分验证。硬件 RTC 当前输入、初始化、完整星期／
时段／事件刷新仍未确认；不能套用水银规则。水银保存快照也不代表模拟器实时状态。
