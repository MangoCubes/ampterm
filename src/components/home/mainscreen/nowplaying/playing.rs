use std::time::Duration;

use crate::{
    action::{Action, StateType},
    components::Component,
    stateful::Stateful,
};
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

pub struct Playing {
    state: CompState,
}

struct CompState {
    vol: f32,
    speed: f32,
    pos: Duration,
    title: String,
    artist: String,
    album: String,
}

impl Playing {
    pub fn new(
        title: String,
        artist: String,
        album: String,
        vol: f32,
        speed: f32,
        pos: Duration,
    ) -> Self {
        Self {
            state: CompState {
                vol,
                speed,
                pos,
                title,
                artist,
                album,
            },
        }
    }
    fn gen_block() -> Block<'static> {
        Block::bordered().border_style(Style::new().white())
    }
}

impl Component for Playing {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::PlayerState(s) = action {
            match s {
                StateType::Position(pos) => self.state.pos = pos,
                StateType::Volume(v) => self.state.vol = v,
                StateType::Speed(s) => self.state.speed = s,
            };
        };
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw(format!("{} - {}", self.state.artist, self.state.title)).bold(),
                Line::raw(format!("{}", self.state.album)),
                Line::raw(format!("{}", self.state.pos.as_secs())),
            ])
            .block(Self::gen_block())
            .wrap(Wrap { trim: false }),
            area,
        );
        Ok(())
    }
}
