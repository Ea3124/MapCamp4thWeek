// client/src/main.rs

// Imports
mod views;
mod blockchain;
mod network;

use blockchain::blockchain_db::Problem;
use tokio::sync::mpsc::unbounded_channel;
use views::problem_solving::view_problem_solving;
use views::chain_info::view_chain_info;
use views::transaction_verification::view_transaction_verification;

use blockchain::blockchain_db::BlockChainDB;
use blockchain::blockchain_db::Block;

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

// 메시지 열거형
#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    SubmitSolution,
    InputChanged(usize, usize, String), // (행, 열, 새로운 값)
    LoadChainInfo,                      // 체인 정보를 로드하는 메시지
    ResetDB,          // DB 초기화 메시지
    AddRandomBlock,   // 블록 추가 메시지

    // 서버 전송 후 결과를 받는 메시지
    SubmitSolutionFinished(Result<(), String>),
    SubmitValidationFinished(Result<(), String>),  // ← 블록 검증 메시지 전송 완료 후 수신
    // 새로운 메시지: 서버로부터의 메시지 수신
    ServerMessage(netServerMessage),

    // 거래 관련 메시지
    TransactionSubmit(String, String, u32), // (sender, receiver, amount)
    TransactionFinished(Result<(), String>),
}

// 메인 상태 구조체
struct BlockchainClientGUI {
    active_tab: usize,
    solution_input: [[String; 4]; 4], // 4x4 정답 입력 상태
    transaction_input: (String, String, String), // (sender, receiver, amount)
    blocks: Vec<Block>,               // 로드된 블록 리스트
    db: BlockChainDB,                 // DB 인스턴스
    // 추가: 서버 메시지를 수신하기 위한 채널
    server_msg_receiver: tokio::sync::mpsc::UnboundedReceiver<netServerMessage>,
}

impl BlockchainClientGUI {
    fn new(db_path: &str) -> (Self, tokio::sync::mpsc::UnboundedSender<netServerMessage>) {
        let db = BlockChainDB::new(db_path);

        // 시작 시 DB에서 기존 블록들을 불러옵니다.
        let blocks = db.load_all_blocks();

        // 채널 생성 (서버 -> 클라이언트 메시지)
        let (tx, rx) = unbounded_channel();

        (
            BlockchainClientGUI {
                active_tab: 0,
                solution_input: Default::default(),
                transaction_input: (String::new(), String::new(), String::new()),
                blocks,
                db,
                server_msg_receiver: rx,
            },
            tx, // sender 반환
        )
    }

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
        let (state, _tx) = BlockchainClientGUI::new("blockchain_db");
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
        let (mut state, tx) = BlockchainClientGUI::new("blockchain_db");

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
                    solution: vec![
                        vec![1, 2, 3, 13],
                        vec![5, 11, 10, 8],
                        vec![9, 7, 6, 12],
                        vec![4, 14, 15, 1],
                    ],    
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

        // --------------------------------------
        // 1) WebSocket 수신: 서버가 새 블록을 전달
        // --------------------------------------
        Message::ServerMessage(netServerMessage::Block(block)) => {
            // (1) 로컬 DB 저장
            println!("Received block: {:?}", block);
            self.db.save_block(&block);
            self.blocks.push(block.clone());

            // (2) 지금 즉시 '동기' 로직으로 블록 검증 (수명 문제 X)
            let is_valid = self.verify_block(&block);

            // (3) "검증 결과"만 소유(복사)해 서버 전송을 준비
            let validation_result = ValidationResult {
                is_valid,
                node_id: "client_node_id".to_string(),
            };

            // (4) `Command::perform(...)`에 넘길 Future는 'static 으로!
            //     validation_result는 move하여 Future 내부 소유로 만듦
            let future = async move {
                let server_url = "http://143.248.196.38:3000";
                // 서버에 검증 결과 전송
                network::submit_validation_result(server_url, &validation_result)
                    .await
                    .map_err(|e| e.to_string())
            };

            // (5) 비동기 Future → SubmitValidationFinished
            return Command::perform(future, Message::SubmitValidationFinished);
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
        },
        Message::ServerMessage(netServerMessage::Problem(_)) => todo!()
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
                view_chain_info(&self.blocks),
            )
            .push(
                2,
                TabLabel::Text("블록 검증".to_owned()),
                view_transaction_verification(),
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
