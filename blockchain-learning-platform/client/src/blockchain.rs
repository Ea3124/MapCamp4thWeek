// client/src/blockchain.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub problem: Vec<Vec<String>>,     // 블록에 포함된 문제 (서버에서 마방진 생성 후 일부 가려서 클라에 전송송)
    pub solution: Vec<Vec<String>>, // 노드가 제출한 풀이 -> 아래와 통합?     
    pub prev_solution: Vec<Vec<String>>,       // 이전 블록의 해시 -> 이전 블록문제의 정답으로 교체 
    pub node_id: String,         // 블록을 생성한 노드 ID
    pub data: String, // 거래 내역
}

pub struct BlockChain {
    chain: Vec<Block>,
}

pub impl Block {
    // 새로운 블록 생성 (서버에서 받은 요청 검증 후)
    fn new(
        index: u64, 
        problem: Vec<Vec<String>>,
        solution: Vec<Vec<String>>, 
        previous_solution: Vec<Vec<String>>,
        node_id: String, 
        data: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut block = Block {
            index,
            timestamp,
            problem,
            solution,
            prev_solution,
            node_id,
            data,
        };
    }

}
/* 
pub impl BlockChain {
    // 새로운 블록체인 초기화
    fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
        };
        blockchain.create_genesis_block();
        blockchain
    }

    // 제네시스 블록 생성
    fn create_genesis_block(&mut self) {
        let genesis_block = Block::new(0, "Genesis Block".to_string(), "0".to_string());
        self.chain.push(genesis_block);
    }

    // 최신 블록 가져오기
    fn get_latest_block(&self) -> &Block {
        self.chain.last().expect("체인이 비어 있음")
    }

    // 새로운 블록 추가
    fn add_block(&mut self, data: String) {
        let latest_block = self.get_latest_block();
        let mut new_block = Block::new(
            latest_block.index + 1,
            data,
            latest_block.hash.clone(),
        );
        new_block.mine_block(self.difficulty);
        self.chain.push(new_block);
    }

}
    */  // 미완성