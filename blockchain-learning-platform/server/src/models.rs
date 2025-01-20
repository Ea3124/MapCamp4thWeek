// server/src/models.rs

use serde::{Serialize, Deserialize};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Block {
//     pub index: u64,
//     pub timestamp: String,
//     pub data: String,
//     pub prev_hash: String,
//     pub hash: String,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub problem: Problem,        // 블록에 포함된 문제
    pub solution: Vec<Vec<u32>>, // 노드가 제출한 풀이 -> 아래와 통합?
    pub hash: String,            // 현재 블록의 해시 -> 현재 정답으로 교체
    pub prev_hash: String,       // 이전 블록의 해시 -> 이전 블록문제의 정답으로 교체 
    pub node_id: String,         // 블록을 생성한 노드 ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: u64,
    pub matrix: Vec<Vec<u32>>, // 마방진 문제를 위한 2D 배열
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult { // 다수결로 합의 보는 알고리즘 
    pub block_hash: String,
    pub is_valid: bool,
    pub node_id: String,
}