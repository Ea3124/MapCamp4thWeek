use iced::{
    alignment::{Alignment, Horizontal},
    widget::{Button, container, text, Column, Row, Container},
    Element, Length, Color, Border, Shadow, Theme,
};
use crate::Message;
use crate::blockchain::blockchain_db::Block;

/// 사용자 정의 스타일: 파란색 컨테이너
struct BlueContainer;

impl container::StyleSheet for BlueContainer {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance { 
            text_color: None, 
            background: Some(Color::from_rgb(0.1, 0.4, 0.8).into()), 
            border: Border::default(), 
            shadow: Shadow::default(), 
        }
    }
}

impl From<BlueContainer> for iced::theme::Container {
    fn from(style: BlueContainer) -> Self {
        iced::theme::Container::Custom(Box::new(style))
    }
}

/// 블록 검증 뷰: 로컬 블록과 서버에서 제안된 블록을 표시
pub fn view_block_verification<'a>(
    last_block: Option<&'a Block>,
    server_block: Option<&'a (Block, bool)>,
) -> Element<'a, Message> {
    // Helper function to create a styled block container
    fn create_block_container<'a>(block_info: Column<'a, Message>) -> Container<'a, Message> {
        Container::new(block_info)
            .padding(10)
            .width(Length::Fill)
            .style(BlueContainer)
    }

    // Helper function to format a matrix as a Column of text
    fn format_matrix(matrix: &Vec<Vec<u32>>) -> Column<'_, Message> {
        matrix.iter().fold(Column::new().spacing(5), |col, row| {
            let row_text = row.iter().map(|val| format!("{}", val)).collect::<Vec<_>>().join(", ");
            col.push(text(row_text))
        })
    }

    // Function to build block information similar to chain_info.rs
    fn build_block_info<'a>(block: &'a Block) -> Column<'a, Message> {
        // Timestamp and Node ID
        let timestamp_node_row = Row::new()
            .spacing(10)
            .push(text(format!("Timestamp: {}", block.timestamp)))
            .push(text(format!("Node ID: {}", block.node_id)));

        // Problem section
        let problem_section = Column::new()
            .spacing(10)
            .push(text("Problem:").size(16))
            .push(format_matrix(&block.problem.matrix));

        // Solution section
        let solution_section = Column::new()
            .spacing(10)
            .push(text("Solution:").size(16))
            .push(format_matrix(&block.solution));

        // Prev_Solution section
        let prev_solution_section = Column::new()
            .spacing(10)
            .push(text("Prev_Solution:").size(16))
            .push(format_matrix(&block.prev_solution));

        // Main section with Problem, Solution, Prev_Solution
        let main_section = Row::new()
            .spacing(20)
            .push(problem_section)
            .push(solution_section)
            .push(prev_solution_section);

        // Data row
        let data_row = Row::new().push(text(format!("Data: {}", block.data)));

        // Combine all sections into a single column
        Column::new()
            .spacing(10)
            .push(timestamp_node_row)
            .push(main_section)
            .push(data_row)
    }

    // Local Last Block Section
    let local_section = {
        let title = text("Last Local Block").size(20);
        let content = if let Some(block) = last_block {
            build_block_info(block)
        } else {
            Column::new().push(text("There are no local blocks"))
        };

        Container::new(
            Column::new()
                .spacing(10)
                .push(title)
                .push(create_block_container(content))
        )
        .padding(10)
        .width(Length::FillPortion(1))
    };

    // Proposed Block from Server Section
    let server_section: Element<Message> = {
        let title = text("Proposed Block from Server").size(20);

        if let Some((block, is_verified)) = server_block {
            // Build block info
            let block_info = build_block_info(block)
                .push(text(format!(
                    "Verification Status: {}",
                    if *is_verified { "Verified" } else { "Pending" }
                )));

            // Buttons for verification
            let buttons = Row::new()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(
                    Button::new(text("Accept"))
                        .padding(10)
                        .on_press(Message::VerifyBlock),
                )
                .push(
                    Button::new(text("Reject"))
                        .padding(10)
                        .on_press(Message::RejectBlock),
                );

            // Combine block info and buttons
            let content = Column::new()
                .spacing(20)
                .push(create_block_container(block_info))
                .push(
                    Container::new(buttons)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center),
                );

            Container::new(
                Column::new()
                    .spacing(10)
                    .push(title)
                    .push(content)
            )
            .padding(10)
            .width(Length::FillPortion(1))
            .into()
        } else {
            Container::new(
                Column::new()
                    .spacing(10)
                    .push(title)
                    .push(
                        Container::new(text("There are no proposed blocks"))
                            .padding(10)
                            .width(Length::Fill)
                            .style(BlueContainer)
                    )
            )
            .padding(10)
            .width(Length::FillPortion(1))
            .into()
        }
    };

    // Arrange Local and Server sections side by side
    let content = Row::new()
        .spacing(20)
        .push(local_section)
        .push(server_section);

    // Wrap the content in a container with padding
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
}
