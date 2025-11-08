use crate::components::traits::renderable::Renderable;
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

pub struct Loading {}

impl Loading {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderable for Loading {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new("Loading...")
                .block(Block::default().padding(Padding::new(0, 0, area.height / 2, 0)))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            area,
        );
        Ok(())
    }
}
