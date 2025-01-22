// main.rs

// Imports
mod views;
mod blockchain;
mod network;
mod transaction;

use views::problem_solving::view_problem_solving;
use views::chain_info::view_chain_info;
use views::block_verification::view_block_verification;

use blockchain::blockchain::{Block, BlockChainDB};
use transaction::transaction::{Transaction, TransactionDB};

// ------------------------------
// iced 관련 import 정리
// ------------------------------
use iced::{
    executor,
    Application,  // Application 트레이트
    Command,      // iced::Command
    Element,
    Length,
    Settings,     // iced::Settings
    Theme,
    widget::container,
};

use iced_aw::{TabLabel, Tabs};
use rand::{Rng, thread_rng};

// 메시지 열거형
#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    SubmitSolution,
    InputChanged(usize, usize, String), // (행, 열, 새로운 값)
    ResetDB,          // DB 초기화 메시지
    AddRandomBlock,   // 블록 추가 메시지
    // **서버 전송 후 결과를 받는 메시지**
    SubmitSolutionFinished(Result<(), String>),
    // 새로 추가된 블록 검증 관련 메시지
    VerifyBlock,      // 서버에서 받은(가정) 블록을 로컬 체인에 추가(검증 통과)
    RejectBlock,      // 서버 블록을 무시(검증 실패)
    // --- 트랜잭션 관련 ---
    AddRandomTransaction, // 트랜잭션 추가
    ResetTxDB,           // 트랜잭션 DB 초기화
}

// 메인 상태 구조체
struct BlockchainClientGUI {
    active_tab: usize,
    solution_input: [[String; 4]; 4], // 4x4 정답 입력 상태
    blocks: Vec<Block>,               // 로드된 블록 리스트
    db: BlockChainDB,                 // DB 인스턴스
    /// (가정) 서버에서 받은 블록(하드코딩)
    proposed_block: Option<Block>,
    // 트랜잭션 관련
    transactions: Vec<Transaction>,
    tx_db: TransactionDB,
}

impl BlockchainClientGUI {
    fn new(db_path: &str, tx_db_path: &str) -> Self {
        let db = BlockChainDB::new(db_path);

        // DB를 열고 블록이 없는 경우 제네시스 블록 생성
        if db.load_latest_index().is_none() {
            db.reset_db();  // reset_db 내부에서 제네시스 블록 생성
        }

        // 시작 시 DB에서 기존 블록들을 불러옵니다.
        let blocks = db.load_all_blocks();

        // 트랜잭션 DB
        let tx_db = TransactionDB::new(tx_db_path);
        let transactions = tx_db.load_all_transactions();

        // (예시) 서버에서 받은 블록(하드코딩) - 실제론 통신을 통해 받아올 예정
        // 여기서는 DB의 마지막 인덱스 + 1로 index 설정
        let latest_index = db.load_latest_index().unwrap_or(0);
        let server_block = Block::new(
            latest_index + 1,
            vec![vec!["Hard-coded Problem".into()]],
            vec![vec!["Hard-coded Solution".into()]],
            vec![],                    // prev_solution
            "ServerNode".into(),      // node_id
            "Proposed block from server".into()
        );

        BlockchainClientGUI {
            active_tab: 0,
            solution_input: Default::default(),
            blocks,
            db,
            proposed_block: Some(server_block), // 서버 블록을 미리 넣어둠

            transactions,
            tx_db,
        }
    }

    /// 임의의 블록 추가
    fn add_random_block(&mut self) {
        let mut rng = thread_rng();
        let problem = vec![vec!["1".to_string(), "2".to_string()]];
        let solution = vec![vec!["3".to_string(), "4".to_string()]];
        let node_id = format!("Node{}", rng.gen_range(1..1000));
        let data = format!("Random Data {}", rng.gen::<u32>());

        let latest_index = self.db.load_latest_index().unwrap_or(0);

        let latest_block = match self.db.load_block(latest_index) {
            Some(block) => block,
            None => {
                let genesis_block = Block::new(
                    0,
                    vec![],
                    vec![],
                    vec![],
                    "GenesisNode".into(),
                    "Genesis Block".into(),
                );
                self.db.save_block(&genesis_block);
                self.db.save_latest_index(0);
                genesis_block
            }
        };

        let new_block = Block::new(
            latest_block.index + 1,
            problem,
            solution,
            latest_block.solution.clone(),
            node_id,
            data,
        );

        self.db.save_block(&new_block);
        self.db.save_latest_index(new_block.index);

        // 갱신
        self.blocks = self.db.load_all_blocks();
    }

    /// DB 초기화
    fn reset_db(&mut self) {
        self.db.reset_db();
        self.blocks = self.db.load_all_blocks();
    }

    /// 임의의 트랜잭션 추가
    fn add_random_transaction(&mut self) {
        let mut rng = thread_rng();
        let sender = format!("Sender{}", rng.gen_range(1..1000));
        let receiver = format!("Receiver{}", rng.gen_range(1..1000));
        let payment = rng.gen_range(1..5000) as u64;

        let latest_index = self.tx_db.load_latest_index().unwrap_or(0);

        let new_tx = Transaction::new(
            latest_index + 1,
            sender,
            receiver,
            payment
        );

        self.tx_db.save_transaction(&new_tx);
        self.tx_db.save_latest_index(new_tx.index);

        self.transactions = self.tx_db.load_all_transactions();

        println!("Random transaction added: {:?}", new_tx);
    }

    /// 트랜잭션 DB 초기화
    fn reset_tx_db(&mut self) {
        self.tx_db.reset_db();
        self.transactions = self.tx_db.load_all_transactions();
        println!("Transaction DB has been reset!");
    }
}

// Default 구현 (Application 초기화 등에 사용)
impl Default for BlockchainClientGUI {
    fn default() -> Self {
        Self::new("blockchain_db", "transaction_db")
    }
}

//------------------------//
// iced::Application 구현 //
//------------------------//
impl Application for BlockchainClientGUI {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    // main에서 넘겨줄 Flags (여기서는 사용 X)
    type Flags = ();

    // 앱 시작 시 호출되는 함수
    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            BlockchainClientGUI::default(),
            Command::none(),
        )
    }

    // 윈도우 타이틀 설정
    fn title(&self) -> String {
        String::from("블록체인 클라이언트")
    }

    // 메시지 처리 (상태 업데이트)
    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::TabSelected(i) => {
                self.active_tab = i;
                Command::none()
            }
            Message::SubmitSolution => {
                println!("Solution submitted! Now sending to server...");
    
                // 1) 4x4 string matrix -> Vec<Vec<u32>> 변환 (파싱)
                let parsed_solution = self
                    .solution_input
                    .iter()
                    .map(|row| {
                        row.iter()
                            .filter_map(|val| val.parse::<u32>().ok()) // 파싱 실패하면 버림
                            .collect::<Vec<u32>>()
                    })
                    .collect::<Vec<Vec<u32>>>();
    
                // 2) 서버로 보낼 BlockForServer 구성 (임시 값들로 예시)
                use crate::network::BlockForServer;
                let block_data = BlockForServer {
                    index: 0,
                    timestamp: "temp-timestamp".to_string(),
                    solution: parsed_solution,
                    hash: "temp-hash".to_string(),
                    prev_hash: "temp-prev-hash".to_string(),
                    node_id: "client-node-id".to_string(),
                };
    
                // 3) 비동기 전송 - Command::perform 사용
                //    http://143.248.196.38:3000 등 실제 서버 주소로 교체
                let server_url = "http://143.248.196.38:3000";
                let future = async move {
                    match crate::network::submit_solution_block(server_url, &block_data).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    }
                };
    
                // 이때 즉시 return을 하지 않고,
                // Command::perform(...)를 반환하여 iced가 비동기 처리 후 메시지를 다시 보냄
                Command::perform(future, Message::SubmitSolutionFinished)
            }
            Message::SubmitSolutionFinished(result) => {
                match result {
                    Ok(()) => println!("Server accepted the solution block successfully!"),
                    Err(err_msg) => eprintln!("Error submitting solution block: {}", err_msg),
                }
                Command::none()
            }
            Message::InputChanged(row, col, value) => {
                self.solution_input[row][col] = value;
                Command::none()
            }
            // 테스트
            Message::ResetDB => {
                // 자유 함수이므로 self가 아닌 self
                self.reset_db();
                println!("DB has been reset!");
                Command::none()
            }
            Message::AddRandomBlock => {
                self.add_random_block();
                println!("Random block added!");
                Command::none()
            }
            // --- (새로 추가된 메시지) 서버 블록 검증 ---
            // 검증 통과 -> 실제 체인(DB)에 추가
            Message::VerifyBlock => {
                // 서버에서 받은 블록(proposed_block)이 존재하는지 확인
                if let Some(proposed) = self.proposed_block.take() {
                    // 로컬 DB의 최신 블록 인덱스와 블록 정보를 가져옴
                    let latest_index = self.db.load_latest_index().unwrap_or(0);
                    let latest_block = self.db.load_block(latest_index).unwrap_or_else(|| {
                        // 만약 로컬에 블록이 없다면 제네시스 블록을 기본값으로 사용
                        Block::new(
                            0,
                            vec![],
                            vec![],
                            vec![],
                            "GenesisNode".into(),
                            "Genesis Block".into()
                        )
                    });

                    // 서버에서 받은 블록의 데이터를 활용하되, 인덱스는 최신 블록 + 1로 설정하여 새 블록 생성
                    let new_block = Block::new(
                        latest_block.index + 1,      // 새로운 인덱스
                        proposed.problem.clone(),    // 서버로부터 받은 문제 정보 (필요시, 기존 로직에 맞게 조정)
                        proposed.solution.clone(),   // 서버로부터 받은 해결책 정보
                        latest_block.solution.clone(), // 이전 해결책(현재 로컬의 마지막 블록의 해결책)
                        proposed.node_id.clone(),    // 서버 노드 ID
                        proposed.data.clone()        // 서버 블록 데이터
                    );

                    // 새로운 블록을 로컬 DB에 저장
                    self.db.save_block(&new_block);
                    self.db.save_latest_index(new_block.index);
                    self.blocks = self.db.load_all_blocks();

                    println!("블록 검증 통과! 새로운 블록을 추가했습니다: {:?}", new_block);
                } else {
                    println!("검증할 블록이 없습니다.");
                }
                Command::none()
            }
            // 검증 실패 -> 아무 것도 안 함(무시)
            Message::RejectBlock => {
                println!("블록 검증 실패! 제안된 블록을 폐기합니다.");
                self.proposed_block = None;
                Command::none()
            }
            // --- 트랜잭션 관련 ---
            Message::AddRandomTransaction => {
                self.add_random_transaction();
                Command::none()
            }
            Message::ResetTxDB => {
                self.reset_tx_db();
                Command::none()
            }
        }
    }

    // 뷰(화면) 구성
    fn view(&self) -> Element<'_, Self::Message> {
        view(self)
    }

    // 테마 지정
    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}

//---------------------//
// 업데이트 로직 함수  //
//---------------------//


//---------------//
// 메인 뷰 함수  //
//---------------//
fn view<'a>(state: &'a BlockchainClientGUI) -> Element<'a, Message> {
    println!("현재 활성 탭: {}", state.active_tab); // 디버그 로그

    let tabs = Tabs::new(Message::TabSelected)
        .push(
            0,
            TabLabel::Text("코인 채굴하기".to_owned()),
            view_problem_solving(state),
        )
        .push(
            1,
            TabLabel::Text("로컬 체인 정보 & 거래 내역".to_owned()),
            view_chain_info(&state.blocks, &state.transactions),
        )
        .push(
            2,
            TabLabel::Text("블록 검증".to_owned()),
            view_block_verification(state.blocks.last(), state.proposed_block.as_ref()),
        )
        .set_active_tab(&state.active_tab);

    container(tabs)
        .width(Length::Fill)
        .height(Length::Fill)
        // 기존: .center_x(Length::Fill), .center_y(Length::Fill) -> 인자 없는 메서드로 변경
        .center_x()
        .center_y()
        .into()
}

//--------------//
// main 함수    //
//--------------//
fn main() -> iced::Result {
    BlockchainClientGUI::run(Settings::default())
}
