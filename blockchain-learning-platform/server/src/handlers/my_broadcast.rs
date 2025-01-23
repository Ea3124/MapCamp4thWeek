// server/src/handlers/my_broadcast.rs

use axum::{
    extract::ws::{Message as WsMessage, WebSocket},
    extract::{Extension, Json},
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::{broadcast::Sender as BroadcastSender, broadcast::Receiver as BroadcastReceiver, mpsc::Sender as MpscSender, Mutex};
use serde_json::json;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::models::{self, Block, Problem, ServerMessage, Transaction, ValidationResult};
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
    let matrix = generate_incomplete_magic_square(4); // 4개의 빈 칸 생성

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
    Extension(tx): Extension<Arc<BroadcastSender<String>>>, 
    Extension(server): Extension<Arc<Mutex<Server>>>,
) -> impl IntoResponse {
    println!("Received block in handle_block_submission: {:?}", block);

    {
        // ================
        // 1) 서버 잠금
        // ================
        let mut guard = server.lock().await;

        // ================
        // 2) 이미 블록이 있나?
        // ================
        if guard.current_block.is_some() {
            // *에러 발생시키지 않음*
            // 대신 "이미 제출됨" 이라는 문구와 함께 200 OK
            println!("A block was already submitted. Ignoring new block.");
            return (StatusCode::OK, "Block already submitted. Ignoring new block.");
        }

        // ================
        // 3) 없다면 세팅
        // ================
        guard.set_new_block(block.clone());
    }

    // ================
    // 4) 브로드캐스트
    // ================
    let message = ServerMessage::Block(block);
    let serialized_message = serde_json::to_string(&message).unwrap();

    if let Err(e) = tx.send(serialized_message) {
        eprintln!("Failed to broadcast block: {}", e);
        // "절대 오류를 일으키지 마라" → 상태코드 200 + 로그만 출력
        return (StatusCode::OK, "Failed to broadcast block, but no error raised.");
    }

    (StatusCode::OK, "Block submitted and broadcasted successfully")
}

// =============== 서버(합의/거래 흐름) 구조체 ===============
pub struct Server {
    current_block: Option<Block>,
    votes: HashMap<String /* node_id */, bool>,
    total_nodes: usize,
    is_problem_solved: bool, // 문제 해결 상태 추가
}

impl Server {
    /// `validation_sender`를 외부에서 전달받아 사용하도록 수정
    pub fn new(total_nodes: usize, _validation_sender: MpscSender<ValidationResult>) -> Self {
        Server {
            current_block: None,
            votes: HashMap::new(),
            total_nodes,
            is_problem_solved: false, // 초기 상태 설정
        }
    }

     // 문제를 설정할 때 상태도 초기화
     pub fn set_new_block(&mut self, block: Block) {
        self.current_block = Some(block);
        self.votes.clear();
        self.is_problem_solved = false; // 새 문제이므로 상태 초기화
    }

    pub fn add_vote(&mut self, node_id: String, is_valid: bool) {
        self.votes.insert(node_id, is_valid);
    }

    // pub fn check_consensus(&self) -> bool {
    //     let valid_count = self.votes.values().filter(|&&v| v).count();
    //     valid_count > self.total_nodes / 2
    // }

    pub fn check_consensus(&self) -> bool {
        self.votes.values().any(|&v| v)
    }
    // pub fn check_consensus(&self) -> bool {
    //     let valid_count = self.votes.values().filter(|&&v| v).count();
    //     valid_count >= 2 // 두 개 이상의 true 투표가 있을 경우
    // }

    /// 문제 해결 상태 업데이트
    pub fn mark_problem_as_solved(&mut self) {
        self.is_problem_solved = true;
    }

    /// 문제 해결 여부 확인
    pub fn problem_solved(&self) -> bool {
        self.is_problem_solved
    }

    /// 다수결 검증 로직 처리
    /// 합의 달성 시 과반수 결과를 출력
    pub async fn process_consensus(
        &mut self, 
        validation_result: ValidationResult,
        problem_tx: Arc<BroadcastSender<Problem>>, // 두 번째 인자 추가
    ) {
        // 1) 투표 기록
        self.add_vote(validation_result.node_id, validation_result.is_valid);
        
        // 2) 다수결 체크
        if self.check_consensus() && !self.is_problem_solved {
            println!("Consensus reached with at least one valid vote.");
    
            // 문제 해결 상태 업데이트
            self.mark_problem_as_solved();
    
            // 새 문제 브로드캐스트
            let new_matrix = generate_incomplete_magic_square(4);
            let new_problem = Problem { matrix: new_matrix };
            if let Err(e) = problem_tx.send(new_problem.clone()) {
                eprintln!("Failed to broadcast new problem after consensus: {}", e);
            } else {
                println!("New problem broadcasted after consensus.");
            }
    
            // 서버 상태 초기화: current_block을 None으로 설정
            self.current_block = None;
            println!("Server state reset. Ready to accept new block submissions.");
        } else {
            println!("No consensus reached or problem already solved. Current votes: {:?}", self.votes);
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
