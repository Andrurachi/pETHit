use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use pethit_execution::Transaction;
use pethit_storage::SharedStorage;
use pethit_txpool::SharedTxPool;
use serde::Deserialize;
use std::net::SocketAddr;

// Data transfer Object
#[derive(Deserialize)]
struct PutTransactionRequest {
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct GetTransactionRequest {
    key: String,
}

#[derive(Clone)]
struct AppState {
    txpool: SharedTxPool,
    storage: SharedStorage,
}

// Handler
// This function runs when someone hits the POST /send_tx endpoint.
async fn send_transaction(
    State(state): State<AppState>,
    Json(payload): Json<PutTransactionRequest>,
) -> String {
    let tx = Transaction {
        key: payload.key.into_bytes(),
        value: payload.value.into_bytes(),
    };

    // Add it to the pool from the state
    if let Err(e) = state.txpool.add(tx.clone()) {
        return format!("Error adding to pool: {}", e);
    }

    // Reply to the user
    println!("Added to pool: Key={:?}", String::from_utf8_lossy(&tx.key));
    "Transaction received and printed!".to_string()
}

// Handler
// This function runs when someone hits the GET /get_tx endpoint.
async fn get_transaction(
    State(state): State<AppState>,
    Json(payload): Json<GetTransactionRequest>,
) -> String {
    let key = payload.key.into_bytes();

    // Add it to the pool from the state
    let value = match state.storage.get(&key) {
        Some(value) => value,
        None => {
            return "Error getting transaction".to_string();
        }
    };

    // Reply to the user
    println!(
        "Got transaction: Key={:?} with value={:?}",
        String::from_utf8_lossy(&key),
        String::from_utf8_lossy(&value)
    );
    "Transaction retrieved and printed!".to_string()
}

// The Server Builder
pub async fn start_server(txpool: SharedTxPool, storage: SharedStorage) {
    // Create the state object
    let state = AppState { txpool, storage };

    // Build the router and inject the state
    let app = Router::new()
        .route("/send_tx", post(send_transaction))
        .route("/get_tx", get(get_transaction))
        .with_state(state);

    // Define the address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("RPC Server listening on {}", addr);

    //start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
