//#![deny(warnings)]
//! This example demonstrates how to interact with a contract that is already deployed onchain using
//! the `ContractInstance` interface.

use alloy::{
    contract::{ContractInstance, Interface},
    dyn_abi::DynSolValue,
    primitives::{U256, address},
    providers::{ProviderBuilder},
    json_abi::JsonAbi,
    rpc::types::TransactionRequest,
    network::{EthereumWallet, TransactionBuilder, Ethereum, NetworkWallet},
    signers::local::LocalSigner,
};
use alloy::signers::local::{coins_bip39::English, MnemonicBuilder};
use alloy::providers::Provider;
use alloy::primitives::Address;
use eyre::Result;
use k256::{
    ecdsa::{self, SigningKey},
};
use alloy::consensus::Transaction;

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure `anvil` is available in $PATH.
    let provider = ProviderBuilder::new().on_anvil_with_wallet();


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
    send_transaction(&provider, created_wallet.address(), alice, U256::from(1_000_000_000_000_000_u128)).await?;
    send_signed_transaction(&provider, created_wallet2.address(), created_wallet.address(), network_wallet, U256::from(4_000_000)).await?;


    // code: https://etherscan.io/address/0xdac17f958d2ee523a2206206994597c13d831ec7#code
    let contract_address = address!("0xdAC17F958D2ee523a2206206994597C13D831ec7");

    println!("parsing abi..");
    let transfer_abi = JsonAbi::parse([
        "function transfer(address recipient, uint256 amount) public returns (bool)",
        "function totalSupply() view returns (uint256)",
        "function balanceOf(address who) returns (uint256)",
        "function symbol() public view returns (string memory)",
    ])?;
    println!("Done parsing abi..");

    // Create a new `ContractInstance` of the `Counter` contract from the abi
    let contract = ContractInstance::new(contract_address, provider.clone(), Interface::new(transfer_abi.clone()));

    // Set the number to 42.
    let number_value = DynSolValue::from(U256::from(42_000_000));
    let recepient = alloy::primitives::Address::random();

    let recepient = alice.clone();

    // this call works on anvil, but not on the anvil forked
    // if called on the anvil fork provider: error code -32603: EVM error InvalidFEOpcode
    let tx_hash = contract.function("transfer", &[DynSolValue::from(recepient), number_value])?.send().await?.watch().await?;

    println!("Done...{tx_hash}");

    let rpc_url = "https://eth.merkle.io";
    let forked_provider =
        ProviderBuilder::new().on_anvil_with_wallet_and_config(|anvil| anvil.fork(rpc_url))?;
    let contract = ContractInstance::new(contract_address, forked_provider, Interface::new(transfer_abi));

    // this is alice address
    let alice_balance = contract.function("balanceOf", &[DynSolValue::from(alice)])?.call().await?;
    dbg!(alice_balance);

    // this call only works on provider that is forked from mainnet? eth.
    let balance: Vec<DynSolValue> = contract.function("totalSupply", &[])?.call().await?;
    dbg!(balance);


    let symbol: Vec<DynSolValue> = contract.function("symbol", &[])?.call().await?;
    dbg!(symbol);

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
        .with_gas_price(20_000_000_000)
        .with_gas_limit(21_000)
        .with_value(amount);
    let dest_balance = provider.get_balance(dest_addr).await;
    dbg!(dest_balance);
    let source_balance = provider.get_balance(source_addr).await;
    dbg!(source_balance);

    let builder = provider.send_transaction(tx.clone()).await?;
    let node_hash = *builder.tx_hash();
    let receipt = builder.get_receipt().await?;
    dbg!(receipt);
    let pending_tx =
        provider.get_transaction_by_hash(node_hash).await?.expect("Pending transaction not found");

    println!("Transaction sent with nonce: {}", pending_tx.nonce());

    let dest_after_balance = provider.get_balance(dest_addr).await;
    dbg!(dest_after_balance);
    let source_after_balance = provider.get_balance(source_addr).await;
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
        .with_gas_price(20_000_000_000)
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


    let dest_after_balance = provider.get_balance(dest_addr).await;
    dbg!(dest_after_balance);
    let source_after_balance = provider.get_balance(source_addr).await;
    dbg!(source_after_balance);


    Ok(())
}
