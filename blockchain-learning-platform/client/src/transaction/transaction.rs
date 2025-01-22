use rocksdb::{DB, Options};
use serde::{Serialize, Deserialize};
use bincode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub index: u64,
    pub sender: String,
    pub receiver: String,
    pub payment: u64,
}

impl Transaction {
    pub fn new(index: u64, sender: String, receiver: String, payment: u64) -> Self {
        Transaction {
            index,
            sender,
            receiver,
            payment,
        }
    }
}

pub struct TransactionDB {
    db: DB,
}

impl TransactionDB {
    pub fn new(db_path: &str) -> Self {
        let mut options = Options::default();
        options.create_if_missing(true);
        let db = DB::open(&options, db_path)
            .expect("Failed to initialize TransactionDB");
        TransactionDB { db }
    }

    /// 트랜잭션 저장
    pub fn save_transaction(&self, tx: &Transaction) {
        let key = format!("tx_{:08}", tx.index);
        let value = bincode::serialize(tx).expect("Failed to serialize transaction");
        self.db.put(key.as_bytes(), value).expect("Failed to save transaction");
    }

    /// 특정 index의 트랜잭션 로드
    pub fn load_transaction(&self, index: u64) -> Option<Transaction> {
        let key = format!("tx_{:08}", index);
        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => bincode::deserialize(&value).ok(),
            _ => None,
        }
    }

    /// 최신 트랜잭션 인덱스 저장
    pub fn save_latest_index(&self, index: u64) {
        let value = bincode::serialize(&index).expect("Failed to serialize latest tx index");
        self.db.put("latest_tx_index", value).expect("Failed to save latest_tx_index");
    }

    /// 최신 트랜잭션 인덱스 로드
    pub fn load_latest_index(&self) -> Option<u64> {
        match self.db.get("latest_tx_index") {
            Ok(Some(value)) => bincode::deserialize(&value).ok(),
            _ => None,
        }
    }

    /// 모든 트랜잭션 로드
    pub fn load_all_transactions(&self) -> Vec<Transaction> {
        let mut txs = Vec::new();
        for item in self.db.iterator(rocksdb::IteratorMode::Start) {
            if let Ok((key, value)) = item {
                if key.starts_with(b"tx_") {
                    if let Ok(tx) = bincode::deserialize::<Transaction>(&value) {
                        txs.push(tx);
                    }
                }
            }
        }
        // index 기준 정렬
        txs.sort_by_key(|tx| tx.index);
        txs
    }

    /// 트랜잭션 DB 초기화
    pub fn reset_db(&self) {
        let mut batch = rocksdb::WriteBatch::default();
        // tx_로 시작하는 키 전부 삭제
        for item in self.db.iterator(rocksdb::IteratorMode::Start) {
            if let Ok((key, _)) = item {
                if key.starts_with(b"tx_") {
                    batch.delete(key);
                }
            }
        }
        // latest_tx_index 삭제
        batch.delete(b"latest_tx_index");

        // 일괄 적용
        self.db.write(batch).expect("Failed to reset TransactionDB");
    }
}
