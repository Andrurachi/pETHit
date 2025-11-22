use axum::{Json, Router, routing::post};
use pethit_execution::Transaction;
use serde::Deserialize;
use std::net::SocketAddr;

// Data transfer Object
#[derive(Deserialize)]
struct TransactionRequest {
    key: String,
    value: String,
}

// Handler
// This function runs when someone hits the POST /send_tx endpoint.
async fn send_transaction(Json(payload): Json<TransactionRequest>) -> String {
    let tx = Transaction {
        key: payload.key.into_bytes(),
        value: payload.value.into_bytes(),
    };

    // Add it to mempool (just print it for now).
    // Use {:?} so we need Debug on Transaction
    println!("Received tx: Key={:?}, Value={:?}", tx.key, tx.value);

    // Reply to the user
    "Transaction received and printed!".to_string()
}

// The Server Builder
pub async fn start_server() {
    // Define routes
    let app = Router::new().route("/send_tx", post(send_transaction));

    // Define the address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("RPC Server listening on {}", addr);

    //start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
