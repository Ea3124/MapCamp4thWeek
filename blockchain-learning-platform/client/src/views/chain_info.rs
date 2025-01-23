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
            // Index (상단 왼쪽)
            let index_row = Row::new().push(text(format!("Index: {}", block.index)));

            // Timestamp와 Node ID (Index 아래에 가로로 배치)
            let timestamp_node_row = Row::new()
                .spacing(10)
                .push(text(format!("Timestamp: {}", block.timestamp)))
                .push(text(format!("Node ID: {}", block.node_id)));

            // Problem (4x4 형태로 표시)
            let problem_matrix = block.problem.matrix.iter().fold(Column::new().spacing(5), |col, row| {
                let row_text = row.iter().map(|val| format!("{}", val)).collect::<Vec<_>>().join(", ");
                col.push(text(row_text))
            });

            let problem_section = Column::new()
                .spacing(10)
                .push(text("Problem:").size(16))
                .push(problem_matrix);

            // Solution (2D Vec 형태로 표시)
            let solution_matrix = block.solution.iter().fold(Column::new().spacing(5), |col, row| {
                let row_text = row.iter().map(|val| format!("{}", val)).collect::<Vec<_>>().join(", ");
                col.push(text(row_text))
            });

            let solution_section = Column::new()
                .spacing(10)
                .push(text("Solution:").size(16))
                .push(solution_matrix);

            // Prev_Solution (2D Vec 형태로 표시)
            let prev_solution_matrix = block.prev_solution.iter().fold(Column::new().spacing(5), |col, row| {
                let row_text = row.iter().map(|val| format!("{}", val)).collect::<Vec<_>>().join(", ");
                col.push(text(row_text))
            });

            let prev_solution_section = Column::new()
                .spacing(10)
                .push(text("Prev_Solution:").size(16))
                .push(prev_solution_matrix);

            // Problem, Solution, Prev_Solution을 가로로 배치
            let main_section = Row::new()
                .spacing(20) // 섹션 간 간격 조정
                .push(problem_section)
                .push(solution_section)
                .push(prev_solution_section);

            // Data (가장 하단)
            let data_row = Row::new().push(text(format!("Data: {}", block.data)));

            // 전체 레이아웃 구성
            let block_info = Column::new()
                .spacing(10)
                .push(index_row)
                .push(timestamp_node_row)
                .push(main_section)
                .push(data_row);

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
