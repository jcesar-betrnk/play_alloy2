use alloy::{
    network::Ethereum,
    primitives::{Address, U256},
    rpc::{client::RpcClient, transport::HttpTransport},
    signers::{LocalWallet, Signer},
    transactions::eip2718::TxEnvelope,
    transactions::TransactionBuilder,
};
use std::env;
use tokio;
use anyhow::Result;

const USDT_CONTRACT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7"; // USDT Mainnet Address
const TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb]; // ERC-20 transfer function selector

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Load private key from environment variable
    let private_key = env::var("PRIVATE_KEY")?;
    let wallet: LocalWallet<Ethereum> = private_key.parse()?;

    // Connect to an Ethereum RPC node
    let rpc_url = env::var("RPC_URL")?; // Example: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID"
    let transport = HttpTransport::new(&rpc_url)?;
    let client = RpcClient::new(transport);

    // Define recipient and amount
    let recipient: Address = "0xRecipientAddressHere".parse()?; // Replace with actual recipient
    let amount = U256::from(1_000_000); // 1 USDT (USDT has 6 decimals)

    // Encode function call data
    let mut call_data = Vec::with_capacity(68);
    call_data.extend_from_slice(&TRANSFER_SELECTOR);
    call_data.extend_from_slice(&recipient.0);
    call_data.extend_from_slice(&amount.to_be_bytes());

    // Get the wallet address
    let sender = wallet.address();

    // Get nonce for the sender
    let nonce: U256 = client.get_transaction_count(sender, None).await?;

    // Get gas price
    let gas_price: U256 = client.get_gas_price().await?;

    // Build transaction
    let tx = TransactionBuilder::new()
        .to(USDT_CONTRACT.parse::<Address>()?)
        .value(U256::ZERO)
        .gas_price(gas_price)
        .gas_limit(U256::from(100_000)) // Set a reasonable gas limit
        .nonce(nonce)
        .data(call_data)
        .chain_id(1) // Mainnet Chain ID
        .build();

    // Sign transaction
    let signed_tx: TxEnvelope = wallet.sign_transaction_sync(&tx)?.into();

    // Send transaction
    let tx_hash = client.send_raw_transaction(signed_tx).await?;

    println!("Transaction sent! Hash: {tx_hash:?}");

    Ok(())
}
