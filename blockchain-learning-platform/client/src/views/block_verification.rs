use iced::{
    alignment::Alignment,
    widget::{container, text, Column, Row, Button, Container},
    Element, Length,
};
use crate::Message;
use crate::blockchain::blockchain::Block;

pub fn view_block_verification<'a>(
    last_block: Option<&'a Block>,
    server_block: Option<&'a Block>,
) -> Element<'a, Message> {
    let mut content = Column::new()
        .padding(20)
        .align_items(Alignment::Start)
        .spacing(10)
        .push(text("Block Verification").size(24))
        .push(text("Result").size(18));

    // 로컬 DB의 마지막 블록 표시 (인덱스 제외) - Container로 감싸기
    if let Some(block) = last_block {
        let local_info = Row::new()
            .spacing(10)
            .push(text("Local Last Block:"))
            .push(text(format!("Node ID: {}", block.node_id)))
            .push(text(format!("Data: {}", block.data)))
            .push(text(format!("Timestamp: {}", block.timestamp)));

        let framed_local = Container::new(local_info)
            .padding(10)
            .width(Length::Fill)
            .style(iced::theme::Container::Box);

        content = content.push(framed_local);
    } else {
        content = content.push(text("There are no local blocks"));
    }

    // 서버에서 받은 블록 표시 및 버튼 배치 - Container로 감싸기
    if let Some(block) = server_block {
        let server_info = Row::new()
            .spacing(10)
            .push(text("Proposed Block from Server:"))
            .push(text(format!("Node ID: {}", block.node_id)))
            .push(text(format!("Data: {}", block.data)))
            .push(text(format!("Timestamp: {}", block.timestamp)));

        let framed_server = Container::new(server_info)
            .padding(10)
            .width(Length::Fill)
            .style(iced::theme::Container::Box);

        let buttons = Row::new()
            .spacing(20)
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

        content = content
            .push(framed_server)
            .push(buttons);
    } else {
        content = content.push(text("There are no proposed blocks"));
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
