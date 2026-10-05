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
        self.profile.capabilities.require(
            self.profile.capabilities.individual_sprites,
            "individual_sprites",
        )?;
        let species = self.valid_species(id)?;
        let b = &self.data;
        let rules = self.profile.sprite_rules;
        let female = rules.female.filter(|tables| {
            b[tables.flags + id as usize] != 0
                && crate::pokemon::gender(species.gender_ratio, pid) == "female"
        });
        let sprites = female.map_or(self.profile.sprites, |tables| tables.sprites);
        let letter = unown_letter(pid);
        let picture_id = if id == rules.unown_species && letter != 0 {
            rules.unown_b_sprite + letter - 1
        } else {
            id
        };
        // Extra Unown picture indices are graphics records, not species IDs.
        let mut sprite = lz77(b, pointer(b, sprites + picture_id as usize * 8)?)?;
        if id == rules.second_frame_species {
            sprite = bytes(&sprite, 2048, 2048)?.to_vec();
        }
        if id == rules.spinda_species {
            spinda_spots(&mut sprite, bytes(b, rules.spinda_spots, 4 * 36)?, pid)?;
        }
        let table = if shiny {
            female.map_or(self.profile.shiny_palettes, |tables| tables.shiny_palettes)
        } else {
            female.map_or(self.profile.palettes, |tables| tables.palettes)
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
        let palette_offset = std::iter::once(self.profile.object_palettes)
            .chain(self.profile.object_palette_supplements.iter().copied())
            .flat_map(|table| (0..table.count).map(move |i| table.offset + i * table.stride))
            .find(|o| u16(b, *o + 4).ok() == Some(tag))
            .ok_or_else(|| err("object_palette", tag))?;
        let palette = bytes(b, pointer(b, palette_offset)?, 32)?;
        tiled_sprite(tiles, palette, width, height)
    }
    pub fn map_image_warnings(&self, id: &str) -> Result<Vec<&'static str>> {
        let maps = self.maps()?;
        let map = maps
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| err("map_id", id))?;
        let mut warnings = Vec::new();
        let rules = self.profile.map_graphics;
        if rules.native_mismatch_headers.contains(&map.header) {
            warnings.push("mapNativeLayoutMismatch");
        }
        if rules.extended_layer_types {
            let b = &self.data;
            let blocks = pointer(b, map.layout + 12)?;
            let attributes = [
                pointer(b, pointer(b, map.layout + 16)? + 20)?,
                pointer(b, pointer(b, map.layout + 20)? + 20)?,
            ];
            for cell in 0..map.width as usize * map.height as usize {
                let id = u16(b, blocks + cell * 2)? as usize & 1023;
                let (set, local) = if id < rules.primary_metatiles {
                    (0, id)
                } else {
                    (1, id - rules.primary_metatiles)
                };
                if (u32(b, attributes[set] + local * 4)? >> 28) & 7 > 4 {
                    warnings.push("mapUnknownLayerType");
                    break;
                }
            }
        }
        Ok(warnings)
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
        let rules = self.profile.map_graphics;
        let primary = Tileset::read(
            b,
            pointer(b, m.layout + 16)?,
            rules.primary_tiles,
            rules.extended_layer_types,
        )?;
        let secondary = Tileset::read(
            b,
            pointer(b, m.layout + 20)?,
            1024 - rules.primary_tiles,
            rules.extended_layer_types,
        )?;
        let palette = map_palette(
            &primary.palette,
            &secondary.palette,
            self.profile.map_palette_banks,
        )?;
        let rgba = render_map(
            b,
            blocks,
            m.width as usize,
            m.height as usize,
            rules,
            [&primary, &secondary],
            &palette,
        )?;
        png(w as u32, h as u32, &rgba)
    }
}
// The engine can partition primary/secondary tiles independently of metatiles.
// Expanded maps also have a third background layer; format belongs to the profile.
fn render_map(
    b: &[u8],
    blocks: usize,
    width: usize,
    height: usize,
    rules: crate::profile::MapGraphics,
    sets: [&Tileset; 2],
    palette: &[u8],
) -> Result<Vec<u8>> {
    let [primary, secondary] = sets;
    let w = width * 16;
    let h = height * 16;
    let mut rgba = vec![0; w * h * 4];
    for by in 0..height {
        for bx in 0..width {
            let id = u16(b, blocks + (by * width + bx) * 2)? as usize & 1023;
            let (set, local_id) = if id < rules.primary_metatiles {
                (primary, id)
            } else {
                (secondary, id - rules.primary_metatiles)
            };
            let meta = set.metatiles + local_id * rules.layers * 8;
            // Mirror the native BG3/BG2/BG1 assignments, including its filler
            // tile and transparent index zero. Type 3 retains the 16-byte stride.
            let read_layer = |layer: usize| -> Result<[u16; 4]> {
                let mut entries = [0; 4];
                for (part, entry) in entries.iter_mut().enumerate() {
                    *entry = u16(b, meta + (layer * 4 + part) * 2)?;
                }
                Ok(entries)
            };
            let entries = if let Some(attributes) = set.attributes {
                match (u32(b, attributes + local_id * 4)? >> 28) & 7 {
                    0 | 1 => vec![[0x3014; 4], read_layer(0)?, read_layer(1)?],
                    2 => vec![read_layer(0)?, read_layer(1)?, [0; 4]],
                    3 => vec![read_layer(0)?, read_layer(1)?, read_layer(2)?],
                    4 => vec![read_layer(0)?, [0; 4], read_layer(1)?],
                    // The native hook leaves the previous BG cells intact for
                    // unknown types. There is no deterministic static replacement.
                    _ => Vec::new(),
                }
            } else {
                (0..rules.layers)
                    .map(read_layer)
                    .collect::<Result<Vec<_>>>()?
            };
            if rules.extended_layer_types {
                for py in 0..16 {
                    for px in 0..16 {
                        let off = ((by * 16 + py) * w + bx * 16 + px) * 4;
                        rgba[off..off + 4].copy_from_slice(&color(u16(palette, 0)?, 255));
                    }
                }
            }
            for (layer, entries) in entries.iter().enumerate() {
                for (part, e) in entries.iter().copied().enumerate() {
                    let tile = (e & 1023) as usize;
                    let (set, local_tile) = if tile < rules.primary_tiles {
                        (primary, tile)
                    } else {
                        (secondary, tile - rules.primary_tiles)
                    };
                    let bank = (e >> 12) as usize;
                    for py in 0..8 {
                        for px in 0..8 {
                            let tx = if e & 0x400 != 0 { 7 - px } else { px };
                            let ty = if e & 0x800 != 0 { 7 - py } else { py };
                            let off = local_tile * 32 + ty * 4 + tx / 2;
                            let v = *set.tiles.get(off).unwrap_or(&0);
                            let index = if tx % 2 == 0 { v & 15 } else { v >> 4 };
                            if (rules.extended_layer_types || layer > 0) && index == 0 {
                                continue;
                            }
                            let x = bx * 16 + (part % 2) * 8 + px;
                            let y = by * 16 + (part / 2) * 8 + py;
                            rgba[(y * w + x) * 4..(y * w + x + 1) * 4].copy_from_slice(&color(
                                u16(palette, bank * 32 + index as usize * 2)?,
                                255,
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(rgba)
}
/// Map entries address one shared palette, independently of their tile graphics.
fn map_palette(primary: &[u8], secondary: &[u8], banks: [usize; 2]) -> Result<[u8; 512]> {
    let [primary_banks, secondary_banks] = banks;
    if primary_banks > 16 || secondary_banks > 16 - primary_banks {
        return Err(err("map_palette_banks", format!("{banks:?}")));
    }
    let split = primary_banks * 32;
    let end = split + secondary_banks * 32;
    let mut palette = [0; 512];
    palette[..split].copy_from_slice(bytes(primary, 0, split)?);
    // Secondary palette pointers still address a full palette, including unused
    // primary banks. Match the engine's source and destination offsets.
    palette[split..end].copy_from_slice(bytes(secondary, split, end - split)?);
    palette[..2].fill(0); // The engine forces the backdrop color to black.
    Ok(palette)
}
struct Tileset {
    tiles: Vec<u8>,
    palette: Vec<u8>,
    metatiles: usize,
    attributes: Option<usize>,
}
impl Tileset {
    fn read(b: &[u8], o: usize, tile_count: usize, extended_layer_types: bool) -> Result<Self> {
        bytes(b, o, 24)?;
        let p = pointer(b, o + 4)?;
        let tiles = if b[o] != 0 {
            lz77(b, p)?
        } else {
            bytes(b, p, tile_count * 32)?.to_vec()
        };
        let palette = bytes(b, pointer(b, o + 8)?, 512)?.to_vec();
        Ok(Self {
            tiles,
            palette,
            metatiles: pointer(b, o + 12)?,
            attributes: if extended_layer_types {
                Some(pointer(b, o + 20)?)
            } else {
                None
            },
        })
    }
}

#[cfg(test)]
mod appearance_tests {
    use super::*;

    #[test]
    fn map_pixels_respect_profile_partitions_layers_flips_and_palette_banks() {
        // A one-cell map using the first secondary metatile, the last primary
        // tile, and the first secondary tile. No copyrighted assets are needed.
        for profile in [
            crate::profile::BW,
            crate::profile::DP,
            crate::profile::ROCKET,
        ] {
            let rules = profile.map_graphics;
            let mut data = vec![0; 128];
            crate::binary::put16(&mut data, 0, rules.primary_metatiles as u16);
            let mut primary = Tileset {
                tiles: vec![0; rules.primary_tiles * 32],
                palette: vec![0; 512],
                metatiles: 32,
                attributes: rules.extended_layer_types.then_some(112),
            };
            let mut secondary = Tileset {
                tiles: vec![0; 32],
                palette: vec![0; 512],
                metatiles: 64,
                attributes: rules.extended_layer_types.then_some(116),
            };
            primary.tiles[(rules.primary_tiles - 1) * 32..].fill(0x11);
            secondary.tiles[31] = 0x20; // Bottom-right pixel, moved to top-left by XY flip.
            let bank = profile.map_palette_banks[0];
            crate::binary::put16(&mut primary.palette, 2, 31);
            crate::binary::put16(&mut secondary.palette, bank * 32 + 4, 31 << 5);
            for part in 0..4 {
                crate::binary::put16(&mut data, 64 + part * 2, (rules.primary_tiles - 1) as u16);
            }
            // The topmost layer supplies just one green pixel per quadrant;
            // zero pixels must preserve the red base, including through layer 2.
            for part in 0..4 {
                crate::binary::put16(
                    &mut data,
                    64 + ((rules.layers - 1) * 4 + part) * 2,
                    rules.primary_tiles as u16 | 0xc00 | ((bank as u16) << 12),
                );
            }
            let palette = map_palette(
                &primary.palette,
                &secondary.palette,
                profile.map_palette_banks,
            )
            .unwrap();
            let pixels =
                render_map(&data, 0, 1, 1, rules, [&primary, &secondary], &palette).unwrap();
            for y in 0..16 {
                for x in 0..16 {
                    let expected = if x % 8 == 0 && y % 8 == 0 {
                        [0, 255, 0, 255]
                    } else {
                        [255, 0, 0, 255]
                    };
                    assert_eq!(
                        &pixels[(y * 16 + x) * 4..(y * 16 + x + 1) * 4],
                        &expected,
                        "{} ({x},{y})",
                        profile.id
                    );
                }
            }
        }
    }

    #[test]
    fn extended_metatiles_read_third_layer_without_changing_base_stride() {
        let rules = crate::mercury::PROFILE.map_graphics;
        let mut data = vec![0; 128];
        crate::binary::put16(&mut data, 2, 1);
        crate::binary::put32(&mut data, 96, 3 << 28);
        crate::binary::put32(&mut data, 100, 2 << 28);
        for part in 0..4 {
            crate::binary::put16(&mut data, 32 + part * 2, 1);
            // The following metatile's base is also the type-3 cell's third layer.
            crate::binary::put16(&mut data, 48 + part * 2, 3);
            crate::binary::put16(&mut data, 56 + part * 2, 2);
        }
        let mut primary = Tileset {
            tiles: vec![0; rules.primary_tiles * 32],
            palette: vec![0; 512],
            metatiles: 32,
            attributes: Some(96),
        };
        for tile in 1..4 {
            primary.tiles[tile * 32..(tile + 1) * 32].fill(tile as u8 * 17);
        }
        let secondary = Tileset {
            tiles: vec![0; 32],
            palette: vec![0; 512],
            metatiles: 64,
            attributes: Some(112),
        };
        let mut palette = [0; 512];
        crate::binary::put16(&mut palette, 2, 31);
        crate::binary::put16(&mut palette, 4, 31 << 5);
        crate::binary::put16(&mut palette, 6, 31 << 10);
        let rgba = render_map(&data, 0, 2, 1, rules, [&primary, &secondary], &palette).unwrap();
        for y in 0..16 {
            for x in 0..32 {
                let expected = if x < 16 {
                    [0, 0, 255, 255]
                } else {
                    [0, 255, 0, 255]
                };
                assert_eq!(&rgba[(y * 32 + x) * 4..(y * 32 + x + 1) * 4], &expected);
            }
        }
    }

    #[test]
    fn extended_transparency_uses_native_filler_and_shared_backdrop() {
        let rules = crate::mercury::PROFILE.map_graphics;
        let mut data = vec![0; 128];
        crate::binary::put16(&mut data, 2, 1);
        crate::binary::put32(&mut data, 100, 2 << 28);
        let mut primary = Tileset {
            tiles: vec![0; rules.primary_tiles * 32],
            palette: vec![0; 512],
            metatiles: 32,
            attributes: Some(96),
        };
        primary.tiles[20 * 32..21 * 32].fill(0x11);
        let secondary = Tileset {
            tiles: vec![0; 32],
            palette: vec![0; 512],
            metatiles: 64,
            attributes: Some(112),
        };
        let mut palette = [0; 512];
        crate::binary::put16(&mut palette, 3 * 32 + 2, 31);
        let rgba = render_map(&data, 0, 2, 1, rules, [&primary, &secondary], &palette).unwrap();
        for y in 0..16 {
            for x in 0..32 {
                let expected = if x < 16 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 0, 255]
                };
                assert_eq!(&rgba[(y * 32 + x) * 4..(y * 32 + x + 1) * 4], &expected);
            }
        }
    }

    #[test]
    fn map_palette_uses_bank_ownership_and_skips_unused_source_banks() {
        let primary = [0x11; 512];
        let secondary = [0x22; 512];
        let palette = map_palette(&primary, &secondary, [6, 7]).unwrap();
        assert_eq!(&palette[..2], &[0, 0]);
        assert!(palette[2..192].iter().all(|&v| v == 0x11));
        assert!(palette[192..416].iter().all(|&v| v == 0x22));
        assert!(palette[416..].iter().all(|&v| v == 0));
        assert!(map_palette(&primary, &secondary, [17, 0]).is_err());
        assert!(map_palette(&primary, &secondary, [6, 11]).is_err());
        assert!(map_palette(&primary[..191], &secondary, [6, 7]).is_err());
        assert!(map_palette(&primary, &secondary[..415], [6, 7]).is_err());
    }

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
