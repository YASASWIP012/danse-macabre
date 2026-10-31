//! The player's side of the game: everything secret happens here.

pub mod seat;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
