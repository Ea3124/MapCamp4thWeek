use iced::{
    alignment::Alignment,
    widget::{container, text, Column},
    Element, Length
};
use crate::Message;

pub fn view_chain_info<'a>() -> Element<'a, Message> {
    let content = Column::new()
        .padding(20)
        .align_x(Alignment::Start)
        .push(text("Information about BlockChain stored in Local DB").size(24))
        .push(text("Information").size(18));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}