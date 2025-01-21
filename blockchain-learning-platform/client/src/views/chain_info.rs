use iced::{
    alignment::Alignment,
    widget::{container, text, Column, Row},
    Element, Length,
};
use crate::Message;
use crate::blockchain::blockchain_db::Block;

pub fn view_chain_info<'a>(blocks: &'a [Block]) -> Element<'a, Message> {
    let mut content = Column::new()
        .padding(20)
        .align_x(Alignment::Start)
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

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}