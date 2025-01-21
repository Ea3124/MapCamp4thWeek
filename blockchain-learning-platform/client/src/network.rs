// client/src/network.rs

use crate::Block;
use reqwest::Client;
use std::error::Error;
use tokio::sync::mpsc;
use reqwest_eventsource::{EventSource, Event};

use serde::{Deserialize, Serialize}; // Deserialize를 위해

use iced::futures::StreamExt;


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

#[derive(Debug, Deserialize)]
pub struct BlockFromServer {
    pub index: u64,
    pub timestamp: String,
    pub solution: Vec<Vec<u32>>,
    pub hash: String,
    pub prev_hash: String,
    pub node_id: String,
    // 서버 Block 필드에 맞춰 필요하면 추가
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

pub async fn listen_blocks_sse(
    server_url: &str,
    mut block_sender: mpsc::Sender<BlockFromServer>, 
) -> Result<(), Box<dyn Error>> {
    // SSE Endpoint
    let url = format!("{}/blocks_sse", server_url);
    let client = Client::new();

    let mut es = EventSource::new(client.get(&url))?;
    println!("SSE connection opened to {}", url);

    while let Some(event) = es.next().await {
        match event {
            Ok(Event::Message(msg)) => {
                // SSE data 필드가 block의 JSON 문자열
                let data = msg.data;
                // JSON 파싱
                if let Ok(block) = serde_json::from_str::<BlockFromServer>(&data) {
                    println!("Received new block via SSE: {:?}", block);

                    // 만약 다른 로직(iced 메시지 전송 등)이 필요하면 channel로 보낸다거나
                    if let Err(e) = block_sender.try_send(block) {
                        eprintln!("Failed to forward block: {}", e);
                    }
                }
            },
            Ok(Event::Open) => {
                println!("SSE connection is now open.");
            },
            Ok(other_event) => {
                println!("Unhandled SSE event: {:?}", other_event);
            }
            // Ok(Event::Retry) => {
                // println!("Server sent a retry signal.");
            // }
            Err(e) => {
                eprintln!("SSE error: {:?}", e);
                // 연결이 끊기거나 에러 발생 시 재시도 로직 etc.
                break;
            }
        }
    }

    println!("SSE connection closed.");
    Ok(())
}
