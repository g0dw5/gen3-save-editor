use crate::{binary::*, err, rom::Rom, Result};
pub fn lz77(b: &[u8], o: usize) -> Result<Vec<u8>> {
    let head = bytes(b, o, 4)?;
    if head[0] != 0x10 {
        return Err(err("lz77_header", o));
    }
    let size = head[1] as usize | ((head[2] as usize) << 8) | ((head[3] as usize) << 16);
    if size > 4 * 1024 * 1024 {
        return Err(err("resource_size", size));
    }
    let mut out = Vec::with_capacity(size);
    let mut src = o + 4;
    while out.len() < size {
        let flags = bytes(b, src, 1)?[0];
        src += 1;
        for bit in 0..8 {
            if out.len() == size {
                break;
            }
            if flags & (128 >> bit) == 0 {
                out.push(bytes(b, src, 1)?[0]);
                src += 1;
            } else {
                let pair = bytes(b, src, 2)?;
                src += 2;
                let n = (pair[0] >> 4) as usize + 3;
                let distance = (((pair[0] & 15) as usize) << 8) + pair[1] as usize + 1;
                if distance > out.len() {
                    return Err(err("lz77_backref", o));
                }
                for _ in 0..n {
                    if out.len() == size {
                        break;
                    }
                    out.push(out[out.len() - distance]);
                }
            }
        }
    }
    Ok(out)
}
fn color(v: u16, alpha: u8) -> [u8; 4] {
    let c = |n: u16| ((n << 3) | (n >> 2)) as u8;
    [c(v & 31), c((v >> 5) & 31), c((v >> 10) & 31), alpha]
}
pub fn png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut e = png::Encoder::new(&mut out, width, height);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        let mut w = e.write_header().map_err(|e| err("png", e))?;
        w.write_image_data(rgba).map_err(|e| err("png", e))?;
    }
    Ok(out)
}
impl Rom {
    pub fn sprite(&self, id: u16, shiny: bool) -> Result<Vec<u8>> {
        self.valid_species(id)?;
        let b = &self.data;
        let sprite = lz77(b, pointer(b, self.profile.sprites + id as usize * 8)?)?;
        let table = if shiny {
            self.profile.shiny_palettes
        } else {
            self.profile.palettes
        };
        let pal = lz77(b, pointer(b, table + id as usize * 8)?)?;
        bytes(&sprite, 0, 2048)?;
        bytes(&pal, 0, 32)?;
        let mut rgba = vec![0; 64 * 64 * 4];
        for y in 0..64 {
            for x in 0..64 {
                let tile = (y / 8) * 8 + x / 8;
                let v = sprite[tile * 32 + (y % 8) * 4 + (x % 8) / 2];
                let index = if x % 2 == 0 { v & 15 } else { v >> 4 };
                rgba[(y * 64 + x) * 4..(y * 64 + x + 1) * 4].copy_from_slice(&color(
                    u16(&pal, index as usize * 2)?,
                    if index == 0 { 0 } else { 255 },
                ));
            }
        }
        png(64, 64, &rgba)
    }
    pub fn map_image(&self, id: &str) -> Result<Vec<u8>> {
        let maps = self.maps()?;
        let m = maps
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| err("map_id", id))?;
        let w = m.width as usize * 16;
        let h = m.height as usize * 16;
        if w.checked_mul(h).is_none_or(|n| n > 4_000_000) {
            return Err(err("resource_size", format!("{w}x{h}")));
        }
        let b = &self.data;
        let blocks = pointer(b, m.layout + 12)?;
        let primary = Tileset::read(b, pointer(b, m.layout + 16)?)?;
        let secondary = Tileset::read(b, pointer(b, m.layout + 20)?)?;
        let mut rgba = vec![0; w * h * 4];
        for by in 0..m.height as usize {
            for bx in 0..m.width as usize {
                let id = u16(b, blocks + (by * m.width as usize + bx) * 2)? as usize & 1023;
                let set = if id < 512 { &primary } else { &secondary };
                let meta = set.metatiles + (id % 512) * 16;
                for layer in 0..2 {
                    for part in 0..4 {
                        let e = u16(b, meta + (layer * 4 + part) * 2)?;
                        let tile = (e & 1023) as usize;
                        let set = if tile < 512 { &primary } else { &secondary };
                        let bank = (e >> 12) as usize;
                        for py in 0..8 {
                            for px in 0..8 {
                                let tx = if e & 0x400 != 0 { 7 - px } else { px };
                                let ty = if e & 0x800 != 0 { 7 - py } else { py };
                                let off = (tile % 512) * 32 + ty * 4 + tx / 2;
                                let v = *set.tiles.get(off).unwrap_or(&0);
                                let index = if tx % 2 == 0 { v & 15 } else { v >> 4 };
                                if layer == 1 && index == 0 {
                                    continue;
                                }
                                let x = bx * 16 + (part % 2) * 8 + px;
                                let y = by * 16 + (part / 2) * 8 + py;
                                rgba[(y * w + x) * 4..(y * w + x + 1) * 4].copy_from_slice(&color(
                                    u16(&set.palette, bank * 32 + index as usize * 2)?,
                                    255,
                                ));
                            }
                        }
                    }
                }
            }
        }
        png(w as u32, h as u32, &rgba)
    }
}
struct Tileset {
    tiles: Vec<u8>,
    palette: Vec<u8>,
    metatiles: usize,
}
impl Tileset {
    fn read(b: &[u8], o: usize) -> Result<Self> {
        bytes(b, o, 24)?;
        let p = pointer(b, o + 4)?;
        let tiles = if b[o] != 0 {
            lz77(b, p)?
        } else {
            bytes(b, p, 0x4000)?.to_vec()
        };
        let palette = bytes(b, pointer(b, o + 8)?, 512)?.to_vec();
        Ok(Self {
            tiles,
            palette,
            metatiles: pointer(b, o + 12)?,
        })
    }
}
