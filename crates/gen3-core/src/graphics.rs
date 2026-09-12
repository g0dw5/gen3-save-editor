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
/// Decode GBA 4bpp tiles, preserving transparent palette index zero.
pub fn tiled_sprite(tiles: &[u8], palette: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    if width == 0
        || height == 0
        || width > 128
        || height > 128
        || !width.is_multiple_of(8)
        || !height.is_multiple_of(8)
    {
        return Err(err("sprite_dimensions", format!("{width}x{height}")));
    }
    bytes(tiles, 0, width * height / 2)?;
    bytes(palette, 0, 32)?;
    let mut rgba = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let tile = (y / 8) * (width / 8) + x / 8;
            let value = tiles[tile * 32 + (y % 8) * 4 + (x % 8) / 2];
            let index = if x % 2 == 0 { value & 15 } else { value >> 4 };
            rgba[(y * width + x) * 4..(y * width + x + 1) * 4].copy_from_slice(&color(
                u16(palette, index as usize * 2)?,
                if index == 0 { 0 } else { 255 },
            ));
        }
    }
    png(width as u32, height as u32, &rgba)
}
/// The two low bits of each PID byte select one of 28 Unown letters.
pub fn unown_letter(pid: u32) -> u16 {
    (((pid & 0x03000000) >> 18 | (pid & 0x00030000) >> 12 | (pid & 0x00000300) >> 6 | (pid & 3))
        % 28) as u16
}

/// Recolor only the body shades covered by the four ROM-supplied spot masks.
fn spinda_spots(tiles: &mut [u8], masks: &[u8], mut pid: u32) -> Result<()> {
    bytes(tiles, 0, 2048)?;
    bytes(masks, 0, 4 * 36)?;
    for spot in masks.as_chunks::<36>().0.iter().take(4) {
        let x = spot[0] as i32 + (pid & 15) as i32 - 8;
        let y = spot[1] as i32 + ((pid >> 4) & 15) as i32 - 8;
        pid >>= 8;
        for row in 0..16 {
            let mask = u16(spot, 2 + row * 2)?;
            for column in 0..16 {
                let (px, py) = (x + column, y + row as i32);
                if mask & (1 << column) == 0 || !(0..64).contains(&px) || !(0..64).contains(&py) {
                    continue;
                }
                let (px, py) = (px as usize, py as usize);
                let offset = (py / 8 * 8 + px / 8) * 32 + py % 8 * 4 + px % 8 / 2;
                let shift = (px % 2) * 4;
                let index = (tiles[offset] >> shift) & 15;
                if (1..=3).contains(&index) {
                    tiles[offset] += 4 << shift;
                }
            }
        }
    }
    Ok(())
}

impl Rom {
    pub fn sprite(&self, id: u16, shiny: bool) -> Result<Vec<u8>> {
        self.pokemon_sprite(id, shiny, 0)
    }
    /// Static front picture. PID drives persistent individual appearances;
    /// temporary battle transformations are not inferred from a boxed Pokémon.
    pub fn pokemon_sprite(&self, id: u16, shiny: bool, pid: u32) -> Result<Vec<u8>> {
        self.valid_species(id)?;
        let b = &self.data;
        let rules = self.profile.sprite_rules;
        let letter = unown_letter(pid);
        let picture_id = if id == rules.unown_species && letter != 0 {
            rules.unown_b_sprite + letter - 1
        } else {
            id
        };
        // Extra Unown picture indices are graphics records, not species IDs.
        let mut sprite = lz77(
            b,
            pointer(b, self.profile.sprites + picture_id as usize * 8)?,
        )?;
        if id == rules.second_frame_species {
            sprite = bytes(&sprite, 2048, 2048)?.to_vec();
        }
        if id == rules.spinda_species {
            spinda_spots(&mut sprite, bytes(b, rules.spinda_spots, 4 * 36)?, pid)?;
        }
        let table = if shiny {
            self.profile.shiny_palettes
        } else {
            self.profile.palettes
        };
        let pal = lz77(b, pointer(b, table + id as usize * 8)?)?;
        tiled_sprite(&sprite, &pal, 64, 64)
    }
    pub fn trainer_sprite(&self, portrait: u16) -> Result<Vec<u8>> {
        let table = self.profile.trainer_sprites;
        if portrait as usize >= table.count {
            return Err(err("trainer_portrait", portrait));
        }
        let b = &self.data;
        let tiles = lz77(
            b,
            pointer(b, table.offset + portrait as usize * table.stride)?,
        )?;
        let palette = lz77(
            b,
            pointer(b, self.profile.trainer_palettes + portrait as usize * 8)?,
        )?;
        tiled_sprite(&tiles, &palette, 64, 64)
    }
    pub fn object_sprite(&self, graphics: u16) -> Result<Vec<u8>> {
        let bank = (graphics >> 8) as usize;
        let id = (graphics & 255) as usize;
        let table = self
            .profile
            .object_graphics
            .get(bank)
            .ok_or_else(|| err("object_graphics", graphics))?;
        if id >= table.count {
            return Err(err("object_graphics_dynamic", graphics));
        }
        let b = &self.data;
        let info = pointer(b, table.offset + id * table.stride)?;
        let width = u16(b, info + 8)? as usize;
        let height = u16(b, info + 10)? as usize;
        let tag = u16(b, info + 2)?;
        let images = pointer(b, info + 28)?;
        let tiles = bytes(b, pointer(b, images)?, u16(b, images + 4)? as usize)?;
        let palette_table = self.profile.object_palettes;
        let palette_offset = (0..palette_table.count)
            .map(|i| palette_table.offset + i * palette_table.stride)
            .find(|o| u16(b, *o + 4).ok() == Some(tag))
            .ok_or_else(|| err("object_palette", tag))?;
        let palette = bytes(b, pointer(b, palette_offset)?, 32)?;
        tiled_sprite(tiles, palette, width, height)
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

#[cfg(test)]
mod appearance_tests {
    use super::*;

    #[test]
    fn spot_masks_preserve_transparency_outlines_and_overlap() {
        let mut tiles = vec![0; 2048];
        for x in 0..16 {
            tiles[x / 8 * 32 + x % 8 / 2] |= (x as u8) << (x % 2 * 4);
        }
        let mut masks = vec![0; 144];
        for i in 0..4 {
            masks[i * 36..i * 36 + 4].copy_from_slice(&[8, 8, 255, 255]);
        }
        spinda_spots(&mut tiles, &masks, 0).unwrap();
        let expected = [0, 5, 6, 7, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        for (x, expected) in expected.into_iter().enumerate() {
            assert_eq!(
                (tiles[x / 8 * 32 + x % 8 / 2] >> (x % 2 * 4)) & 15,
                expected
            );
        }
        assert!(spinda_spots(&mut tiles[..2047], &masks, 0).is_err());
        assert!(spinda_spots(&mut tiles, &masks[..143], 0).is_err());
    }
}
