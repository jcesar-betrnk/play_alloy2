#![deny(warnings)]
//! This example demonstrates how to interact with a contract that is already deployed onchain using
//! the `ContractInstance` interface.

use alloy::{
    contract::{ContractInstance, Interface},
    dyn_abi::DynSolValue,
    primitives::{U256, address},
    providers::{ProviderBuilder},
    json_abi::JsonAbi,
    signers::local::LocalSigner,
};
use alloy::signers::local::{coins_bip39::English, MnemonicBuilder};
use alloy::providers::Provider;
use eyre::Result;
use k256::{
    ecdsa::{SigningKey},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure `anvil` is available in $PATH.
    let provider = ProviderBuilder::new().on_anvil_with_wallet();
    let gas_price = provider.get_gas_price().await?;
    dbg!(gas_price);


    let alice = address!("0x70997970C51812dc3A010C7d01b50e0d17dc79C8");

    // code: https://etherscan.io/address/0xdac17f958d2ee523a2206206994597c13d831ec7#code
    let contract_address = address!("0xdAC17F958D2ee523a2206206994597C13D831ec7");

    println!("parsing abi..");
    let usdt_abi = JsonAbi::parse([
        "function transfer(address recipient, uint256 amount) public returns (bool)",
        "function transferFrom2(address _from, address _to, uint _value) public returns (bool)",
        "function totalSupply() view returns (uint256)",
        "function balanceOf(address who) returns (uint256)",
        "function symbol() public view returns (string memory)",
    ])?;
    println!("Done parsing abi..");

    // Create a new `ContractInstance` of the `Counter` contract from the abi
    let contract = ContractInstance::new(contract_address, provider.clone(), Interface::new(usdt_abi.clone()));


    let phrase = "work man brother plunge mystery proud hollow address reunion sauce theory bonus";
    let index = 0u32;
    let password = "pwd123";

    let created_wallet = create_wallet(index, phrase, password)?;
    dbg!(&created_wallet);

    // Set the number to 42.
    let number_value = DynSolValue::from(U256::from(42_000_000));

    let phrase2 = "work man sister plunge mystery proud hollow address reunion sauce theory bonus";
    let index2 = 0u32;
    let password2 = "pass123";

    let created_wallet2 = create_wallet(index2, phrase2, password2)?;
    dbg!(&created_wallet2);

    // this call works on anvil, but not on the anvil forked
    // if called on the anvil fork provider: error code -32603: EVM error InvalidFEOpcode
    let tx_hash = contract.function("transfer", &[DynSolValue::from(created_wallet2.address()), number_value.clone()])?.send().await?.watch().await?;
    println!("Done simple transfer...{tx_hash}");

    let tx_hash = contract.function("transferFrom2", &[DynSolValue::from(created_wallet2.address()), DynSolValue::from(created_wallet.address()), number_value])?.send().await?.watch().await?;
    println!("Done calling transferFrom...{tx_hash}");

    // query the balance of the recepient
    // ISSUE:
    // - [ ] Error: contract call to `balanceOf` returned no data ("0x"); the called address might not be a contract
    //   - Maybe because the contract is not deployed on this provider instance yet.
    //      - but then why I was able to call `transfer` function
    //          - maybe that `transfer` function is on the native eth token
    //   - [ ] Try running on an anvil fork that has advanced into a block where the USDT contract
    //   has already been deployed.
    //
    //let recepient_balance = contract.function("balanceOf", &[DynSolValue::from(created_wallet2.address())])?.call().await?;
    //dbg!(recepient_balance);

    let rpc_url = "https://eth.merkle.io";
    let forked_provider =
        ProviderBuilder::new().on_anvil_with_wallet_and_config(|anvil| anvil.fork(rpc_url))?;
    let contract = ContractInstance::new(contract_address, forked_provider, Interface::new(usdt_abi));

    // this is alice address
    let alice_balance = contract.function("balanceOf", &[DynSolValue::from(alice)])?.call().await?;
    dbg!(alice_balance);

    // query the balance of the recepient
    // returns 0, because this is a different provider. Alice and Bob preset accounts don't exist
    // here.
    // - [ ] Try connecting into the main-net and use USDT to see if it can sucessfully do a
    // transfer and query balance.
    let recepient_balance = contract.function("balanceOf", &[DynSolValue::from(created_wallet2.address())])?.call().await?;
    dbg!(recepient_balance);

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

