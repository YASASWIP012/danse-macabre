//! The statements of the game, built the same way by prover and verifier.

use alloc::{vec, vec::Vec};

use crate::{
    group::{small, Group, Pt, Sc, GENERATOR, IDENTITY},
    sigma::{Branch, Clause, Eq},
};

/// An ElGamal ciphertext: `(r*G, M + r*Y)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cipher {
    pub c1: Pt,
    pub c2: Pt,
}

pub const CARD_VILLAGER: u8 = 5;
pub const CARD_WOLF: u8 = 7;

pub fn card_point<G: Group>(card: u8) -> Pt {
    G::mul_base(&small(card))
}

/// Context bound into every proof: action, game, round, seat, and any extra binding.
pub fn ctx(action: &[u8], game: u64, round: u32, seat: u32, extra: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(action.len() + 18 + extra.len());
    out.push(action.len() as u8);
    out.extend_from_slice(action);
    out.extend_from_slice(&game.to_le_bytes());
    out.extend_from_slice(&round.to_le_bytes());
    out.extend_from_slice(&seat.to_le_bytes());
    out.extend_from_slice(extra);
    out
}

/// `cipher` encrypts `plain` under `joint` with witness `r`.
fn encrypts<G: Group>(cipher: &Cipher, plain: &Pt, joint: &Pt) -> Clause {
    vec![
        Eq { base: GENERATOR, target: cipher.c1 },
        Eq { base: *joint, target: G::sub(&cipher.c2, plain) },
    ]
}

fn diff<G: Group>(a: &Cipher, b: &Cipher) -> Cipher {
    Cipher { c1: G::sub(&a.c1, &b.c1), c2: G::sub(&a.c2, &b.c2) }
}

/// Add two ciphertexts, which adds their plaintexts.
pub fn sum<G: Group>(a: &Cipher, b: &Cipher) -> Cipher {
    Cipher { c1: G::add(&a.c1, &b.c1), c2: G::add(&a.c2, &b.c2) }
}

/// Knowledge of the secret key for `key`, bound to `ctx`. Used on joining, which stops a rogue
/// joint key, and to authorise fee payments.
pub fn key_statement(key: &Pt) -> Vec<Branch> {
    vec![vec![vec![Eq { base: GENERATOR, target: *key }]]]
}

/// A 2x2 switch: the outputs re-encrypt the inputs, either straight or crossed.
pub fn switch_statement<G: Group>(
    joint: &Pt,
    a: &Cipher,
    b: &Cipher,
    out_a: &Cipher,
    out_b: &Cipher,
) -> Vec<Branch> {
    let zero = IDENTITY;
    vec![
        vec![
            encrypts::<G>(&diff::<G>(out_a, a), &zero, joint),
            encrypts::<G>(&diff::<G>(out_b, b), &zero, joint),
        ],
        vec![
            encrypts::<G>(&diff::<G>(out_a, b), &zero, joint),
            encrypts::<G>(&diff::<G>(out_b, a), &zero, joint),
        ],
    ]
}

/// Decryption shares for many ciphertexts by one seat: each share is `x * base` for the seat's key.
pub fn shares_statement(key: &Pt, bases: &[Pt], shares: &[Pt]) -> Vec<Branch> {
    assert_eq!(bases.len(), shares.len());
    let mut clause = vec![Eq { base: GENERATOR, target: *key }];
    for (base, share) in bases.iter().zip(shares) {
        clause.push(Eq { base: *base, target: *share });
    }
    vec![vec![clause]]
}

/// One clause list where `votes[pick]` encrypts one and the rest encrypt zero.
fn one_hot<G: Group>(joint: &Pt, votes: &[Cipher], pick: Option<usize>) -> Branch {
    let mut branch = Vec::with_capacity(votes.len());
    for (index, vote) in votes.iter().enumerate() {
        let plain = if Some(index) == pick { GENERATOR } else { IDENTITY };
        branch.push(encrypts::<G>(vote, &plain, joint));
    }
    branch
}

/// The clause that says the prover holds this seat's key.
fn holds_key(seat_key: &Pt) -> Clause {
    vec![Eq { base: GENERATOR, target: *seat_key }]
}

/// A day vote by the holder of `seat_key`: exactly one of the `targets` gets one vote.
pub fn vote_statement<G: Group>(joint: &Pt, seat_key: &Pt, votes: &[Cipher], targets: &[usize]) -> Vec<Branch> {
    targets
        .iter()
        .map(|t| {
            let mut branch = vec![holds_key(seat_key)];
            branch.extend(one_hot::<G>(joint, votes, Some(*t)));
            branch
        })
        .collect()
}

/// A night action by the holder of `seat_key`. Branch 0: every vote encrypts zero. Branch `1 + k`: the seat's card is the wolf,
/// shown by decrypting its private residual card under its own key, and exactly `targets[k]` gets
/// one vote. Villagers can only ever take branch 0, and nobody can tell which branch was taken.
pub fn night_statement<G: Group>(
    joint: &Pt,
    seat_key: &Pt,
    residual: &Cipher,
    votes: &[Cipher],
    targets: &[usize],
) -> Vec<Branch> {
    let wolf = card_point::<G>(CARD_WOLF);
    let is_wolf: Clause = vec![
        Eq { base: GENERATOR, target: *seat_key },
        Eq { base: residual.c1, target: G::sub(&residual.c2, &wolf) },
    ];
    let mut empty = vec![holds_key(seat_key)];
    empty.extend(one_hot::<G>(joint, votes, None));
    let mut branches = vec![empty];
    for target in targets {
        let mut branch = vec![is_wolf.clone()];
        branch.extend(one_hot::<G>(joint, votes, Some(*target)));
        branches.push(branch);
    }
    branches
}

/// The plaintext count behind `point`, if it is `m * G` for a small `m`.
pub fn small_count<G: Group>(point: &Pt, max: u32) -> Option<u32> {
    let mut candidate = IDENTITY;
    for m in 0..=max {
        if candidate == *point {
            return Some(m);
        }
        candidate = G::add(&candidate, &GENERATOR);
    }
    None
}

/// The zero scalar.
pub const ZERO: Sc = [0u8; 32];
