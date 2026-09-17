# Pokémon record integrity / 宝可梦记录完整性

## Validation order

A valid flash-sector checksum does not establish the integrity of every Pokémon
inside it. A game can save an already damaged in-memory record and produce valid
sector checksums around that record.

For each nonzero 80-byte boxed or 100-byte party record, the editor now:

1. Decrypts using the full 32-bit PID XOR trainer ID and restores substructure order.
2. Validates the decrypted checksum before reading species or deciding a slot is empty.
3. Rejects the Bad Egg header flag, even when the checksum matches.
4. Interprets fields and applies the existing species, move and item checks.

The same integrity gate covers slot lookup, full-storage scans, imports, direct
edits and conversion to party format. Errors include the affected location and,
for checksum failures, the stored and calculated values. Free editing does not
bypass these checks. `unpack` and `decode` remain available for read-only forensic
inspection; write paths must use `checked_unpack` first.

Previously, a nonzero record whose decrypted species was zero and whose
`hasSpecies` flag was unset could be treated as empty before its checksum was
checked. That could hide damage and allow the slot to be overwritten. A record
with a matching checksum but a Bad Egg flag was also accepted. These gaps are
fixed; they are not evidence that the editor caused a particular damaged save.

## PID edits and recovery

Changing a PID requires decrypting with the old identity, retaining canonical
data, reordering for the new PID and encrypting with the new identity. For
example, truncating `0x9ABC1357` to `0x00001357` by changing only header bytes
leaves the ciphertext inconsistent. Replacing the checksum alone is not a repair.

Recover from a known-good record only after comparing the full record and
establishing which bytes changed. Preserve the user's source, verify the repaired
output independently, and never roll back an entire current save merely to
recover one Pokémon. Do not automatically substitute a backup bank with older
game progress.

Regression coverage includes all 24 destination PID orders, high-half truncation,
the `u32` limit, corrupted records masquerading as empty slots, Bad Egg flags,
failed-load preservation, and refusal to write an invalid export over a valid
destination. Fixtures are synthetic; no user Pokémon or ROM bytes are committed.

## Storage drag/drop audit

The storage drag payload contains only a source location, the ROM fingerprint,
and the copy modifier. It does not serialize a Pokemon or a PID. The command is
`Action::Transfer`; occupied destinations swap records, and empty destinations
receive the source record before its original slot is cleared. Box moves copy
all 80 bytes; party conversion preserves those bytes and adds the party tail.
Desktop commands acquire a backend mutex and mutate a cloned save, which must
pass validation before becoming the current state.

A frontend race was reproduced in commit `fb9d6d4` (0.1.5) and the pre-fix current
UI: `inFlight.current` was tested before `await guard()` but assigned afterwards.
Two synthetic drop events in the same JavaScript turn could both pass the test.
With two occupied slots, the actual backend performed two swaps and restored the
original placement. The fix reserves the lock before the first await and releases
it in `finally`. This prevents duplicate submissions; it is not proof that the
race caused a historical Pokemon corruption.

Verification covers three transfer modes for each of the 420 box destinations,
both active save banks and fourteen physical sector rotations, using an
independent physical-byte oracle. It compares the whole save, including the
inactive bank and unowned padding, rather than only the displayed PID or checksum.
Additional tests exercise party conversions, undo/redo and disk export.
`scripts/test_drag_transactions.py` checks duplicate drop events in the real UI
with held synthetic API responses, subsequent drags and the unsaved-field guard.

Private replay against a known-good backup performed 840 transfers with both the
0.1.5 core and the current core; each final file matched the input exactly.
Chrome/Playwright tests also exercised both UIs against their respective real
Rust backends: moving to an empty box, swapping occupied slots, Alt-copying,
undoing, duplicate drops and downloading the result. Independent binary checks
found no record-header or ciphertext changes from the transfers. These tests do
not reproduce the original phone/emulator session or every WebKit event sequence;
the historical high-half PID truncation remains unexplained.

## 中文说明

外层保存区校验正确，不代表里面每只宝可梦都正确。游戏可能把已经损坏的内存记录
保存下来，并为整个保存区生成正确校验值。

现在会在解释种族和判定空槽之前检查宝可梦内部校验，并拒绝已标记为坏蛋的记录。
导入、编辑、转入同行、遍历盒子与导出均受保护，自由编辑也不能绕过。

本次修正了“损坏记录可能被当成空槽”和“校验正确但带坏蛋标志仍被接受”的防护
漏洞。它们不等于某个具体坏档的已确认根因；存档中也没有记录是哪一个程序写过某个
字节，缺少中间备份或可复现步骤时，应明确保留来源不确定性。

PID 是加密密钥及数据排列依据。不能只改 PID 的头部字节，也不能通过重算校验和
掩盖损坏。恢复应依据已知正确的记录，只修改已证实的损坏范围，保留原文件备份并
重新验证，避免为了恢复一只宝可梦而回滚其他游戏进度。

拖拽复核发现并修复了前端重复提交竞态：等待异步确认前未占用锁，可能让同一轮事件
中的两次放下都提交。实际旧版后端复现的是交换两次后回到原位；未复现 PID 高两字节
清零。0.1.5 和当前核心对正常备份各重放 840 次移动／交换，最终完整文件均与输入
逐字节一致；浏览器操作和导出回读也保持原始记录不变。不能把这次发现的竞态直接
认定为历史坏蛋根因，也不能据有限复现声称排除了所有拖拽或模拟器问题。
