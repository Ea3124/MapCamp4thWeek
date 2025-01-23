use iced::{
    alignment::{Alignment,Horizontal},
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

pub fn view_block_verification<'a>(
    last_block: Option<&'a Block>,
    server_block: Option<&'a (Block, bool)>,
) -> Element<'a, Message> {
    // Local Last Block 섹션: 제목과 정보 (1번 코드 형식 적용)
    let local_section = {
        let title = text("Local Last Block").size(20);
        let content = if let Some(block) = last_block {
            // 첫 번째 행: Timestamp, Node ID (Index는 생략; 필요 시 추가 가능)
            let top_row = Row::new()
                .spacing(10)
                .push(text(format!("Timestamp: {}", block.timestamp)))
                .push(text(format!("Node ID: {}", block.node_id)));
            // 두 번째 행: Problem, Solution, Prev_Solution
            let middle_row = Row::new()
                .spacing(10)
                .push(text(format!("Problem: {:?}", block.problem)))
                .push(text(format!("Solution: {:?}", block.solution)))
                .push(text(format!("Prev_Solution: {:?}", block.prev_solution)));
            // 세 번째 행: Data
            let bottom_row = Row::new()
                .spacing(10)
                .push(text(format!("Data: {}", block.data)));
            // 세 행을 Column으로 묶기
            Column::new()
                .spacing(5)
                .push(top_row)
                .push(middle_row)
                .push(bottom_row)
        } else {
            Column::new().push(text("There are no local blocks"))
        };
        
        Container::new(
            Column::new()
                .spacing(10)
                .push(title)
                .push(
                    Container::new(content)
                        .padding(10)
                        .width(Length::Fill)
                        .style(BlueContainer)
                )
        )
        .padding(10)
        .width(Length::Fill)
    };

    // Proposed Block from Server 섹션: 제목, 정보 및 버튼 (1번 코드 형식 적용)
    let server_section: Element<Message> = {
        let title = text("Proposed Block from Server").size(20);
        
        if let Some((block, is_verified)) = server_block {
            // 첫 번째 행: Timestamp, Node ID
            let top_row = Row::new()
                .spacing(10)
                .push(text(format!("Timestamp: {}", block.timestamp)))
                .push(text(format!("Node ID: {}", block.node_id)))
                .push(text(format!(
                    "Verification Status: {}",
                    if *is_verified { "Verified" } else { "Pending" }
                )));
        
            // 두 번째 행: Problem, Solution, Prev_Solution
            let middle_row = Row::new()
                .spacing(10)
                .push(text(format!("Problem: {:?}", block.problem)))
                .push(text(format!("Solution: {:?}", block.solution)))
                .push(text(format!("Prev_Solution: {:?}", block.prev_solution)));
        
            // 세 번째 행: Data
            let bottom_row = Row::new()
                .spacing(10)
                .push(text(format!("Data: {}", block.data)));
            // 블록 정보를 묶은 컬럼
            let server_info = Column::new()
                .spacing(5)
                .push(top_row)
                .push(middle_row)
                .push(bottom_row);
            
            let buttons = Row::new()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(
                    Button::new(text("pass"))
                        .padding(10)
                        .on_press(Message::VerifyBlock),
                )
                .push(
                    Button::new(text("fail"))
                        .padding(10)
                        .on_press(Message::RejectBlock),
                );

            let content = Column::new()
                .spacing(20)
                .push(
                    Container::new(server_info)
                        .padding(10)
                        .width(Length::Fill)
                        .style(BlueContainer),
                )
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
            .width(Length::Fill)
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
            .width(Length::Fill)
            .into()
        }
    };

    // Local과 Server 섹션을 가로로 배치
    let content = Row::new()
        .spacing(20)
        .push(local_section)
        .push(server_section);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
}
