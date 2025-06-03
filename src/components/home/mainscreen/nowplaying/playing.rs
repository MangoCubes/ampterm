use std::time::Duration;

use crate::{
    action::{Action, StateType},
    components::Component,
};
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
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
    length: i32,
}

impl Playing {
    pub fn new(
        title: String,
        artist: String,
        album: String,
        length: i32,
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
                length,
            },
        }
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
        } else if let Action::InQueue {
            current,
            next,
            vol,
            speed,
            pos,
        } = action
        {
            self.state.pos = Duration::default();
        }
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let areas = vertical.split(area);
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw(format!("{} - {}", self.state.artist, self.state.title)).bold(),
                Line::raw(format!("{}", self.state.album)),
            ])
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .border_style(Style::new().white()),
            )
            .wrap(Wrap { trim: false }),
            areas[0],
        );
        let label = format!(
            "{:02}:{:02} / {:02}:{:02}",
            self.state.pos.as_secs() / 60,
            self.state.pos.as_secs() % 60,
            self.state.length / 60,
            self.state.length % 60,
        );
        frame.render_widget(
            Gauge::default()
                .label(label)
                .percent(((self.state.pos.as_secs() as i32 * 100) / self.state.length) as u16)
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                        .border_style(Style::new().white()),
                ),
            areas[1],
        );
        Ok(())
    }
}
