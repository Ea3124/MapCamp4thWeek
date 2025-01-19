use axum::{Router, routing::{post, get}, extract::Extension, Json};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

use crate::models::Block;
use crate::handlers::my_broadcast;

pub fn create_routes(tx: Arc<Sender<Block>>) -> Router {
    Router::new()
        .route(
            "/broadcast",
            post({
                let tx = tx.clone();
                move |Json(block): Json<Block>| {
                    my_broadcast::handle_broadcast(Json(block), Extension(tx.clone()))
                }
            }),
        )
        .route("/",get(|| async { "Hello, World!" }))
}
