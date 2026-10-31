# Proof

A real five-seat game on Esmeralda, played from the site. The wolf won, and The Gravedigger turned the card over on-chain at the end.

- Template: [63523893016c13646e1f44a3595dbdc0b9a8fd98a0f7e26549860029be94a5a5](https://explorer.tari.mw/template/63523893016c13646e1f44a3595dbdc0b9a8fd98a0f7e26549860029be94a5a5)
- WASM sha256: `819a0fa622edd3ecf4fe50d291ffacbfa2d53f50ec2f0acb4c6a26b6140c4ca5`
- Component: [component_3ce57c95a3a7644274d539ad76ff5d270e3b852ba537db7fc8c2a07f7d2fe976](https://explorer.tari.mw/substate/component_3ce57c95a3a7644274d539ad76ff5d270e3b852ba537db7fc8c2a07f7d2fe976)
- Table: 1

Every transaction below was sealed by a throwaway key and paid for by the fee pot. No account appears in any of them.

| # | Move | Transaction |
|---|---|---|
| 1 | Table opened | [48aba9d6e9a9df85…](https://explorer.tari.mw/tx/48aba9d6e9a9df854a289dfa2141ac97baad988e9e91945fa30de9fd79882776) |
| 2 | Five keys registered | [d07681ce1eab4deb…](https://explorer.tari.mw/tx/d07681ce1eab4debc6fb8b71afe9aab6b9245bad79b3fa2d3eeae4b8790e8324) |
| 3 | Seats shuffle, with a proof for every switch | [39e0d7f973cec1ee…](https://explorer.tari.mw/tx/39e0d7f973cec1eec2ede215f4136ff390f4fa558a12a7d7129f8479cebbe412) |
| 4 | Seats strip their layers from the others' cards | [ae07b2f62078f610…](https://explorer.tari.mw/tx/ae07b2f62078f6106b9a40d98f50d0ea4d715d9af1ae3da830f8217ea2144610) |
| 5 | Sealed night actions, all the same size | [744c1479ffaa3bec…](https://explorer.tari.mw/tx/744c1479ffaa3beca1d17c20960daefbdf19e1cd97e5929272e55096c4d3a764) |
| 6 | The night's total opens | [df69e198c2ebf2c5…](https://explorer.tari.mw/tx/df69e198c2ebf2c578cdfc5d4e838fe174ae09c13258877372307b6a0a7b5721) |
| 7 | The dead seat turns its card face up | [e72627fe88654418…](https://explorer.tari.mw/tx/e72627fe886544189596e514d2edbef1fd8a35f4559098c806dd425a81574722) |
| 8 | Sealed day votes | [e0fd8113e60e9ad1…](https://explorer.tari.mw/tx/e0fd8113e60e9ad1930f684098716c5223e73c6fa8533efcdd0757a4e06cab49) |
| 9 | The day's total opens | [c419fe3bde5e3e5f…](https://explorer.tari.mw/tx/c419fe3bde5e3e5fdbdfa5f879b237a85391df8421d447b7947ed6d79f82a10e) |
| 10 | The dead seat turns its card face up | [697031144ec3212d…](https://explorer.tari.mw/tx/697031144ec3212d3cce4e3a3c6f4a4fc2c6bc27dd5f83da734a241618b4f178) |
| 11 | Sealed night actions, all the same size | [eeb890f856142d11…](https://explorer.tari.mw/tx/eeb890f856142d11fa170f4812404ea5156b4d4333e497dd44ca0ceb74f3cb86) |
| 12 | The night's total opens | [69777a891fd5f5e1…](https://explorer.tari.mw/tx/69777a891fd5f5e19febfa917da72667da3e20a998cfd75a8b79eaad61132d4d) |
| 13 | The dead seat turns its card face up | [acb761196d83759a…](https://explorer.tari.mw/tx/acb761196d83759a6703c8c8d5536695be13252d19b93652a8969b960647b56b) |
| 14 | The wolf turns its card over | [40a90cf69bbcf6c5…](https://explorer.tari.mw/tx/40a90cf69bbcf6c5479d653c23eee7e0090218688579bd8f1cbc63a6470b257f) |

The transactions in the night and day rows carry all the seats' moves in one transaction, because this game was played solo with four ghosts. In group play each player posts their own.

Test count: 25 engine tests and 8 prover tests.
