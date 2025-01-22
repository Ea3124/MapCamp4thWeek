use iced::{
    alignment::Alignment,
    widget::{button, container, text, Column, Row, Scrollable, Container},
    Element, Length, Color, Border, Shadow, Theme,
};
use crate::Message;
use crate::blockchain::blockchain_db::Block;
use crate::transaction::transaction::Transaction;

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
/// 블록과 거래내역(트랜잭션)을 함께 표시하는 뷰
pub fn view_chain_info<'a>(
    blocks: &'a [Block],
    transactions: &'a [Transaction],
) -> Element<'a, Message> {
    let blocks_scrollable = Scrollable::new(
        blocks.iter().fold(Column::new().spacing(10), |col, block| {
            // 첫 번째 행: Index, Timestamp, Node ID
            let top_row = Row::new()
                .spacing(10)
                .push(text(format!("Index: {}", block.index)))
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

            // 위의 세 행을 Column으로 묶음
            let block_info = Column::new()
                .spacing(5)
                .push(top_row)
                .push(middle_row)
                .push(bottom_row);

            let framed_block = Container::new(block_info)
                .padding(10)
                .width(Length::Fill)
                .style(BlueContainer); // 사용자 정의 스타일 적용

            col.push(framed_block)
        })
    )
    .height(Length::FillPortion(9));

    let blocks_section = Column::new()
        .spacing(10)
        .push(text("Local BlockChain").size(20))
        .push(
            Container::new(blocks_scrollable)
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .push(
            Row::new()
                .spacing(10)
                .push(button("Reset Block DB").padding(10).on_press(Message::ResetDB))
                .push(button("Add Random Block").padding(10).on_press(Message::AddRandomBlock)),
        );

    let tx_scrollable = Scrollable::new(
        transactions.iter().fold(Column::new().spacing(10), |col, tx| {
            let tx_row = Row::new()
                .spacing(10)
                .push(text(format!("TxIndex: {}", tx.index)))
                .push(text(format!("Sender: {}", tx.sender)))
                .push(text(format!("Receiver: {}", tx.receiver)))
                .push(text(format!("Payment: {}", tx.payment)));

            let framed_tx = Container::new(tx_row)
                .padding(10)
                .width(Length::Fill);

            col.push(framed_tx)
        })
    )
    .height(Length::FillPortion(9));

    let tx_section = Column::new()
        .spacing(10)
        .push(text("Transaction Memory").size(20))
        .push(
            Container::new(tx_scrollable)
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .push(
            Row::new()
                .spacing(10)
                .push(button("Reset TX DB").padding(10).on_press(Message::ResetTxDB))
                .push(button("Add Random TX").padding(10).on_press(Message::AddRandomTransaction)),
        );

    let info_row = Row::new()
        .spacing(20)
        .push(Container::new(blocks_section).width(Length::FillPortion(1)))
        .push(Container::new(tx_section).width(Length::FillPortion(1)));

    let content = Column::new()
        .padding(20)
        .align_items(Alignment::Start)
        .spacing(10)
        .push(info_row);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
