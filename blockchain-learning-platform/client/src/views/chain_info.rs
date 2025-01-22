use iced::{
    alignment::Alignment,
    widget::{button, container, text, Column, Row, Scrollable, Container},
    Element, Length,
};
use crate::Message;
use crate::blockchain::blockchain::Block;

pub fn view_chain_info<'a>(blocks: &'a [Block]) -> Element<'a, Message> {
    // 스크롤 가능한 블록 리스트 생성
    let blocks_scrollable = Scrollable::new(
        blocks.iter().fold(Column::new().spacing(10), |column, block| {
            // 각 블록 정보를 Row로 구성한 후 Container로 감쌈
            let block_row = Row::new()
                .spacing(10)
                .push(text(format!("Index: {}", block.index)))
                .push(text(format!("Node ID: {}", block.node_id)))
                .push(text(format!("Data: {}", block.data)))
                .push(text(format!("Timestamp: {}", block.timestamp)));

            // Container를 사용해 블록을 사각형 프레임으로 감싸기
            let framed_block = Container::new(block_row)
                .padding(10)
                .width(Length::Fill)
                .style(iced::theme::Container::Box);  // 기본 테두리 스타일 사용

            column.push(framed_block)
        })
    )
    .height(Length::FillPortion(4));

    // DB 초기화 및 랜덤 블록 추가 버튼 생성
    let buttons = Row::new()
        .spacing(10)
        .push(
            button("Reset DB")
                .padding(10)
                .on_press(Message::ResetDB)
        )
        .push(
            button("Add Random Block")
                .padding(10)
                .on_press(Message::AddRandomBlock)
        );

    let content = Column::new()
        .padding(20)
        .align_items(Alignment::Start)
        .spacing(10)
        .push(text("Local Blockchain Information").size(24))
        .push(text("Stored Blocks:").size(18))
        .push(
            Container::new(blocks_scrollable)
                .padding(10)
        )
        .push(buttons);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
