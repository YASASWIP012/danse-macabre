//! The browser's entry points. Secrets stay in the tab; everything crosses as hex.

use shroud::{statements::Cipher, Pt, Sc};
use wasm_bindgen::prelude::*;

use crate::seat::{random_scalar, Seat};

fn pt(s: &str) -> Pt {
    hex::decode(s).expect("hex").try_into().expect("32 bytes")
}

fn sc(s: &str) -> Sc {
    pt(s)
}

fn seat(game: u64, index: u32, secret: &str) -> Seat {
    use shroud::{group::Group, native::Native};
    let secret = sc(secret);
    Seat { game, index, secret, key: Native::mul_base(&secret) }
}

fn ciphers(s: &str) -> Vec<Cipher> {
    hex::decode(s)
        .expect("hex")
        .chunks(64)
        .map(|c| Cipher { c1: c[..32].try_into().unwrap(), c2: c[32..].try_into().unwrap() })
        .collect()
}

fn encode(list: &[Cipher]) -> Vec<u8> {
    list.iter().flat_map(|c| [c.c1, c.c2].concat()).collect()
}

fn points(s: &str) -> Vec<Pt> {
    hex::decode(s).expect("hex").chunks(32).map(|p| p.try_into().unwrap()).collect()
}

fn indexes(s: &[u32]) -> Vec<usize> {
    s.iter().map(|i| *i as usize).collect()
}

#[wasm_bindgen]
pub fn new_secret() -> String {
    hex::encode(random_scalar())
}

#[wasm_bindgen]
pub fn public_key(secret: &str) -> String {
    use shroud::{group::Group, native::Native};
    hex::encode(Native::mul_base(&sc(secret)))
}

#[wasm_bindgen]
pub fn join_proof(game: u64, index: u32, secret: &str) -> String {
    hex::encode(seat(game, index, secret).join_proof())
}

#[wasm_bindgen]
pub fn fee_proof(game: u64, index: u32, secret: &str, nonce: u32, sealer: &str) -> String {
    hex::encode(seat(game, index, secret).fee_proof(nonce, &pt(sealer)))
}

#[wasm_bindgen]
pub fn ticket_proof(game: u64, secret: &str, nonce: u32, sealer: &str) -> String {
    hex::encode(seat(game, 0, secret).ticket_proof(nonce, &pt(sealer)))
}

#[wasm_bindgen]
pub fn payout_fee_proof(game: u64, index: u32, secret: &str, nonce: u32, sealer: &str) -> String {
    hex::encode(seat(game, index, secret).payout_fee_proof(nonce, &pt(sealer)))
}

#[wasm_bindgen]
pub fn invite_proof(game: u64, ticket: &str, index: u32, key: &str) -> String {
    hex::encode(seat(game, 0, ticket).invite_proof(index, &pt(key)))
}

#[wasm_bindgen]
pub fn ante_proof(game: u64, index: u32, secret: &str, payout_key: &str) -> String {
    hex::encode(seat(game, index, secret).ante_proof(&pt(payout_key)))
}

#[wasm_bindgen]
pub fn shuffle_proof(game: u64, index: u32, secret: &str, steps: &str) -> String {
    hex::encode(seat(game, index, secret).shuffle_proof(&hex::decode(steps).expect("hex")))
}

#[wasm_bindgen]
pub fn payout_proof(game: u64, index: u32, secret: &str, action: &str, sealer: &str) -> String {
    hex::encode(seat(game, index, secret).payout_proof(action.as_bytes(), &pt(sealer)))
}

/// Every switch output and proof, in network order.
#[wasm_bindgen]
pub fn shuffle(game: u64, index: u32, secret: &str, deck: &str, joint: &str) -> String {
    let steps = seat(game, index, secret).shuffle(&ciphers(deck), &pt(joint));
    let mut out = Vec::new();
    for s in steps {
        out.extend_from_slice(&s.out_a.c1);
        out.extend_from_slice(&s.out_a.c2);
        out.extend_from_slice(&s.out_b.c1);
        out.extend_from_slice(&s.out_b.c2);
        out.extend_from_slice(&s.proof);
    }
    hex::encode(out)
}

/// Returns `shares|proof`.
#[wasm_bindgen]
pub fn shares(game: u64, index: u32, secret: &str, action: &str, round: u32, bases: &str) -> String {
    let (shares, proof) = seat(game, index, secret).shares(action.as_bytes(), round, &points(bases));
    format!("{}|{}", hex::encode(shares.concat()), hex::encode(proof))
}

#[wasm_bindgen]
pub fn read_card(game: u64, index: u32, secret: &str, residual: &str) -> u8 {
    seat(game, index, secret).read_card(&ciphers(residual)[0]).unwrap_or(0)
}

/// Returns `votes|proof`. `pick` is ignored unless this seat holds the wolf. Use -1 for no pick.
#[wasm_bindgen]
pub fn night(game: u64, index: u32, secret: &str, round: u32, joint: &str, residual: &str, targets: &[u32], seats: u32, pick: i32) -> String {
    let pick = (pick >= 0).then_some(pick as usize);
    let (votes, proof) = seat(game, index, secret).night(round, &pt(joint), &ciphers(residual)[0], &indexes(targets), seats as usize, pick);
    format!("{}|{}", hex::encode(encode(&votes)), hex::encode(proof))
}

/// Returns `votes|proof`.
#[wasm_bindgen]
pub fn vote(game: u64, index: u32, secret: &str, round: u32, joint: &str, targets: &[u32], seats: u32, pick: u32) -> String {
    let (votes, proof) = seat(game, index, secret).vote(round, &pt(joint), &indexes(targets), seats as usize, pick as usize);
    format!("{}|{}", hex::encode(encode(&votes)), hex::encode(proof))
}
