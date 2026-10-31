//! Shroud: the proofs behind a game with no dealer.
//!
//! Everything here runs in two places. The prover runs in the player's browser. The verifier runs
//! inside the Ootle template, on the engine's Ristretto and hashing intrinsics. Both share this one
//! implementation through the [`Group`] trait, so there is no second copy of the rules to drift.
#![no_std]
extern crate alloc;

pub mod group;
pub mod sigma;
pub mod statements;
pub mod network;

#[cfg(feature = "native")]
pub mod native;

pub use group::{Group, Pt, Sc, GENERATOR, IDENTITY};
pub use sigma::{prove, verify, Branch, Clause, Eq};
