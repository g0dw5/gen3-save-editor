//! Binary formats and verified capabilities are independent of table addresses.
use crate::{binary::bytes, err, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PokemonCodec {
    Gen3,
    Rocket21,
}

/// A field owns only these bits, never the rest of its containing word.
#[derive(Clone, Copy, Debug)]
pub struct Field {
    byte: usize,
    shift: u8,
    width: u8,
}
impl Field {
    pub const fn new(byte: usize, shift: u8, width: u8) -> Self {
        assert!(width > 0 && width <= 32 && shift < 32);
        Self { byte, shift, width }
    }
    pub fn max(self) -> u32 {
        (u64::MAX >> (64 - self.width)) as u32
    }
    pub fn read(self, data: &[u8]) -> Result<u32> {
        let len = (self.shift as usize + self.width as usize).div_ceil(8);
        let mut word = [0u8; 8];
        word[..len].copy_from_slice(bytes(data, self.byte, len)?);
        Ok(((u64::from_le_bytes(word) >> self.shift) & self.max() as u64) as u32)
    }
    pub fn write(self, data: &mut [u8], value: u32) -> Result<()> {
        if value > self.max() {
            return Err(err("field_range", value));
        }
        let len = (self.shift as usize + self.width as usize).div_ceil(8);
        let mut word = [0u8; 8];
        word[..len].copy_from_slice(bytes(data, self.byte, len)?);
        let mask = (self.max() as u64) << self.shift;
        let value = (u64::from_le_bytes(word) & !mask) | ((value as u64) << self.shift);
        data[self.byte..self.byte + len].copy_from_slice(&value.to_le_bytes()[..len]);
        Ok(())
    }
}

pub struct PokemonFields {
    pub experience: Field,
    pub pp_ups: Field,
    pub friendship: Field,
    pub ball: Field,
    pub ability: Field,
    pub nature_override: Option<Field>,
    pub ribbons: Option<Field>,
    pub default_ball: u8,
    pub language: Field,
    pub bad_egg: Field,
    pub has_species: Field,
    pub header_egg: Field,
    pub markings: Field,
    pub ot_name: usize,
}
impl PokemonCodec {
    pub fn fields(self) -> PokemonFields {
        match self {
            Self::Gen3 => PokemonFields {
                experience: Field::new(4, 0, 32),
                pp_ups: Field::new(8, 0, 8),
                friendship: Field::new(9, 0, 8),
                ball: Field::new(38, 11, 4),
                ability: Field::new(40, 31, 1),
                nature_override: None,
                ribbons: Some(Field::new(44, 0, 32)),
                default_ball: 4,
                language: Field::new(18, 0, 8),
                bad_egg: Field::new(19, 0, 1),
                has_species: Field::new(19, 1, 1),
                header_egg: Field::new(19, 2, 1),
                markings: Field::new(27, 0, 4),
                ot_name: 20,
            },
            Self::Rocket21 => PokemonFields {
                experience: Field::new(4, 0, 23),
                pp_ups: Field::new(7, 0, 8),
                friendship: Field::new(8, 0, 8),
                ball: Field::new(9, 0, 5),
                ability: Field::new(47, 0, 2),
                nature_override: Some(Field::new(9, 5, 5)),
                ribbons: None,
                default_ball: 1,
                language: Field::new(18, 0, 3),
                bad_egg: Field::new(18, 3, 1),
                has_species: Field::new(18, 4, 1),
                header_egg: Field::new(18, 5, 1),
                markings: Field::new(26, 0, 4),
                ot_name: 19,
            },
        }
    }
}

/// Table formats are independently selectable: future hybrids need not inherit
/// all of another engine's layouts to reuse one reader.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpeciesFormat {
    Gen3,
    Expanded36,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MoveFormat {
    Gen3,
    Expanded20,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearnsetFormat {
    Packed9Bit,
    MoveLevel16,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct RomFormats {
    pub species: SpeciesFormat,
    pub moves: MoveFormat,
    pub learnsets: LearnsetFormat,
}
impl RomFormats {
    pub const GEN3: Self = Self {
        species: SpeciesFormat::Gen3,
        moves: MoveFormat::Gen3,
        learnsets: LearnsetFormat::Packed9Bit,
    };
    pub const ROCKET21: Self = Self {
        species: SpeciesFormat::Expanded36,
        moves: MoveFormat::Expanded20,
        learnsets: LearnsetFormat::MoveLevel16,
    };
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Capabilities {
    pub save_edit: bool,
    pub rom_edit: bool,
    pub world: bool,
    pub dex: bool,
    pub complete_learnsets: bool,
    pub individual_sprites: bool,
    pub battle_forms: bool,
}
impl Capabilities {
    pub const DARK_PHANTOM: Self = Self {
        save_edit: true,
        rom_edit: true,
        world: true,
        dex: true,
        complete_learnsets: true,
        individual_sprites: true,
        battle_forms: false,
    };
    pub const ROCKET_READ_ONLY: Self = Self {
        save_edit: false,
        rom_edit: false,
        world: false,
        dex: false,
        complete_learnsets: false,
        individual_sprites: false,
        battle_forms: true,
    };
    pub fn require(self, available: bool, feature: &str) -> Result<()> {
        if available {
            Ok(())
        } else {
            Err(err("unsupported_feature", feature))
        }
    }
}
