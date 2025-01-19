// server/src/routes.rs

use axum::{Router, routing::post};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

use crate::{
    broadcast::handle_broadcast,
    models::Block
};

pub fn create_routes(tx: Arc<Sender<Block>>) -> Router {
    Router::new()
        .route("/broadcast", post(handle_broadcast))
        // 다른 라우트가 필요하다면 .route(...)를 추가
}
