//! Whole games on the real Ootle engine, in-process.

use danse_client::seat::{encrypt, random_scalar, Seat};
use danse_macabre::{DAY, DAY_OPEN, DEAL, NIGHT, NIGHT_OPEN, OVER, REVEAL, SHUFFLE, WINNER_VILLAGE, WINNER_WOLF};
use shroud::{
    group::Group,
    native::Native,
    statements::{self as st, Cipher, CARD_WOLF},
    Pt, IDENTITY,
};
use tari_template_lib::types::{bytes::Bytes, Amount, ComponentAddress};
use tari_template_test_tooling::{crypto::RistrettoSecretKey, transaction::{args, Transaction}, TemplateTest};

const N: usize = 5;

struct World {
    test: TemplateTest,
    game: ComponentAddress,
    account: ComponentAddress,
    key: RistrettoSecretKey,
    table: u64,
    seats: Vec<Seat>,
    nonce: u32,
}

struct View {
    phase: u8,
    round: u32,
    alive: u32,
    turn: u8,
    winner: u8,
    wolf: u8,
    joint: Bytes,
    deck: Bytes,
    residual: Bytes,
    tally: Bytes,
}

fn b(v: Vec<u8>) -> Bytes {
    Bytes::from_vec(v)
}

fn world() -> World {
    let mut test = TemplateTest::new(env!("CARGO_MANIFEST_DIR"), [".".to_string()].iter().map(|s| s.as_str()));
    test.enable_auto_add_proofs_from_signers();
    let (account, _proof, key) = test.create_funded_account();
    let template = test.get_template_address("DanseMacabre");
    let tx = test
        .transaction()
        .call_method(account, "withdraw", args![tari_template_lib::prelude::TARI_TOKEN, Amount::from(100_000_000u64)])
        .put_last_instruction_output_on_workspace("pot")
        .call_function(template, "new", args![Workspace("pot")])
        .finish()
        .seal(&key);
    let result = test.execute_expect_success(tx, vec![]);
    let game = result.finalize.result.accept().unwrap().up_iter()
        .find_map(|(id, _)| id.as_component_address().filter(|a| *a != account))
        .expect("component created");
    let ticket = Native::mul_base(&random_scalar());
    let table: u64 = test.call_method(game, "open_table", args![b(ticket.to_vec()), 0u64], vec![]);
    let seats = (0..N as u32).map(|i| Seat::new(table, i)).collect();
    World { test, game, account, key, table, seats, nonce: 0 }
}

impl World {
    fn call(&mut self, method: &str, a: Vec<tari_template_test_tooling::transaction::builder::named_args::NamedArg>) -> Transaction {
        self.test.transaction().call_method(self.game, method, a).finish().seal(&self.key)
    }
    fn ok(&mut self, method: &str, a: Vec<tari_template_test_tooling::transaction::builder::named_args::NamedArg>) {
        let tx = self.call(method, a);
        self.test.execute_expect_success(tx, vec![]);
    }
    fn refused(&mut self, method: &str, a: Vec<tari_template_test_tooling::transaction::builder::named_args::NamedArg>, reason: &str) {
        let tx = self.call(method, a);
        let why = format!("{:?}", self.test.execute_expect_failure(tx, vec![]));
        assert!(why.contains(reason), "expected {reason}, got {why}");
    }
    fn state(&mut self) -> View {
        let t = self.table;
        let v: (u8, u32, u32, u8, u8, u8, Bytes, Bytes, Bytes, Bytes) =
            self.test.call_method(self.game, "table_view", args![t], vec![]);
        View { phase: v.0, round: v.1, alive: v.2, turn: v.3, winner: v.4, wolf: v.5, joint: v.6, deck: v.7, residual: v.8, tally: v.9 }
    }
    fn joint(&mut self) -> Pt { self.state().joint.to_vec().try_into().unwrap() }

    fn join_all(&mut self) {
        for s in self.seats.clone() {
            let t = self.table;
            self.ok("join", args![t, s.index as u8, b(s.key.to_vec()), b(s.join_proof())]);
        }
    }
    fn shuffle_all(&mut self) {
        let joint = self.joint();
        for s in self.seats.clone() {
            let t = self.table;
            let deck = ciphers(&self.state().deck);
            self.ok("shuffle", args![t, s.index as u8, b(encode_steps(&s.shuffle(&deck, &joint)))]);
        }
    }
    fn deal_all(&mut self) {
        let deck = ciphers(&self.state().deck);
        for s in self.seats.clone() {
            let bases: Vec<Pt> = (0..N).filter(|i| *i != s.index as usize).map(|i| deck[i].c1).collect();
            let (shares, proof) = s.shares(b"deal", 0, &bases);
            let t = self.table;
            self.ok("deal", args![t, s.index as u8, b(shares.concat()), b(proof)]);
        }
    }
    fn wolf(&mut self) -> usize {
        let residual = ciphers(&self.state().residual);
        self.seats.iter().position(|s| s.read_card(&residual[s.index as usize]) == Some(CARD_WOLF)).unwrap()
    }
    fn night(&mut self, pick: usize) {
        let state = self.state();
        let joint = self.joint();
        let residual = ciphers(&state.residual);
        for s in self.seats.clone() {
            if state.alive & (1 << s.index) == 0 { continue; }
            let targets: Vec<usize> = (0..N).filter(|i| *i != s.index as usize && state.alive & (1 << i) != 0).collect();
            let (votes, proof) = s.night(state.round, &joint, &residual[s.index as usize], &targets, N, Some(pick));
            let t = self.table;
            self.ok("night", args![t, s.index as u8, b(encode_ciphers(&votes)), b(proof)]);
        }
    }
    fn vote(&mut self, pick: usize) {
        let state = self.state();
        let joint = self.joint();
        for s in self.seats.clone() {
            if state.alive & (1 << s.index) == 0 { continue; }
            let targets: Vec<usize> = (0..N).filter(|i| *i != s.index as usize && state.alive & (1 << i) != 0).collect();
            let choice = if s.index as usize == pick { targets[0] } else { pick };
            let (votes, proof) = s.vote(state.round, &joint, &targets, N, choice);
            let t = self.table;
            self.ok("vote", args![t, s.index as u8, b(encode_ciphers(&votes)), b(proof)]);
        }
    }
    fn open(&mut self) {
        let state = self.state();
        let bases: Vec<Pt> = ciphers(&state.tally).iter().map(|c| c.c1).collect();
        let round = state.round + if state.phase == DAY_OPEN { 1 << 16 } else { 0 };
        for s in self.seats.clone() {
            if state.alive & (1 << s.index) == 0 { continue; }
            let (shares, proof) = s.shares(b"open", round, &bases);
            let t = self.table;
            self.ok("open_tally", args![t, s.index as u8, b(shares.concat()), b(proof)]);
        }
    }
    fn reveal(&mut self) {
        let state = self.state();
        let s = self.seats[state.turn as usize].clone();
        let t = self.table;
        self.ok("reveal", args![t, s.index as u8, b(s.secret.to_vec())]);
    }
    fn ready(&mut self) -> usize {
        self.join_all(); self.shuffle_all(); self.deal_all();
        self.wolf()
    }
}

fn ciphers(bytes: &[u8]) -> Vec<Cipher> {
    bytes.chunks(64).map(|c| Cipher { c1: c[..32].try_into().unwrap(), c2: c[32..].try_into().unwrap() }).collect()
}
fn encode_ciphers(list: &[Cipher]) -> Vec<u8> {
    list.iter().flat_map(|c| [c.c1, c.c2].concat()).collect()
}
fn encode_steps(steps: &[danse_client::seat::ShuffleStep]) -> Vec<u8> {
    steps.iter().flat_map(|s| [s.out_a.c1.to_vec(), s.out_a.c2.to_vec(), s.out_b.c1.to_vec(), s.out_b.c2.to_vec(), s.proof.clone()].concat()).collect()
}

#[test]
fn a_full_game_the_village_wins() {
    let mut w = world();
    let wolf = w.ready();
    assert_eq!(w.state().phase, NIGHT);
    let victim = (wolf + 1) % N;
    w.night(victim);
    assert_eq!(w.state().phase, NIGHT_OPEN);
    w.open();
    assert_eq!(w.state().phase, REVEAL);
    assert_eq!(w.state().turn as usize, victim);
    w.reveal();
    assert_eq!(w.state().phase, DAY);
    w.vote(wolf);
    w.open();
    w.reveal();
    let s = w.state();
    assert_eq!((s.phase, s.winner, s.wolf as usize), (OVER, WINNER_VILLAGE, wolf));
}

#[test]
fn a_full_game_the_wolf_wins() {
    let mut w = world();
    let wolf = w.ready();
    let villagers: Vec<usize> = (0..N).filter(|i| *i != wolf).collect();
    w.night(villagers[0]);
    w.open();
    w.reveal();
    w.vote(villagers[1]);
    w.open();
    w.reveal();
    assert_eq!(w.state().phase, NIGHT);
    assert_eq!(w.state().round, 2);
    w.night(villagers[2]);
    w.open();
    w.reveal();
    let s = w.state();
    assert_eq!((s.phase, s.winner), (OVER, WINNER_WOLF));
}

#[test]
fn a_rogue_key_without_proof_of_possession_is_refused() {
    let mut w = world();
    let t = w.table;
    let s = w.seats[0].clone();
    let other = w.seats[1].join_proof();
    w.refused("join", args![t, 0u8, b(s.key.to_vec()), b(other)], "Bad key proof");
}

#[test]
fn shuffling_out_of_turn_is_refused() {
    let mut w = world();
    w.join_all();
    assert_eq!(w.state().phase, SHUFFLE);
    let joint = w.joint();
    let deck = ciphers(&w.state().deck);
    let s = w.seats[1].clone();
    let t = w.table;
    w.refused("shuffle", args![t, 1u8, b(encode_steps(&s.shuffle(&deck, &joint)))], "Not your shuffle");
}

#[test]
fn a_shuffle_that_adds_a_second_wolf_is_refused() {
    let mut w = world();
    w.join_all();
    let joint = w.joint();
    let deck = ciphers(&w.state().deck);
    let s = w.seats[0].clone();
    let mut steps = s.shuffle(&deck, &joint);
    steps[3].out_b.c2 = Native::add(&steps[3].out_b.c2, &shroud::GENERATOR);
    let t = w.table;
    w.refused("shuffle", args![t, 0u8, b(encode_steps(&steps))], "Bad shuffle proof");
}

#[test]
fn a_seat_cannot_act_twice_in_one_night() {
    let mut w = world();
    let wolf = w.ready();
    let state = w.state();
    let joint = w.joint();
    let residual = ciphers(&state.residual);
    let s = w.seats[(wolf + 1) % N].clone();
    let targets: Vec<usize> = (0..N).filter(|i| *i != s.index as usize).collect();
    let (votes, proof) = s.night(1, &joint, &residual[s.index as usize], &targets, N, None);
    let t = w.table;
    w.ok("night", args![t, s.index as u8, b(encode_ciphers(&votes)), b(proof.clone())]);
    w.refused("night", args![t, s.index as u8, b(encode_ciphers(&votes)), b(proof)], "You already acted");
}

#[test]
fn a_night_proof_cannot_be_replayed_by_another_seat() {
    let mut w = world();
    let wolf = w.ready();
    let state = w.state();
    let joint = w.joint();
    let residual = ciphers(&state.residual);
    let s = w.seats[(wolf + 1) % N].clone();
    let thief = (wolf + 2) % N;
    let targets: Vec<usize> = (0..N).filter(|i| *i != s.index as usize).collect();
    let (votes, proof) = s.night(1, &joint, &residual[s.index as usize], &targets, N, None);
    let t = w.table;
    w.refused("night", args![t, thief as u8, b(encode_ciphers(&votes)), b(proof)], "Bad night proof");
}

#[test]
fn a_villager_cannot_post_a_kill() {
    let mut w = world();
    let wolf = w.ready();
    let state = w.state();
    let joint = w.joint();
    let s = w.seats[(wolf + 1) % N].clone();
    let (votes, _): (Vec<Cipher>, Vec<_>) = (0..N).map(|i| encrypt(if i == wolf { &shroud::GENERATOR } else { &IDENTITY }, &joint)).unzip();
    let residual = ciphers(&state.residual)[s.index as usize];
    let targets: Vec<usize> = (0..N).filter(|i| *i != s.index as usize).collect();
    let statement = st::night_statement::<Native>(&joint, &s.key, &residual, &votes, &targets);
    let ctx = st::ctx(b"night", s.game, 1, s.index, &[]);
    let forged = shroud::sigma::prove::<Native>(&ctx, &statement, 0, &(0..N).map(|_| random_scalar()).collect::<Vec<_>>(), &mut random_scalar);
    let t = w.table;
    w.refused("night", args![t, s.index as u8, b(encode_ciphers(&votes)), b(forged)], "Bad night proof");
}

#[test]
fn a_false_reveal_is_refused() {
    let mut w = world();
    let wolf = w.ready();
    w.night((wolf + 1) % N);
    w.open();
    let turn = w.state().turn;
    let t = w.table;
    w.refused("reveal", args![t, turn, b(random_scalar().to_vec())], "Wrong secret");
}

#[test]
fn the_dead_do_not_act() {
    let mut w = world();
    let wolf = w.ready();
    let victim = (wolf + 1) % N;
    w.night(victim);
    w.open();
    w.reveal();
    let state = w.state();
    assert_eq!(state.phase, DAY);
    let joint = w.joint();
    let s = w.seats[victim].clone();
    let targets: Vec<usize> = vec![wolf];
    let (votes, proof) = s.vote(state.round, &joint, &targets, N, wolf);
    let t = w.table;
    w.refused("vote", args![t, victim as u8, b(encode_ciphers(&votes)), b(proof)], "The dead do not act");
}

#[test]
fn a_tied_day_kills_nobody() {
    let mut w = world();
    let wolf = w.ready();
    let villagers: Vec<usize> = (0..N).filter(|i| *i != wolf).collect();
    w.night(villagers[0]);
    w.open();
    w.reveal();
    let state = w.state();
    let joint = w.joint();
    let alive: Vec<usize> = (0..N).filter(|i| state.alive & (1 << i) != 0).collect();
    for (i, seat) in alive.iter().enumerate() {
        let s = w.seats[*seat].clone();
        let pick = alive[(i + 1) % alive.len()];
        let targets: Vec<usize> = alive.iter().copied().filter(|x| x != seat).collect();
        let (votes, proof) = s.vote(state.round, &joint, &targets, N, pick);
        let t = w.table;
        w.ok("vote", args![t, *seat as u8, b(encode_ciphers(&votes)), b(proof)]);
    }
    w.open();
    let s = w.state();
    assert_eq!((s.phase, s.round, s.alive.count_ones()), (NIGHT, 2, 4));
    assert_eq!(DEAL + DAY, 8);
}

/// A throwaway key with no account plays a move, and only the fee pot pays.
fn sealed_by_stranger(w: &mut World, seat: usize, method: &str, a: Vec<tari_template_test_tooling::transaction::builder::named_args::NamedArg>) -> Transaction {
    let t = w.table;
    let s = w.seats[seat].clone();
    let nonce = w.nonce;
    w.nonce += 1;
    let proof = s.fee_proof(nonce);
    let (stranger, _) = w.test.new_key_pair(200 + nonce as u8);
    w.test
        .transaction()
        .with_fee_instructions_builder(|b| {
            b.call_method(w.game, "pay_seat_fee", args![t, seat as u8, b_(proof), 3_000_000u64])
        })
        .call_method(w.game, method, a)
        .finish()
        .seal(&stranger)
}

/// Runs a transaction with fees on, the way the network will.
fn priced(w: &mut World, tx: Transaction) -> tari_template_test_tooling::engine_types::commit_result::ExecuteResult {
    w.test.enable_fees();
    let result = w.test.execute_expect_success(tx, vec![]);
    w.test.disable_fees();
    result
}

fn b_(v: Vec<u8>) -> Bytes {
    Bytes::from_vec(v)
}

#[test]
fn a_stranger_key_plays_a_move_and_only_the_pot_pays() {
    let mut w = world();
    w.join_all();
    let before: Amount = w.test.call_method(w.game, "pot_balance", args![], vec![]);
    let joint = w.joint();
    let deck = ciphers(&w.state().deck);
    let s = w.seats[0].clone();
    let t = w.table;
    let steps = encode_steps(&s.shuffle(&deck, &joint));
    let tx = sealed_by_stranger(&mut w, 0, "shuffle", args![t, 0u8, b(steps)]);
    let result = priced(&mut w, tx);
    let fee = result.finalize.fee_receipt.total_fees_paid();
    println!("one shuffle: fee {fee}");
    assert_eq!(w.state().turn, 1);
    let after: Amount = w.test.call_method(w.game, "pot_balance", args![], vec![]);
    assert!(after < before);
}

#[test]
fn every_seat_in_one_transaction_stays_inside_the_budget() {
    let mut w = world();
    w.join_all();
    let joint = w.joint();
    let deck = ciphers(&w.state().deck);
    let s = w.seats[0].clone();
    let t = w.table;
    let steps = encode_steps(&s.shuffle(&deck, &joint));
    let tx = sealed_by_stranger(&mut w, 0, "shuffle", args![t, 0u8, b(steps)]);
    priced(&mut w, tx);
    for seat in 1..N {
        let joint = w.joint();
        let deck = ciphers(&w.state().deck);
        let s = w.seats[seat].clone();
        let steps = encode_steps(&s.shuffle(&deck, &joint));
        let tx = sealed_by_stranger(&mut w, 0, "shuffle", args![t, seat as u8, b(steps)]);
        priced(&mut w, tx);
    }
    let state = w.state();
    assert_eq!(state.phase, DEAL);
    let deck = ciphers(&state.deck);
    let mut builder = w.test.transaction();
    let proof = w.seats[0].fee_proof(w.nonce);
    builder = builder.with_fee_instructions_builder(|f| f.call_method(w.game, "pay_seat_fee", args![t, 0u8, b_(proof), 3_000_000u64]));
    for s in w.seats.clone() {
        let bases: Vec<Pt> = (0..N).filter(|i| *i != s.index as usize).map(|i| deck[i].c1).collect();
        let (shares, proof) = s.shares(b"deal", 0, &bases);
        builder = builder.call_method(w.game, "deal", args![t, s.index as u8, b(shares.concat()), b(proof)]);
    }
    let (stranger, _) = w.test.new_key_pair(250);
    let tx = builder.finish().seal(&stranger);
    let result = priced(&mut w, tx);
    println!("deal for five seats: fee {}", result.finalize.fee_receipt.total_fees_paid());
    w.nonce += 1;
    assert_eq!(w.state().phase, NIGHT);

    let state = w.state();
    let joint = w.joint();
    let residual = ciphers(&state.residual);
    let proof = w.seats[0].fee_proof(w.nonce);
    let mut builder = w.test.transaction().with_fee_instructions_builder(|f| f.call_method(w.game, "pay_seat_fee", args![t, 0u8, b_(proof), 3_000_000u64]));
    for s in w.seats.clone() {
        let targets: Vec<usize> = (0..N).filter(|i| *i != s.index as usize).collect();
        let (votes, proof) = s.night(1, &joint, &residual[s.index as usize], &targets, N, Some(targets[0]));
        builder = builder.call_method(w.game, "night", args![t, s.index as u8, b(encode_ciphers(&votes)), b(proof)]);
    }
    let (stranger, _) = w.test.new_key_pair(251);
    let tx = builder.finish().seal(&stranger);
    let result = priced(&mut w, tx);
    println!("night for five seats: fee {}", result.finalize.fee_receipt.total_fees_paid());
    assert_eq!(w.state().phase, NIGHT_OPEN);
}

#[test]
fn a_stale_fee_proof_is_refused() {
    let mut w = world();
    w.join_all();
    let t = w.table;
    let proof = w.seats[0].fee_proof(99);
    let (stranger, _) = w.test.new_key_pair(252);
    let tx = w
        .test
        .transaction()
        .with_fee_instructions_builder(|f| f.call_method(w.game, "pay_seat_fee", args![t, 0u8, b_(proof), 3_000_000u64]))
        .call_method(w.game, "pot_balance", args![])
        .finish()
        .seal(&stranger);
    assert!(w.test.try_execute(tx, vec![]).map(|r| r.finalize.result.any_accept().is_none()).unwrap_or(true));
}

fn stake(w: &mut World, seat: usize, amount: u64) {
    let t = w.table;
    let tx = w
        .test
        .transaction()
        .call_method(w.account, "withdraw", args![tari_template_lib::prelude::TARI_TOKEN, Amount::from(amount)])
        .put_last_instruction_output_on_workspace("stake")
        .call_method(w.game, "ante", args![t, seat as u8, Workspace("stake")])
        .finish()
        .seal(&w.key);
    w.test.execute_expect_success(tx, vec![]);
}

fn finish_with_village_win(w: &mut World, wolf: usize) {
    w.night((wolf + 1) % N);
    w.open();
    w.reveal();
    w.vote(wolf);
    w.open();
    w.reveal();
    assert_eq!(w.state().winner, WINNER_VILLAGE);
}

fn payout(amount: u64) -> tari_template_test_tooling::support::stealth::StealthSecretTransferData {
    tari_template_test_tooling::support::stealth::generate_mint_statement([amount], 0u64, None)
}

#[test]
fn stakes_close_at_nightfall() {
    let mut w = world();
    w.join_all();
    w.shuffle_all();
    w.deal_all();
    let wolf = w.wolf();
    let winner = (wolf + 1) % N;
    // Stakes close at nightfall, so this table is too late.
    let t = w.table;
    let tx = w.test.transaction()
        .call_method(w.account, "withdraw", args![tari_template_lib::prelude::TARI_TOKEN, Amount::from(1000u64)])
        .put_last_instruction_output_on_workspace("stake")
        .call_method(w.game, "ante", args![t, winner as u8, Workspace("stake")])
        .finish().seal(&w.key);
    let why = format!("{:?}", w.test.execute_expect_failure(tx, vec![]));
    assert!(why.contains("The first night has begun"), "{why}");
}

#[test]
fn stakes_pay_the_winners_and_only_once() {
    let mut w = world();
    w.join_all();
    stake(&mut w, 0, 1_000);
    stake(&mut w, 1, 1_000);
    w.shuffle_all();
    w.deal_all();
    let wolf = w.wolf();
    finish_with_village_win(&mut w, wolf);
    let t = w.table;
    let winners: Vec<usize> = [0usize, 1].into_iter().filter(|s| *s != wolf).collect();
    let share = 2_000 / winners.len() as u64;
    let seat = winners[0];
    let data = payout(share);
    let commitment = *data.statement.outputs_statement.outputs[0].commitment();
    let proof = w.seats[seat].collect_proof(commitment.as_ref());

    // A proof for one output can't be used to pay a different one.
    let other = payout(share);
    w.refused("collect", args![t, seat as u8, other.statement.clone(), b(proof.clone())], "Bad collect proof");
    let wrong = payout(share - 1);
    let wrong_commitment = *wrong.statement.outputs_statement.outputs[0].commitment();
    let wrong_proof = w.seats[seat].collect_proof(wrong_commitment.as_ref());
    w.refused("collect", args![t, seat as u8, wrong.statement.clone(), b(wrong_proof)], "Wrong amount");

    w.ok("collect", args![t, seat as u8, data.statement.clone(), b(proof.clone())]);
    let again = payout(share);
    let again_commitment = *again.statement.outputs_statement.outputs[0].commitment();
    let again_proof = w.seats[seat].collect_proof(again_commitment.as_ref());
    w.refused("collect", args![t, seat as u8, again.statement.clone(), b(again_proof)], "Already collected");
}

#[test]
fn a_seat_that_lost_collects_nothing() {
    let mut w = world();
    w.join_all();
    stake(&mut w, 0, 1_000);
    stake(&mut w, 1, 1_000);
    stake(&mut w, 2, 1_000);
    stake(&mut w, 3, 1_000);
    stake(&mut w, 4, 1_000);
    w.shuffle_all();
    w.deal_all();
    let wolf = w.wolf();
    finish_with_village_win(&mut w, wolf);
    let t = w.table;
    let data = payout(1_250);
    let commitment = *data.statement.outputs_statement.outputs[0].commitment();
    let proof = w.seats[wolf].collect_proof(commitment.as_ref());
    w.refused("collect", args![t, wolf as u8, data.statement.clone(), b(proof)], "This seat lost");
}
