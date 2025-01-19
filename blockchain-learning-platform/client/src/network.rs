// client/src/network.rs

use crate::blockchain::Block;
use reqwest::Client;
use std::error::Error;

pub async fn broadcast_block(
    server_url: &str,
    block: &Block
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = format!("{}/broadcast", server_url);

    let resp = client.post(&url)
        .json(block)
        .send()
        .await?
        .error_for_status()?;

    println!("Broadcast response: {:?}", resp.text().await?);
    Ok(())
}

// 예) 서버에서 오는 블록을 구독(웹소켓/SSE)하는 로직(선택)
// pub async fn start_ws_listener(ws_url: &str) -> Result<(), Box<dyn Error>> {
//     // 1. 웹소켓 연결
//     // 2. 메시지 수신 시 블록 역직렬화
//     // 3. 검증 후 RocksDB에 저장
//     // ...
//     unimplemented!()
// }
