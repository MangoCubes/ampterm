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
    widgets::{Block, Gauge, Paragraph, Wrap},
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
    cover: Option<String>,
}

impl Playing {
    pub fn new(
        title: String,
        artist: String,
        album: String,
        cover: Option<String>,
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
            cover,
        }
    }
}

impl Component for Playing {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::FromPlayerWorker(FromPlayerWorker::State(s)) = action {
            match s {
                StateType::Position(pos) => self.pos = pos,
                StateType::Volume(v) => self.vol = v,
                StateType::Speed(s) => self.speed = s,
            };
        } else if let Action::FromPlayerWorker(FromPlayerWorker::InQueue {
            items: _,
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
        let border = Block::bordered().border_style(Style::new().white());
        let inner = border.inner(area);
        let vertical = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]);
        let areas = vertical.split(inner);
        frame.render_widget(border, area);
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw(format!("{} - {}", self.artist, self.title)).bold(),
                Line::raw(format!("{}", self.album)),
            ])
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
        frame.render_widget(Gauge::default().label(label).percent(adjusted), areas[1]);

        Ok(())
    }
}
