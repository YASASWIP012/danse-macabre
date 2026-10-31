//! Plays the cryptography of a whole game with the verifier the template runs.

use danse_client::seat::*;
use shroud::{
    group::Group,
    native::Native,
    network,
    sigma::verify,
    statements::{self as st, Cipher, CARD_VILLAGER, CARD_WOLF},
    Pt, IDENTITY,
};

const N: usize = 5;

fn table() -> (Vec<Seat>, Pt) {
    let seats: Vec<Seat> = (0..N as u32).map(|i| Seat::new(7, i)).collect();
    let mut joint = IDENTITY;
    for seat in &seats {
        joint = Native::add(&joint, &seat.key);
    }
    (seats, joint)
}

fn apply_shuffle(seat: &Seat, deck: &[Cipher], joint: &Pt, steps: &[ShuffleStep]) -> Option<Vec<Cipher>> {
    let mut wires = deck.to_vec();
    for (step, ((a, b), s)) in network::switches(deck.len()).into_iter().zip(steps).enumerate() {
        let ctx = st::ctx(b"shuffle", seat.game, step as u32, seat.index, &[]);
        let statement = st::switch_statement::<Native>(joint, &wires[a], &wires[b], &s.out_a, &s.out_b);
        if !verify::<Native>(&ctx, &statement, &s.proof) {
            return None;
        }
        wires[a] = s.out_a;
        wires[b] = s.out_b;
    }
    Some(wires)
}

fn deal(seats: &[Seat], joint: &Pt) -> (Vec<Cipher>, Vec<Cipher>) {
    let mut deck = starting_deck(N);
    for seat in seats {
        let steps = seat.shuffle(&deck, joint);
        deck = apply_shuffle(seat, &deck, joint, &steps).expect("honest shuffle verifies");
    }
    let residuals = (0..N)
        .map(|i| {
            let mut c2 = deck[i].c2;
            for seat in seats.iter().filter(|s| s.index as usize != i) {
                c2 = Native::sub(&c2, &Native::mul(&deck[i].c1, &seat.secret));
            }
            Cipher { c1: deck[i].c1, c2 }
        })
        .collect();
    (deck, residuals)
}

#[test]
fn network_routes_every_permutation() {
    fn permutations(items: Vec<usize>) -> Vec<Vec<usize>> {
        if items.len() <= 1 {
            return vec![items];
        }
        let mut out = vec![];
        for i in 0..items.len() {
            let mut rest = items.clone();
            let head = rest.remove(i);
            for mut tail in permutations(rest) {
                tail.insert(0, head);
                out.push(tail);
            }
        }
        out
    }
    for destinations in permutations((0..N).collect()) {
        let mut wires: Vec<usize> = (0..N).collect();
        for ((a, b), cross) in network::switches(N).into_iter().zip(network::route(&destinations)) {
            if cross {
                wires.swap(a, b);
            }
        }
        for (position, card) in wires.iter().enumerate() {
            assert_eq!(destinations[*card], position);
        }
    }
}

#[test]
fn exactly_one_wolf_is_dealt_and_each_seat_reads_its_own_card() {
    let (seats, joint) = table();
    let (_, residuals) = deal(&seats, &joint);
    let cards: Vec<u8> = seats.iter().zip(&residuals).map(|(s, r)| s.read_card(r).unwrap()).collect();
    assert_eq!(cards.iter().filter(|c| **c == CARD_WOLF).count(), 1);
    assert_eq!(cards.iter().filter(|c| **c == CARD_VILLAGER).count(), N - 1);
}

#[test]
fn a_seat_cannot_read_a_neighbours_card() {
    let (seats, joint) = table();
    let (_, residuals) = deal(&seats, &joint);
    assert_eq!(seats[1].read_card(&residuals[0]), None);
}

#[test]
fn a_shuffle_that_swaps_in_a_new_card_fails() {
    let (seats, joint) = table();
    let deck = starting_deck(N);
    let mut steps = seats[0].shuffle(&deck, &joint);
    steps[0].out_a.c2 = Native::add(&steps[0].out_a.c2, &shroud::GENERATOR);
    assert!(apply_shuffle(&seats[0], &deck, &joint, &steps).is_none());
}

#[test]
fn a_shuffle_replayed_by_another_seat_fails() {
    let (seats, joint) = table();
    let deck = starting_deck(N);
    let steps = seats[0].shuffle(&deck, &joint);
    assert!(apply_shuffle(&seats[1], &deck, &joint, &steps).is_none());
}

#[test]
fn the_wolfs_night_action_and_a_villagers_verify_the_same_way() {
    let (seats, joint) = table();
    let (_, residuals) = deal(&seats, &joint);
    let targets = [0, 1, 2, 3, 4];
    let mut lengths = vec![];
    for (seat, residual) in seats.iter().zip(&residuals) {
        let (votes, proof) = seat.night(1, &joint, residual, &targets, N, Some(2));
        let statement = st::night_statement::<Native>(&joint, &seat.key, residual, &votes, &targets);
        let ctx = st::ctx(b"night", seat.game, 1, seat.index, &[]);
        assert!(verify::<Native>(&ctx, &statement, &proof));
        lengths.push(proof.len());
    }
    assert!(lengths.windows(2).all(|w| w[0] == w[1]), "every seat sends the same size");
}

#[test]
fn a_villager_cannot_forge_a_kill() {
    let (seats, joint) = table();
    let (_, residuals) = deal(&seats, &joint);
    let (seat, residual) = seats
        .iter()
        .zip(&residuals)
        .find(|(s, r)| s.read_card(r) == Some(CARD_VILLAGER))
        .unwrap();
    // A real vote for seat 0 with a proof built for the empty branch.
    let (votes, _) = {
        let (a, r0) = encrypt(&shroud::GENERATOR, &joint);
        let rest: Vec<_> = (1..N).map(|_| encrypt(&IDENTITY, &joint)).collect();
        let mut v = vec![a];
        let mut w = vec![r0];
        for (c, r) in rest {
            v.push(c);
            w.push(r);
        }
        (v, w)
    };
    let targets = [0, 1, 2, 3, 4];
    let statement = st::night_statement::<Native>(&joint, &seat.key, residual, &votes, &targets);
    let ctx = st::ctx(b"night", seat.game, 1, seat.index, &[]);
    let forged = shroud::sigma::prove::<Native>(
        &ctx,
        &statement,
        0,
        &(0..N).map(|_| random_scalar()).collect::<Vec<_>>(),
        &mut random_scalar,
    );
    assert!(!verify::<Native>(&ctx, &statement, &forged));
}

#[test]
fn summed_votes_open_to_the_victim_and_only_the_total() {
    let (seats, joint) = table();
    let (_, residuals) = deal(&seats, &joint);
    let targets = [0, 1, 2, 3, 4];
    let posts: Vec<_> = seats
        .iter()
        .zip(&residuals)
        .map(|(s, r)| s.night(1, &joint, r, &targets, N, Some(3)).0)
        .collect();
    let mut totals: Vec<Cipher> = vec![Cipher { c1: IDENTITY, c2: IDENTITY }; N];
    for post in &posts {
        for (total, vote) in totals.iter_mut().zip(post) {
            *total = st::sum::<Native>(total, vote);
        }
    }
    let bases: Vec<Pt> = totals.iter().map(|t| t.c1).collect();
    let mut opened: Vec<Pt> = totals.iter().map(|t| t.c2).collect();
    for seat in &seats {
        let (shares, proof) = seat.shares(b"open", 1, &bases);
        let ctx = st::ctx(b"open", seat.game, 1, seat.index, &[]);
        assert!(verify::<Native>(&ctx, &st::shares_statement(&seat.key, &bases, &shares), &proof));
        for (o, s) in opened.iter_mut().zip(&shares) {
            *o = Native::sub(o, s);
        }
    }
    let counts: Vec<u32> = opened.iter().map(|p| st::small_count::<Native>(p, 5).unwrap()).collect();
    assert_eq!(counts, vec![0, 0, 0, 1, 0]);
}
