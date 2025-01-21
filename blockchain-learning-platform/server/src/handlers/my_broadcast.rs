// server/src/handlers/my_broadcast.rs

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::{broadcast::Sender, mpsc::Sender as MpscSender, Mutex};
use tokio::sync::mpsc;

use crate::models::{Block, Problem, ValidationResult, Transaction};
use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;

// =============== 문제 브로드캐스트 ===============
pub async fn broadcast_problem(
    Extension(tx): Extension<Arc<Sender<Problem>>>,
){
    let problem = Problem {
        id: rand::thread_rng().gen(), 
        matrix: vec![
            vec![8, 1, 6],
            vec![3, 5, 7],
            vec![4, 9, 2],
        ],
    };

    if let Err(e) = tx.send(problem) {
        eprintln!("Failed to broadcast problem: {}", e);
    }
}

// =============== 블록 제출 & 검증 요청 ===============
pub async fn handle_block_submission(
    Json(block): Json<Block>,
    Extension(tx): Extension<Arc<Sender<Block>>>,
    Extension(validation_sender): Extension<MpscSender<ValidationResult>>,
) -> impl IntoResponse {
    // 블록 브로드캐스트
    if let Err(e) = tx.send(block.clone()) {
        eprintln!("Failed to broadcast block: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to broadcast block");
    }

    // 검증 요청 (합의는 다른 태스크에서 진행)
    if let Err(e) = validation_sender.send(ValidationResult {
        block_hash: block.hash.clone(),
        is_valid: false, // 일단 false 로 가정
        node_id: "server".to_string(),
    }).await {
        eprintln!("Failed to send validation request: {}", e);
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
        problem_tx: Arc<Sender<Problem>>,
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
    async fn start_transaction_flow(&mut self, problem_tx: Arc<Sender<Problem>>) {
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
            id: rand::thread_rng().gen(),
            matrix: vec![
                vec![2, 9, 4],
                vec![7, 5, 3],
                vec![6, 1, 8],
            ],
        };
        if let Err(e) = problem_tx.send(new_problem) {
            eprintln!("Failed to broadcast new problem after transaction flow: {}", e);
        } else {
            println!("A new problem has been broadcast after transaction flow.");
        }
    }
}
