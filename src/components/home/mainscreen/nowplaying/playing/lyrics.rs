use std::time::Duration;

use color_eyre::eyre::Result;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
    style::Stylize,
    text::Line,
    Frame,
};

use crate::{
    action::{Action, FromPlayerWorker, StateType},
    components::traits::{renderable::Renderable, simplecomp::SimpleComp},
    lyricsclient::getlyrics::ParsedLyrics,
};

pub struct Lyrics {
    lyrics: ParsedLyrics,
    current_time: Duration,
}

impl Lyrics {
    pub fn new(found: String) -> Self {
        Self {
            lyrics: ParsedLyrics::from(found),
            current_time: Duration::default(),
        }
    }
}

impl Renderable for Lyrics {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical =
            Layout::vertical([Constraint::Max(1), Constraint::Max(1), Constraint::Max(1)]);
        let areas = vertical.split(area);
        if let Some(l) = self.lyrics.get_lyrics(self.current_time) {
            frame.render_widget(Line::raw(l.lyric).bold(), areas[0]);
        } else {
            frame.render_widget(Line::raw("𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮").bold(), areas[0]);
        }
        Ok(())
    }
}

impl SimpleComp for Lyrics {
    fn update(&mut self, action: Action) {
        if let Action::FromPlayerWorker(FromPlayerWorker::StateChange(StateType::Position(d))) =
            action
        {
            self.current_time = d;
        }
    }
}
