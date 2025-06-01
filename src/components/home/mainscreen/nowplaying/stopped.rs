use crate::{action::Action, components::Component};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

pub struct Stopped {}

impl Stopped {
    pub fn new() -> Self {
        Self {}
    }
    fn gen_block() -> Block<'static> {
        Block::bordered().border_style(Style::new().white())
    }
}

impl Component for Stopped {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action;
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        let _ = key;
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new("Select a music!")
                .block(Self::gen_block().padding(Padding::new(0, 0, area.height / 2, 0)))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            area,
        );
        Ok(())
    }
}
