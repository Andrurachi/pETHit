use alloy_primitives::{Address, B256, U256};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use k256::ecdsa::{RecoveryId, Signature};
use pethit_consensus::SharedChain;
use pethit_execution::{SignedTransaction, Transaction};
use pethit_txpool::SharedTxPool;
use serde::Deserialize;
use std::net::SocketAddr;
use std::str::FromStr;

// Data transfer Object (simple types are used (String, u8) that are guaranteed to work with Serde.)
#[derive(Deserialize)]
struct PutTransactionRequest {
    to: String,
    value: String,
    nonce: u64,
    signature: String,
    recovery_id: u8,
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
    // Parse payload address
    let to_address = match Address::from_str(&payload.to) {
        Ok(addr) => addr,
        Err(_) => return "Invalid 'to' address format".to_string(),
    };

    // Parse payload value
    let val = match U256::from_str(&payload.value) {
        Ok(v) => v,
        Err(_) => return "Invalid 'value' format".to_string(),
    };

    // Parse payload signature
    // Strip the "0x" prefix if it exists
    let sig_hex = payload
        .signature
        .strip_prefix("0x")
        .unwrap_or(&payload.signature);
    let sig_bytes = match hex::decode(sig_hex) {
        Ok(b) => b,
        Err(_) => return "Invalid signature hex".to_string(),
    };
    let signature = match Signature::try_from(sig_bytes.as_slice()) {
        Ok(s) => s,
        Err(_) => return "Invalid signature bytes".to_string(),
    };

    // Parse payload Recovery ID
    let rec_id = match RecoveryId::try_from(payload.recovery_id) {
        Ok(rec) => rec,
        Err(_) => return "Invalid 'Recovery ID' format".to_string(),
    };

    let tx = Transaction {
        to: to_address,
        value: val,
        nonce: payload.nonce,
    };

    let sig_tx = SignedTransaction {
        transaction: tx.clone(),
        signature,
        recovery_id: rec_id,
    };

    let k_hash = tx.hash();

    // Add it to the pool from the state
    if let Err(e) = state.txpool.add(k_hash, sig_tx) {
        return format!("Error adding to pool: {}", e);
    }

    // Reply to the user
    println!("\n Added to pool tx with hash={:?}", k_hash);
    "Transaction received and printed!".to_string()
}

// TODO: Refactor get_tx so it is searched in the chain, not in the storage.
// Probably will require a new block method to return a tx given the hash. Is there a fast way to get a tx?

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
