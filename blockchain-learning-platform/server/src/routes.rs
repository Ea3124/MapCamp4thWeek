// server/src/routes.rs

use axum::{
    Router, 
    routing::{post, get}, 
    extract::Extension, 
    Json
};
use std::sync::Arc;
use tokio::sync::{broadcast::Sender, mpsc::Sender as MpscSender, Mutex};

use crate::models::{Block, Problem, ValidationResult, Transaction};
use crate::handlers::my_broadcast::{self, Server};

pub fn create_routes(
    tx: Arc<Sender<Block>>,
    problem_tx: Arc<Sender<Problem>>,
    validation_sender: MpscSender<ValidationResult>,
    server: Arc<Mutex<Server>>, // 거래 핸들러에서 서버 상태 접근용
) -> Router {
    Router::new()
        // 문제 브로드캐스트
        .route(
            "/broadcast_problem",
            get(my_broadcast::broadcast_problem),
        )
        .layer(Extension(problem_tx.clone()))

        // 블록 제출
        .route(
            "/submit_block",
            post({
                let tx = Arc::clone(&tx);
                let validation_sender = validation_sender.clone();
                move |Json(block): Json<Block>| async move {
                    my_broadcast::handle_block_submission(
                        Json(block),
                        Extension(Arc::clone(&tx)),
                        Extension(validation_sender.clone()),
                    )
                    .await;
                    "Block submitted"
                }
            }),
        )
        .layer(Extension(Arc::clone(&tx)))

        // ** 거래(트랜잭션) 제출 경로 추가 **
        .route(
            "/transaction",
            post({
                let server_clone = Arc::clone(&server);
                move |Json(transaction): Json<Transaction>| async move {
                    my_broadcast::handle_transaction(
                        Json(transaction),
                        Extension(server_clone.clone()),
                    )
                    .await
                }
            }),
        )

        // 기본 경로
        .route("/", get(|| async { "Hello, World!" }))
}
