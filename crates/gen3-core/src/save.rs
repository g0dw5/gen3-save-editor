//! Sector ownership and lossless logical block access for Emerald-family saves.
use crate::{
    binary::*,
    err,
    pokemon::{self, Pokemon, PokemonPatch, Policy},
    profile::SaveLayout,
    rom::Rom,
    Result,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const SECTOR: usize = 4096;
const SLOT: usize = 14 * SECTOR;
const SIGNATURE: u32 = 0x08012025;

#[derive(Clone)]
pub struct Save {
    pub data: Vec<u8>,
    pub layout: SaveLayout,
    pub active_slot: usize,
    pub counter: u32,
    pub sections: [usize; 14],
    pub backup_valid: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Location {
    Party { slot: usize },
    Box { box_index: usize, slot: usize },
}
#[derive(Serialize)]
pub struct StoredPokemon {
    pub location: Location,
    pub pokemon: Pokemon,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trainer {
    pub name: String,
    pub gender: u8,
    pub tid: u16,
    pub sid: u16,
    pub hours: u16,
    pub minutes: u8,
    pub seconds: u8,
    pub money: u32,
    pub coins: u16,
    pub registered_item: u16,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainerPatch {
    pub name: Option<String>,
    pub gender: Option<u8>,
    pub tid: Option<u16>,
    pub sid: Option<u16>,
    pub hours: Option<u16>,
    pub minutes: Option<u8>,
    pub seconds: Option<u8>,
    pub money: Option<u32>,
    pub coins: Option<u16>,
    pub registered_item: Option<u16>,
}
#[derive(Clone, Copy, Serialize)]
pub struct Pocket {
    pub id: &'static str,
    pub offset: usize,
    pub count: usize,
    pub encrypted: bool,
    pub category: u8,
}
pub const POCKETS: [Pocket; 6] = [
    Pocket {
        id: "pc",
        offset: 0x498,
        count: 50,
        encrypted: false,
        category: 0,
    },
    Pocket {
        id: "items",
        offset: 0x560,
        count: 30,
        encrypted: true,
        category: 1,
    },
    Pocket {
        id: "key_items",
        offset: 0x5d8,
        count: 30,
        encrypted: true,
        category: 4,
    },
    Pocket {
        id: "balls",
        offset: 0x650,
        count: 16,
        encrypted: true,
        category: 2,
    },
    Pocket {
        id: "tmhm",
        offset: 0x690,
        count: 64,
        encrypted: true,
        category: 3,
    },
    Pocket {
        id: "berries",
        offset: 0x790,
        count: 46,
        encrypted: true,
        category: 5,
    },
];
#[derive(Serialize)]
pub struct BagEntry {
    pub pocket: String,
    pub slot: usize,
    pub item: u16,
    pub quantity: u16,
}
#[derive(Serialize)]
pub struct BoxInfo {
    pub index: usize,
    pub name: String,
    pub wallpaper: u8,
    pub count: usize,
}
#[derive(Serialize)]
pub struct DexFlag {
    pub number: u16,
    pub seen: bool,
    pub owned: bool,
}
pub fn sector_checksum(b: &[u8]) -> u16 {
    let sum = b
        .as_chunks::<4>()
        .0
        .iter()
        .fold(0u32, |a, c| a.wrapping_add(u32::from_le_bytes(*c)));
    ((sum >> 16) + (sum & 65535)) as u16
}
fn slot(data: &[u8], base: usize, layout: SaveLayout) -> Result<([usize; 14], u32)> {
    let mut refs = BTreeMap::new();
    let mut counter = None;
    for i in 0..14 {
        let o = base + i * SECTOR;
        let b = bytes(data, o, SECTOR)?;
        let id = u16(b, 0xff4)? as usize;
        if id >= 14 || u32(b, 0xff8)? != SIGNATURE {
            return Err(err("save_sector", i));
        }
        if refs.insert(id, o).is_some() {
            return Err(err("save_duplicate_sector", id));
        }
        if sector_checksum(&b[..layout.sizes[id]]) != u16(b, 0xff6)? {
            return Err(err("save_checksum", id));
        }
        let n = u32(b, 0xffc)?;
        if counter.is_some_and(|c| c != n) {
            return Err(err("save_mixed_counter", i));
        }
        counter = Some(n);
    }
    Ok((std::array::from_fn(|id| refs[&id]), counter.unwrap()))
}
impl Save {
    pub fn open(data: Vec<u8>, layout: SaveLayout) -> Result<Self> {
        if data.len() != 0x20000 {
            return Err(err(
                "save_size",
                format!("expected 131072 battery-save bytes, got {}", data.len()),
            ));
        }
        let a = slot(&data, 0, layout);
        let b = slot(&data, SLOT, layout);
        let backup_valid = a.is_ok() && b.is_ok();
        let (active_slot, (sections, counter)) = match (a, b) {
            (Ok(a), Ok(b)) => {
                if b.1.wrapping_sub(a.1) as i32 > 0 {
                    (1, b)
                } else {
                    (0, a)
                }
            }
            (Ok(a), Err(_)) => (0, a),
            (Err(_), Ok(b)) => (1, b),
            (Err(a), Err(b)) => return Err(err("save_no_valid_slot", format!("A {a}; B {b}"))),
        };
        let s = Self {
            data,
            layout,
            active_slot,
            counter,
            sections,
            backup_valid,
        };
        if s.party_count() > 6 {
            return Err(err("party_count", s.party_count()));
        }
        Ok(s)
    }
    pub fn logical(&self, ids: std::ops::RangeInclusive<usize>) -> Vec<u8> {
        ids.flat_map(|i| {
            self.data[self.sections[i]..self.sections[i] + self.layout.sizes[i]].to_vec()
        })
        .collect()
    }
    fn write_logical(&mut self, ids: std::ops::RangeInclusive<usize>, data: &[u8]) -> Result<()> {
        let total: usize = ids.clone().map(|i| self.layout.sizes[i]).sum();
        if data.len() != total {
            return Err(err("block_size", data.len()));
        }
        let mut pos = 0;
        for i in ids {
            let n = self.layout.sizes[i];
            let off = self.sections[i];
            self.data[off..off + n].copy_from_slice(&data[pos..pos + n]);
            pos += n;
            self.fix(i);
        }
        Ok(())
    }
    fn fix(&mut self, id: usize) {
        let o = self.sections[id];
        let sum = sector_checksum(&self.data[o..o + self.layout.sizes[id]]);
        put16(&mut self.data, o + 0xff6, sum);
    }
    pub fn party_count(&self) -> usize {
        self.data[self.sections[1] + self.layout.party_count] as usize
    }
    fn location_offset(&self, loc: Location) -> Result<(usize, usize)> {
        match loc {
            Location::Party { slot } => {
                if slot >= 6 {
                    return Err(err("location", slot));
                }
                Ok((self.layout.party + slot * 100, 100))
            }
            Location::Box { box_index, slot } => {
                if box_index >= self.layout.boxes || slot >= self.layout.slots {
                    return Err(err("location", format!("{box_index}:{slot}")));
                }
                Ok((4 + (box_index * self.layout.slots + slot) * 80, 80))
            }
        }
    }
    pub fn raw(&self, loc: Location) -> Result<Vec<u8>> {
        let (o, n) = self.location_offset(loc)?;
        match loc {
            Location::Party { slot } => {
                if slot >= self.party_count() {
                    return Ok(vec![0; 100]);
                }
                Ok(bytes(&self.data, self.sections[1] + o, n)?.to_vec())
            }
            Location::Box { .. } => Ok(bytes(&self.logical(5..=13), o, n)?.to_vec()),
        }
    }
    pub fn pokemon(&self, loc: Location, rom: &Rom) -> Result<Option<Pokemon>> {
        let raw = self.raw(loc)?;
        if raw.iter().all(|b| *b == 0) {
            return Ok(None);
        }
        let c = pokemon::unpack(&raw)?;
        if u16(&c, 0)? == 0 && raw[19] & 2 == 0 {
            return Ok(None);
        }
        let p = pokemon::decode(&raw, rom)?;
        if p.species == 0 {
            return Err(err("pokemon_empty_flags", format!("{loc:?}")));
        }
        Ok(Some(p))
    }
    fn write_raw(&mut self, loc: Location, raw: &[u8]) -> Result<()> {
        let (o, n) = self.location_offset(loc)?;
        if raw.len() != n {
            return Err(err("pokemon_size", raw.len()));
        }
        match loc {
            Location::Party { .. } => {
                let off = self.sections[1] + o;
                self.data[off..off + n].copy_from_slice(raw);
                self.fix(1);
            }
            Location::Box { .. } => {
                let mut b = self.logical(5..=13);
                b[o..o + n].copy_from_slice(raw);
                self.write_logical(5..=13, &b)?;
            }
        }
        Ok(())
    }
    fn write_party(&mut self, party: &[Vec<u8>]) -> Result<()> {
        if party.len() > 6 {
            return Err(err("party_count", party.len()));
        }
        let base = self.sections[1];
        self.data[base + self.layout.party_count] = party.len() as u8;
        for i in 0..6 {
            let off = base + self.layout.party + i * 100;
            if let Some(raw) = party.get(i) {
                if raw.len() != 100 {
                    return Err(err("pokemon_size", raw.len()));
                }
                self.data[off..off + 100].copy_from_slice(raw);
            } else {
                self.data[off..off + 100].fill(0);
            }
        }
        self.fix(1);
        Ok(())
    }
    pub fn all(&self, rom: &Rom) -> Result<Vec<StoredPokemon>> {
        let mut out = Vec::new();
        for i in 0..self.party_count() {
            let location = Location::Party { slot: i };
            if let Some(pokemon) = self.pokemon(location, rom)? {
                out.push(StoredPokemon { location, pokemon });
            } else {
                return Err(err("party_gap", i));
            }
        }
        // Assemble storage once instead of once per slot.
        let storage = self.logical(5..=13);
        for b in 0..self.layout.boxes {
            for i in 0..self.layout.slots {
                let off = 4 + (b * self.layout.slots + i) * 80;
                let raw = &storage[off..off + 80];
                if raw.iter().all(|b| *b == 0) {
                    continue;
                }
                let c = pokemon::unpack(raw)?;
                if u16(&c, 0)? == 0 && raw[19] & 2 == 0 {
                    continue;
                }
                out.push(StoredPokemon {
                    location: Location::Box {
                        box_index: b,
                        slot: i,
                    },
                    pokemon: pokemon::decode(raw, rom)?,
                });
            }
        }
        Ok(out)
    }
    pub fn validate(&self, rom: &Rom) -> Result<()> {
        slot(&self.data, self.active_slot * SLOT, self.layout)?;
        for row in self.all(rom)? {
            if !row.pokemon.checksum_ok {
                return Err(err("pokemon_checksum", format!("{:?}", row.location)));
            }
            rom.valid_species(row.pokemon.species)?;
            rom.item(row.pokemon.held_item)?;
            for id in row.pokemon.moves {
                rom.move_info(id)?;
            }
        }
        Ok(())
    }
    pub fn edit(
        &mut self,
        loc: Location,
        patch: &PokemonPatch,
        rom: &Rom,
        policy: Policy,
    ) -> Result<Vec<pokemon::Finding>> {
        if self.pokemon(loc, rom)?.is_none() {
            return Err(err("empty_slot", format!("{loc:?}")));
        }
        let (raw, findings) = pokemon::edit(&self.raw(loc)?, patch, rom, policy)?;
        self.write_raw(loc, &raw)?;
        Ok(findings)
    }
    pub fn insert(&mut self, loc: Location, raw: &[u8], rom: &Rom) -> Result<()> {
        if self.pokemon(loc, rom)?.is_some() {
            return Err(err("occupied_slot", format!("{loc:?}")));
        }
        let p = pokemon::decode(raw, rom)?;
        if !p.checksum_ok {
            return Err(err("pokemon_checksum", p.species));
        }
        rom.valid_species(p.species)?;
        rom.item(p.held_item)?;
        if rom.is_mail(p.held_item) {
            return Err(err("mail_attachment", "import/insert"));
        }
        for id in p.moves {
            rom.move_info(id)?;
        }
        match loc {
            Location::Party { .. } => {
                let n = self.party_count();
                if n >= 6 {
                    return Err(err("party_full", n));
                }
                let mut rows = (0..n)
                    .map(|i| self.raw(Location::Party { slot: i }))
                    .collect::<Result<Vec<_>>>()?;
                rows.push(pokemon::to_party(raw, rom)?);
                self.write_party(&rows)
            }
            Location::Box { .. } => self.write_raw(loc, &raw[..80]),
        }
    }
    pub fn remove(&mut self, loc: Location, rom: &Rom) -> Result<()> {
        if self
            .pokemon(loc, rom)?
            .is_some_and(|p| rom.is_mail(p.held_item))
        {
            return Err(err("mail_attachment", "delete"));
        }
        if self.pokemon(loc, rom)?.is_none() {
            return Err(err("empty_slot", format!("{loc:?}")));
        }
        match loc {
            Location::Party { slot } => {
                let n = self.party_count();
                if n <= 1 {
                    return Err(err("last_party", "at least one party Pokemon"));
                }
                let rows = (0..n)
                    .filter(|i| *i != slot)
                    .map(|i| self.raw(Location::Party { slot: i }))
                    .collect::<Result<Vec<_>>>()?;
                self.write_party(&rows)
            }
            Location::Box { .. } => self.write_raw(loc, &[0; 80]),
        }
    }
    pub fn transfer(&mut self, from: Location, to: Location, copy: bool, rom: &Rom) -> Result<()> {
        if from == to {
            return Ok(());
        }
        let source = self
            .pokemon(from, rom)?
            .ok_or_else(|| err("empty_slot", format!("{from:?}")))?;
        if !source.checksum_ok {
            return Err(err("pokemon_checksum", source.species));
        }
        let target = self.pokemon(to, rom)?;
        if rom.is_mail(source.held_item)
            || target.as_ref().is_some_and(|p| rom.is_mail(p.held_item))
        {
            return Err(err("mail_attachment", "transfer"));
        }
        let raw = self.raw(from)?;
        if copy {
            return self.insert(to, &raw, rom);
        }
        if let Some(p) = target {
            if !p.checksum_ok {
                return Err(err("pokemon_checksum", p.species));
            }
            let other = self.raw(to)?;
            let converted = |r: &[u8], loc| {
                if matches!(loc, Location::Party { .. }) {
                    if r.len() == 100 {
                        Ok(r.to_vec())
                    } else {
                        pokemon::to_party(r, rom)
                    }
                } else {
                    Ok(r[..80].to_vec())
                }
            };
            let a = converted(&raw, to)?;
            let b = converted(&other, from)?;
            self.write_raw(from, &b)?;
            self.write_raw(to, &a)
        } else {
            // Check the source before insertion so a failed removal is atomic.
            if matches!(from, Location::Party { .. })
                && self.party_count() <= 1
                && !matches!(to, Location::Party { .. })
            {
                return Err(err("last_party", "move"));
            }
            if let (Location::Party { slot: src }, Location::Party { .. }) = (from, to) {
                let n = self.party_count();
                let mut rows = (0..n)
                    .filter(|i| *i != src)
                    .map(|i| self.raw(Location::Party { slot: i }))
                    .collect::<Result<Vec<_>>>()?;
                rows.push(raw);
                return self.write_party(&rows);
            }
            self.insert(to, &raw, rom)?;
            self.remove(from, rom)
        }
    }
    pub fn trainer(&self, rom: &Rom) -> Result<Trainer> {
        let a = self.sections[0];
        let b = self.sections[1];
        let key = u32(&self.data, a + self.layout.key)?;
        Ok(Trainer {
            name: rom.codec.decode(&self.data[a..a + 7]),
            gender: self.data[a + 8],
            tid: u16(&self.data, a + 10)?,
            sid: u16(&self.data, a + 12)?,
            hours: u16(&self.data, a + 14)?,
            minutes: self.data[a + 16],
            seconds: self.data[a + 17],
            money: u32(&self.data, b + self.layout.money)? ^ key,
            coins: u16(&self.data, b + self.layout.coins)? ^ (key as u16),
            registered_item: u16(&self.data, b + 0x496)?,
        })
    }
    pub fn edit_trainer(&mut self, p: &TrainerPatch, rom: &Rom) -> Result<()> {
        let a = self.sections[0];
        let b = self.sections[1];
        let key = u32(&self.data, a + self.layout.key)?;
        if let Some(v) = &p.name {
            let encoded = rom.codec.encode(v, 7)?;
            self.data[a..a + 7].copy_from_slice(&encoded);
            self.data[a + 7] = 255;
        }
        if let Some(v) = p.gender {
            if v > 1 {
                return Err(err("range", "gender"));
            }
            self.data[a + 8] = v;
        }
        if let Some(v) = p.tid {
            put16(&mut self.data, a + 10, v);
        }
        if let Some(v) = p.sid {
            put16(&mut self.data, a + 12, v);
        }
        if let Some(v) = p.hours {
            put16(&mut self.data, a + 14, v);
        }
        for (off, v) in [(16, p.minutes), (17, p.seconds)] {
            if let Some(v) = v {
                if v > 59 {
                    return Err(err("range", "time"));
                }
                self.data[a + off] = v;
            }
        }
        if let Some(v) = p.money {
            if v > 999999 {
                return Err(err("range", "money"));
            }
            put32(&mut self.data, b + self.layout.money, v ^ key);
        }
        if let Some(v) = p.coins {
            if v > 9999 {
                return Err(err("range", "coins"));
            }
            put16(&mut self.data, b + self.layout.coins, v ^ (key as u16));
        }
        if let Some(v) = p.registered_item {
            rom.item(v)?;
            put16(&mut self.data, b + 0x496, v);
        }
        self.fix(0);
        self.fix(1);
        Ok(())
    }
    pub fn bag(&self) -> Result<Vec<BagEntry>> {
        let b = self.sections[1];
        let key = u32(&self.data, self.sections[0] + self.layout.key)? as u16;
        let mut out = Vec::new();
        for p in POCKETS {
            for i in 0..p.count {
                let o = b + p.offset + i * 4;
                let id = u16(&self.data, o)?;
                out.push(BagEntry {
                    pocket: p.id.into(),
                    slot: i,
                    item: id,
                    quantity: if id == 0 {
                        0
                    } else {
                        u16(&self.data, o + 2)? ^ if p.encrypted { key } else { 0 }
                    },
                });
            }
        }
        Ok(out)
    }
    pub fn edit_bag(
        &mut self,
        pocket: &str,
        index: usize,
        id: u16,
        quantity: u16,
        rom: &Rom,
        policy: Policy,
    ) -> Result<()> {
        let p = POCKETS
            .iter()
            .find(|p| p.id == pocket)
            .ok_or_else(|| err("pocket", pocket))?;
        if index >= p.count {
            return Err(err("location", index));
        }
        let item = rom.item(id)?;
        if id != 0 && quantity == 0 {
            return Err(err("quantity", "nonempty item requires quantity"));
        }
        if policy == Policy::Standard && id != 0 {
            if p.category != 0 && item.pocket != p.category {
                return Err(err("item_pocket", id));
            }
            let max = if p.id == "key_items" {
                1
            } else if matches!(p.id, "pc" | "tmhm" | "berries") {
                999
            } else {
                99
            };
            if quantity > max {
                return Err(err("quantity", max));
            }
        }
        let key = if p.encrypted {
            u32(&self.data, self.sections[0] + self.layout.key)? as u16
        } else {
            0
        };
        let o = self.sections[1] + p.offset + index * 4;
        put16(&mut self.data, o, id);
        put16(
            &mut self.data,
            o + 2,
            if id == 0 { key } else { quantity ^ key },
        );
        self.fix(1);
        Ok(())
    }
    pub fn boxes(&self, rom: &Rom) -> Result<Vec<BoxInfo>> {
        let b = self.logical(5..=13);
        let names = 4 + self.layout.boxes * self.layout.slots * 80;
        let mut out = Vec::new();
        for i in 0..self.layout.boxes {
            let count = (0..30)
                .filter(|j| {
                    u16(
                        &pokemon::unpack(&b[4 + (i * 30 + j) * 80..4 + (i * 30 + j + 1) * 80])
                            .unwrap(),
                        0,
                    )
                    .unwrap()
                        != 0
                })
                .count();
            out.push(BoxInfo {
                index: i,
                name: rom.codec.decode(&b[names + i * 9..names + (i + 1) * 9]),
                wallpaper: b[names + self.layout.boxes * 9 + i],
                count,
            });
        }
        Ok(out)
    }
    pub fn edit_box(&mut self, index: usize, name: &str, wallpaper: u8, rom: &Rom) -> Result<()> {
        if index >= self.layout.boxes || wallpaper > 15 {
            return Err(err("range", "box"));
        }
        let mut b = self.logical(5..=13);
        let names = 4 + self.layout.boxes * self.layout.slots * 80;
        b[names + index * 9..names + index * 9 + 8].copy_from_slice(&rom.codec.encode(name, 8)?);
        b[names + index * 9 + 8] = 0xff;
        b[names + self.layout.boxes * 9 + index] = wallpaper;
        self.write_logical(5..=13, &b)
    }
    pub fn sort_box(&mut self, index: usize, rom: &Rom) -> Result<()> {
        if index >= self.layout.boxes {
            return Err(err("location", index));
        }
        let mut rows = Vec::new();
        for i in 0..self.layout.slots {
            let loc = Location::Box {
                box_index: index,
                slot: i,
            };
            if let Some(p) = self.pokemon(loc, rom)? {
                rows.push((p.species, p.level, self.raw(loc)?));
            }
        }
        rows.sort_by_key(|r| (r.0, r.1));
        for i in 0..self.layout.slots {
            let raw = rows
                .get(i)
                .map(|r| r.2.clone())
                .unwrap_or_else(|| vec![0; 80]);
            self.write_raw(
                Location::Box {
                    box_index: index,
                    slot: i,
                },
                &raw,
            )?;
        }
        Ok(())
    }
    pub fn dex(&self) -> Result<Vec<DexFlag>> {
        let a = self.sections[0];
        Ok((1..=416)
            .map(|n| {
                let i = (n - 1) as usize;
                DexFlag {
                    number: n,
                    owned: self.data[a + 0x28 + i / 8] & (1 << (i % 8)) != 0,
                    seen: self.data[a + 0x5c + i / 8] & (1 << (i % 8)) != 0,
                }
            })
            .collect())
    }
    pub fn edit_dex(&mut self, n: u16, seen: bool, owned: bool) -> Result<()> {
        if !(1..=416).contains(&n) {
            return Err(err("range", "dex number"));
        }
        let i = (n - 1) as usize;
        let mask = 1u8 << (i % 8);
        let a = self.sections[0];
        let mut main = self.logical(1..=4);
        for (off, v) in [(0x28, owned), (0x5c, seen || owned)] {
            let b = &mut self.data[a + off + i / 8];
            *b = (*b & !mask) | if v { mask } else { 0 };
        }
        for off in [0x988, 0x3b24] {
            let b = &mut main[off + i / 8];
            *b = (*b & !mask) | if seen || owned { mask } else { 0 };
        }
        self.fix(0);
        self.write_logical(1..=4, &main)
    }
}
