//! Contest feeding bounds, not an inverse reconstruction of feeding history.
use crate::{binary::bytes, err, rom::Rom, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ContestRules {
    pub flavor_preferences: usize,
    pub npc_blender: Option<NpcBlender>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct NpcBlender {
    pub berries: usize,
    pub berry_count: usize,
    pub berry_stride: usize,
    pub flavors_offset: usize,
    pub opponents: usize,
    pub master: usize,
    /// A deliberately generous bound established from this engine's speed and
    /// progress transitions. This is not a claim that this RPM is attainable.
    pub rpm_upper_bound: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContestScope {
    Npc,
}

#[derive(Debug, Serialize)]
pub struct ContestCheck {
    /// A necessary-condition check can disprove a history, never certify one.
    pub status: &'static str,
    pub can_feed: bool,
    pub minimum_sheen_lower_bound: Option<u16>,
    pub weights: Option<[u16; 5]>,
}

pub fn preferences(rom: &Rom, nature: u8) -> Result<[i8; 5]> {
    if nature >= 25 {
        return Err(err("range", "nature"));
    }
    let rules = rom
        .profile
        .contest
        .ok_or_else(|| err("unsupported_feature", "contest_feeding"))?;
    let mut out = [0; 5];
    for (i, b) in bytes(&rom.data, rules.flavor_preferences + nature as usize * 5, 5)?
        .iter()
        .enumerate()
    {
        out[i] = *b as i8;
        if !(-1..=1).contains(&out[i]) {
            return Err(err("rom_structure", "contest flavor preference"));
        }
    }
    Ok(out)
}

/// Native feeding semantics, including the gain-dependent preference direction.
/// The block is a supplied input, not a claim that its recipe is obtainable.
pub fn feed(condition: [u8; 6], block: [u8; 6], preferences: [i8; 5]) -> Result<[u8; 6]> {
    if condition[5] == 255 {
        return Err(err("contest_full", "sheen is already 255"));
    }
    let gain: i32 = (0..5)
        .map(|i| block[i] as i32 * preferences[i] as i32)
        .sum();
    let mut after = condition;
    for i in 0..5 {
        let amount = block[i] as i32;
        let adjustment = if gain != 0 && preferences[i] as i32 == gain.signum() {
            (amount + 5) / 10 * preferences[i] as i32
        } else {
            0
        };
        after[i] = (condition[i] as i32 + amount + adjustment).clamp(0, 255) as u8;
    }
    after[5] = condition[5].saturating_add(block[5]);
    Ok(after)
}

/// All ordinary NPC recipes, including the Blender Master and the ROM's default
/// Enigma Berry. Event-replaced Enigma data and link blending are outside scope.
/// Gains are optimistic: always reward liked flavors, never penalize disliked
/// ones. Thus the bound remains valid for every RPM below the verified ceiling.
pub fn npc_upper_blocks(rom: &Rom, nature: u8) -> Result<Vec<[u16; 6]>> {
    let rules = rom
        .profile
        .contest
        .and_then(|r| r.npc_blender)
        .ok_or_else(|| err("unsupported_feature", "contest_npc"))?;
    let tastes = preferences(rom, nature)?;
    let berries = (0..rules.berry_count)
        .map(|i| {
            Ok(bytes(
                &rom.data,
                rules.berries + i * rules.berry_stride + rules.flavors_offset,
                6,
            )?
            .try_into()
            .unwrap())
        })
        .collect::<Result<Vec<[u8; 6]>>>()?;
    if berries.len() != 43 {
        return Err(err(
            "rom_structure",
            "NPC berry selector expects 43 berries",
        ));
    }
    let opponents = bytes(&rom.data, rules.opponents, 30)?;
    let masters = bytes(&rom.data, rules.master, 5)?;
    let mut blocks = Vec::new();
    for berry in 0..berries.len() {
        let set = if berry == 42 {
            // The native selector retains the first smallest flavor.
            (0..5).min_by_key(|i| berries[berry][*i]).unwrap() + 5
        } else if berry < 5 {
            berry
        } else {
            berry % 5 + 5
        };
        for players in 2..=4 {
            for master in [false, true] {
                if master && players != 2 {
                    continue;
                }
                let mut ingredients = vec![berry];
                if master {
                    let id = masters[set % 5] as usize;
                    ingredients.push(if (30..35).contains(&berry) {
                        id.checked_sub(5)
                            .ok_or_else(|| err("rom_structure", "master berry"))?
                    } else {
                        id
                    });
                } else {
                    ingredients.extend(
                        opponents[set * 3..set * 3 + players - 1]
                            .iter()
                            .map(|v| *v as usize),
                    );
                }
                let mut sums = [0u16; 6];
                for id in ingredients {
                    let fruit = berries
                        .get(id)
                        .ok_or_else(|| err("rom_structure", "NPC berry ID"))?;
                    for i in 0..6 {
                        sums[i] += fruit[i] as u16;
                    }
                }
                let mut raw = [0i32; 5];
                for i in 0..5 {
                    raw[i] = sums[i] as i32 - sums[(i + 1) % 5] as i32;
                }
                let negative = raw.iter().filter(|v| **v < 0).count() as i32;
                let mut block = [0u16; 6];
                let feel = (sums[5] / players as u16).saturating_sub(players as u16);
                if feel == 0 {
                    return Err(err("rom_structure", "zero-feel berry requires a new bound"));
                }
                block[5] = feel.min(255);
                for i in 0..5 {
                    let v = ((raw[i] - negative).max(0)
                        * (100 + rules.rpm_upper_bound as i32 / 333)
                        + 50)
                        / 100;
                    let v = v.min(255) as u16;
                    block[i] = v + if tastes[i] > 0 { (v + 5) / 10 } else { 0 };
                }
                // Include every black-block outcome as an optimistic envelope,
                // even if this particular recipe never produces a black block.
                let mut black = [2; 6];
                black[5] = block[5];
                blocks.extend([block, black]);
            }
        }
    }
    blocks.sort_unstable();
    blocks.dedup();
    Ok(blocks)
}

pub fn check_npc(rom: &Rom, nature: u8, condition: [u8; 6]) -> Result<ContestCheck> {
    let blocks = npc_upper_blocks(rom, nature)?;
    let mut result = ContestCheck {
        status: "not_disproved",
        can_feed: condition[5] < 255,
        minimum_sheen_lower_bound: Some(0),
        weights: None,
    };
    if condition[..5].iter().all(|v| *v == 0) {
        return Ok(result);
    }
    // Small integer separating weights. Passing is deliberately inconclusive;
    // any separating vector is a reproducible certificate of impossibility.
    let mut lower_bound = 0u16;
    for code in 0..8u32.pow(5) {
        let mut n = code;
        let mut weights = [0u16; 5];
        for w in &mut weights {
            *w = (n % 8 + 1) as u16;
            n /= 8;
        }
        let target: u32 = (0..5)
            .map(|i| weights[i] as u32 * condition[i] as u32)
            .sum();
        let mut rate = (0u32, 1u32);
        let mut last = 0u32;
        for b in &blocks {
            let gain: u32 = (0..5).map(|i| weights[i] as u32 * b[i] as u32).sum();
            if gain * rate.1 > rate.0 * b[5] as u32 {
                rate = (gain, b[5] as u32);
            }
            last = last.max(gain);
        }
        if rate.0 == 0 {
            continue;
        }
        let required = (target * rate.1).div_ceil(rate.0).min(255) as u16;
        lower_bound = lower_bound.max(required);
        // At 255 the last block is allowed to cross the cap; prior feel <=254.
        let possible = if condition[5] == 255 {
            target * rate.1 <= 254 * rate.0 + last * rate.1
        } else {
            target * rate.1 <= condition[5] as u32 * rate.0
        };
        if !possible {
            result.status = "outside_npc_bound";
            result.weights = Some(weights);
            // No misleading numeric minimum if no final sheen can work.
            result.minimum_sheen_lower_bound = if target * rate.1 > 254 * rate.0 + last * rate.1 {
                None
            } else {
                Some(lower_bound)
            };
            return Ok(result);
        }
    }
    result.minimum_sheen_lower_bound = Some(lower_bound);
    Ok(result)
}
