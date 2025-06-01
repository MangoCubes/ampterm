use std::time::Duration;

use crate::{action::Action, components::Component, hasparams::HasParams, noparams::NoParams};
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

pub struct PlayingState {
    pub vol: f32,
    pub speed: f32,
    pub pos: Duration,
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

impl Component for Playing {}

impl HasParams<&PlayingState> for Playing {
    fn draw_params(&mut self, frame: &mut Frame, area: Rect, state: &PlayingState) -> Result<()> {
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw(format!("{} - {}", self.artist, self.title)).bold(),
                Line::raw(format!("{}", self.album)),
                Line::raw(format!("{}", state.pos.as_secs())),
            ])
            .block(Self::gen_block())
            .wrap(Wrap { trim: false }),
            area,
        );
        Ok(())
    }
}
