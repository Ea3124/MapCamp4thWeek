use iced::{
    alignment::{Alignment, Horizontal, Vertical},
    widget::{button, column, container, text, text_input, Row, Container},
    Element, Length, Border, Shadow, Theme,
};
use crate::Message;

/// 사용자 정의 스타일: 테두리
struct BorderStyle;

impl container::StyleSheet for BorderStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,              
            background: None,               
            border: Border {
                width: 1.0,                    // Set border width
                radius: 0.0.into(),                   // Set border radius
                color: iced::Color::BLACK,     // Set border color
            },
            shadow: Shadow::default(),                   
        }
    }
}

impl From<BorderStyle> for iced::theme::Container {
    fn from(style: BorderStyle) -> Self {
        iced::theme::Container::Custom(Box::new(style))
    }
}

// 뷰함수
pub fn view_problem_solving<'a>(state: &'a crate::BlockchainClientGUI) -> Element<'a, Message> {
    let problem_matrix = if let Some(problem) = &state.current_problem {
        &problem.matrix
    } else {
        // 수신한 문제가 없으면 기존에 쓰던 예시를 사용
        &vec![
            vec![0,0,0,0],
            vec![0,0,0,0],
            vec![0,0,0,0],
            vec![0,0,0,0],
        ]
    };

    let problem_view = column![
        text("4x4 Magic Square Problem").size(24),
        column(
            problem_matrix.iter().map(|row| {
                Row::with_children(
                    row.iter().map(|&num| {
                        // 1) num == 0일 때 빈 문자열로 표시
                        let display_text = if num == 0 {
                            "".to_string() // 빈 문자열
                        } else {
                            num.to_string() // 숫자를 문자열로 변환
                        };

                        container(
                            text(display_text)
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(Horizontal::Center)
                                .vertical_alignment(Vertical::Center)
                        )
                        .style(BorderStyle)
                        .width(Length::Fixed(50.0))
                        .height(Length::Fixed(50.0))
                        .into()
                    }).collect::<Vec<_>>()
                )
                .spacing(10)
                .into()
            })
        ).spacing(10)
    ]
    .spacing(20) 
    .align_items(Alignment::Center);

    let solution_inputs = column![
        text("Your Solution").size(24),
        column(
            (0..4).map(|i| {
                Row::with_children(
                    (0..4).map(|j| {
                        text_input("", &state.solution_input[i][j])
                            .on_input(move |value| Message::InputChanged(i, j, value))
                            .padding(5)
                            .width(Length::Fixed(50.0))
                            .into()
                    }).collect::<Vec<_>>()
                )
                .spacing(10)
                .into()
            })
        )
        .spacing(10),
    ];

    container(
        column![
            problem_view,
            solution_inputs,
            button("Submit")
                .padding(10)
                .on_press(Message::SubmitSolution),
        ]
        .spacing(20)
        .align_items(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
