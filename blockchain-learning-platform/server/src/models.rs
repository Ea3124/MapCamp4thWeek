// server/src/models.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    // pub problem: Problem,        // 블록에 포함된 문제
    pub solution: Vec<Vec<u32>>, // 노드가 제출한 풀이
    pub hash: String,            // 현재 블록의 해시
    pub prev_hash: String,       // 이전 블록의 해시
    pub node_id: String,         // 블록을 생성한 노드 ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: u64,
    pub matrix: Vec<Vec<u32>>, // 예: 마방진 문제용 2D 배열
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub block_hash: String,
    pub is_valid: bool,
    pub node_id: String,
}

// ------------------------------
// 새로 추가: Transaction 구조체
// ------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender_id: String,
    pub receiver_id: String,
    pub amount: u64,
}
