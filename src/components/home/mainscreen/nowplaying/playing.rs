use crate::{action::Action, components::Component, noparams::NoParams};
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

pub struct Playing {
    title: String,
    artist: String,
    album: String,
}

impl Playing {
    pub fn new(title: String, artist: String, album: String) -> Self {
        Self {
            title,
            artist,
            album,
        }
    }
    fn gen_block() -> Block<'static> {
        Block::bordered().border_style(Style::new().white())
    }
}

impl Component for Playing {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action;
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        let _ = key;
        Ok(None)
    }
}

impl NoParams for Playing {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw(format!("{}", self.title)),
                Line::raw(format!("{}", self.artist)),
                Line::raw(format!("{}", self.album)),
            ])
            .block(Self::gen_block())
            .wrap(Wrap { trim: false }),
            area,
        );
        Ok(())
    }
}
