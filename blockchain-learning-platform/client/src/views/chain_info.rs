use iced::{
    alignment::Alignment,
    widget::{button,container, text, Column, Row},
    Element, Length,
};
use crate::Message;
use crate::blockchain::blockchain_db::Block;

pub fn view_chain_info<'a>(blocks: &'a [Block]) -> Element<'a, Message> {
    let mut content = Column::new()
        .padding(20)
        .align_items(Alignment::Start)
        .push(text("Local Blockchain Information").size(24))
        .push(text("Stored Blocks:").size(18));

    // 블록 데이터를 UI로 추가
    for block in blocks {
        let block_info = Row::new()
            .spacing(10)
            .push(text(format!("Index: {}", block.index)))
            .push(text(format!("Node ID: {}", block.node_id)))
            .push(text(format!("Data: {}", block.data)))
            .push(text(format!("Timestamp: {}", block.timestamp)));

        content = content.push(block_info);
    }

    // 아래 두 개의 버튼을 추가
    // 1) DB 초기화
    content = content.push(
        button("Reset DB")
            .padding(10)
            .on_press(Message::ResetDB),
    );

    // 2) 블록 추가
    content = content.push(
        button("Add Random Block")
            .padding(10)
            .on_press(Message::AddRandomBlock),
    );

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
