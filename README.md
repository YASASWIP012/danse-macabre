# Danse Macabre

Werewolf with no game master, on Tari Ootle. Five players shuffle one wolf into the deck together, so nobody knows the deal, including whoever opened the table. Each night every living seat posts one sealed action. The wolf's names a victim and the others are proved empty. Only the sum is decrypted, so the village learns who died and never who did it.

## Why this needs Tari

The cryptography would run on any chain with fast curve operations. The rest of the game would not. On a public chain the account paying for a move is a clue about who made it. Here each move is sealed by a throwaway key and the table's fee pot pays, so no wallet ever touches a seat.

## What's here

| Path | What it is |
|---|---|
| `crates/shroud` | A reusable library for hidden-state games: one proof system for shuffles, shared decryption and sealed actions. The same code verifies in the template and in the browser. |
| `template` | The Ootle template that runs the game. It holds no secret and trusts no seat. |
| `crates/client` | The player's prover, compiled to WebAssembly. |
| `web` | The site. |
| `THREAT_MODEL.md` | What's hidden, what isn't, and which test plays each attack. |

## Run the tests

```bash
cargo test -p danse-client
```

```bash
cd template && cargo test --release
```

The second suite plays whole games and every attack in `THREAT_MODEL.md` on the real Ootle engine, in-process.

MIT licensed. Testnet only, not audited.
