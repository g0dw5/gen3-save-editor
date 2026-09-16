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
