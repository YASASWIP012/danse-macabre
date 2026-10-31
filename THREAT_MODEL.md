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

| Attack | What stops it | Test |
|---|---|---|
| Rogue key: the last seat cancels the others and reads everything | Proof of possession on join | `a_rogue_key_without_proof_of_possession_is_refused` |
| Slipping a second wolf into the deck | Every switch proves it re-encrypted the same two cards | `a_shuffle_that_adds_a_second_wolf_is_refused` |
| Shuffling out of turn | Turn check | `shuffling_out_of_turn_is_refused` |
| A villager posting a kill | The only branch with a vote needs the wolf card | `a_villager_cannot_post_a_kill` |
| Copying another seat's proof | Game, round, seat and the full statement are hashed into every proof | `a_night_proof_cannot_be_replayed_by_another_seat` |
| Acting twice in a night | One action per seat per phase | `a_seat_cannot_act_twice_in_one_night` |
| Lying about a dead seat's card | The published secret must match the seat key | `a_false_reveal_is_refused` |
| Voting from the grave | Dead seats can't act | `the_dead_do_not_act` |
| A biased shuffle | The client samples a uniform permutation and routes it, instead of flipping each switch | `network_routes_every_permutation` |
| Voting for several victims | Each branch fixes exactly one target | `a_villager_cannot_forge_a_kill`, `night_statement` |

## Known limits

- **Stalling.** All five seats must take part, so one closed tab stops the game. After two epochs (about 96 minutes) anyone can close the table. There is no forfeit of stakes yet.
- **Last to open sees first.** The last seat to post a share can work out the total a moment early. It can stall but it can't change the result.
- **Timing.** A wolf who deliberates posts later than a villager who doesn't. The client should wait until the phase is about to close before posting, and between humans this is not yet enforced.
- **Provable cards.** A player can prove their own card to someone else. Flipping your card over at a real table works the same way, so this is a house rule.
- **Solo play.** The four ghosts run in your own tab, and you could read their cards. The privacy only means anything between separate people.
- **Fee pot.** Tables that may open per epoch are capped and each table has a fee budget. Someone could still burn through the cap on purpose.
