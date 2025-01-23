// client/src/main.rs

// Imports
mod views;
mod blockchain;
mod network;
mod transaction;

use blockchain::blockchain_db::Problem;
use tokio::sync::mpsc::unbounded_channel;
use views::problem_solving::view_problem_solving;
use views::chain_info::view_chain_info;
use views::block_verification::view_block_verification;

use blockchain::blockchain_db::{Block, BlockChainDB};
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

// 추가: network 모듈 관련 임포트
use crate::network::ServerMessage as netServerMessage;
use crate::network::ValidationResult;

use std::sync::Arc;
use tokio::sync::Mutex;
use iced::subscription::unfold;
use futures::Stream;
use iced::{Subscription, Event, event::Status};

// 메시지 열거형
#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    SubmitSolution,
    InputChanged(usize, usize, String), // (행, 열, 새로운 값)
    LoadChainInfo,                      // 체인 정보를 로드하는 메시지 ***
    ResetDB,          // DB 초기화 메시지
    AddRandomBlock,   // 블록 추가 메시지

    // 서버 전송 후 결과를 받는 메시지
    SubmitSolutionFinished(Result<(), String>),
    SubmitValidationFinished(Result<(), String>),  // ← 블록 검증 메시지 전송 완료 후 수신 **
    // 새로운 메시지: 서버로부터의 메시지 수신
    ServerMessage(netServerMessage), // ***
    VerifyBlock,      // 서버에서 받은(가정) 블록을 로컬 체인에 추가(검증 통과)
    RejectBlock,      // 서버 블록을 무시(검증 실패)

    ReceivedProposedBlock(Option<netServerMessage>),

    // --- 트랜잭션 관련 ---
    AddRandomTransaction, // 트랜잭션 추가
    ResetTxDB,           // 트랜잭션 DB 초기화

    // 거래 관련 메시지
    TransactionSubmit(String, String, u32), // (sender, receiver, amount)// ***
    TransactionFinished(Result<(), String>),// ***

    NoMoreMessages,
    
}

// 메인 상태 구조체
struct BlockchainClientGUI {
    active_tab: usize,
    solution_input: [[String; 4]; 4], // 4x4 정답 입력 상태
    transaction_input: (String, String, String), // (sender, receiver, amount)
    blocks: Vec<Block>,               // 로드된 블록 리스트
    db: BlockChainDB,                 // DB 인스턴스
    // 추가: 서버 메시지를 수신하기 위한 채널
    server_msg_receiver: Option<Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<netServerMessage>>>>,
    // /// (가정) 서버에서 받은 블록(하드코딩)
    proposed_block: Option<Block>,
    // 트랜잭션 관련
    transactions: Vec<Transaction>,
    tx_db: TransactionDB,
}

impl BlockchainClientGUI {
    // fn new(db_path: &str) -> (Self, tokio::sync::mpsc::UnboundedSender<netServerMessage>) { //*** 
    fn new(db_path: &str, tx_db_path: &str)
        -> (Self, tokio::sync::mpsc::UnboundedSender<netServerMessage>) {
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

        // 2) 채널 생성
        let (tx, rx) = unbounded_channel::<netServerMessage>();

        // 3) Arc<Mutex<...>>로 감싸기
        let rx_arc = Arc::new(Mutex::new(rx));

        // 4) 구조체 생성
        let gui = BlockchainClientGUI {
            active_tab: 0,
            solution_input: Default::default(),
            transaction_input: (String::new(), String::new(), String::new()),
            blocks,
            db,
            // 바뀐 부분
            server_msg_receiver: Some(rx_arc),
            proposed_block: None,
            transactions,
            tx_db,
        };
        (gui, tx)
    }

    // pub async fn process_server_messages(&mut self) {
    //     if let Some(receiver) = &mut self.server_msg_receiver {
    //         while let Some(message) = receiver.recv().await {
    //             match message {
    //                 netServerMessage::Block(block) => {
    //                     println!("Received Block: {:?}", block);
    //                     // 수신한 Block을 처리
    //                     self.blocks.push(block);
    //                 }
    //                 netServerMessage::Problem(problem) => {
    //                     println!("Received Problem: {:?}", problem);
    //                     // Problem 처리 로직 (필요 시 추가)
    //                 }
    //             }
    //         }
    //     } else {
    //         eprintln!("server_msg_receiver is None. Cannot process server messages.");
    //     }
    // }

    /// 임의의 블록 추가
    fn add_random_block(&mut self) {
        let mut rng = thread_rng();
        let problem1 = Problem {
            matrix: vec![
                vec![1, 2, 3, 4],
                vec![5, 6, 7, 8],
                vec![9, 10, 11, 12],
                vec![13, 14, 15, 16],
            ],
        };
        let problem2 = Problem {
            matrix: vec![
                vec![1, 2, 3, 4],
                vec![5, 6, 7, 8],
                vec![9, 10, 11, 12],
                vec![13, 14, 15, 16],
            ],
        };
        
        let solution = vec![vec![3, 4]];

        let node_id = format!("Node{}", rng.gen_range(1..1000));
        let data = format!("Random Data {}", rng.gen::<u32>());

        let latest_index = self.db.load_latest_index().unwrap_or(0);

        let latest_block = match self.db.load_block(latest_index) {
            Some(block) => block,
            None => {
                let genesis_block = Block::new(
                    0,
                    problem1,
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
            problem2,
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

    /// 블록 검증 로직 (여기서는 간단하게 true 반환)
    fn verify_block(&self, _block: &Block) -> bool {
        // 실제 검증 로직을 여기에 구현
        // 예: 해시 검사, nonce 검사 등등
        true
    }

    /// 블록을 검증하고 그 결과를 서버에 제출
    async fn validate_and_submit_block(&self, block: &Block) -> Result<(), String> {
        // 1) 블록 유효성 검사
        let is_valid = self.verify_block(block);

        // 2) 서버로 전송할 검증결과 구조체 생성
        let validation_result = ValidationResult {
            is_valid,
            node_id: "client_node_id".to_string(), // 실제 노드 ID를 사용
        };

        // 서버 URL은 실제 서버 주소로 변경
        let server_url = "http://143.248.196.38:3000";

        // 3) 서버에 POST 전송
        if let Err(e) = network::submit_validation_result(server_url, &validation_result).await {
            Err(format!("Failed to submit validation result: {}", e))
        } else {
            Ok(())
        }
    }

    /// 거래를 서버에 제출하는 함수
    async fn submit_transaction(&self, sender_id: String, receiver_id: String, amount: u32) -> Result<(), String> {
        let transaction = network::Transaction {
            sender_id,
            receiver_id,
            amount,
        };

        // 서버 URL은 실제 서버 주소로 변경
        let server_url = "http://143.248.196.38:3000";

        if let Err(e) = network::submit_transaction(server_url, &transaction).await {
            Err(format!("Failed to submit transaction: {}", e))
        } else {
            Ok(())
        }
    }
}

// Default 구현 (Application 초기화 등에 사용)
impl Default for BlockchainClientGUI {
    fn default() -> Self {
        let (state, _tx) = BlockchainClientGUI::new("blockchain_db", "transaction_db");
        state
    }
}

//------------------------//
// iced::Application 구현 //
//------------------------//
impl Application for BlockchainClientGUI {
    // Executor (스레드 풀 설정)
    type Executor = executor::Default;

    // 메시지 타입
    type Message = Message;

    // 사용할 테마
    type Theme = Theme;

    // main에서 넘겨줄 Flags (여기서는 사용 X)
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (mut state, tx) = BlockchainClientGUI::new("blockchain_db","transaction");

        // WebSocket 연결을 비동기로 시작
        let server_url = "http://143.248.196.38:3000"; // 실제 서버 주소로 변경
        let ws_command = Command::perform(
            async move {
                network::connect_to_websocket(server_url, tx).await
            },
            |_| Message::LoadChainInfo, // 성공/실패와 상관없이 LoadChainInfo 발생
        );

        (state, ws_command)
    }

    // 윈도우 타이틀 설정
    fn title(&self) -> String {
        String::from("블록체인 클라이언트")
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Some(rx_arc) = &self.server_msg_receiver {
            // (1) Arc::clone
            let cloned = Arc::clone(rx_arc);
    
            unfold("my-sub", cloned, |rx_arc| async move {
                let mut guard = rx_arc.lock().await;
                let message = match guard.recv().await {
                    Some(msg) => Message::ServerMessage(msg),
                    None => Message::NoMoreMessages,
                };
            
                drop(guard); // 명시적 락 해제
            
                (message, rx_arc)
            })
            
        } else {
            Subscription::none()
        }
    }
    

    // 메시지 처리 (상태 업데이트)
    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            // 탭 변경
            Message::TabSelected(i) => {
                self.active_tab = i;
                Command::none()
            }

            // 1) '코인 채굴하기' 탭에서 '풀이 제출' 버튼 누른 경우
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

                let example_problem = Problem {
                    matrix: vec![
                        vec![1, 2, 3, 4],
                        vec![5, 6, 7, 8],
                        vec![9, 10, 11, 12],
                        vec![13, 14, 15, 16],
                    ],
                };

                // 2) 서버로 보낼 BlockForServer 구성 (임시 값들로 예시)
                use network::BlockForServer;
                let block_data = BlockForServer {
                    index: 0, // 실제 블록 인덱스로 교체
                    timestamp: "temp-timestamp".to_string(), // 실제 타임스탬프로 교체
                    problem: example_problem,
                    // solution: parsed_solution,
                    solution: parsed_solution.clone(),    
                    prev_solution: vec![
                        vec![1, 2, 3, 13],
                        vec![5, 11, 10, 8],
                        vec![9, 7, 6, 12],
                        vec![4, 14, 15, 1],
                    ],           
                    node_id: "client-node-id".to_string(),
                    data: "data".to_string(),
                };

                // 3) 비동기 전송 - Command::perform 사용
                //    http://143.248.196.38:3000 등 실제 서버 주소로 교체
                let server_url = "http://143.248.196.38:3000";
                let future = async move {
                    match network::submit_solution_block(server_url, &block_data).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    }
                };

                // Command::perform(...)를 반환하여 iced가 비동기 처리 후 메시지를 다시 보냄
                Command::perform(future, Message::SubmitSolutionFinished)
            }
            // 1)-b) 블록 제출에 대한 결과 처리
            Message::SubmitSolutionFinished(result) => {
                match result {
                    Ok(()) => println!("Server accepted the solution block successfully!"),
                    Err(err_msg) => eprintln!("Error submitting solution block: {}", err_msg),
                }
                Command::none()
            }

            // 2) '코인 채굴하기' 탭의 입력 필드
            Message::InputChanged(row, col, value) => {
                // 블록 풀이 입력 필드
                if row < 4 && col < 4 {
                    self.solution_input[row][col] = value;
                }
                // 만약 거래 입력 필드를 여기서 처리한다면 추가 로직 필요 (지금은 미사용)
                Command::none()
            }

            // 3) 로컬 체인 정보 로드
            Message::LoadChainInfo => {
                self.blocks = self.db.load_all_blocks();
                Command::none()
            }

            // 4) 체인 리셋
            Message::ResetDB => {
                self.reset_db();
                println!("DB has been reset!");
                Command::none()
            }

            // 5) 랜덤 블록 추가
            Message::AddRandomBlock => {
                self.add_random_block();
                println!("Random block added!");
                Command::none()
            }
            
            Message::VerifyBlock => {
                // 서버에서 받은 블록이 있는지 확인
                if let Some(proposed) = self.proposed_block.take() {
                    // 로컬 DB의 최신 블록
                    let latest_index = self.db.load_latest_index().unwrap_or(0);
                    let latest_block = self.db.load_block(latest_index).unwrap_or_else(|| {
                        // 로컬이 비어있다면 Genesis 블록 생성
                        Block::new(
                            0,
                            Problem { matrix: vec![vec![0;4];4] },
                            vec![],
                            vec![],
                            "GenesisNode".into(),
                            "Genesis Block".into()
                        )
                    });
                    
                    // 새 블록 생성
                    let new_block = Block::new(
                        latest_block.index + 1,
                        proposed.problem.clone(),
                        proposed.solution.clone(),
                        latest_block.solution.clone(), // 이전 블록의 solution
                        proposed.node_id.clone(),
                        proposed.data.clone()
                    );
                    
                    // 유효성 검사(예: self.verify_block(&new_block)) 후 DB 저장
                    self.db.save_block(&new_block);
                    self.db.save_latest_index(new_block.index);
                    self.blocks = self.db.load_all_blocks();
            
                    println!("검증 성공! 새로운 블록을 DB에 추가했습니다: {:?}", new_block);
                } else {
                    println!("검증할 블록이 없습니다: proposed_block이 None");
                }
                Command::none()
            }
            
            
            // 검증 실패 -> 아무 것도 안 함(무시)
            Message::RejectBlock => {
                println!("블록 검증 실패! 제안된 블록을 폐기합니다.");
                self.server_msg_receiver = None; // Replace with a new receiver
                Command::none()
            }

        // --------------------------------------
        // 1) WebSocket 수신: 서버가 새 블록을 전달
        // --------------------------------------
                    // 서버 메시지 처리: Block
                    Message::ServerMessage(netServerMessage::Block(block)) => {
                        println!("서버에서 블록을 수신하였습니다: {:?}", block);
                        self.proposed_block = Some(block.clone());
        
                        // 블록 검증 및 제출
                        let validation_result = ValidationResult {
                            is_valid: self.verify_block(&block),
                            node_id: "client_node_id".to_string(),
                        };
        
                        let future = async move {
                            let server_url = "http://143.248.196.38:3000";
                            network::submit_validation_result(server_url, &validation_result)
                                .await
                                .map_err(|e| e.to_string())
                        };
        
                        Command::perform(future, Message::SubmitValidationFinished)
                    }
            
        // ---------------------------------------------------------
        // 2) 거래 전송: 굳이 &self 메서드를 직접 async로 안 쓰는 방식
        // ---------------------------------------------------------
        Message::TransactionSubmit(sender, receiver, amount) => {
            // (1) 거래 데이터 (소유권) 생성
            let transaction = network::Transaction {
                sender_id: sender,
                receiver_id: receiver,
                amount,
            };
            // (2) 별도의 동기/검증 로직이 필요하다면, 여기서 self를 사용하고 즉시 끝냄
            //     (예: self.db 잔액 조회)

            // (3) 나머지 통신은 'static Future 로
            let future = async move {
                let server_url = "http://143.248.196.38:3000";
                network::submit_transaction(server_url, &transaction)
                    .await
                    .map_err(|e| e.to_string())
            };
            return Command::perform(future, Message::TransactionFinished);
        }

        // --- 트랜잭션 관련 --- ***
        Message::AddRandomTransaction => {
            self.add_random_transaction();
            Command::none()
        }
        Message::ResetTxDB => {
            self.reset_tx_db();
            Command::none()
        }

        Message::NoMoreMessages => {
            // 채널이 닫힌 뒤에 계속 들어오는 “더미” 메시지
            // 특별히 할 일이 없다면 그냥 Command::none()
            println!("NoMoreMessages: channel is closed. Doing nothing...");
            Command::none()
        }

        // ---------------------
        // 3) 검증/트랜잭션 후처리
        // ---------------------
        Message::SubmitValidationFinished(result) => {
            match result {
                Ok(()) => println!("Validation result submitted successfully!"),
                Err(err_msg) => eprintln!("Error submitting validation result: {}", err_msg),
            }
            Command::none()
        }
        Message::TransactionFinished(result) => {
            match result {
                Ok(()) => println!("Transaction completed successfully!"),
                Err(err_msg) => eprintln!("Error submitting transaction: {}", err_msg),
            }
            Command::none()
        }
        // 서버 메시지 처리: Problem
        Message::ServerMessage(netServerMessage::Problem(problem)) => {
            println!("Received Problem: {:?}", problem);
            // Problem 처리 로직 추가
            Command::none()
        }
        Message::ReceivedProposedBlock(server_message) => todo!(),
    }
}

    // 뷰(화면) 구성
    fn view(&self) -> Element<'_, Message> {
        // 탭 구성
        let tabs = Tabs::new(Message::TabSelected)
            .push(
                0,
                TabLabel::Text("코인 채굴하기".to_owned()),
                view_problem_solving(self),
            )
            .push(
                1,
                TabLabel::Text("로컬 체인 정보 & 거래 내역".to_owned()),
                view_chain_info(&self.blocks, &self.transactions),
            )
            .push(
                2,
                TabLabel::Text("블록 검증".to_owned()),
                view_block_verification(self.blocks.last(), self.proposed_block.as_ref()),
            )
            .set_active_tab(&self.active_tab);

        container(tabs)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

// 메인 함수
fn main() -> iced::Result {
    BlockchainClientGUI::run(Settings::default())
}
