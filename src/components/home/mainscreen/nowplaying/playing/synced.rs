use std::time::Duration;

use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
    style::Stylize,
    text::Line,
    Frame,
};

use crate::{
    action::action::{Action, QueryAction},
    components::traits::{handlequery::HandleQuery, renderable::Renderable},
    lyricsclient::getlyrics::ParsedLyrics,
    playerworker::player::{FromPlayerWorker, StateType},
};

pub struct Synced {
    lyrics: ParsedLyrics,
    current_time: Duration,
}

impl Synced {
    pub fn new(found: String) -> Self {
        Self {
            lyrics: ParsedLyrics::from(found),
            current_time: Duration::default(),
        }
    }
}

impl Renderable for Synced {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let vertical =
            Layout::vertical([Constraint::Max(1), Constraint::Max(1), Constraint::Max(1)]);
        let areas = vertical.split(area);
        let (prev, current, next) = self.lyrics.get_lyrics(self.current_time);
        frame.render_widget(
            Line::raw(if let Some(l) = prev {
                format!("  {}", l.lyric)
            } else {
                "  𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮".to_string()
            }),
            areas[0],
        );
        frame.render_widget(
            Line::raw(if let Some(l) = current {
                format!("> {}", l.lyric)
            } else {
                "> 𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮".to_string()
            })
            .bold(),
            areas[1],
        );
        frame.render_widget(
            Line::raw(if let Some(l) = next {
                format!("  {}", l.lyric)
            } else {
                "  𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮".to_string()
            }),
            areas[2],
        );
    }
}

impl HandleQuery for Synced {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
        if let QueryAction::FromPlayerWorker(FromPlayerWorker::StateChange(StateType::Position(
            d,
        ))) = action
        {
            self.current_time = d;
        }
        None
    }
}
