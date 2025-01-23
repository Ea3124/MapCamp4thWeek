// server/src/main.rs

use axum::{Router};
use axum::extract::Extension;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::task;
use tower::{ServiceBuilder};
use tower::limit::ConcurrencyLimitLayer;

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
    let problem_tx = Arc::new(problem_tx);

    // --------------------------
    // 2) Block 전용 채널 생성
    // --------------------------
    let (block_tx, _block_rx) = broadcast::channel::<String>(100); // String 타입으로 변경
    let block_tx = Arc::new(block_tx);

    // --------------------------
    // 3-2) Validation 채널 생성
    // --------------------------
    let (validation_tx, validation_rx) = mpsc::channel::<models::ValidationResult>(100);

    // ------------------------------------
    // 4) 서버(합의/거래 흐름 관리) 구조체 생성
    // ------------------------------------
    let server = handlers::my_broadcast::Server::new(100, validation_tx.clone());
    let server = Arc::new(Mutex::new(server));

    // ----------------------------
    // 5) 검증 결과를 처리하는 태스크
    // ----------------------------
    let server_clone_for_validation = Arc::clone(&server);
    let problem_tx_for_validation = Arc::clone(&problem_tx);
    task::spawn(async move {
        handle_validation_results(server_clone_for_validation, validation_rx, problem_tx_for_validation).await;
    });

    // ----------------------------
    // 6) 라우터 생성 및 서버 시작
    // ----------------------------
    let app: Router = routes::create_routes(
        Arc::clone(&block_tx),
        Arc::clone(&problem_tx),
        validation_tx.clone(), // 동일한 validation_tx를 전달
        Arc::clone(&server),
    )
    // 동시 30개 요청 처리 제한
    .layer(ServiceBuilder::new().layer(ConcurrencyLimitLayer::new(30)))
    // 추가로 필요한 Extension 주입
    .layer(Extension(Arc::clone(&block_tx)))
    .layer(Extension(Arc::clone(&problem_tx)))
    .layer(Extension(Arc::clone(&server)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// 검증 결과를 처리하는 비동기 함수
async fn handle_validation_results(
    server: Arc<Mutex<handlers::my_broadcast::Server>>,
    mut validation_rx: mpsc::Receiver<models::ValidationResult>,
    problem_tx: Arc<broadcast::Sender<models::Problem>>,
) {
    while let Some(validation_result) = validation_rx.recv().await {
        println!(
            "Received validation result from node {}: {:?}",
            validation_result.node_id, validation_result.is_valid
        );
        // 서버 락 획득 후 합의 로직 처리
        let mut server_guard = server.lock().await;
        server_guard.process_consensus(validation_result, Arc::clone(&problem_tx)).await;
    }
    eprintln!("Validation receiver dropped");
}
