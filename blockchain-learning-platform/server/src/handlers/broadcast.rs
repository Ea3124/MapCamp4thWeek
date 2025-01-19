// server/src/broadcast.rs

use axum::{
    Json,
    extract::Extension,
    response::IntoResponse
};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

use crate::models::Block;

pub async fn handle_broadcast(
    // JSON 바디로 들어온 Block
    Json(block): Json<Block>,
    // Axum의 Extension을 통해 주입된 broadcast::Sender
    Extension(tx): Extension<Arc<Sender<Block>>>,
) -> impl IntoResponse {
    // block을 다른 subscriber(클라이언트)에게 전송
    if let Err(e) = tx.send(block) {
        eprintln!("Failed to broadcast block: {}", e);
        return "Failed to broadcast block";
    }
    "Block broadcasted successfully"
}
