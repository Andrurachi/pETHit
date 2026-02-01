use alloy_primitives::B256;
use alloy_rlp::Decodable;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use pethit_consensus::SharedChain;
use pethit_execution::SignedTransaction;
use pethit_txpool::SharedTxPool;
use serde::Deserialize;
use std::net::SocketAddr;
use std::str::FromStr;

// Raw hex the wallet sends
#[derive(Deserialize)]
struct PutTransactionRequest {
    pub raw_tx: String,
}

// #[derive(Deserialize)]
// struct GetTransactionRequest {
//     hash: String,
// }

#[derive(Deserialize)]
struct GetBlockRequest {
    hash: String,
}

#[derive(Clone)]
struct AppState {
    txpool: SharedTxPool,
    chain: SharedChain,
}

// Handler
// This function runs when someone hits the POST /send_tx endpoint.
async fn send_transaction(
    State(state): State<AppState>,
    Json(payload): Json<PutTransactionRequest>,
) -> String {
    // Strip "0x" and Decode Hex
    let hex_data = payload.raw_tx.strip_prefix("0x").unwrap_or(&payload.raw_tx);

    let rlp_bytes = match hex::decode(hex_data) {
        Ok(b) => b,
        Err(_) => return "Error: Invalid Hex string".to_string(),
    };

    // Decode RLP to SignedTransaction
    let sig_tx = match SignedTransaction::decode(&mut rlp_bytes.as_slice()) {
        Ok(tx) => tx,
        Err(e) => return format!("Error decoding RLP: {}", e),
    };

    // Calculate hash and add to the pool
    let tx_hash = alloy_primitives::keccak256(&rlp_bytes);

    if let Err(e) = state.txpool.add(tx_hash, sig_tx) {
        return format!("Error adding to the pool: {}", e);
    }

    // Reply to the user
    println!("\n Added to pool tx with hash={:?}", tx_hash);
    "Transaction received!".to_string()
}

// TODO: Implement get_account by address

// TODO: Refactor get_tx so it is searched in the chain, not in the storage.
// Probably will require a new block method to return a tx given the hash. Is there a fast way to get a tx?
// Probably this will be implemented in iteration 4 after the block history is part of the db, not some random chain variable
// // Handler
// // This function runs when someone hits the GET /get_tx endpoint.
// async fn get_transaction(
//     State(state): State<AppState>,
//     Json(payload): Json<GetTransactionRequest>,
// ) -> String {
//     let tx_hash = payload.hash.into_bytes();

//     // Get it from the shared storage
//     let tx = match state.storage.get(&tx_hash) {
//         Some(tx) => tx,
//         None => {
//             return "Error getting transaction from hash".to_string();
//         }
//     };

//     // Reply to the user
//     println!(
//         "Got transaction with that hash. Sent to={:?} with value={:?}",
//         String::from_utf8_lossy(&tx.to),
//         String::from_utf8_lossy(&tx.value)
//     );
//     "Transaction retrieved and printed!".to_string()
// }

// Handler
// This function runs when someone hits the GET /get_block endpoint.
async fn get_block_by_hash(
    State(state): State<AppState>,
    Json(payload): Json<GetBlockRequest>,
) -> String {
    let hash = match B256::from_str(&payload.hash) {
        Ok(hash) => hash,
        Err(_) => return "Invalid hash format".to_string(),
    };

    // Get it from the shared blockchain
    let block = match state.chain.get_block_by_hash(hash) {
        Some(block) => block,
        None => {
            return "Error getting block".to_string();
        }
    };

    // Reply to the user
    format!(
        "Found Block!\nNumber: {}\nHash: {}\nParent: {}\nTxs: {} \n",
        block.id,
        block.k_hash,
        block.parent_hash,
        block.transactions.len()
    )
}

// The Server Builder
pub async fn start_server(txpool: SharedTxPool, chain: SharedChain) {
    // Create the state object
    let state = AppState { txpool, chain };

    // Build the router and inject the state
    let app = Router::new()
        .route("/send_tx", post(send_transaction))
        //.route("/get_tx", get(get_transaction))
        .route("/get_block", get(get_block_by_hash))
        .with_state(state);

    // Define the address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("RPC Server listening on {}", addr);

    //start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
