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
// 핵심: iced 관련 import 정리
// ------------------------------
use iced::{
    Application, // iced 루트에서 Application 트레이트
    Element,
    Length,
    Settings,    // iced 루트에서 Settings 구조체
    Theme,
    executor,    // executor::Default 사용
    widget::container,
};
use iced_aw::{TabLabel, Tabs};
use tokio::process::Command;

// 메시지 열거형
#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    SubmitSolution,
    InputChanged(usize, usize, String), // (행, 열, 새로운 값)
    LoadChainInfo,                      // 체인 정보를 로드하는 메시지
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

        BlockchainClientGUI {
            active_tab: 0,
            solution_input: Default::default(),
            blocks: Vec::new(),
            db,
        }
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
            println!("Solution submitted!");
        }
        Message::InputChanged(row, col, value) => {
            state.solution_input[row][col] = value;
        }
        Message::LoadChainInfo => {
            // DB에서 체인 정보 로드
            state.blocks = state.db.load_all_blocks();
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
        .center_x()
        .center_y()
        .into()
}

//--------------//
// main 함수   //
//--------------//
fn main() -> iced::Result {
    // 앱 실행
    BlockchainClientGUI::run(Settings::default())
}
