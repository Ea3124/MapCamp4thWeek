use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

use crate::models::Block;


pub async fn handle_broadcast(
    Json(block): Json<Block>,
    Extension(tx): Extension<Arc<Sender<Block>>>,
) -> impl IntoResponse {
    if let Err(e) = tx.send(block) {
        eprintln!("Failed to broadcast block: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to broadcast block");
    }
    (StatusCode::OK, "Block broadcasted successfully")
}
