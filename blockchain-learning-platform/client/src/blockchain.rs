// client/src/blockchain.rs

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;

use crate::utils::current_timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
}

/// 새 블록을 생성 (이전 블록을 알고 있어야 함)
pub fn create_block(last_block: &Block, data: &str) -> Block {
    let new_index = last_block.index + 1;
    let timestamp = current_timestamp();
    let prev_hash = last_block.hash.clone();

    let hash = calculate_hash(new_index, &timestamp, data, &prev_hash);

    Block {
        index: new_index,
        timestamp,
        data: data.to_string(),
        prev_hash,
        hash,
    }
}

/// 블록 해시를 계산(sha256)
fn calculate_hash(index: u64, timestamp: &str, data: &str, prev_hash: &str) -> String {
    let input = format!("{}{}{}{}", index, timestamp, data, prev_hash);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// 새 블록이 이전 블록과 잘 연결되는지 검증
pub fn validate_block(new_block: &Block, last_block: &Block) -> bool {
    // 1) prev_hash가 last_block.hash와 일치하는가?
    if new_block.prev_hash != last_block.hash {
        return false;
    }
    // 2) 해시 값이 올바른가?
    let recalculated = calculate_hash(
        new_block.index,
        &new_block.timestamp,
        &new_block.data,
        &new_block.prev_hash
    );
    if new_block.hash != recalculated {
        return false;
    }
    true
}
