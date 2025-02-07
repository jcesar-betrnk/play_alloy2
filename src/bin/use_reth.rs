use alloy::{
    node_bindings::Reth,
};
use alloy::{
    primitives::{U256, address},
    providers::{ProviderBuilder},
    rpc::types::TransactionRequest,
    network::{EthereumWallet, TransactionBuilder, Ethereum, NetworkWallet},
    signers::local::LocalSigner,
};
use alloy::signers::local::{coins_bip39::English, MnemonicBuilder};
use alloy::providers::Provider;
use alloy::primitives::Address;
use eyre::Result;
use k256::{
    ecdsa::{SigningKey},
};
use alloy::consensus::Transaction;


#[tokio::main]
async fn main() -> Result<()> {
    // Spin up a local Reth node.
    // Ensure `reth` is available in $PATH.
    let reth = Reth::new().dev().disable_discovery().instance(1).spawn();
    let provider = ProviderBuilder::new().on_http(reth.endpoint().parse()?);

    let chain_id = provider.get_chain_id().await?;

    println!("Reth running at: {} with chain id: {chain_id}", reth.endpoint());

    assert_eq!(chain_id, 1337);
    assert_eq!(reth.http_port(), 8545);
    assert_eq!(reth.ws_port(), 8546);
    assert_eq!(reth.auth_port(), Some(8551));
    assert_eq!(reth.p2p_port(), None);

    let gas_price = provider.get_gas_price().await?;
    dbg!(gas_price);

    let phrase = "work man brother plunge mystery proud hollow address reunion sauce theory bonus";
    let index = 0u32;
    let password = "pwd123";

    let created_wallet = create_wallet(index, phrase, password)?;
    dbg!(&created_wallet);

    let phrase2 = "work man sister plunge mystery proud hollow address reunion sauce theory bonus";
    let index2 = 0u32;
    let password2 = "pass123";

    let created_wallet2 = create_wallet(index2, phrase2, password2)?;
    dbg!(&created_wallet2);

    let network_wallet = EthereumWallet::from(created_wallet.clone());


    let alice = address!("0x70997970C51812dc3A010C7d01b50e0d17dc79C8");
    //send_transaction(&provider, created_wallet.address(), alice, U256::from(1_000_000_000_000_000_u128)).await?;
    send_signed_transaction(&provider, created_wallet2.address(), created_wallet.address(), network_wallet, U256::from(4_000_000)).await?;

    Ok(())
}

fn create_wallet(index: u32, phrase: &str, password: &str) -> Result<LocalSigner<SigningKey>>{
    let wallet: LocalSigner<SigningKey> = MnemonicBuilder::<English>::default()
        .phrase(phrase)
        .index(index)?
        .password(password)
        .build()?;
    Ok(wallet)
}

async fn send_transaction(provider: &impl Provider, dest_addr: Address, source_addr: Address, amount: U256) -> Result<()> {
    let tx = TransactionRequest::default()
        .with_to(dest_addr)
        .with_from(source_addr)
        .with_gas_price(1_000_000_000)
        .with_gas_limit(21_000)
        .with_value(amount);
    let dest_balance = provider.get_balance(dest_addr).await?;
    dbg!(dest_balance);
    let source_balance = provider.get_balance(source_addr).await?;
    dbg!(source_balance);

    let builder = provider.send_transaction(tx.clone()).await?;
    let node_hash = *builder.tx_hash();
    let receipt = builder.get_receipt().await?;
    dbg!(receipt);
    let pending_tx =
        provider.get_transaction_by_hash(node_hash).await?.expect("Pending transaction not found");

    println!("Transaction sent with nonce: {}", pending_tx.nonce());

    let dest_after_balance = provider.get_balance(dest_addr).await?;
    dbg!(dest_after_balance);
    let source_after_balance = provider.get_balance(source_addr).await?;
    dbg!(source_after_balance);

    println!("Transaction sent with nonce: {}", pending_tx.nonce());

    Ok(())
}

async fn send_signed_transaction(provider: &impl Provider, dest_addr: Address, source_addr: Address, signer_wallet: impl NetworkWallet<Ethereum>, amount: U256) -> Result<()> {
    println!("chain_id: {:?}", provider.get_chain_id().await?);
    let tx = TransactionRequest::default()
        .with_to(dest_addr)
        .with_nonce(0)
        .with_from(source_addr)
        .with_gas_price(900_000_000)
        .with_gas_limit(21_000)
        .with_value(amount);
    let dest_balance = provider.get_balance(dest_addr).await?;
    dbg!(dest_balance);
    let source_balance = provider.get_balance(source_addr).await?;
    dbg!(source_balance);

    //let signer_wallet = provider.wallet();
    println!("using the signing wallet..");
    let tx_envelope = tx.build(&signer_wallet).await?;
    println!("Sending the transaction envelope...");
    let receipt = provider.send_tx_envelope(tx_envelope).await?.get_receipt().await?;
    println!("sent transaction: {}", receipt.transaction_hash);


    let dest_after_balance = provider.get_balance(dest_addr).await?;
    dbg!(dest_after_balance);
    let source_after_balance = provider.get_balance(source_addr).await?;
    dbg!(source_after_balance);


    Ok(())
}
