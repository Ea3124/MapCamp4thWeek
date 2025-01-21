use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::{broadcast::Sender, mpsc::Sender as MpscSender};
use tokio::sync::{mpsc};

use crate::models::{Block, Problem, ValidationResult};
use rand::Rng;
use std::collections::HashMap;


/// 문제를 생성하고 브로드캐스트하는 핸들러
pub async fn broadcast_problem(
    Extension(tx): Extension<Arc<Sender<Problem>>>,
){
    // 새로운 문제 생성
    let problem = Problem {
        id: rand::thread_rng().gen(), // `rand::Rng`에서 `gen()` 호출
        matrix: vec![
            vec![8, 1, 6],
            vec![3, 5, 7],
            vec![4, 9, 2],
        ],
    };

    // 메시지를 브로드캐스트
    if let Err(e) = tx.send(problem) {
        eprintln!("Failed to broadcast problem: {}", e);
    }
}

/// 블록 제출을 처리하고 검증 요청을 브로드캐스트하는 핸들러
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

    // 검증 요청: 검증 결과를 기다릴 필요 없음
    if let Err(e) = validation_sender.send(ValidationResult {
        block_hash: block.hash.clone(),
        is_valid: false, // 기본값으로 false
        node_id: "server".to_string(),
    }).await {
        eprintln!("Failed to send validation request: {}", e);
    }

    (StatusCode::OK, "Block submitted and broadcasted successfully")
}

pub struct Server {
    // 블록에 대한 상태
    current_block: Option<Block>,
    // 노드별 투표 결과
    votes: HashMap<String /* node_id */, bool>,
    // 노드 수 or 합의 조건
    total_nodes: usize,
}

impl Server {
    pub fn new(total_nodes: usize) -> (Self, mpsc::Sender<ValidationResult>) {
        let (tx, rx) = mpsc::channel::<ValidationResult>(100);

        let server = Server {
            current_block: None,
            votes: HashMap::new(),
            total_nodes,
        };
        (server, tx)
    }

    pub fn set_new_block(&mut self, block: Block) {
        self.current_block = Some(block);
        self.votes.clear(); // 새 블록이므로 투표 결과 초기화
    }

    pub fn add_vote(&mut self, node_id: String, is_valid: bool) {
        self.votes.insert(node_id, is_valid);
    }

    pub fn check_consensus(&self) -> bool {
        let valid_count = self.votes.values().filter(|&&v| v).count();
        valid_count > self.total_nodes / 2
    }

        // 새로 추가
        pub async fn process_consensus(&mut self, validation_result: ValidationResult) {
            // 1) 노드별 투표 기록 추가
            self.add_vote(validation_result.node_id, validation_result.is_valid);
    
            // 2) 다수결(합의) 여부 확인
            if self.check_consensus() {
                println!("Consensus reached! Block is approved.");
                // TODO: 승인된 블록을 전체 노드에 다시 브로드캐스트하는 로직 등 필요하다면 여기에
                //       block이 self.current_block에 있다고 가정하면, 그걸 broadcast할 수도 있음
            } else {
                println!("Still waiting for more votes...");
            }
        }
}
