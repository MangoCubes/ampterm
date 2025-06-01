use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

use crate::{components::Component, noparams::NoParams};

pub struct Loading {
    url: String,
    username: String,
}

impl Loading {
    pub fn new(url: String, username: String) -> Self {
        Self { url, username }
    }
}
impl Component for Loading {}

impl NoParams for Loading {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [horizontal] = Layout::horizontal([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .areas(area);
        let [centered] = Layout::vertical([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .areas(horizontal);
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw("Logging in with the credentials in the configuration..."),
                Line::raw(format!("URL: {}", self.url)),
                Line::raw(format!("Username: {}", self.username)),
            ])
            .centered()
            .block(Block::bordered().padding(Padding::new(0, 0, (area.height / 2) - 1, 0)))
            .wrap(Wrap { trim: false }),
            centered,
        );
        Ok(())
    }
}
