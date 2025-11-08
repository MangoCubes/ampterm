use crate::components::traits::simplecomponent::SimpleComponent;
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    text::Line,
    widgets::{Block, Padding, Paragraph},
    Frame,
};

pub struct Error {
    error: String,
}

impl Error {
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

impl SimpleComponent for Error {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw("Error!"),
                Line::raw(format!("{}", self.error)),
                Line::raw(format!("Reload with 'R'")),
            ])
            .block(Block::default().padding(Padding::new(0, 0, (area.height / 2) - 1, 0)))
            .alignment(Alignment::Center),
            area,
        );
        Ok(())
    }
}
