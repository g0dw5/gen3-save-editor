//! Encoding facts only; no glyph bitmaps or game text is distributed.
use crate::{err, Result};
use std::collections::HashMap;
#[derive(Clone)]
pub struct Codec {
    decode: HashMap<Vec<u8>, String>,
    encode: HashMap<char, Vec<u8>>,
}
impl Default for Codec {
    fn default() -> Self {
        Self::new()
    }
}
impl Codec {
    pub fn new() -> Self {
        let mut decode = HashMap::new();
        let mut encode = HashMap::new();
        for line in include_str!("../data/charmap.txt").lines() {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                if key.len() % 2 != 0 {
                    continue;
                }
                let raw: Option<Vec<u8>> = (0..key.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&key[i..i + 2], 16).ok())
                    .collect();
                if let Some(raw) = raw {
                    if raw.is_empty() {
                        continue;
                    }
                    let value = if key == "00" { " " } else { value };
                    decode.insert(raw.clone(), value.to_string());
                    if value.chars().count() == 1 {
                        encode.entry(value.chars().next().unwrap()).or_insert(raw);
                    }
                }
            }
        }
        decode.insert(vec![0x71], "\u{2009}".into());
        encode.insert('\u{2009}', vec![0x71]);
        Self { decode, encode }
    }
    pub fn decode(&self, b: &[u8]) -> String {
        let mut s = String::new();
        let mut i = 0;
        while i < b.len() {
            if b[i] == 0xff {
                break;
            }
            if matches!(b[i], 0xfe | 0xfa | 0xfb) {
                s.push('\n');
                i += 1;
                continue;
            }
            if i + 1 < b.len() {
                if let Some(v) = self.decode.get(&b[i..i + 2]) {
                    s.push_str(v);
                    i += 2;
                    continue;
                }
            }
            if let Some(v) = self.decode.get(&b[i..i + 1]) {
                s.push_str(v);
            } else {
                s.push_str(&format!("{{{:02X}}}", b[i]));
            }
            i += 1;
        }
        s.trim_end_matches(' ').to_string()
    }
    pub fn encode(&self, s: &str, capacity: usize) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        for c in s.chars() {
            out.extend(
                self.encode
                    .get(&c)
                    .ok_or_else(|| err("text_character", c))?,
            );
        }
        if out.len() > capacity {
            return Err(err(
                "text_length",
                format!("{} > {capacity} bytes", out.len()),
            ));
        }
        out.resize(capacity, 0xff);
        Ok(out)
    }
}
