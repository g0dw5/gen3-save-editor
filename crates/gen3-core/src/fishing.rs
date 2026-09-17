//! Save-seeded Emerald fishing spots. Ordinary wild tables do not contain these.
use crate::{binary::*, err, profile::FeebasRules, rom::Rom, save::Save, world::Map, Result};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FishingSpot {
    pub x: u16,
    pub y: u16,
    pub spot_id: u16,
}
#[derive(Debug, Serialize)]
pub struct FishingReport {
    pub map_id: String,
    pub species: u16,
    pub min_level: u8,
    pub max_level: u8,
    /// Conditional on a successful fishing encounter on a selected water tile.
    pub percent: u8,
    pub seed: Option<u16>,
    pub spots: Vec<FishingSpot>,
}

fn selected_spots(seed: u16, count: u16) -> Result<Vec<u16>> {
    if count < 4 {
        return Err(err("fishing_rule", "fewer than four fishing spots"));
    }
    let mut rng = seed as u32;
    let mut spots = Vec::with_capacity(6);
    // Bound work for malformed adapters. Duplicate rolls are intentional.
    for _ in 0..4096 {
        rng = rng.wrapping_mul(0x41c64e6d).wrapping_add(0x3039);
        let mut id = (rng >> 16) as u16 % count;
        if id == 0 {
            id = count;
        }
        if id >= 4 {
            spots.push(id);
            if spots.len() == 6 {
                return Ok(spots);
            }
        }
    }
    Err(err("fishing_rule", "spot selection did not terminate"))
}

fn locate(rom: &Rom, map: &Map, rules: FeebasRules, selected: &[u16]) -> Result<Vec<FishingSpot>> {
    let b = &rom.data;
    let blocks = pointer(b, map.layout + 12)?;
    let attributes = [16, 20]
        .map(|offset| pointer(b, map.layout + offset).and_then(|tileset| pointer(b, tileset + 16)));
    let [primary, secondary] = attributes;
    let attributes = [primary?, secondary?];
    let mut spots = Vec::new();
    let mut last_y = None;
    for section in 0..3 {
        let row = rules.water_sections + section * 6;
        let lo = u16(b, row)?;
        let hi = u16(b, row + 2)?;
        // The engine restarts numbering at each ROM-defined section base.
        // Do not renumber all water tiles globally: this hack has extra tiles.
        let mut id = u16(b, row + 4)?;
        if lo > hi || u32::from(hi) >= map.height || last_y.is_some_and(|y| lo <= y) {
            return Err(err("fishing_rule", "invalid water section bounds"));
        }
        last_y = Some(hi);
        for y in lo..=hi {
            for x in 0..map.width {
                let tile = u16(
                    b,
                    blocks + (y as usize * map.width as usize + x as usize) * 2,
                )? as usize
                    & 1023;
                let behavior = (u16(b, attributes[tile / 512] + (tile % 512) * 2)? & 255) as usize;
                let flags = bytes(b, rules.behavior_flags + behavior, 1)?[0];
                if flags & 2 == 0 || behavior == 0x13 {
                    continue;
                }
                id = id
                    .checked_add(1)
                    .ok_or_else(|| err("fishing_rule", "spot ID overflow"))?;
                if selected.contains(&id) {
                    spots.push(FishingSpot {
                        x: x as u16,
                        y,
                        spot_id: id,
                    });
                }
            }
        }
    }
    Ok(spots)
}

impl Rom {
    pub fn fishing_spots(&self, save: Option<&Save>) -> Result<Option<FishingReport>> {
        let Some(rules) = self.profile.feebas else {
            return Ok(None);
        };
        let record = bytes(&self.data, rules.wild_record, 4)?;
        let species = u16(record, 2)?;
        self.valid_species(species)?;
        if record[0] == 0 || record[0] > record[1] || record[1] > 100 {
            return Err(err("fishing_rule", "invalid encounter levels"));
        }
        let seed = save
            .map(|s| u16(&s.logical(1..=4), rules.seed_offset))
            .transpose()?;
        let spots = if let Some(seed) = seed {
            let map = self
                .maps()?
                .into_iter()
                .find(|m| m.id == rules.map_id)
                .ok_or_else(|| err("map_id", rules.map_id))?;
            locate(self, &map, rules, &selected_spots(seed, rules.spot_count)?)?
        } else {
            Vec::new()
        };
        Ok(Some(FishingReport {
            map_id: rules.map_id.into(),
            species,
            min_level: record[0],
            max_level: record[1],
            percent: 50,
            seed,
            spots,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_seed_vector_and_bounded_invalid_rule() {
        assert_eq!(
            selected_spots(0x1234, 447).unwrap(),
            [247, 306, 425, 132, 230, 377]
        );
        assert!(selected_spots(0, 3).is_err());
        // Six selections may include the same tile; never retry duplicates.
        assert_eq!(selected_spots(0, 4).unwrap(), [4; 6]);
    }

    #[test]
    fn section_bases_waterfall_and_secondary_tileset() {
        let mut b = vec![0; 0x2000000];
        put32(&mut b, 0x10c, 0x08000200);
        put32(&mut b, 0x110, 0x08000300);
        put32(&mut b, 0x114, 0x08000320);
        put32(&mut b, 0x310, 0x08000400);
        put32(&mut b, 0x330, 0x08000800);
        put16(&mut b, 0x402, 0x10);
        put16(&mut b, 0x404, 0x13);
        put16(&mut b, 0x800, 0x10);
        b[0xc10] = 2;
        b[0xc13] = 2;
        for (i, tile) in [1, 2, 0, 512, 1, 1, 1, 0, 1].iter().enumerate() {
            put16(&mut b, 0x200 + i * 2, *tile);
        }
        for (i, value) in [0, 0, 0, 1, 1, 100, 2, 2, 200].iter().enumerate() {
            put16(&mut b, 0xd00 + i * 2, *value);
        }
        let rom = Rom::fixture(b);
        let map = Map {
            id: "0-34".into(),
            name: "test".into(),
            group: 0,
            number: 34,
            region: 0,
            width: 3,
            height: 3,
            layout: 256,
            header: 0,
            map_type: 0,
            events: None,
            objects: vec![],
            scripts: vec![],
        };
        let rules = FeebasRules {
            map_id: "0-34",
            seed_offset: 0,
            wild_record: 0,
            water_sections: 0xd00,
            behavior_flags: 0xc00,
            spot_count: 447,
        };
        let result = locate(&rom, &map, rules, &[1, 101, 103, 202]).unwrap();
        assert_eq!(
            result
                .iter()
                .map(|s| (s.x, s.y, s.spot_id))
                .collect::<Vec<_>>(),
            [(0, 0, 1), (0, 1, 101), (2, 1, 103), (2, 2, 202)]
        );
    }
}
