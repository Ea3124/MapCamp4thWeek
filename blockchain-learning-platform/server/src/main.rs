// server/src/main.rs

use axum::Router;
use axum::extract::Extension;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, mpsc};
use tokio::task;

mod models;
mod routes;
mod handlers {
    pub mod my_broadcast;
}

#[tokio::main]
async fn main() {
        // --------------------------
    // 1) Problem 전용 채널 생성
    // --------------------------
    let (problem_tx, _problem_rx) = broadcast::channel::<models::Problem>(100);

    // --------------------------
    // 2) Block 전용 채널 생성
    // --------------------------
    let (block_tx, _block_rx) = broadcast::channel::<models::Block>(100);
    // Arc 로 감싸기
    let block_tx = Arc::new(block_tx);

    // --------------------------
    // 3) Validation 채널 생성
    // --------------------------
    let (validation_tx, validation_rx) = mpsc::channel::<models::ValidationResult>(100);

    // ------------------------------------------------------------------
    // (선택) 서버 구조체 초기화 - your custom Server / Sender (if needed)
    // ------------------------------------------------------------------
    // 예: my_broadcast.rs 안에 Sender::new(capacity: usize) -> (Server, MpscSender<ValidationResult>)
    let (server, validation_sender) = handlers::my_broadcast::Server::new(100);

    // -------------------------------------------------------
    // 4) 검증 결과를 처리하는 비동기 태스크 (합의 로직 등 수행)
    // -------------------------------------------------------
    task::spawn(handle_validation_results(server, validation_rx));

    // 라우터 생성
    // route table보고 어떻게 broadcast할지 설정
    let app: Router = routes::create_routes(block_tx.clone(), validation_sender);

    // 서버 시작
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// 검증 결과를 처리하는 비동기 함수
async fn handle_validation_results(
    mut server: handlers::my_broadcast::Server,
    mut validation_rx: mpsc::Receiver<models::ValidationResult>,
) {
    while let Some(validation_result) = validation_rx.recv().await {
        // 검증 결과 처리
        println!(
            "Received validation result from node {}: {:?}",
            validation_result.node_id, validation_result.is_valid
        );
        
        // 합의 로직 수행
        server.process_consensus(validation_result).await;
    }
}