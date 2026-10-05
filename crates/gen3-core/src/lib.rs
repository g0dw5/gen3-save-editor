//! ROM-backed Gen III editing. No UI, process-global ROM or bundled game assets.
pub mod acquisition;
pub mod adapter;
pub mod app;
pub mod binary;
pub mod breeding;
pub mod breeding_production;
pub mod cheats;
pub mod clock;
pub mod collection;
pub mod contest;
pub mod daycare_state;
pub mod event_state;
pub mod fishing;
pub mod graphics;
pub mod map_events;
pub mod native_trainer;
pub mod navigation;
pub mod pokemon;
pub mod profile;
pub mod rom;
pub mod save;
pub mod script_pokemon;
pub mod script_resources;
pub mod script_teaching;
pub mod session;
#[cfg(test)]
mod tests;
pub mod text;
pub mod world;

use serde::Serialize;

#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[error("{code}: {detail}")]
pub struct Error {
    pub code: &'static str,
    pub detail: String,
}
pub type Result<T> = std::result::Result<T, Error>;
pub fn err(code: &'static str, detail: impl ToString) -> Error {
    Error {
        code,
        detail: detail.to_string(),
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        err("io", e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        err("json", e)
    }
}

pub mod forms;
pub mod mercury;

pub mod relations;

pub mod ultimate;
mod ultimate_battle;
mod ultimate_ev;

pub mod wild_items;
