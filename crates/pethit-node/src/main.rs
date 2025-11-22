use pethit_rpc::start_server;

#[tokio::main] // turns `main` into an async function
async fn main() {
    println!("Starting pETHit node...");

    // Start the RPC server. Pause here until the server stops (never)
    start_server().await;
}
