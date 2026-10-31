use curve25519_dalek::scalar::Scalar;
use rand::{rngs::OsRng, seq::SliceRandom};
use shroud::{
    group::{small, Group},
    native::Native,
    network,
    sigma::prove,
    statements::{self as st, Cipher, CARD_VILLAGER, CARD_WOLF},
    Pt, Sc, GENERATOR, IDENTITY,
};

pub fn random_scalar() -> Sc {
    Scalar::random(&mut OsRng).to_bytes()
}

fn rng() -> impl FnMut() -> Sc {
    random_scalar
}

/// Encrypts `plain` under `joint`, returning the ciphertext and its randomness.
pub fn encrypt(plain: &Pt, joint: &Pt) -> (Cipher, Sc) {
    let r = random_scalar();
    let c1 = Native::mul_base(&r);
    let c2 = Native::add(plain, &Native::mul(joint, &r));
    (Cipher { c1, c2 }, r)
}

fn reencrypt(card: &Cipher, joint: &Pt) -> (Cipher, Sc) {
    let (zero, r) = encrypt(&IDENTITY, joint);
    (st::sum::<Native>(card, &zero), r)
}

/// The starting deck, one wolf among villagers, in the clear. The first shuffle encrypts it.
pub fn starting_deck(seats: usize) -> Vec<Cipher> {
    (0..seats)
        .map(|i| Cipher {
            c1: IDENTITY,
            c2: st::card_point::<Native>(if i == 0 { CARD_WOLF } else { CARD_VILLAGER }),
        })
        .collect()
}

/// One round of the shuffle: the output of every switch and its proof.
pub struct ShuffleStep {
    pub out_a: Cipher,
    pub out_b: Cipher,
    pub proof: Vec<u8>,
}

/// Secrets for one seat at one table. In a real game each player's tab holds its own.
#[derive(Clone)]
pub struct Seat {
    pub game: u64,
    pub index: u32,
    pub secret: Sc,
    pub key: Pt,
}

impl Seat {
    pub fn new(game: u64, index: u32) -> Self {
        let secret = random_scalar();
        let key = Native::mul_base(&secret);
        Self { game, index, secret, key }
    }

    pub fn join_proof(&self) -> Vec<u8> {
        let ctx = st::ctx(b"join", self.game, 0, self.index, &[]);
        prove::<Native>(&ctx, &st::key_statement(&self.key), 0, &[self.secret], &mut rng())
    }

    /// Authorises the fee pot to pay for the transaction sealed by `sealer`.
    pub fn fee_proof(&self, nonce: u32, sealer: &[u8]) -> Vec<u8> {
        let ctx = st::ctx(b"fee", self.game, nonce, self.index, sealer);
        prove::<Native>(&ctx, &st::key_statement(&self.key), 0, &[self.secret], &mut rng())
    }

    /// Authorises the pot to pay for a join, from whoever holds the invite link.
    pub fn ticket_proof(&self, nonce: u32, sealer: &[u8]) -> Vec<u8> {
        let ctx = st::ctx(b"ticket", self.game, nonce, 0, sealer);
        prove::<Native>(&ctx, &st::key_statement(&self.key), 0, &[self.secret], &mut rng())
    }

    /// Lets `key` sit in `seat`. The holder of the invite ticket makes this.
    pub fn invite_proof(&self, seat: u32, key: &Pt) -> Vec<u8> {
        let ctx = st::ctx(b"invite", self.game, 0, seat, key);
        prove::<Native>(&ctx, &st::key_statement(&self.key), 0, &[self.secret], &mut rng())
    }

    /// Stakes on this seat, naming the separate key that will claim the money. The seat's own key
    /// goes public if it dies, so it can't be the one that collects.
    pub fn ante_proof(&self, payout_key: &Pt) -> Vec<u8> {
        let ctx = st::ctx(b"ante", self.game, 0, self.index, payout_key);
        prove::<Native>(&ctx, &st::key_statement(&self.key), 0, &[self.secret], &mut rng())
    }

    /// Signs one shuffle. Without this, anyone could shuffle for a seat with a known permutation.
    pub fn shuffle_proof(&self, steps: &[u8]) -> Vec<u8> {
        let ctx = st::ctx(b"shuffle-by", self.game, 0, self.index, &Native::hash_scalar(steps));
        prove::<Native>(&ctx, &st::key_statement(&self.key), 0, &[self.secret], &mut rng())
    }

    /// Lets the pot pay for a claim, signed by the payout key.
    pub fn payout_fee_proof(&self, nonce: u32, sealer: &[u8]) -> Vec<u8> {
        let ctx = st::ctx(b"payout-fee", self.game, nonce, self.index, sealer);
        prove::<Native>(&ctx, &st::key_statement(&self.key), 0, &[self.secret], &mut rng())
    }

    /// Claims money with the payout key in the transaction sealed by `sealer`. `action` is
    /// `collect` or `refund`.
    pub fn payout_proof(&self, action: &[u8], sealer: &[u8]) -> Vec<u8> {
        let ctx = st::ctx(action, self.game, 0, self.index, sealer);
        prove::<Native>(&ctx, &st::key_statement(&self.key), 0, &[self.secret], &mut rng())
    }

    pub fn shuffle(&self, deck: &[Cipher], joint: &Pt) -> Vec<ShuffleStep> {
        let n = deck.len();
        let mut destinations: Vec<usize> = (0..n).collect();
        destinations.shuffle(&mut OsRng);
        self.shuffle_with(deck, joint, &destinations)
    }

    /// Shuffle to fixed destinations. Tests use this to deal known hands.
    pub fn shuffle_with(&self, deck: &[Cipher], joint: &Pt, destinations: &[usize]) -> Vec<ShuffleStep> {
        let mut wires = deck.to_vec();
        let settings = network::route(destinations);
        let mut steps = Vec::new();
        for (step, ((a, b), cross)) in network::switches(deck.len()).into_iter().zip(settings).enumerate() {
            let (in_a, in_b) = (wires[a], wires[b]);
            let (first, second) = if cross { (in_b, in_a) } else { (in_a, in_b) };
            let (out_a, ra) = reencrypt(&first, joint);
            let (out_b, rb) = reencrypt(&second, joint);
            let ctx = st::ctx(b"shuffle", self.game, step as u32, self.index, &[]);
            let statement = st::switch_statement::<Native>(joint, &in_a, &in_b, &out_a, &out_b);
            let proof = prove::<Native>(&ctx, &statement, cross as usize, &[ra, rb], &mut rng());
            wires[a] = out_a;
            wires[b] = out_b;
            steps.push(ShuffleStep { out_a, out_b, proof });
        }
        steps
    }

    /// Shares for a list of bases, with one proof over all of them.
    pub fn shares(&self, action: &[u8], round: u32, bases: &[Pt]) -> (Vec<Pt>, Vec<u8>) {
        let shares: Vec<Pt> = bases.iter().map(|b| Native::mul(b, &self.secret)).collect();
        let ctx = st::ctx(action, self.game, round, self.index, &[]);
        let statement = st::shares_statement(&self.key, bases, &shares);
        let proof = prove::<Native>(&ctx, &statement, 0, &[self.secret], &mut rng());
        (shares, proof)
    }

    /// Reads this seat's card from its residual ciphertext.
    pub fn read_card(&self, residual: &Cipher) -> Option<u8> {
        let plain = Native::sub(&residual.c2, &Native::mul(&residual.c1, &self.secret));
        [CARD_WOLF, CARD_VILLAGER].into_iter().find(|c| Native::mul_base(&small(*c)) == plain)
    }

    fn one_hot(joint: &Pt, seats: usize, pick: Option<usize>) -> (Vec<Cipher>, Vec<Sc>) {
        (0..seats)
            .map(|i| encrypt(if Some(i) == pick { &GENERATOR } else { &IDENTITY }, joint))
            .unzip()
    }

    /// The night action. `pick` is only honoured for the wolf; every other seat sends the same
    /// shape of message with a proof of the empty branch.
    pub fn night(
        &self,
        round: u32,
        joint: &Pt,
        residual: &Cipher,
        targets: &[usize],
        seats: usize,
        pick: Option<usize>,
    ) -> (Vec<Cipher>, Vec<u8>) {
        let wolf = self.read_card(residual) == Some(CARD_WOLF);
        let pick = if wolf { pick } else { None };
        let (votes, randomness) = Self::one_hot(joint, seats, pick);
        let statement = st::night_statement::<Native>(joint, &self.key, residual, &votes, targets);
        let real = match pick {
            None => 0,
            Some(target) => targets.iter().position(|t| *t == target).expect("pick must be a target") + 1,
        };
        let mut witnesses = vec![self.secret];
        witnesses.extend(randomness);
        let ctx = st::ctx(b"night", self.game, round, self.index, &[]);
        (votes.clone(), prove::<Native>(&ctx, &statement, real, &witnesses, &mut rng()))
    }

    pub fn vote(&self, round: u32, joint: &Pt, targets: &[usize], seats: usize, pick: usize) -> (Vec<Cipher>, Vec<u8>) {
        let (votes, randomness) = Self::one_hot(joint, seats, Some(pick));
        let statement = st::vote_statement::<Native>(joint, &self.key, &votes, targets);
        let k = targets.iter().position(|t| *t == pick).expect("pick must be a target");
        let ctx = st::ctx(b"vote", self.game, round, self.index, &[]);
        let mut witnesses = vec![self.secret];
        witnesses.extend(randomness);
        (votes.clone(), prove::<Native>(&ctx, &statement, k, &witnesses, &mut rng()))
    }
}
