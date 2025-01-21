// main.rs

// Imports: 뷰 함수들과 라이브러리리
mod views;
use views::problem_solving::view_problem_solving;
use views::chain_info::view_chain_info;
use views::transaction_verification::view_transaction_verification;

use iced::{
    theme::Theme,
    widget::{container},
    Element, Length,
    application,
};
use iced_aw::{TabLabel, Tabs};

// 구조체 선언
#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    SubmitSolution,
    InputChanged(usize, usize, String), // (행, 열, 새로운 값)
}

#[derive(Default)]
struct BlockchainClientGUI {
    active_tab: usize,
    solution_input: [[String; 4]; 4], // 4x4 정답 입력 상태
}

// 업데이트: 탭 전환
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
    }
}

// 메인 뷰
fn view<'a>(state: &'a BlockchainClientGUI) -> Element<'a, Message> {
    println!("현재 활성 탭: {}", state.active_tab); // debug
    let tabs = Tabs::new(Message::TabSelected)
        .push(
            0,
            TabLabel::Text("코인 채굴하기".to_owned()),
            view_problem_solving(state),
        )
        .push(
            1,
            TabLabel::Text("로컬 체인 정보 & 거래 내역".to_owned()),
            view_chain_info(),
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
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into() 
}


fn main() -> iced::Result {
    application(
        "블록체인 클라이언트", // 타이틀
        update,                // 업데이트 함수
        view,                  // 뷰 함수
    )
    .theme(|_| Theme::Dark)     // 테마 설정 (선택 사항)
    .run()
}