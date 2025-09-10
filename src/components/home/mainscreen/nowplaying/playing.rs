use std::time::Duration;

use crate::{
    action::{Action, FromPlayerWorker, StateType},
    components::traits::component::Component,
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
            vol,
            speed,
            pos,
            title,
            artist,
            album,
            length,
        }
    }
}

impl Component for Playing {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::FromPlayerWorker(FromPlayerWorker::PlayerState(s)) = action {
            match s {
                StateType::Position(pos) => self.pos = pos,
                StateType::Volume(v) => self.vol = v,
                StateType::Speed(s) => self.speed = s,
            };
        } else if let Action::FromPlayerWorker(FromPlayerWorker::InQueue {
            play: _,
            vol: _,
            speed: _,
            pos: _,
        }) = action
        {
            self.pos = Duration::default();
        }
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let areas = vertical.split(area);
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw(format!("{} - {}", self.artist, self.title)).bold(),
                Line::raw(format!("{}", self.album)),
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
            self.pos.as_secs() / 60,
            self.pos.as_secs() % 60,
            self.length / 60,
            self.length % 60,
        );
        let percent = ((self.pos.as_secs() as i32 * 100) / self.length) as u16;
        let adjusted = if percent > 100 { 100 } else { percent };
        frame.render_widget(
            Gauge::default().label(label).percent(adjusted).block(
                Block::default()
                    .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                    .border_style(Style::new().white()),
            ),
            areas[1],
        );
        Ok(())
    }
}
