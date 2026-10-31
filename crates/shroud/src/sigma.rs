//! One proof system for every statement in the game: an OR of ANDs of discrete-log equalities.
//!
//! An [`Eq`] says `target = w * base`. A [`Clause`] is a list of them sharing one witness `w`. A
//! [`Branch`] is a list of clauses that must all hold, and a statement is a list of branches of
//! which at least one holds. The verifier learns that one does, and nothing about which.
//!
//! The Fiat-Shamir hash covers the context, the shape, every base and target, and every commitment,
//! so a proof can't be replayed into another game, night, seat or statement.

use alloc::vec::Vec;

use crate::group::{Group, Pt, Sc};

const DOMAIN: &[u8] = b"shroud/sigma/v1";

#[derive(Clone, Debug)]
pub struct Eq {
    pub base: Pt,
    pub target: Pt,
}

pub type Clause = Vec<Eq>;
pub type Branch = Vec<Clause>;

fn clause_count(branches: &[Branch]) -> usize {
    branches.iter().map(|b| b.len()).sum()
}

/// The proof length in bytes for a statement of this shape.
pub fn proof_len(branches: &[Branch]) -> usize {
    (branches.len() + clause_count(branches)) * 32
}

fn absorb_statement(t: &mut Vec<u8>, ctx: &[u8], branches: &[Branch]) {
    t.extend_from_slice(DOMAIN);
    t.extend_from_slice(&(ctx.len() as u32).to_le_bytes());
    t.extend_from_slice(ctx);
    t.extend_from_slice(&(branches.len() as u32).to_le_bytes());
    for branch in branches {
        t.extend_from_slice(&(branch.len() as u32).to_le_bytes());
        for clause in branch {
            t.extend_from_slice(&(clause.len() as u32).to_le_bytes());
            for eq in clause {
                t.extend_from_slice(&eq.base);
                t.extend_from_slice(&eq.target);
            }
        }
    }
}

fn scalar_at(bytes: &[u8], index: usize) -> Sc {
    let mut s = [0u8; 32];
    s.copy_from_slice(&bytes[index * 32..(index + 1) * 32]);
    s
}

/// Checks `proof` against the statement. Malformed input returns false.
pub fn verify<G: Group>(ctx: &[u8], branches: &[Branch], proof: &[u8]) -> bool {
    if branches.is_empty() || proof.len() != proof_len(branches) {
        return false;
    }
    let mut transcript = Vec::new();
    absorb_statement(&mut transcript, ctx, branches);

    let mut sum = [0u8; 32];
    let mut next = branches.len();
    for (index, branch) in branches.iter().enumerate() {
        let c = scalar_at(proof, index);
        if !G::scalar_ok(&c) {
            return false;
        }
        sum = G::sc_add(&sum, &c);
        let minus_c = G::sc_sub(&[0u8; 32], &c);
        for clause in branch {
            let s = scalar_at(proof, next);
            next += 1;
            if !G::scalar_ok(&s) {
                return false;
            }
            for eq in clause {
                // T = s * base - c * target
                let t = G::msm(&[eq.base, eq.target], &[s, minus_c]);
                transcript.extend_from_slice(&t);
            }
        }
    }
    G::hash_scalar(&transcript) == sum
}

/// Proves the statement, given the index of a branch that holds and one witness per clause of it.
/// `rand` must return uniform scalars. This never runs in the template.
pub fn prove<G: Group>(
    ctx: &[u8],
    branches: &[Branch],
    real: usize,
    witnesses: &[Sc],
    rand: &mut dyn FnMut() -> Sc,
) -> Vec<u8> {
    assert_eq!(branches[real].len(), witnesses.len(), "one witness per clause");

    let mut challenges = Vec::with_capacity(branches.len());
    let mut responses: Vec<Vec<Sc>> = Vec::with_capacity(branches.len());
    let mut nonces = Vec::new();
    let mut transcript = Vec::new();
    absorb_statement(&mut transcript, ctx, branches);

    for (index, branch) in branches.iter().enumerate() {
        if index == real {
            challenges.push([0u8; 32]);
            responses.push(Vec::new());
            for clause in branch {
                let k = rand();
                nonces.push(k);
                for eq in clause {
                    transcript.extend_from_slice(&G::mul(&eq.base, &k));
                }
            }
        } else {
            let c = rand();
            let minus_c = G::sc_sub(&[0u8; 32], &c);
            let mut list = Vec::with_capacity(branch.len());
            for clause in branch {
                let s = rand();
                for eq in clause {
                    let t = G::msm(&[eq.base, eq.target], &[s, minus_c]);
                    transcript.extend_from_slice(&t);
                }
                list.push(s);
            }
            challenges.push(c);
            responses.push(list);
        }
    }

    let mut real_c = G::hash_scalar(&transcript);
    for (index, c) in challenges.iter().enumerate() {
        if index != real {
            real_c = G::sc_sub(&real_c, c);
        }
    }
    challenges[real] = real_c;
    responses[real] = nonces
        .iter()
        .zip(witnesses)
        .map(|(k, w)| G::sc_add(k, &G::sc_mul(&real_c, w)))
        .collect();

    let mut out = Vec::with_capacity(proof_len(branches));
    for c in &challenges {
        out.extend_from_slice(c);
    }
    for list in &responses {
        for s in list {
            out.extend_from_slice(s);
        }
    }
    out
}
