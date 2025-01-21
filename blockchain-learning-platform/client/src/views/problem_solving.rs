use iced::{
    alignment::{Alignment, Horizontal, Vertical},
    widget::{button, column, container, text, text_input, Row},
    Element, Length,
};
use crate::Message;

pub fn view_problem_solving<'a>(state: &'a crate::BlockchainClientGUI) -> Element<'a, Message> {
    let problem_grid = vec![
        vec![16, 2, 3, 13],
        vec![5, 11, 10, 8],
        vec![9, 7, 6, 12],
        vec![4, 14, 15, 1],
    ];

    let problem_view = column![
        text("4x4 Magic Square Problem").size(24),
        column(
            problem_grid.into_iter().map(|row| {
                Row::with_children(
                    row.into_iter().map(|num| {
                        container(
                            text(num.to_string())
                                .width(Length::Fill)
                                .height(Length::Fill)
                                // align_x -> horizontal_alignment
                                .horizontal_alignment(Horizontal::Center)
                                .vertical_alignment(Vertical::Center)
                        )
                        .width(Length::Fixed(50.0))
                        .height(Length::Fixed(50.0))
                        .into()
                    }).collect::<Vec<_>>()
                )
                .spacing(10)
                .into()
            })
        )
        .spacing(10),
    ];

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
        // align_x -> align_items
        .align_items(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    // center_x(Length::Fill) -> center_x()
    // center_y(Length::Fill) -> center_y()
    .center_x()
    .center_y()
    .into()
}
