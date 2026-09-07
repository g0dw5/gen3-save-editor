use crate::{err, Result};
pub fn bytes(b: &[u8], o: usize, n: usize) -> Result<&[u8]> {
    b.get(o..o.checked_add(n).ok_or_else(|| err("bounds", o))?)
        .ok_or_else(|| err("bounds", format!("{o:#x}+{n:#x} / {:#x}", b.len())))
}
pub fn u16(b: &[u8], o: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(bytes(b, o, 2)?.try_into().unwrap()))
}
pub fn u32(b: &[u8], o: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(bytes(b, o, 4)?.try_into().unwrap()))
}
pub fn put16(b: &mut [u8], o: usize, v: u16) {
    b[o..o + 2].copy_from_slice(&v.to_le_bytes());
}
pub fn put32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
pub fn pointer(b: &[u8], o: usize) -> Result<usize> {
    let p = u32(b, o)?;
    let off = p
        .checked_sub(0x08000000)
        .ok_or_else(|| err("pointer", format!("{p:#x}")))? as usize;
    bytes(b, off, 1)?;
    Ok(off)
}
pub fn hash(b: &[u8]) -> String {
    use md5::{Digest, Md5};
    format!("{:x}", Md5::digest(b))
}
pub fn sha256(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(b))
}
