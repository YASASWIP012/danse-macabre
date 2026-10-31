# Threat model

Testnet only, and not audited. The honest claim of Danse Macabre is narrow, so here it is in full.

## What the game hides

- **Who holds the wolf card.** Cards are encrypted under a key made from all five seats. A seat can only read its own card once the other four have stripped their layers.
- **The deal itself.** Each seat reorders the deck in secret. One seat that keeps its order to itself is enough, even if the other four collude.
- **Who acted at night.** Every living seat posts one sealed action. They are the same size and take the same path through the verifier. Only the sum is decrypted.
- **Who voted for whom.** Day votes are sealed and summed the same way.
- **Which wallet is in which seat.** Moves are sealed by a throwaway key and paid for by the table's fee pot, so no account appears.

## What stays public

That a table exists, the five seat keys, which seats have posted, timing, fees, who died, and the card of each dead seat. Seat slots are public, so the game does not hide *that* seat 3 posted. It hides what seat 3's post means.

## Attacks and what stops them

Every row has a test that plays the attack on the real Ootle engine and checks the reason it fails.

| Attack | What stops it | Test |
|---|---|---|
| Rogue key: the last seat cancels the others and reads everything | Proof of possession on join | `a_rogue_key_without_proof_of_possession_is_refused` |
| A stranger takes a seat, or fills the table | Each key needs an invite signed by the ticket | `a_join_needs_the_ticket` |
| Slipping a second wolf into the deck | Every switch proves it re-encrypted the same two cards | `a_shuffle_that_adds_a_second_wolf_is_refused` |
| Shuffling for another seat with a known order, to learn the deal | A shuffle must be signed by the seat's key | `nobody_else_can_shuffle_for_a_seat` |
| Shuffling out of turn | Turn check | `shuffling_out_of_turn_is_refused` |
| A villager posting a kill | The only branch with a vote needs the wolf card | `a_villager_cannot_post_a_kill` |
| Posting a night action for another seat, to block the kill | Every branch of the night proof needs the seat's key | `nobody_else_can_post_a_seats_night_action` |
| Casting a day vote for another seat | Every branch of the vote proof needs the seat's key | `nobody_else_can_post_a_seats_day_vote` |
| Copying another seat's proof | Game, round, seat and the full statement are hashed into every proof | `a_night_proof_cannot_be_replayed_by_another_seat` |
| Acting twice in a night | One action per seat per phase | `a_seat_cannot_act_twice_in_one_night` |
| Lying about a dead seat's card | The published secret must match the seat key | `a_false_reveal_is_refused` |
| Voting from the grave | Dead seats can't act | `the_dead_do_not_act` |
| A biased shuffle | The client samples a uniform permutation and routes it, instead of flipping each switch | `network_routes_every_permutation` |
| Using a stale fee proof, or lifting one from the mempool | Each proof binds a fresh nonce and the transaction's seal key | `a_stale_fee_proof_is_refused` |
| Copying a payout claim to redirect the money | The claim binds the transaction's seal key | `stakes_pay_the_winners_and_only_once` |
| Taking a dead seat's winnings with its published secret | Stakes name a separate payout key | `a_dead_seats_public_secret_cannot_take_its_payout` |
| A villager claiming a wolf win | Unmasking checks the card under the seat's secret | `a_surviving_wolf_unmasks_and_collects` |
| Staking after reading your card | Stakes close before the deal | `stakes_close_before_the_deal` |

The first version of this template had no key check on shuffles, night actions or votes. An independent review found that, and the rows above come from it.

## Known limits

- **Stalling.** All seats must take part, so one closed tab stops the game. Anyone can close the table between 48 and 96 minutes after its last move, and every stake goes back to its owner. A player who stalls loses nothing, so a losing wolf can refuse to reveal and turn a loss into a refund.
- **Last to open sees first.** The last seat to post a share can work out the total a moment early. It can stall, as above, but it can't change the result.
- **Timing.** Every player picks a name at night and the client waits a random moment, so a wolf who deliberates looks like a villager who does the same. A player using their own client could still leak through timing.
- **Provable cards.** A player can prove their own card to someone else. Flipping your card over at a real table works the same way, so this is a house rule.
- **Solo play.** The four ghosts run in your own tab, and you could read their cards. The privacy only means something between separate people.
- **Fee pot.** Tables that may open per epoch are capped and each table has a fee budget. A determined stranger could still burn through the epoch cap on purpose.
- **Stake unlinking is only as good as the coin.** A stake comes from a stealth output, and that output's own history still links to whatever funded it.
