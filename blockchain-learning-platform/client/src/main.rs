// client/src/main.rs

use std::error::Error;
use crate::{
    blockchain::{Block, create_block, validate_block},
    db::{init_db, store_block, get_last_block},
    network::broadcast_block,
    utils::{log_info, log_error}
};

mod blockchain;
mod db;
mod network;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. RocksDB 초기화
    let db = init_db("client_db")?;

    // 2. 마지막 블록 조회 (최초 실행 시 없을 수도 있으므로 None 처리)
    let last_block = match get_last_block(&db) {
        Some(b) => b,
        None => {
            // 초기 제네시스 블록 생성 (단순 예시)
            let genesis_block = Block {
                index: 0,
                timestamp: "2025-01-20T00:00:00Z".to_string(),
                data: "Genesis Block".to_string(),
                prev_hash: "0".to_string(),
                hash: "GENESIS_HASH".to_string(),
            };
            store_block(&db, &genesis_block)?;
            genesis_block
        }
    };

    // 3. 새 블록 생성
    let data = "My new block data";
    let new_block = create_block(&last_block, data);

    // 4. 블록 검증
    if !validate_block(&new_block, &last_block) {
        log_error("Invalid block! Aborting.");
        return Ok(());
    }

    // 5. 로컬 DB에 저장
    store_block(&db, &new_block)?;
    log_info(&format!("Stored block #{}", new_block.index));

    // 6. 서버로 브로드캐스트
    let server_url = "http://127.0.0.1:3000";
    broadcast_block(server_url, &new_block).await?;
    log_info("Block broadcasted successfully");

    // (선택) 서버 메시지 수신 (웹소켓/SSE 등)
    // network::start_ws_listener("ws://127.0.0.1:3000/subscribe").await?;

    Ok(())
}
