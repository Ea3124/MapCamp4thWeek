// client/src/db.rs
use rocksdb::DB;
use serde_json;
use std::error::Error;
use crate::blockchain::Block;

pub fn init_db(path: &str) -> Result<DB, Box<dyn Error>> {
    let db = DB::open_default(path)?;
    Ok(db)
}

/// 블록 저장
pub fn store_block(db: &DB, block: &Block) -> Result<(), Box<dyn Error>> {
    let key = format!("block_{}", block.index);
    let value = serde_json::to_string(block)?;
    db.put(key.as_bytes(), value.as_bytes())?;

    // last_index 갱신
    db.put(b"last_index", block.index.to_string().as_bytes())?;
    Ok(())
}

/// 마지막 블록 가져오기
pub fn get_last_block(db: &DB) -> Option<Block> {
    // 저장된 last_index 키 확인
    let last_index_data = db.get(b"last_index").ok()??;
    let last_index_str = String::from_utf8(last_index_data).ok()?;
    let last_index: u64 = last_index_str.parse().ok()?;

    let block_key = format!("block_{}", last_index);
    let block_data = db.get(block_key.as_bytes()).ok()??;
    let block_json = String::from_utf8(block_data).ok()?;
    let block: Block = serde_json::from_str(&block_json).ok()?;
    Some(block)
}
