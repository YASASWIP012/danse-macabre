//! # Danse Macabre
//!
//! Werewolf with no game master. Five players shuffle one wolf card into the deck together, so
//! nobody, including whoever opened the table, knows the deal. Each night every living seat posts
//! one sealed action of the same shape. The wolf's names a victim, the others are proved empty, and
//! only the sum is decrypted: the village learns who died and never who did it.
//!
//! The template holds no secret and trusts no seat. Every step is checked here with the same
//! verifier the players' browsers use (see the `shroud` crate). Seats act from throwaway keys and
//! the table's fee pot pays, so no wallet is tied to a seat. See THREAT_MODEL.md for what stays
//! visible.
#![allow(clippy::too_many_arguments)]

use shroud::{
    group::{Group, Pt, Sc},
    network,
    sigma::{proof_len, verify},
    statements::{self as st, Cipher, CARD_WOLF},
    GENERATOR,
    IDENTITY,
};
use std::collections::BTreeMap;

use tari_template_lib::{prelude::*, types::bytes::Bytes};

pub const SEATS: usize = 5;
/// The most one transaction may draw from the fee pot, in µT.
pub const MAX_TX_FEE: u64 = 5_000_000;
/// The most one game may draw from the fee pot, in µT.
pub const MAX_GAME_FEES: u64 = 200_000_000;
/// Tables that may open per epoch, which bounds what strangers can draw from the pot.
pub const MAX_OPENS_PER_EPOCH: u32 = 12;
/// A table may be closed this many epoch boundaries after its last move. An epoch is about 48
/// minutes, so that is somewhere between 48 and 96 minutes.
pub const STALL_EPOCHS: u64 = 2;
/// The stake, in µT. Every staked seat stakes the same.
pub const ANTE: u64 = 1_000_000;
/// Finished tables kept in state.
pub const KEEP_FINISHED: u64 = 8;

const SWITCH_LEN: usize = 128 + 192;

pub const LOBBY: u8 = 0;
pub const SHUFFLE: u8 = 1;
pub const DEAL: u8 = 2;
pub const NIGHT: u8 = 3;
pub const NIGHT_OPEN: u8 = 4;
pub const REVEAL: u8 = 5;
pub const DAY: u8 = 6;
pub const DAY_OPEN: u8 = 7;
pub const OVER: u8 = 8;

pub const WINNER_NONE: u8 = 0;
pub const WINNER_VILLAGE: u8 = 1;
pub const WINNER_WOLF: u8 = 2;
pub const WINNER_STALLED: u8 = 3;

/// The engine's intrinsics, behind Shroud's group interface.
pub struct Engine;

fn pk(p: &Pt) -> RistrettoPublicKeyBytes {
    RistrettoPublicKeyBytes::from(*p)
}

fn sc(s: &Sc) -> Scalar32Bytes {
    Scalar32Bytes::try_from(s.as_slice()).expect("scalar length")
}

fn pt_out(p: RistrettoPublicKeyBytes) -> Pt {
    let mut out = [0u8; 32];
    out.copy_from_slice(p.as_ref());
    out
}

fn sc_out(s: Scalar32Bytes) -> Sc {
    let mut out = [0u8; 32];
    out.copy_from_slice(s.as_ref());
    out
}

impl Group for Engine {
    fn msm(points: &[Pt], scalars: &[Sc]) -> Pt {
        let p: Vec<_> = points.iter().map(pk).collect();
        let s: Vec<_> = scalars.iter().map(sc).collect();
        pt_out(intrinsics::ristretto_msm(&p, &s))
    }

    fn add(a: &Pt, b: &Pt) -> Pt {
        pt_out(intrinsics::ristretto_add(&pk(a), &pk(b)))
    }

    fn sub(a: &Pt, b: &Pt) -> Pt {
        pt_out(intrinsics::ristretto_sub(&pk(a), &pk(b)))
    }

    fn sc_add(a: &Sc, b: &Sc) -> Sc {
        sc_out(intrinsics::scalar_add(&sc(a), &sc(b)))
    }

    fn sc_sub(a: &Sc, b: &Sc) -> Sc {
        sc_out(intrinsics::scalar_sub(&sc(a), &sc(b)))
    }

    fn sc_mul(a: &Sc, b: &Sc) -> Sc {
        sc_out(intrinsics::scalar_mul(&sc(a), &sc(b)))
    }

    fn hash_scalar(data: &[u8]) -> Sc {
        let wide = intrinsics::hash_sha512(data);
        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(wide.as_ref());
        sc_out(intrinsics::scalar_from_uniform_bytes(&bytes))
    }

    fn point_ok(p: &Pt) -> bool {
        intrinsics::ristretto_is_canonical(&pk(p))
    }

    fn scalar_ok(s: &Sc) -> bool {
        intrinsics::scalar_is_canonical(&sc(s))
    }
}

fn point_at(bytes: &[u8], index: usize) -> Pt {
    let mut p = [0u8; 32];
    p.copy_from_slice(&bytes[index * 32..(index + 1) * 32]);
    assert!(Engine::point_ok(&p), "Point is not canonical");
    p
}

fn cipher_at(bytes: &[u8], index: usize) -> Cipher {
    Cipher { c1: point_at(bytes, index * 2), c2: point_at(bytes, index * 2 + 1) }
}

fn set_point(bytes: &mut [u8], index: usize, p: &Pt) {
    bytes[index * 32..(index + 1) * 32].copy_from_slice(p);
}

fn set_cipher(bytes: &mut [u8], index: usize, c: &Cipher) {
    set_point(bytes, index * 2, &c.c1);
    set_point(bytes, index * 2 + 1, &c.c2);
}

fn ciphers(bytes: &[u8]) -> Vec<Cipher> {
    (0..bytes.len() / 64).map(|i| cipher_at(bytes, i)).collect()
}

#[template]
mod danse_macabre {
    use super::*;

    pub struct DanseMacabre {
        tables: BTreeMap<u64, Table>,
        next_table: u64,
        pot: Vault,
        /// Stakes wait here until the game ends.
        stakes: Vault,
        open_epoch: u64,
        opens_this_epoch: u32,
    }

    impl DanseMacabre {
        pub fn new(pot: Bucket) -> Component<Self> {
            assert!(pot.resource_address() == TARI_TOKEN, "The fee pot holds TARI");
            Component::new(Self {
                tables: BTreeMap::new(),
                next_table: 1,
                pot: Vault::from_bucket(pot),
                stakes: Vault::new_empty(TARI_TOKEN),
                open_epoch: 0,
                opens_this_epoch: 0,
            })
            .with_access_rules(AccessRules::allow_all())
            .create()
        }

        /// Anyone may top up the pot.
        pub fn add_to_pot(&mut self, tari: Bucket) {
            self.pot.deposit(tari);
        }

        /// Open a table. `ticket` is a key held by whoever has the invite link, and it pays for
        /// joining. `fee` is drawn from the pot, up to a cap.
        pub fn open_table(&mut self, ticket: Bytes, fee: u64) -> u64 {
            let now = Consensus::current_epoch();
            self.count_open();
            self.draw(fee);

            assert!(ticket.len() == 32, "Bad ticket");
            assert!(point_at(&ticket, 0) != IDENTITY, "Bad ticket");
            let id = self.next_table;
            self.next_table += 1;

            // A finished table with money still in it stays until the money has gone.
            self.tables.retain(|old, t| t.phase != OVER || *old + KEEP_FINISHED >= id || !all_paid(t));
            self.tables.insert(id, Table {
                seats: SEATS as u8,
                phase: LOBBY,
                round: 0,
                acted: 0,
                alive: (1 << SEATS) - 1,
                turn: 0,
                after_reveal: NIGHT,
                winner: WINNER_NONE,
                wolf: u8::MAX,
                last_victim: u8::MAX,
                keys: Bytes::from_vec(vec![0u8; SEATS * 32]),
                joined: 0,
                joint: Bytes::from_vec(IDENTITY.to_vec()),
                deck: Bytes::from_vec(Vec::new()),
                residual: Bytes::from_vec(Vec::new()),
                tally: Bytes::from_vec(Vec::new()),
                opening: Bytes::from_vec(Vec::new()),
                revealed: Bytes::from_vec(vec![0u8; SEATS * 32]),
                ticket,
                staked: 0,
                payouts: Bytes::from_vec(vec![0u8; SEATS * 32]),
                collected: 0,
                fee_nonce: 0,
                fees_used: 0,
                stall_at: now + STALL_EPOCHS,
            });
            emit_event("Danse.Opened", metadata! { "table" => id.to_string() });
            id
        }

        /// Fee stage: pay for this transaction from the pot. A living seat at the table may ask.
        /// The proof binds a fresh nonce and this transaction's seal key, so it can't be lifted out
        /// of the mempool. No account and no wallet appear anywhere in the transaction.
        pub fn pay_seat_fee(&mut self, table: u64, seat: u8, proof: Bytes, amount: u64) {
            let t = self.tables.get(&table).expect("No such table");
            assert!((seat as usize) < SEATS && t.joined & (1 << seat) != 0, "Empty seat");
            assert!(t.phase != OVER || !all_paid(t), "This table is finished");
            assert!(t.phase == OVER || t.alive & (1 << seat) != 0 || t.turn == seat, "The dead pay no fees");
            let key = point_at(&t.keys, seat as usize);
            let sealer = CallerContext::transaction_signer_public_key();
            let ctx = st::ctx(b"fee", table, t.fee_nonce, seat as u32, sealer.as_ref());
            assert!(verify::<Engine>(&ctx, &st::key_statement(&key), &proof), "Bad fee proof");
            self.charge(table, amount);
        }

        pub fn pay_ticket_fee(&mut self, table: u64, proof: Bytes, amount: u64) {
            let t = self.tables.get(&table).expect("No such table");
            assert!(t.phase == LOBBY, "Tickets only work in the lobby");
            let key = point_at(&t.ticket, 0);
            let sealer = CallerContext::transaction_signer_public_key();
            let ctx = st::ctx(b"ticket", table, t.fee_nonce, 0, sealer.as_ref());
            assert!(verify::<Engine>(&ctx, &st::key_statement(&key), &proof), "Bad ticket proof");
            self.charge(table, amount);
        }

        /// Fee stage, for opening a table. Capped per call, and limited per epoch here because the
        /// fee stage commits even if the rest of the transaction fails.
        pub fn pay_open_fee(&mut self, amount: u64) {
            self.count_open();
            self.draw(amount);
        }

        /// Takes a seat. `invite` proves the ticket holder let this key sit here, and `proof` that
        /// the joiner holds the key, which stops a rogue joint key.
        pub fn join(&mut self, table: u64, seat: u8, key: Bytes, proof: Bytes, invite: Bytes) {
            let t = self.table(table);
            assert!(t.phase == LOBBY, "This table is full");
            let i = seat as usize;
            assert!(i < SEATS && t.joined & (1 << seat) == 0, "Seat is taken");
            assert!(key.len() == 32, "Bad key");
            let y = point_at(&key, 0);
            assert!(y != IDENTITY, "Bad key");
            let ctx = st::ctx(b"join", table, 0, seat as u32, &[]);
            assert!(verify::<Engine>(&ctx, &st::key_statement(&y), &proof), "Bad key proof");
            let ctx = st::ctx(b"invite", table, 0, seat as u32, &y);
            let ticket = point_at(&t.ticket, 0);
            assert!(verify::<Engine>(&ctx, &st::key_statement(&ticket), &invite), "Bad invite");

            set_point(&mut t.keys, i, &y);
            t.joined |= 1 << seat;
            let joint = Engine::add(&point_at(&t.joint, 0), &y);
            t.joint = Bytes::from_vec(joint.to_vec());

            if t.joined == (1 << SEATS) - 1 {
                t.phase = SHUFFLE;
                let mut deck = vec![0u8; SEATS * 64];
                for card in 0..SEATS {
                    let plain = st::card_point::<Engine>(if card == 0 { CARD_WOLF } else { st::CARD_VILLAGER });
                    set_cipher(&mut deck, card, &Cipher { c1: IDENTITY, c2: plain });
                }
                t.deck = Bytes::from_vec(deck);
                touch(t);
            }
            emit_event("Danse.Joined", metadata! { "table" => table.to_string(), "seat" => seat.to_string() });
        }

        /// One seat's shuffle: the output of every switch with its proof, in network order.
        pub fn shuffle(&mut self, table: u64, seat: u8, steps: Bytes, proof: Bytes) {
            let t = self.table(table);
            assert!(t.phase == SHUFFLE && t.turn == seat, "Not your shuffle");
            let key = point_at(&t.keys, seat as usize);
            let ctx = st::ctx(b"shuffle-by", table, 0, seat as u32, &Engine::hash_scalar(&steps));
            assert!(verify::<Engine>(&ctx, &st::key_statement(&key), &proof), "Only the seat can shuffle");
            let joint = point_at(&t.joint, 0);
            let switches = network::switches(SEATS);
            assert!(steps.len() == switches.len() * SWITCH_LEN, "Bad shuffle length");

            let mut deck = t.deck.to_vec();
            for (index, (a, b)) in switches.into_iter().enumerate() {
                let step = &steps[index * SWITCH_LEN..(index + 1) * SWITCH_LEN];
                let out_a = cipher_at(step, 0);
                let out_b = cipher_at(step, 1);
                let statement = st::switch_statement::<Engine>(
                    &joint,
                    &cipher_at(&deck, a),
                    &cipher_at(&deck, b),
                    &out_a,
                    &out_b,
                );
                let ctx = st::ctx(b"shuffle", table, index as u32, seat as u32, &[]);
                assert!(verify::<Engine>(&ctx, &statement, &step[128..]), "Bad shuffle proof");
                set_cipher(&mut deck, a, &out_a);
                set_cipher(&mut deck, b, &out_b);
            }
            t.deck = Bytes::from_vec(deck);
            t.turn += 1;
            if t.turn as usize == SEATS {
                t.phase = DEAL;
                t.residual = t.deck.clone();
                t.acted = 0;
            }
            touch(t);
            emit_event("Danse.Shuffled", metadata! { "table" => table.to_string(), "seat" => seat.to_string() });
        }

        /// A seat strips its layer from every card but its own, so each card ends up readable only
        /// by the seat it was dealt to.
        pub fn deal(&mut self, table: u64, seat: u8, shares: Bytes, proof: Bytes) {
            let t = self.table(table);
            assert!(t.phase == DEAL && t.acted & (1 << seat) == 0, "Not dealing");
            assert!((seat as usize) < SEATS && shares.len() == (SEATS - 1) * 32, "Bad shares");
            let others: Vec<usize> = (0..SEATS).filter(|i| *i != seat as usize).collect();
            let bases: Vec<Pt> = others.iter().map(|i| cipher_at(&t.deck, *i).c1).collect();
            let parts: Vec<Pt> = (0..others.len()).map(|i| point_at(&shares, i)).collect();
            let key = point_at(&t.keys, seat as usize);
            let ctx = st::ctx(b"deal", table, 0, seat as u32, &[]);
            assert!(
                verify::<Engine>(&ctx, &st::shares_statement(&key, &bases, &parts), &proof),
                "Bad deal proof"
            );
            let mut residual = t.residual.to_vec();
            for (i, share) in others.iter().zip(&parts) {
                let c = cipher_at(&residual, *i);
                set_cipher(&mut residual, *i, &Cipher { c1: c.c1, c2: Engine::sub(&c.c2, share) });
            }
            t.residual = Bytes::from_vec(residual);
            t.acted |= 1 << seat;
            if t.acted == (1 << SEATS) - 1 {
                t.round = 1;
                begin(t, NIGHT);
            }
            touch(t);
        }

        /// A sealed night action. Every living seat posts exactly one, and they are the same size
        /// whether or not they mean anything.
        pub fn night(&mut self, table: u64, seat: u8, votes: Bytes, proof: Bytes) {
            let t = self.table(table);
            assert!(t.phase == NIGHT, "It is not night");
            assert_can_act(t, seat);
            let posted = read_votes(&votes);
            let joint = point_at(&t.joint, 0);
            let key = point_at(&t.keys, seat as usize);
            let residual = cipher_at(&t.residual, seat as usize);
            let targets = living_except(t, seat);
            let statement = st::night_statement::<Engine>(&joint, &key, &residual, &posted, &targets);
            assert!(proof.len() == proof_len(&statement), "Bad proof length");
            let ctx = st::ctx(b"night", table, t.round, seat as u32, &[]);
            assert!(verify::<Engine>(&ctx, &statement, &proof), "Bad night proof");
            add_votes(t, &posted);
            t.acted |= 1 << seat;
            if t.acted == t.alive {
                begin(t, NIGHT_OPEN);
            }
            touch(t);
            emit_event("Danse.NightAction", metadata! { "table" => table.to_string() });
        }

        /// A sealed day vote: exactly one living seat gets this seat's vote.
        pub fn vote(&mut self, table: u64, seat: u8, votes: Bytes, proof: Bytes) {
            let t = self.table(table);
            assert!(t.phase == DAY, "It is not day");
            assert_can_act(t, seat);
            let posted = read_votes(&votes);
            let joint = point_at(&t.joint, 0);
            let key = point_at(&t.keys, seat as usize);
            let statement = st::vote_statement::<Engine>(&joint, &key, &posted, &living_except(t, seat));
            let ctx = st::ctx(b"vote", table, t.round, seat as u32, &[]);
            assert!(verify::<Engine>(&ctx, &statement, &proof), "Bad vote proof");
            add_votes(t, &posted);
            t.acted |= 1 << seat;
            if t.acted == t.alive {
                begin(t, DAY_OPEN);
            }
            touch(t);
            emit_event("Danse.Voted", metadata! { "table" => table.to_string() });
        }

        /// Living seats post decryption shares for the totals. Only the totals ever open.
        pub fn open_tally(&mut self, table: u64, seat: u8, shares: Bytes, proof: Bytes) {
            let t = self.table(table);
            assert!(t.phase == NIGHT_OPEN || t.phase == DAY_OPEN, "Nothing to open");
            assert_can_act(t, seat);
            assert!(shares.len() == SEATS * 32, "Bad shares");
            let bases: Vec<Pt> = ciphers(&t.tally).iter().map(|c| c.c1).collect();
            let parts: Vec<Pt> = (0..SEATS).map(|i| point_at(&shares, i)).collect();
            let key = point_at(&t.keys, seat as usize);
            let ctx = st::ctx(b"open", table, t.round + phase_offset(t.phase), seat as u32, &[]);
            assert!(
                verify::<Engine>(&ctx, &st::shares_statement(&key, &bases, &parts), &proof),
                "Bad open proof"
            );
            let mut opening = t.opening.to_vec();
            for (i, share) in parts.iter().enumerate() {
                let left = Engine::sub(&point_at(&opening, i), share);
                set_point(&mut opening, i, &left);
            }
            t.opening = Bytes::from_vec(opening);
            t.acted |= 1 << seat;
            if t.acted == t.alive {
                finish_open(t, table);
            }
            touch(t);
        }

        /// A dead seat publishes its secret, which turns its card face up and lets the template
        /// stand in for it in every later opening.
        pub fn reveal(&mut self, table: u64, seat: u8, secret: Bytes) {
            let t = self.table(table);
            assert!(t.phase == REVEAL && t.turn == seat, "Not your reveal");
            assert!(secret.len() == 32, "Bad secret");
            let mut x = [0u8; 32];
            x.copy_from_slice(&secret);
            assert!(Engine::scalar_ok(&x), "Bad secret");
            let i = seat as usize;
            assert!(Engine::mul_base(&x) == point_at(&t.keys, i), "Wrong secret");

            let card = cipher_at(&t.residual, i);
            let plain = Engine::sub(&card.c2, &Engine::mul(&card.c1, &x));
            let was_wolf = plain == st::card_point::<Engine>(CARD_WOLF);
            let mut revealed = t.revealed.to_vec();
            revealed[i * 32..(i + 1) * 32].copy_from_slice(&x);
            t.revealed = Bytes::from_vec(revealed);
            t.alive &= !(1 << seat);
            t.last_victim = seat;
            if was_wolf {
                t.wolf = seat;
            }
            emit_event("Danse.Revealed", metadata! {
                "table" => table.to_string(),
                "seat" => seat.to_string(),
                "wolf" => was_wolf.to_string(),
            });

            if was_wolf {
                finish(t, table, WINNER_VILLAGE);
            } else if t.alive.count_ones() <= 2 {
                finish(t, table, WINNER_WOLF);
            } else {
                let next = t.after_reveal;
                if next == NIGHT {
                    t.round += 1;
                }
                begin(t, next);
            }
            touch(t);
        }

        /// Stake on this seat before the shuffle ends. Pay it from a stealth coin, so the stake
        /// names no wallet. `payout_key` claims the money later, because the seat's own key goes
        /// public if the seat dies.
        pub fn ante(&mut self, table: u64, seat: u8, payout_key: Bytes, proof: Bytes, stake: Bucket) {
            let t = self.table(table);
            assert!(t.phase == LOBBY || t.phase == SHUFFLE, "Stakes have closed");
            assert!((seat as usize) < SEATS && t.joined & (1 << seat) != 0, "Empty seat");
            assert!(t.staked & (1 << seat) == 0, "Already staked");
            assert!(stake.amount() == Amount::from(ANTE), "Wrong stake");
            assert!(payout_key.len() == 32, "Bad payout key");
            let payout = point_at(&payout_key, 0);
            assert!(payout != IDENTITY, "Bad payout key");
            let key = point_at(&t.keys, seat as usize);
            let ctx = st::ctx(b"ante", table, 0, seat as u32, &payout);
            assert!(verify::<Engine>(&ctx, &st::key_statement(&key), &proof), "Only the seat can stake");
            t.staked |= 1 << seat;
            let mut payouts = t.payouts.to_vec();
            set_point(&mut payouts, seat as usize, &payout);
            t.payouts = Bytes::from_vec(payouts);
            self.stakes.deposit(stake);
            emit_event("Danse.Staked", metadata! { "table" => table.to_string(), "seat" => seat.to_string() });
        }

        /// After a wolf win the surviving wolf turns their card over, which names the winner. It
        /// checks the secret against the seat key and the card, so nobody else can claim it.
        pub fn unmask(&mut self, table: u64, seat: u8, secret: Bytes) {
            let t = self.table(table);
            assert!(t.phase == OVER && t.winner == WINNER_WOLF && t.wolf == u8::MAX, "Nothing to unmask");
            assert!((seat as usize) < SEATS && secret.len() == 32, "Bad secret");
            let mut x = [0u8; 32];
            x.copy_from_slice(&secret);
            assert!(Engine::scalar_ok(&x), "Bad secret");
            assert!(Engine::mul_base(&x) == point_at(&t.keys, seat as usize), "Wrong secret");
            let card = cipher_at(&t.residual, seat as usize);
            let plain = Engine::sub(&card.c2, &Engine::mul(&card.c1, &x));
            assert!(plain == st::card_point::<Engine>(CARD_WOLF), "That seat is no wolf");
            t.wolf = seat;
            emit_event("Danse.Unmasked", metadata! { "table" => table.to_string(), "seat" => seat.to_string() });
        }

        /// A winner takes their share as a bucket, and the same transaction turns it into a fresh
        /// stealth output. The proof binds this transaction's seal key, so a copy of it is useless.
        pub fn collect(&mut self, table: u64, seat: u8, proof: Bytes) -> Bucket {
            let t = self.table(table);
            assert!(t.phase == OVER && (seat as usize) < SEATS, "The game isn't over");
            assert!(t.winner == WINNER_VILLAGE || t.winner == WINNER_WOLF, "Nobody won");
            assert!(t.wolf != u8::MAX, "The wolf has not unmasked");
            assert!(t.collected & (1 << seat) == 0, "Already collected");
            assert!(t.staked & (1 << seat) != 0, "No stake on this seat");
            assert!(seat_won(t, seat as usize), "This seat lost");
            check_payout(t, table, seat, b"collect", &proof);
            let winners = staked_winners(t);
            let share = ANTE * t.staked.count_ones() as u64 / winners;
            t.collected |= 1 << seat;
            emit_event("Danse.Collected", metadata! { "table" => table.to_string() });
            self.stakes.withdraw(Amount::from(share))
        }

        /// Gives a stake back when the table stalled, or when no winning seat had staked.
        pub fn refund(&mut self, table: u64, seat: u8, proof: Bytes) -> Bucket {
            let t = self.table(table);
            assert!(t.phase == OVER && (seat as usize) < SEATS, "The game isn't over");
            assert!(t.collected & (1 << seat) == 0, "Already collected");
            assert!(t.staked & (1 << seat) != 0, "No stake on this seat");
            let void = t.winner == WINNER_STALLED || (t.wolf != u8::MAX && staked_winners(t) == 0);
            assert!(void, "This table has a winner");
            check_payout(t, table, seat, b"refund", &proof);
            t.collected |= 1 << seat;
            emit_event("Danse.Refunded", metadata! { "table" => table.to_string() });
            self.stakes.withdraw(Amount::from(ANTE))
        }

        /// Anyone may close a table that has stalled for two epochs.
        pub fn close_stalled(&mut self, table: u64) {
            let t = self.table(table);
            let silent_wolf = t.phase == OVER && t.winner == WINNER_WOLF && t.wolf == u8::MAX;
            assert!(t.phase != OVER || silent_wolf, "Already over");
            assert!(Consensus::current_epoch() >= t.stall_at, "Not stalled");
            finish(t, table, WINNER_STALLED);
        }

        /// Phase, round, living seats, turn, winner, wolf, then the joint key, deck, residual cards
        /// and tally.
        pub fn table_view(&self, table: u64) -> (u8, u32, u32, u8, u8, u8, Bytes, Bytes, Bytes, Bytes) {
            let t = self.tables.get(&table).expect("No such table");
            (
                t.phase,
                t.round,
                t.alive,
                t.turn,
                t.winner,
                t.wolf,
                t.joint.clone(),
                t.deck.clone(),
                t.residual.clone(),
                t.tally.clone(),
            )
        }

        pub fn pot_balance(&self) -> Amount {
            self.pot.balance()
        }

        fn table(&mut self, table: u64) -> &mut Table {
            self.tables.get_mut(&table).expect("No such table")
        }

        fn charge(&mut self, table: u64, amount: u64) {
            assert!(amount <= MAX_TX_FEE, "Fee above the cap");
            let t = self.tables.get_mut(&table).expect("No such table");
            assert!(t.fees_used + amount <= MAX_GAME_FEES, "This table has spent its fees");
            t.fees_used += amount;
            t.fee_nonce += 1;
            self.pot.pay_fee(Amount::from(amount));
        }

        fn count_open(&mut self) {
            let now = Consensus::current_epoch();
            if now != self.open_epoch {
                self.open_epoch = now;
                self.opens_this_epoch = 0;
            }
            assert!(self.opens_this_epoch < MAX_OPENS_PER_EPOCH, "Too many tables this epoch");
            self.opens_this_epoch += 1;
        }

        fn draw(&mut self, amount: u64) {
            assert!(amount <= MAX_TX_FEE, "Fee above the cap");
            if amount > 0 {
                self.pot.pay_fee(Amount::from(amount));
            }
        }
    }

    pub struct Table {
        pub seats: u8,
        pub phase: u8,
        /// Night number, from 1.
        pub round: u32,
        /// Seats that have acted in this phase, as a bitmask.
        pub acted: u32,
        pub alive: u32,
        /// Seat whose turn it is to shuffle or reveal.
        pub turn: u8,
        /// Phase to enter after a reveal.
        pub after_reveal: u8,
        pub winner: u8,
        pub wolf: u8,
        pub last_victim: u8,
        pub keys: Bytes,
        pub joined: u32,
        pub joint: Bytes,
        /// The shuffled deck, as `seats` ciphertexts.
        pub deck: Bytes,
        /// Each card with every other seat's share removed, so only its owner can read it.
        pub residual: Bytes,
        /// Summed sealed votes per target.
        pub tally: Bytes,
        /// Tally second halves with shares removed so far.
        pub opening: Bytes,
        /// Secrets of the dead, published on reveal. Zero until then.
        pub revealed: Bytes,
        pub ticket: Bytes,
        /// Seats that staked, as a bitmask, and the key each named to claim with.
        pub staked: u32,
        pub payouts: Bytes,
        pub collected: u32,
        pub fee_nonce: u32,
        pub fees_used: u64,
        pub stall_at: u64,
    }

    fn seat_won(t: &Table, seat: usize) -> bool {
        (seat as u8 == t.wolf) == (t.winner == WINNER_WOLF)
    }

    fn staked_winners(t: &Table) -> u64 {
        (0..SEATS).filter(|i| seat_won(t, *i) && t.staked & (1 << i) != 0).count() as u64
    }

    /// True once every staked seat has had its money.
    fn all_paid(t: &Table) -> bool {
        if t.staked == 0 {
            return true;
        }
        let owed = if staked_winners(t) == 0 || t.winner == WINNER_STALLED {
            t.staked
        } else {
            (0..SEATS as u32).filter(|i| seat_won(t, *i as usize)).fold(0, |m, i| m | (1 << i)) & t.staked
        };
        t.collected & owed == owed
    }

    fn check_payout(t: &Table, table: u64, seat: u8, action: &[u8], proof: &Bytes) {
        let key = point_at(&t.payouts, seat as usize);
        let sealer = CallerContext::transaction_signer_public_key();
        let ctx = st::ctx(action, table, 0, seat as u32, sealer.as_ref());
        assert!(verify::<Engine>(&ctx, &st::key_statement(&key), proof), "Bad payout proof");
    }

    fn touch(t: &mut Table) {
        t.stall_at = Consensus::current_epoch() + STALL_EPOCHS;
    }

    fn phase_offset(phase: u8) -> u32 {
        if phase == DAY_OPEN { 1 << 16 } else { 0 }
    }

    fn assert_can_act(t: &Table, seat: u8) {
        assert!((seat as usize) < SEATS, "No such seat");
        assert!(t.alive & (1 << seat) != 0, "The dead do not act");
        assert!(t.acted & (1 << seat) == 0, "You already acted");
    }

    fn living_except(t: &Table, seat: u8) -> Vec<usize> {
        (0..SEATS).filter(|i| *i != seat as usize && t.alive & (1 << i) != 0).collect()
    }

    fn read_votes(votes: &Bytes) -> Vec<Cipher> {
        assert!(votes.len() == SEATS * 64, "Bad votes");
        ciphers(votes)
    }

    fn add_votes(t: &mut Table, posted: &[Cipher]) {
        let mut tally = t.tally.to_vec();
        for (i, vote) in posted.iter().enumerate() {
            let total = st::sum::<Engine>(&cipher_at(&tally, i), vote);
            set_cipher(&mut tally, i, &total);
        }
        t.tally = Bytes::from_vec(tally);
    }

    /// Enter a phase with a clean slate.
    fn begin(t: &mut Table, phase: u8) {
        t.phase = phase;
        t.acted = 0;
        if phase == NIGHT || phase == DAY {
            let mut empty = vec![0u8; SEATS * 64];
            for i in 0..SEATS {
                set_cipher(&mut empty, i, &Cipher { c1: IDENTITY, c2: IDENTITY });
            }
            t.tally = Bytes::from_vec(empty);
        }
        if phase == NIGHT_OPEN || phase == DAY_OPEN {
            let opening: Vec<u8> = ciphers(&t.tally).iter().flat_map(|c| c.c2).collect();
            t.opening = Bytes::from_vec(opening);
        }
    }

    fn finish(t: &mut Table, table: u64, winner: u8) {
        t.phase = OVER;
        t.winner = winner;
        touch(t);
        emit_event("Danse.Over", metadata! { "table" => table.to_string(), "winner" => winner.to_string() });
    }

    /// Stands in for the dead, reads the totals and moves the game on.
    fn finish_open(t: &mut Table, table: u64) {
        let was_night = t.phase == NIGHT_OPEN;
        let mut opening = t.opening.to_vec();
        for seat in 0..SEATS {
            if t.alive & (1 << seat) == 0 {
                let mut x = [0u8; 32];
                x.copy_from_slice(&t.revealed[seat * 32..(seat + 1) * 32]);
                for i in 0..SEATS {
                    let share = Engine::mul(&cipher_at(&t.tally, i).c1, &x);
                    let left = Engine::sub(&point_at(&opening, i), &share);
                    set_point(&mut opening, i, &left);
                }
            }
        }
        let counts: Vec<u32> = (0..SEATS)
            .map(|i| st::small_count::<Engine>(&point_at(&opening, i), SEATS as u32).expect("Tally out of range"))
            .collect();
        t.opening = Bytes::from_vec(opening);

        let top = *counts.iter().max().unwrap_or(&0);
        let leaders: Vec<usize> = (0..SEATS).filter(|i| counts[*i] == top).collect();
        let chosen = if top > 0 && leaders.len() == 1 { Some(leaders[0]) } else { None };
        emit_event("Danse.Opened.Tally", metadata! {
            "table" => table.to_string(),
            "chosen" => chosen.map(|c| c as i32).unwrap_or(-1).to_string(),
        });
        t.after_reveal = if was_night { DAY } else { NIGHT };
        match chosen {
            Some(seat) => {
                t.turn = seat as u8;
                begin(t, REVEAL);
            },
            None => {
                if !was_night {
                    t.round += 1;
                }
                begin(t, if was_night { DAY } else { NIGHT });
            },
        }
    }
}

const _: () = {
    assert!(GENERATOR[0] == 0xe2);
};
