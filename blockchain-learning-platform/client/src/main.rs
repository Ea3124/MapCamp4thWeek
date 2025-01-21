// main.rs

// Imports
mod views;
mod blockchain;

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

// 메시지 열거형
#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    SubmitSolution,
    InputChanged(usize, usize, String), // (행, 열, 새로운 값)
    LoadChainInfo,                      // 체인 정보를 로드하는 메시지
    ResetDB,          // DB 초기화 메시지
    AddRandomBlock,   // 블록 추가 메시지
    // **서버 전송 후 결과를 받는 메시지**
    SubmitSolutionFinished(Result<(), String>),
}

// 메인 상태 구조체
struct BlockchainClientGUI {
    active_tab: usize,
    solution_input: [[String; 4]; 4], // 4x4 정답 입력 상태
    blocks: Vec<Block>,               // 로드된 블록 리스트
    db: BlockChainDB,                 // DB 인스턴스
}

impl BlockchainClientGUI {
    fn new(db_path: &str) -> Self {
        let db = BlockChainDB::new(db_path);

        // 시작 시 DB에서 기존 블록들을 불러옵니다.
        let blocks = db.load_all_blocks();

        BlockchainClientGUI {
            active_tab: 0,
            solution_input: Default::default(),
            blocks,
            db,
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
}

// Default 구현 (Application 초기화 등에 사용)
impl Default for BlockchainClientGUI {
    fn default() -> Self {
        Self::new("blockchain_db")
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
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        update(self, message);
        Command::none()
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
fn update(state: &mut BlockchainClientGUI, msg: Message) {
    match msg {
        Message::TabSelected(i) => {
            state.active_tab = i;
        }
        Message::SubmitSolution => {
            println!("Solution submitted! Now sending to server...");

            // 1) 4x4 string matrix -> Vec<Vec<u32>> 변환 (파싱)
            let parsed_solution = state
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
            iced::Command::perform(future, Message::SubmitSolutionFinished);
        }
        Message::SubmitSolutionFinished(result) => {
            match result {
                Ok(()) => println!("Server accepted the solution block successfully!"),
                Err(err_msg) => eprintln!("Error submitting solution block: {}", err_msg),
            }
        }
        Message::InputChanged(row, col, value) => {
            state.solution_input[row][col] = value;
        }
        Message::LoadChainInfo => {
            // DB에서 체인 정보 로드
            state.blocks = state.db.load_all_blocks();
        }
        // 테스트
        Message::ResetDB => {
            // 자유 함수이므로 self가 아닌 state
            state.reset_db();
            println!("DB has been reset!");
        }
        Message::AddRandomBlock => {
            state.add_random_block();
            println!("Random block added!");
        }
    }
}

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
            view_chain_info(&state.blocks),
        )
        .push(
            2,
            TabLabel::Text("블록 검증".to_owned()),
            view_transaction_verification(),
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
