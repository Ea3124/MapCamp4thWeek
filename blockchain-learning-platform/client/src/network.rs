// client/src/network.rs

use crate::blockchain::Block;
use reqwest::Client;
use serde::Serialize;
use std::error::Error;

// 서버와 동일하게 맞춰줄 임시 구조체 (서버의 Block 구조체에 매칭)
#[derive(Serialize)]
pub struct BlockForServer {
    pub index: u64,
    pub timestamp: String,
    pub solution: Vec<Vec<u32>>,
    pub hash: String,
    pub prev_hash: String,
    pub node_id: String,
    // 문제(Problem) 같은 필드도 필요하다면 추가하세요.
    // pub problem: ProblemForServer, etc...
}

/// 실제로 서버에 POST `/submit_block` 요청을 보내는 함수
pub async fn submit_solution_block(
    server_url: &str,
    block_data: &BlockForServer,
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    // 서버 라우트와 동일한 경로 사용 (예: /submit_block)
    let url = format!("{}/submit_block", server_url);

    let resp = client
        .post(&url)
        .json(block_data)
        .send()
        .await?
        .error_for_status()?; // 4xx, 5xx 에러 시 Result Err 로 변환

    println!("Server response: {:?}", resp.text().await?);
    Ok(())
}
