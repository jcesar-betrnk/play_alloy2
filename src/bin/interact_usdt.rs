#![deny(warnings)]
//! This example demonstrates how to interact with a contract that is already deployed onchain using
//! the `ContractInstance` interface.

use alloy::{
    contract::{ContractInstance, Interface},
    dyn_abi::DynSolValue,
    primitives::{U256, address},
    providers::{ProviderBuilder},
    json_abi::JsonAbi,
};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure `anvil` is available in $PATH.
    let provider = ProviderBuilder::new().on_anvil_with_wallet();


    let contract_address = address!("dAC17F958D2ee523a2206206994597C13D831ec7");

    println!("parsing abi..");
    let transfer_abi = JsonAbi::parse([
        "function transfer(address recipient, uint256 amount) public returns (bool)",
        "function totalSupply() view returns (uint256)",
    ])?;
    println!("Done parsing abi..");

    // Create a new `ContractInstance` of the `Counter` contract from the abi
    let contract = ContractInstance::new(contract_address, provider.clone(), Interface::new(transfer_abi.clone()));

    // Set the number to 42.
    let number_value = DynSolValue::from(U256::from(42));
    let recepient = alloy::primitives::Address::random();

    // this is alice address
    //let recepient = address!("70997970C51812dc3A010C7d01b50e0d17dc79C8");

    // this call works on anvil, but not on the anvil forked
    // if called on the anvil fork provider: error code -32603: EVM error InvalidFEOpcode
    let tx_hash = contract.function("transfer", &[DynSolValue::from(recepient), number_value])?.send().await?.watch().await?;

    println!("Done...{tx_hash}");

    let rpc_url = "https://eth.merkle.io";
    let forked_provider =
        ProviderBuilder::new().on_anvil_with_wallet_and_config(|anvil| anvil.fork(rpc_url))?;
    let contract = ContractInstance::new(contract_address, forked_provider, Interface::new(transfer_abi));
    // this call only works on provider that is forked from mainnet? eth.
    let balance: Vec<DynSolValue> = contract.function("totalSupply", &[])?.call().await?;
    dbg!(balance);


    Ok(())
}
