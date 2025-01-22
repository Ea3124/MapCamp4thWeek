// server/src/handlers/my_broadcast.rs

use axum::{
    extract::ws::{Message as WsMessage, WebSocket},
    extract::{Extension, Json},
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::{broadcast::Sender as BroadcastSender,broadcast::Receiver as BroadcastReceiver, mpsc::Sender as MpscSender, Mutex};
use tokio::sync::mpsc;
use serde_json::json;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::models::{Block, Problem, ServerMessage, Transaction, ValidationResult};
use std::collections::HashMap;
use std::time::Duration;

/// 4x4 마방진 생성
fn generate_random_magic_square() -> Vec<Vec<u32>> {
    let base_magic_square = vec![
        vec![16, 2, 3, 13],
        vec![5, 11, 10, 8],
        vec![9, 7, 6, 12],
        vec![4, 14, 15, 1],
    ];

    let mut rng = thread_rng();

    // 행을 랜덤하게 섞음
    let mut rows = base_magic_square.clone();
    rows.shuffle(&mut rng);

    // 열을 랜덤하게 섞음
    let mut cols: Vec<Vec<u32>> = vec![vec![0; 4]; 4];
    let col_order: Vec<usize> = (0..4).collect::<Vec<_>>().choose_multiple(&mut rng, 4).cloned().collect();

    for i in 0..4 {
        for j in 0..4 {
            cols[i][j] = rows[i][col_order[j]];
        }
    }

    cols
}


/// 특정 개수의 값을 0으로 비우는 마방진 생성
fn generate_incomplete_magic_square(num_blank: usize) -> Vec<Vec<u32>> {
    let mut magic_square = generate_random_magic_square();
    let mut rng = thread_rng();

    // 4x4 매트릭스의 인덱스를 모두 수집
    let mut positions: Vec<(usize, usize)> = (0..4).flat_map(|i| (0..4).map(move |j| (i, j))).collect();

    // 비울 인덱스를 랜덤하게 선택
    positions.shuffle(&mut rng);
    let blank_positions = &positions[..num_blank.min(16)];

    // 값 비우기: 비워진 위치를 0으로 설정
    for &(i, j) in blank_positions {
        magic_square[i][j] = 0;
    }

    magic_square
}

// =============== 문제 브로드캐스트 ===============
pub async fn broadcast_problem(
    Extension(tx): Extension<Arc<BroadcastSender<Problem>>>,
){
    // 랜덤 마방진 생성 및 값 비우기
    let matrix = generate_incomplete_magic_square(4); // 5개의 빈 칸 생성

    // Problem 생성
    let problem = Problem { matrix };

    // 문제 브로드캐스트
    match tx.send(problem.clone()) {
        Ok(subscriber_count) => {
            println!(
                "Problem broadcasted successfully to {} subscribers.",
                subscriber_count
            );
        }
        Err(e) => {
            eprintln!("Failed to broadcast problem: {}", e);
        }
    }
}

// =============== 블록 제출 & 검증 요청 ===============
pub async fn handle_block_submission(
    Json(block): Json<Block>,
    Extension(tx): Extension<Arc<BroadcastSender<String>>>, // 직렬화된 JSON 문자열을 보냄
    Extension(_validation_sender): Extension<MpscSender<ValidationResult>>, // 사용하지 않음
) -> impl IntoResponse {
    println!("Received block in handle_block_submission: {:?}", block);

    // ServerMessage로 감싸기
    let message = ServerMessage::Block(block.clone());

    // JSON 문자열로 직렬화
    match serde_json::to_string(&message) {
        Ok(serialized_message) => {
            // 브로드캐스트
            if let Err(e) = tx.send(serialized_message) {
                eprintln!("Failed to broadcast block: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to broadcast block");
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize message: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize message");
        }
    }

    (StatusCode::OK, "Block submitted and broadcasted successfully")
}



// =============== 거래(트랜잭션) 핸들러 ===============
pub async fn handle_transaction(
    Json(tx): Json<Transaction>,
    Extension(server): Extension<Arc<Mutex<Server>>>,
) -> impl IntoResponse {
    let mut guard = server.lock().await;
    if !guard.is_transaction_flow {
        return (
            StatusCode::BAD_REQUEST, 
            "Currently not in transaction flow. Please wait for next transaction window."
        );
    }

    // 실제로는 여기서 sender 잔액 확인, receiver 업데이트 등이 필요
    // 일단 간단하게 로그만
    println!(
        "Transaction received: {} -> {} (amount: {})",
        tx.sender_id, tx.receiver_id, tx.amount
    );

    // 필요한 로직(예: balances) 추가 가능
    // guard.balances.entry(tx.sender_id).and_modify(...).etc

    (StatusCode::OK, "Transaction accepted")
}

// =============== 서버(합의/거래 흐름) 구조체 ===============
pub struct Server {
    current_block: Option<Block>,
    votes: HashMap<String /* node_id */, bool>,
    total_nodes: usize,
    pub is_transaction_flow: bool, // 거래창이 열려 있는지 여부
}

impl Server {
    pub fn new(total_nodes: usize) -> (Self, mpsc::Sender<ValidationResult>) {
        let (tx, rx) = mpsc::channel::<ValidationResult>(100);

        let server = Server {
            current_block: None,
            votes: HashMap::new(),
            total_nodes,
            is_transaction_flow: false, // 초기에는 거래모드 아님
        };
        (server, tx)
    }

    pub fn set_new_block(&mut self, block: Block) {
        self.current_block = Some(block);
        self.votes.clear();
    }

    pub fn add_vote(&mut self, node_id: String, is_valid: bool) {
        self.votes.insert(node_id, is_valid);
    }

    pub fn check_consensus(&self) -> bool {
        let valid_count = self.votes.values().filter(|&&v| v).count();
        valid_count > self.total_nodes / 2
    }

    /// 다수결 검증 로직 처리
    /// 합의 달성 시 -> 30초간 거래 모드 -> 이후 새 문제 브로드캐스트
    pub async fn process_consensus(
        &mut self, 
        validation_result: ValidationResult,
        problem_tx: Arc<BroadcastSender<Problem>>,
    ) {
        // 1) 투표 기록
        self.add_vote(validation_result.node_id, validation_result.is_valid);
        
        // 2) 다수결 체크
        if self.check_consensus() {
            println!("Consensus reached! Block is approved.");

            // ---------------------------
            //   30초간 거래 모드 실행
            // ---------------------------
            self.start_transaction_flow(problem_tx).await;
        } else {
            println!("Still waiting for more votes...");
        }
    }

    /// 실제로 30초간 거래 플로우를 활성화하고,
    /// 이후 종료 시 새 문제를 브로드캐스트
    async fn start_transaction_flow(&mut self, problem_tx: Arc<BroadcastSender<Problem>>) {
        // 1) 거래 창 오픈
        self.is_transaction_flow = true;
        println!("=== Transaction flow started for 30 seconds ===");

        // 2) 30초 기다림 (비동기로 잠깐 락을 풀고 싶다면, 여기서는 별도 스코프 사용 가능)
        //    여기서는 간단히 본 함수에서 sleep
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        // 3) 거래 창 닫기
        self.is_transaction_flow = false;
        println!("=== Transaction flow ended ===");

        // 4) 새 문제 브로드캐스트
        let new_problem = Problem {
            // id: rand::thread_rng().gen(),
            matrix: vec![
                vec![16, 2, 3, 13],
                vec![5, 11, 10, 8],
                vec![9, 7, 6, 12],
                vec![4, 14, 15, 1],
            ],
        };
        if let Err(e) = problem_tx.send(new_problem) {
            eprintln!("Failed to broadcast new problem after transaction flow: {}", e);
        } else {
            println!("A new problem has been broadcast after transaction flow.");
        }
    }
}


// WebSocket 핸들러 함수
pub async fn handle_websocket(
    ws: axum::extract::ws::WebSocketUpgrade,
    problem_tx: Arc<BroadcastSender<Problem>>,
    block_tx: Arc<BroadcastSender<String>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, problem_tx, block_tx))
}




async fn handle_socket(
    mut socket: WebSocket,
    problem_tx: Arc<BroadcastSender<Problem>>,
    block_tx: Arc<BroadcastSender<String>>,
) {
    // 각 채널의 수신기 생성
    let mut problem_rx: BroadcastReceiver<Problem> = problem_tx.subscribe();
    let mut block_rx: BroadcastReceiver<String> = block_tx.subscribe();

    loop {
        tokio::select! {
            // 문제 채널에서 새로운 메시지가 도착한 경우
            Ok(problem) = problem_rx.recv() => {
                let msg = json!({
                    "type": "problem",
                    "data": problem
                });
                if let Err(e) = socket.send(WsMessage::Text(msg.to_string())).await {
                    eprintln!("WebSocket send error: {}", e);
                    break;
                }
            }

            // 블록 채널에서 새로운 메시지가 도착한 경우
            Ok(block) = block_rx.recv() => {
                let block_json: serde_json::Value = serde_json::from_str(&block).expect("Failed to parse block");

                println!("block_json: {}", block_json);
                let msg = json!({
                    "type": "block",
                    "data": block_json["data"] // 중첩 없이 JSON 객체로 포함
                });
                if let Err(e) = socket.send(WsMessage::Text(msg.to_string())).await {
                    eprintln!("WebSocket send error: {}", e);
                    break;
                }
            }

            // WebSocket 연결 종료 처리
            else => {
                break;
            }
        }
    }

    println!("WebSocket connection closed");
}