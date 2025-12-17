use pethit_consensus::Miner;
use pethit_rpc::start_server;
use pethit_storage::SharedStorage;
use pethit_txpool::SharedTxPool;

#[tokio::main] // turns `main` into an async function
async fn main() {
    println!("Starting pETHit node...");

    // Start the shared components
    let shared_storage = SharedStorage::new();
    let shared_txpool = SharedTxPool::new();

    // Setup the Miner
    let miner_txpool = shared_storage.clone();
    let miner_storage = shared_txpool.clone();

    // Launch the Miner in the background
    // `tokio::task::spawn_blocking` is used because the Miner uses `thread::sleep`, which shouldn't block the async executor.
    tokio::task::spawn_blocking(move || {
        let miner = Miner::new(miner_storage, miner_txpool);
        miner.start_mining();
    });

    // Start the RPC server. Pause here until the server stops (never)
    start_server(shared_txpool).await;
}
