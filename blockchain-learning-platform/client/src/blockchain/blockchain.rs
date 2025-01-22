// client/src/blockchain.rs
use crate::blockchain::blockchain_db::{Block, BlockChainDB};

use super::blockchain_db::Problem;

pub struct BlockChain {
    db: BlockChainDB,
}

impl BlockChain {
    // 새로운 블록체인 초기화
    pub fn new(db_path: &str) -> Self {
        let db = BlockChainDB::new(db_path);

        let problem3 = Problem {
            matrix: vec![
                vec![1, 2, 3, 4],
                vec![5, 6, 7, 8],
                vec![9, 10, 11, 12],
                vec![13, 14, 15, 16],
            ],
        };

        // 블록체인 초기화 및 제네시스 블록 생성
        if db.load_latest_index().is_none() {
            let genesis_block = Block::new(
                0,
                problem3,                         // 문제는 비어 있음
                vec![],                         // 풀이도 비어 있음
                vec![],                         // 이전 블록 풀이도 없음
                "GenesisNode".to_string(),      // 제네시스 블록 생성 노드 ID
                "Genesis Block".to_string(),    // 제네시스 블록 데이터
            );
            db.save_block(&genesis_block);
            db.save_latest_index(genesis_block.index);
        }

        BlockChain { db }
    }

    // 최신 블록 가져오기
    pub fn get_latest_block(&self) -> Block {
        let latest_index = self.db.load_latest_index().expect("블록체인이 비어 있습니다.");
        self.db
            .load_block(latest_index)
            .expect("최신 블록을 로드할 수 없습니다.")
    }

    // 새로운 블록 추가
    pub fn add_block(
        &mut self,
        problem: Problem,
        solution: Vec<Vec<u32>>,
        node_id: String,
        data: String,
    ) {
        let latest_block = self.get_latest_block();
        let new_block = Block::new(
            latest_block.index + 1,       // 새 블록의 인덱스는 이전 블록의 인덱스 + 1
            problem,                      // 새로운 문제
            solution,                     // 새로운 풀이
            latest_block.solution.clone(), // 이전 블록의 풀이를 참조
            node_id,                      // 블록 생성 노드 ID
            data,                         // 거래 내역
        );

        self.db.save_block(&new_block);
        self.db.save_latest_index(new_block.index);
    }
}