use alloy_primitives::{Address, U256};
use alloy_rlp::Encodable;
use clap::{Parser, Subcommand};
use k256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
use pethit_execution::{SignedTransaction, Transaction};
use std::str::FromStr;

/// Pethit Wallet CLI
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new random private key and address
    Generate,
    /// Send a transaction
    Send {
        /// Private key in hex format without 0x prefix
        #[arg(long)]
        private_key: String,
        /// Receiver address (0x)
        #[arg(long)]
        to: String,
        /// Amount to send
        #[arg(long)]
        value: u64,
        /// RPC URL
        #[arg(long, default_value = "http://127.0.0.1:8000")]
        rpc: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate => {
            generate_wallet();
        }
        Commands::Send {
            private_key,
            to,
            value,
            rpc,
        } => {
            send_transaction(private_key, to, value, rpc).await?;
        }
    }
    Ok(())
}

fn generate_wallet() {
    // Generate a private random key
    let signing_key = SigningKey::random(&mut OsRng);
    let secret_bytes = signing_key.to_bytes();

    // Derive address
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_encoded_point(false); // uncompressed
    let public_key_slice = &public_key_bytes.as_bytes()[1..]; // Remove first byte (0x04)
    let hash = alloy_primitives::keccak256(public_key_slice);
    let address = Address::from_slice(&hash[12..]); // Last 20 bytes

    println!("New Wallet Generated:");
    println!("Private Key: {}", hex::encode(secret_bytes));
    println!("Address:     {}", address);
    println!("SAVE THIS PRIVATE KEY! IT WILL NOT BE SHOWN AGAIN.");
}

async fn send_transaction(
    private_key_hex: String,
    to_str: String,
    value: u64,
    rpc_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let priv_key_bytes = hex::decode(private_key_hex)?;

    // Load signer from raw bytes
    let signer = SigningKey::from_slice(&priv_key_bytes)?;
    let verifying_key = signer.verifying_key();

    // Derive the address to check nonce
    let public_key_bytes = verifying_key.to_encoded_point(false);
    let hash = alloy_primitives::keccak256(&public_key_bytes.as_bytes()[1..]);
    let from_address = Address::from_slice(&hash[12..]);

    println!("Sending from: {}", from_address);

    // Get nonce from RPC
    let nonce = fetch_nonce(&rpc_url, from_address).await.unwrap_or(0);
    println!("  Nonce: {}", nonce);

    // Create transaction
    let tx = Transaction {
        to: Address::from_str(&to_str)?,
        value: U256::from(value),
        nonce,
    };

    // Sign transaction
    let tx_hash = tx.hash();
    let (signature, recid) = signer.sign_prehash_recoverable(tx_hash.as_slice())?;
    let signed_tx = SignedTransaction {
        transaction: tx,
        signature,
        recovery_id: recid,
    };

    // Encode to RLP
    let mut rlp_bytes = Vec::new();
    signed_tx.encode(&mut rlp_bytes);
    let tx_hex = hex::encode(rlp_bytes);

    // Send tx to RPC
    let send_tx_url = format!("{}/send_tx", rpc_url);
    let client = reqwest::Client::new();
    let response = client
        .post(&send_tx_url)
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "raw_tx": tx_hex
        }))
        .send()
        .await?;

    let response_text = response.text().await?;
    println!("Response: {}", response_text);

    Ok(())
}

// Helper to fetch nonce
async fn fetch_nonce(_rpc_url: &str, _address: Address) -> Result<u64, Box<dyn std::error::Error>> {
    // TODO: Implement actual RPC call here
    Ok(0)
}
