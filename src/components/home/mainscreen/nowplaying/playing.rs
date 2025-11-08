use std::time::Duration;

use crate::{
    action::{Action, FromPlayerWorker, StateType},
    components::traits::{renderable::Renderable, simplecomp::SimpleComp},
    helper::strings::trim_long_str,
    osclient::response::getplaylist::Media,
};
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Gauge, Paragraph, Wrap},
    Frame,
};

pub struct Playing {
    vol: f32,
    speed: f32,
    pos: Duration,
    music: Media,
}

impl Playing {
    pub fn new(music: Media, vol: f32, speed: f32, pos: Duration) -> Self {
        Self {
            vol,
            speed,
            pos,
            music,
        }
    }
}

impl SimpleComp for Playing {
    fn update(&mut self, action: Action) {
        if let Action::FromPlayerWorker(FromPlayerWorker::StateChange(s)) = action {
            match s {
                StateType::Position(pos) => self.pos = pos,
                StateType::Volume(v) => self.vol = v,
                StateType::Speed(s) => self.speed = s,
                StateType::Queue(_queue_change) => {}
                StateType::NowPlaying(Some(now_playing)) => {
                    self.music = now_playing.music;
                }
                _ => {}
            };
        }
    }
}

impl Renderable for Playing {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical = Layout::vertical([
            Constraint::Max(1),
            Constraint::Max(1),
            Constraint::Length(1),
        ]);
        let areas = vertical.split(area);
        frame.render_widget(
            Line::raw(format!(
                "{} - {}",
                trim_long_str(
                    self.music.artist.clone().unwrap_or("Unknown".to_string()),
                    // Need to be a multiple of 30 to ensure the character bound thing is satisfied
                    30
                ),
                self.music.title
            ))
            .bold(),
            areas[0],
        );
        frame.render_widget(
            Line::raw(format!(
                "{}",
                self.music.album.clone().unwrap_or("Unknown".to_string())
            )),
            areas[1],
        );
        if let Some(len) = self.music.duration {
            if len == 0 {
                let label = format!(
                    "{:02}:{:02} / 00:00",
                    self.pos.as_secs() / 60,
                    self.pos.as_secs() % 60,
                );
                frame.render_widget(Line::raw(label), areas[2]);
            } else {
                let label = format!(
                    "{:02}:{:02} / {:02}:{:02}",
                    self.pos.as_secs() / 60,
                    self.pos.as_secs() % 60,
                    len / 60,
                    len % 60,
                );
                let percent = ((self.pos.as_secs() as i32 * 100) / len) as u16;
                let adjusted = if percent > 100 { 100 } else { percent };
                frame.render_widget(Gauge::default().label(label).percent(adjusted), areas[2]);
            }
        } else {
            let label = format!(
                "{:02}:{:02} / ??:??",
                self.pos.as_secs() / 60,
                self.pos.as_secs() % 60,
            );
            frame.render_widget(Line::raw(label), areas[1]);
        }

        Ok(())
    }
}
