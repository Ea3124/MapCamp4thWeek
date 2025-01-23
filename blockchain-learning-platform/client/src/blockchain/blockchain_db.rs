// client/src/blockchain/blockchain_db.rs
use rocksdb::{DB, Options};
use serde::{Serialize, Deserialize};
use bincode;
use chrono::{DateTime, TimeZone, Utc, FixedOffset};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub problem: Problem,      // 숫자 배열
    pub solution: Vec<Vec<u32>>,     // 숫자 배열
    pub prev_solution: Vec<Vec<u32>>,
    pub node_id: String,
    pub data: String,
}

#[derive(Debug,Clone, Serialize, Deserialize )]
pub struct Problem {
    pub matrix: Vec<Vec<u32>>,
}

impl Block {
    pub fn new(
        index: u64,
        problem: Problem,
        solution: Vec<Vec<u32>>,
        prev_solution: Vec<Vec<u32>>,
        node_id: String,
        data: String,
    ) -> Self {
        let sys_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let datetime: DateTime<Utc> = Utc.timestamp_opt(sys_timestamp as i64, 0)
            .single()
            .expect("Invalid timestamp");

        // 한국 시간으로 변환 (UTC+9)
        let kst_offset = FixedOffset::east_opt(9 * 3600)
            .expect("Invalid offset");
        let kst_datetime = datetime.with_timezone(&kst_offset);
        
        let timestamp= kst_datetime.format("%Y-%m-%d %H:%M:%S").to_string();

        Block {
            index,
            timestamp,
            problem,
            solution,
            prev_solution,
            node_id,
            data,
        }
    }
}

pub struct BlockChainDB {
    db: DB,
}

impl BlockChainDB {
    pub fn new(db_path: &str) -> Self {
        let mut options = Options::default();
        options.create_if_missing(true);
        let db = DB::open(&options, db_path).expect("RocksDB 초기화 실패");
        BlockChainDB { db }
    }

    pub fn save_block(&self, block: &Block) {
        let key = format!("block_{:08}", block.index);
        let value = bincode::serialize(block).expect("블록 직렬화 실패");
        self.db.put(key.as_bytes(), value).expect("블록 저장 실패");
    }

    pub fn load_block(&self, index: u64) -> Option<Block> {
        let key = format!("block_{:08}", index);
        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => bincode::deserialize(&value).ok(),
            _ => None,
        }
    }

    pub fn save_latest_index(&self, index: u64) {
        let value = bincode::serialize(&index).expect("인덱스 직렬화 실패");
        self.db.put("latest_block_index", value).expect("최신 블록 인덱스 저장 실패");
    }

    pub fn load_latest_index(&self) -> Option<u64> {
        match self.db.get("latest_block_index") {
            Ok(Some(value)) => bincode::deserialize(&value).ok(),
            _ => None,
        }
    }

    // 모든 블록 로드
    pub fn load_all_blocks(&self) -> Vec<Block> {
        let mut blocks = Vec::new();
        for item in self.db.iterator(rocksdb::IteratorMode::Start) {
            if let Ok((key, value)) = item {
                if key.starts_with(b"block_") {
                    if let Ok(block) = bincode::deserialize::<Block>(&value) {
                        blocks.push(block);
                    }
                }
            }
        }
        blocks.sort_by_key(|block| block.index); // 인덱스 정렬
        blocks
    }
    
    /// DB를 초기화(모든 블록 삭제 후 제네시스 블록 재생성)
    pub fn reset_db(&self) {
        // 1) 모든 block_... 키 삭제
        let mut batch = rocksdb::WriteBatch::default();
        for item in self.db.iterator(rocksdb::IteratorMode::Start) {
            if let Ok((key, _value)) = item {
                if key.starts_with(b"block_") {
                    batch.delete(key);
                }
            }
        }
        // latest_block_index 삭제
        batch.delete(b"latest_block_index");

        // 일괄 적용
        self.db.write(batch).expect("DB 초기화 실패");

        // 2) 제네시스 블록 추가
        let genesis_block = Block::new(
            0,
            Problem{matrix:vec![]},
            vec![],
            vec![],
            "GenesisNode".into(),
            "Genesis Block".into(),
        );
        self.save_block(&genesis_block);
        self.save_latest_index(0);
    }
}