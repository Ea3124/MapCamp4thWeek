// server/src/models.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
}
