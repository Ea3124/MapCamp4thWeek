// server/src/routes.rs

use axum::{
    Router, 
    routing::{post, get}, 
    extract::Extension, 
    Json, 
    response::IntoResponse,
};
use hyper::StatusCode;
use std::sync::Arc;
use tokio::sync::{broadcast::Sender, mpsc::Sender as MpscSender, Mutex};

use crate::models::{Block, Problem, ValidationResult, Transaction};
use crate::handlers::my_broadcast::{self, Server};

// 추가: WebSocket 관련 import
use axum::routing::get as axum_get;
use crate::handlers::my_broadcast::handle_websocket;

pub fn create_routes(
    tx: Arc<Sender<String>>,
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
                        // Extension(validation_sender.clone()), // 사용하지 않으므로 제거
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

        // ** 검증 결과 제출 경로 추가 **
        // 검증 결과 제출 경로 추가
        .route(
            "/submit_validation",
            post({
                let validation_sender = validation_sender.clone();
                move |Json(validation_result): Json<ValidationResult>| async move {
                    if let Err(e) = validation_sender.send(validation_result).await {
                        eprintln!("Failed to send validation result: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to submit validation result");
                    }
                    (StatusCode::OK, "Validation result submitted successfully")
                }
            }),
        )

        // ** WebSocket 라우트 추가 **
        .route(
            "/ws",
            axum_get({
                let problem_tx = Arc::clone(&problem_tx);
                let block_tx = Arc::clone(&tx);  //string  변경
                move |ws: axum::extract::ws::WebSocketUpgrade| {
                    handle_websocket(ws, problem_tx.clone(), block_tx.clone())
                }
            }),
        )

        // 기본 경로
        .route("/", get(|| async { "Hello, World!" }))
}
