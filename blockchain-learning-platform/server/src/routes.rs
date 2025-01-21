use axum::{
    Router, 
    routing::{post, get}, 
    extract::Extension, 
    Json
};
use std::sync::Arc;
use tokio::sync::{broadcast::Sender, mpsc::Sender as MpscSender};

use crate::models::{Block, Problem, ValidationResult};
use crate::handlers::my_broadcast;

pub fn create_routes(
    tx: Arc<Sender<Block>>,
    problem_tx: Arc<Sender<Problem>>,
    validation_sender: MpscSender<ValidationResult>
) -> Router {
    Router::new()
        .route(
            "/broadcast_problem",
            get(
                    my_broadcast::broadcast_problem
            ),
        )
        .layer(Extension(problem_tx.clone()))
        // 블록 제출
        .route(
            "/submit_block",
            post({
                let tx = tx.clone();
                let validation_sender = validation_sender.clone();
                move |Json(block): Json<Block>| async move {
                    my_broadcast::handle_block_submission(
                        Json(block),
                        Extension(tx.clone()),
                        Extension(validation_sender.clone()),
                    )
                    .await;
                    "Block submitted"
                }
            }),
        )
        .layer(Extension(tx.clone()))
        // 기본 경로
        .route("/",get(|| async { "Hello, World!" }))
}
