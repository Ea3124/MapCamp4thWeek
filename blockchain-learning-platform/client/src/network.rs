// client/src/network.rs

use crate::blockchain::blockchain_db::Problem; // blockchain_db.rs에서 가져옴
use crate::Block;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::error::Error;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use tokio_tungstenite::connect_async;
use url::Url;
use serde_json::Value;
use futures::StreamExt;

// 서버와 동일하게 맞춰줄 임시 구조체 (서버의 Block 구조체에 매칭)
#[derive(Serialize)]
pub struct BlockForServer {
    pub index: u64,
    pub timestamp: String,
    pub solution: Vec<Vec<u32>>,     // 숫자 배열
    pub problem: Problem,      // 숫자 배열
    pub prev_solution: Vec<Vec<u32>>,
    pub node_id: String,
    pub data: String,
}



#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    Problem(Problem),
    Block(Block),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub node_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub sender_id: String,
    pub receiver_id: String,
    pub amount: u32,
}

/// 실제로 서버에 POST `/submit_block` 요청을 보내는 함수
pub async fn submit_solution_block(
    server_url: &str,
    block_data: &BlockForServer,
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
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

/// 서버에 검증 결과를 제출하는 함수
pub async fn submit_validation_result(
    server_url: &str,
    validation_result: &ValidationResult,
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = format!("{}/submit_validation", server_url);

    let resp = client
        .post(&url)
        .json(validation_result)
        .send()
        .await?
        .error_for_status()?; // 4xx, 5xx 에러 시 Result Err 로 변환

    println!("Validation result response: {:?}", resp.text().await?);
    Ok(())
}

/// 서버에 거래를 제출하는 함수
pub async fn submit_transaction(
    server_url: &str,
    transaction: &Transaction,
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = format!("{}/transaction", server_url);

    let resp = client
        .post(&url)
        .json(transaction)
        .send()
        .await?
        .error_for_status()?; // 4xx, 5xx 에러 시 Result Err 로 변환

    println!("Transaction response: {:?}", resp.text().await?);
    Ok(())
}
/// WebSocket을 통해 서버와 연결하고 메시지를 수신하는 함수
pub async fn connect_to_websocket(
    server_url: &str,
    sender: UnboundedSender<ServerMessage>,
) -> Result<(), Box<dyn Error>> {
    let ws_url = format!("ws://{}/ws", server_url.trim_start_matches("http://").trim_start_matches("https://"));
    let url = Url::parse(&ws_url)?;

    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connected to {}", ws_url);

    let (mut write, mut read) = ws_stream.split();

    // 수신 태스크
    tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => {
                    if msg.is_text() {
                        let text = msg.into_text().unwrap();

                        // 원본 메시지 로깅
                        println!("Received raw message: {}", text);

                        // JSON 데이터 역직렬화
                        match serde_json::from_str::<Value>(&text) {
                            Ok(json_value) => {
                                if let Some(msg_type) = json_value.get("type").and_then(|v| v.as_str()) {
                                    match msg_type {
                                        "block" => {
                                            if let Some(data) = json_value.get("data") {
                                                match serde_json::from_value::<Value>(data.clone()) {
                                                    Ok(debug_data) => {
                                                        println!("Debug Data: {:?}", debug_data); // 문제 확인용 디버깅
                                                        match serde_json::from_value::<Block>(data.clone()) {
                                                            Ok(block) => {
                                                                println!("Parsed Block: {:?}", block);
                                                                if let Err(e) = sender.send(ServerMessage::Block(block)) {
                                                                    eprintln!("Failed to send Block to UI: {}", e);
                                                                }
                                                            }
                                                            Err(e) => eprintln!("Failed to parse Block: {}", e),
                                                        }
                                                    }
                                                    Err(e) => eprintln!("Failed to debug Block data: {}", e),
                                                }
                                            } else {
                                                eprintln!("Missing 'data' field for Block");
                                            }
                                        }
                                        "problem" => {
                                            if let Some(data) = json_value.get("data") {
                                                match serde_json::from_value::<Problem>(data.clone()) {
                                                    Ok(problem) => {
                                                        println!("Parsed Problem: {:?}", problem);
                                                        if let Err(e) = sender.send(ServerMessage::Problem(problem)) {
                                                            eprintln!("Failed to send Problem to UI: {}", e);
                                                        }
                                                    }
                                                    Err(e) => eprintln!("Failed to parse Problem: {}", e),
                                                }
                                            } else {
                                                eprintln!("Missing 'data' field for Problem");
                                            }
                                        }
                                        _ => eprintln!("Unknown message type: {}", msg_type),
                                    }
                                } else {
                                    eprintln!("Missing 'type' field in message");
                                }
                            }
                            Err(e) => eprintln!("Failed to parse raw message as JSON: {}", e),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    Ok(())
}

