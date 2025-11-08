use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

use crate::components::traits::renderable::Renderable;

pub struct Centered {
    paragraph: Paragraph<'static>,
}

impl Centered {
    pub fn new(msg: Vec<String>) -> Self {
        let lines: Vec<Line> = msg.into_iter().map(|s| Line::raw(s)).collect();
        Centered {
            paragraph: Paragraph::new(lines).centered(),
        }
    }
}

impl Renderable for Centered {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [horizontal] = Layout::horizontal([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .areas(area);
        let [centered] = Layout::vertical([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .areas(horizontal);
        frame.render_widget(
            self.paragraph
                .clone()
                .block(Block::default().padding(Padding::new(0, 0, area.height / 2, 0)))
                .wrap(Wrap { trim: false }),
            centered,
        );
        Ok(())
    }
}
