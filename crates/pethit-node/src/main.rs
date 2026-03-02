use std::fs;
use std::str::FromStr;
use alloy_primitives::{Address, U256};
use pethit_consensus::{Miner, SharedChain};
use pethit_rpc::start_server;
use pethit_storage::{SharedStorage, Account};
use pethit_txpool::SharedTxPool;

// Helper to load genesis state
fn load_genesis_state(storage: &SharedStorage) {
    // Read Json file
    let genesis_str = fs::read_to_string("genesis.json").expect("Failed to read genesis.json. Make sure it exists in the workspace root.");
    let genesis_json: serde_json::Value = serde_json::from_str(&genesis_str).expect("Invalid JSON in genesis.json");

    // Parse the alloc object
    let alloc = genesis_json["alloc"].as_object().expect("Missing 'alloc' object in genesis");

    // Insert each account into storage
    for (addr_str, account_data) in alloc {
        let address = Address::from_str(addr_str).expect("Invalid address format in genesis");

        // Parse balance
        let balance_str = account_data["balance"].as_str().expect("Balance must be a string");
        let balance = U256::from_str(balance_str).expect("Invalid balance format");
        
        let account = Account {
            nonce: 0,
            balance,
        };

        storage.set_account(address, account);
        println!("Funded {} with {} Wei", address, balance);
    }
}


#[tokio::main] // turns `main` into an async function
async fn main() {
    println!("Starting pETHit node...");

    // Start the shared components
    let shared_storage = SharedStorage::new();
    let shared_txpool = SharedTxPool::new();
    let shared_chain = SharedChain::new();

    // Load genesis (only in storage. Will be added to the chain after implementing MPT)
    load_genesis_state(&shared_storage);

    // Setup the Miner
    let miner_txpool = shared_storage.clone();
    let miner_storage = shared_txpool.clone();
    let miner_chain = shared_chain.clone();

    // Launch the Miner in the background
    // `tokio::task::spawn_blocking` is used because the Miner uses `thread::sleep`, which shouldn't block the async executor.
    tokio::task::spawn_blocking(move || {
        let miner = Miner::new(miner_storage, miner_txpool, miner_chain);
        miner.start_mining();
    });

    // Start the RPC server. Pause here until the server stops (never)
    start_server(shared_storage, shared_txpool, shared_chain).await;
}
