// client/src/main.rs

// Imports
mod views;
mod blockchain;
mod network;

use blockchain::blockchain_db::Problem;
use tokio::sync::mpsc::unbounded_channel;
use views::problem_solving::view_problem_solving;
use views::chain_info::view_chain_info;
use views::block_verification::view_block_verification;

use blockchain::blockchain_db::{Block, BlockChainDB};

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
use chrono::{DateTime, TimeZone, Utc, FixedOffset};

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
    proposed_block: Option<(Block, bool)>,
    // 서버에서 받은 현재 문제
    current_problem: Option<Problem>,
    // 내 정보
    my_node_id: String,
    my_balance: u64, 
}

impl BlockchainClientGUI {
    // fn new(db_path: &str) -> (Self, tokio::sync::mpsc::UnboundedSender<netServerMessage>) { //*** 
    fn new(db_path: &str)
        -> (Self, tokio::sync::mpsc::UnboundedSender<netServerMessage>) {
        let db = BlockChainDB::new(db_path);

        // DB를 열고 블록이 없는 경우 제네시스 블록 생성
        if db.load_latest_index().is_none() {
            db.reset_db();  // reset_db 내부에서 제네시스 블록 생성
        }

        // 시작 시 DB에서 기존 블록들을 불러옵니다.
        let blocks = db.load_all_blocks();

        // 내 정보
        let my_node_id = Self::generate_random_node_id();
        let my_balance = 0;

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
            current_problem: None, // 현재 문제 초기화
            my_node_id,
            my_balance,
        };
        (gui, tx)
    }

    fn generate_random_node_id() -> String {
        let mut rng = thread_rng();
        let names = ["SangNamJa", "HeeSeungSim", "SeungJaeLee","MadCamp2024W","JunhoKim","JunhoPark"];
        // 예: A, B, C 중 하나 선택
        let chosen = names[rng.gen_range(0..names.len())];
        // 1~999 범위 중 3자리로 포맷
        let number = rng.gen_range(1..1000);
        format!("{}{:03}", chosen, number)
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
        String::from("상남자특 Rust로 블록체인 배움")
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

                let example_problem = self.current_problem.clone().unwrap_or_else(|| {
                    Problem{matrix: vec![ vec![0;4]; 4 ]}
                });

                let prev_solution = if let Some(last_block) = self.blocks.last() {
                    last_block.solution.clone()
                } else {
                    // 로컬 체인에 아무 블록이 없으면 빈 Vec
                    vec![]
                };

                let my_node_id = self.my_node_id.clone();

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

                // 2) 서버로 보낼 BlockForServer 구성 (임시 값들로 예시)
                use network::BlockForServer;
                let block_data = BlockForServer {
                    index: 0, // 실제 블록 인덱스로 교체
                    timestamp, 
                    problem: example_problem,
                    solution: parsed_solution,    
                    prev_solution,           
                    node_id: my_node_id,
                    data: "10".to_string(),
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
                        if let Some((proposed, _)) = self.proposed_block.take() {
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

                        // [CHANGED CODE] node_id가 자신의 아이디와 같으면 보상
                    if proposed.node_id == self.my_node_id {
                        self.my_balance += 10;
                        println!(
                            "블록 제출자가 나 자신이므로 보상으로 balance를 10 증가! 현재 잔액: {}",
                            self.my_balance
                        );
                    }
                            
                            // 검증 성공, 블록을 로컬 체인에 추가
                            self.db.save_block(&new_block);
                            self.db.save_latest_index(new_block.index);
                            self.blocks = self.db.load_all_blocks();
                            println!("로컬체인: {:?}", self.blocks.clone());
                    
                            // 서버로 검증 결과 전송
                            let validation_result = ValidationResult {
                                is_valid: true, // 검증 성공
                                node_id: "client_node_id".to_string(),
                            };
                            let server_url = "http://143.248.196.38:3000";
                            let future = async move {
                                network::submit_validation_result(server_url, &validation_result)
                                    .await
                                    .map_err(|e| e.to_string())
                            };
                    
                            println!("블록 검증 성공: 블록 추가 및 서버에 결과 전송");
                            return Command::perform(future, Message::SubmitValidationFinished);
                        } else {
                            println!("검증할 블록이 없습니다!");
                        }
                        Command::none()
                    }
                    
            Message::RejectBlock => {
                if let Some((block, _)) = self.proposed_block.take() {
                    // 검증 실패, 블록 폐기
                    println!("블록 검증 실패: 블록 폐기 - {:?}", block);
            
                    // 서버로 검증 실패 결과 전송
                    let validation_result = ValidationResult {
                        is_valid: false, // 검증 실패
                        node_id: "client_node_id".to_string(),
                    };
                    let server_url = "http://143.248.196.38:3000";
                    let future = async move {
                        network::submit_validation_result(server_url, &validation_result)
                            .await
                            .map_err(|e| e.to_string())
                    };
            
                    return Command::perform(future, Message::SubmitValidationFinished);
                } else {
                    println!("검증할 블록이 없습니다!");
                }
                Command::none()
            }
                    

        // --------------------------------------
        // 1) WebSocket 수신: 서버가 새 블록을 전달
        // --------------------------------------
            // 서버 메시지 처리: Block
            Message::ServerMessage(netServerMessage::Block(block)) => {
                println!("서버에서 블록 수신: {:?}", block);
                self.proposed_block = Some((block.clone(), false)); // 검증 대기 상태로 저장
                Command::none()
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

                self.current_problem = Some(problem.clone()); // 수신한 문제를 state에 저장

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
                TabLabel::Text("내 정보".to_owned()),
                view_chain_info(&self.blocks, &self.my_node_id, self.my_balance),
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

