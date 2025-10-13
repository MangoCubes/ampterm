use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

use color_eyre::Result;

use crate::components::traits::component::Component;

pub struct Nothing {}
impl Component for Nothing {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [horizontal] = Layout::horizontal([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .areas(area);
        let [centered] = Layout::vertical([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .areas(horizontal);
        frame.render_widget(
            Paragraph::new(vec![Line::raw("Nothing in the queue")])
                .centered()
                .block(Block::default().padding(Padding::new(0, 0, area.height / 2, 0)))
                .wrap(Wrap { trim: false }),
            centered,
        );
        Ok(())
    }
}
