//! Puts the game on Esmeralda: claims the L1 burn, publishes the template, creates the component.
//!
//!   danse-deploy claim <proof.json>     claim the burn into two spendable outputs
//!   danse-deploy publish <wasm>         publish the template and open the fee pot
//!
//! Keys come from DEPLOYER_KEY and DEPLOYER_VIEW in the environment.

use std::{env, fs, num::NonZeroU64};

use ootle_rs::{
    claim_burn::{ClaimBurn, MinotariBurnClaimProof},
    default_indexer_url,
    key_provider::PrivateKeyProvider,
    keys::OotleSecretKey,
    provider::{Provider, ProviderBuilder, WalletProvider},
    stealth::{Output, StealthTransfer},
    template_types::{
        constants::{TARI, TARI_TOKEN},
        crypto::PedersenCommitmentBytes,
        EncryptedData,
        Hash32,
        UtxoAddress,
    },
    wallet::{NetworkWallet, OotleWallet},
    Network, TransactionRequest,
};
use tari_crypto::{ristretto::RistrettoSecretKey, tari_utilities::ByteArray};
use tari_ootle_transaction::{args, Epoch, Transaction};

const FEE_BUDGET: u64 = 40 * TARI;

#[derive(serde::Deserialize)]
struct BurnProofFile {
    claim_proof: MinotariBurnClaimProof,
    encrypted_data: EncryptedData,
}

fn secret(name: &str) -> RistrettoSecretKey {
    let bytes = hex::decode(env::var(name).expect(name)).expect("hex key");
    RistrettoSecretKey::from_canonical_bytes(&bytes).expect("canonical key")
}

#[tokio::main]
async fn main() {
    let mut argv = env::args().skip(1);
    let command = argv.next().expect("command");
    let network = Network::Esmeralda;
    let keys = OotleSecretKey::new(network, secret("DEPLOYER_KEY"), secret("DEPLOYER_VIEW"));
    let wallet = OotleWallet::from(PrivateKeyProvider::new(keys));
    let me = wallet.default_address().clone();
    let mut provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect(default_indexer_url(network))
        .await
        .expect("indexer");
    let max_epoch = Epoch(provider.get_epoch().await.unwrap().as_u64() + 10);

    match command.as_str() {
        "claim" => {
            let path = argv.next().expect("proof file");
            let file: BurnProofFile = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
            let (unsigned, sealer) = ClaimBurn::new(&provider, file.claim_proof, file.encrypted_data, max_epoch)
                .with_max_fee(50_000u64)
                .with_memo_message("Danse Macabre deployment")
                .prepare()
                .await
                .expect("prepare claim");
            let tx = TransactionRequest::default().with_transaction(unsigned).build(&sealer).await.unwrap();
            let pending = provider.send_transaction(tx).await.expect("submit");
            println!("claim tx {}", pending.tx_id());
            println!("{:?}", pending.watch().await.expect("claim"));
        },
        "publish" => {
            let wasm = fs::read(argv.next().expect("wasm path")).unwrap();
            let commitment = hex::decode(argv.next().expect("input commitment")).unwrap();
            let value: u64 = argv.next().expect("input value").parse().unwrap();
            let commitment = PedersenCommitmentBytes::try_from(commitment.as_slice()).unwrap();
            // Splits the claim into a fee coin and a pot coin for the `open` step.
            let pot = 4000 * TARI;
            let left = NonZeroU64::new(value - FEE_BUDGET - pot).unwrap();
            let (fee, signers) = StealthTransfer::new(TARI_TOKEN, &provider)
                .spend_stealth_input(me.clone(), commitment)
                .to_revealed_output(FEE_BUDGET)
                .to_stealth_output(Output::new(me.clone(), TARI_TOKEN, left))
                .to_stealth_output(Output::new(me.clone(), TARI_TOKEN, NonZeroU64::new(pot).unwrap()))
                .prepare()
                .await
                .unwrap();
            for output in fee.stealth_outputs() {
                println!("output {}", hex::encode(output.commitment().as_ref()));
            }
            let unsigned = Transaction::builder(provider.network(), max_epoch)
                .with_fee_instructions_builder(|b| {
                    b.stealth_transfer(TARI_TOKEN, fee)
                        .put_last_instruction_output_on_workspace("fees")
                        .pay_fee_from_bucket("fees")
                })
                .publish_template(wasm)
                .add_input(TARI_TOKEN)
                .add_input(UtxoAddress::new(TARI_TOKEN, commitment.into()))
                .build_unsigned();
            let authorizer = provider.wallet().stealth_authorizer(signers);
            let tx = TransactionRequest::default().with_transaction(unsigned).build(&authorizer).await.unwrap();
            let pending = provider.send_transaction(tx).await.expect("submit");
            println!("publish tx {}", pending.tx_id());
            println!("{:?}", pending.watch().await.expect("publish"));
        },
        "open" => {
            // Input A pays the fee, input B fills the pot.
            let raw = hex::decode(argv.next().expect("template hex")).unwrap();
            let template = Hash32::try_from(raw.as_slice()).unwrap();
            let a = PedersenCommitmentBytes::try_from(hex::decode(argv.next().unwrap()).unwrap().as_slice()).unwrap();
            let a_value: u64 = argv.next().unwrap().parse().unwrap();
            let b = PedersenCommitmentBytes::try_from(hex::decode(argv.next().unwrap()).unwrap().as_slice()).unwrap();
            let pot: u64 = argv.next().unwrap().parse().unwrap();
            let left = NonZeroU64::new(a_value - FEE_BUDGET).unwrap();
            let (fee, fee_signers) = StealthTransfer::new(TARI_TOKEN, &provider)
                .spend_stealth_input(me.clone(), a)
                .to_revealed_output(FEE_BUDGET)
                .to_stealth_output(Output::new(me.clone(), TARI_TOKEN, left))
                .prepare()
                .await
                .unwrap();
            let (main, main_signers) = StealthTransfer::new(TARI_TOKEN, &provider)
                .spend_stealth_input(me.clone(), b)
                .to_revealed_output(pot)
                .prepare()
                .await
                .unwrap();
            let unsigned = Transaction::builder(provider.network(), max_epoch)
                .with_fee_instructions_builder(|f| {
                    f.stealth_transfer(TARI_TOKEN, fee)
                        .put_last_instruction_output_on_workspace("fees")
                        .pay_fee_from_bucket("fees")
                })
                .stealth_transfer(TARI_TOKEN, main)
                .put_last_instruction_output_on_workspace("pot")
                .call_function(template, "new", args![Workspace("pot")])
                .add_input(TARI_TOKEN)
                .add_input(UtxoAddress::new(TARI_TOKEN, a.into()))
                .add_input(UtxoAddress::new(TARI_TOKEN, b.into()))
                .build_unsigned();
            let authorizer = provider.wallet().stealth_authorizer(fee_signers.merge(main_signers));
            let tx = TransactionRequest::default().with_transaction(unsigned).build(&authorizer).await.unwrap();
            let pending = provider.send_transaction(tx).await.expect("submit");
            println!("open tx {}", pending.tx_id());
            println!("{:?}", pending.watch().await.expect("open"));
        },
        other => panic!("unknown command {other}"),
    }
}
