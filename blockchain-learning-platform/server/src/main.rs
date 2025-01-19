// server/src/main.rs

use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;

mod models;
mod routes;
mod handlers {
    pub mod my_broadcast;
}

#[tokio::main]
async fn main() {
    // broadcast 채널 생성 (Block 타입, capacity=100)
    let (tx, _rx) = broadcast::channel::<models::Block>(100);

    // 여러 핸들러에서 사용할 수 있도록 Arc로 감싸기
    let tx = Arc::new(tx);

    // 라우터 생성
    let app: Router = routes::create_routes(tx);

    // 서버 시작
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
