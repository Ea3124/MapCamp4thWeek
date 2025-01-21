use iced::{
    alignment::Alignment,
    widget::{container, text, Column},
    Element, Length
};
use crate::Message;

pub fn view_transaction_verification<'a>() -> Element<'a, Message> {
    let content = Column::new()
        .padding(20)
        .align_x(Alignment::Start)
        .push(text("Transaction Verification").size(24))
        .push(text("Result").size(18));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}