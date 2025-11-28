mod synced;
mod unsynced;
use std::time::Duration;

use crate::{
    action::{Action, FromPlayerWorker, StateType},
    components::{
        home::mainscreen::nowplaying::playing::{synced::Synced, unsynced::Unsynced},
        lib::centered::Centered,
        traits::{handleaction::HandleActionSimple, renderable::Renderable},
    },
    helper::strings::trim_long_str,
    lyricsclient::getlyrics::GetLyricsParams,
    osclient::response::getplaylist::Media,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{ResponseType, ToQueryWorker},
    },
};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::Gauge,
    Frame,
};

enum LyricsSpace {
    Found(Synced),
    Fetching(Centered),
    NotFound(Centered),
    Error(Centered),
    Plain(Unsynced),
    Disabled,
}

pub struct Playing {
    vol: f32,
    speed: f32,
    pos: Duration,
    music: Media,
    lyrics: LyricsSpace,
}

impl Playing {
    pub fn new(
        music: Media,
        vol: f32,
        speed: f32,
        pos: Duration,
        enable_lyrics: bool,
    ) -> (Self, Option<Action>) {
        if enable_lyrics {
            (
                Self {
                    vol,
                    speed,
                    pos,
                    music: music.clone(),
                    lyrics: LyricsSpace::Fetching(Centered::new(vec![format!(
                        "Searching for lyrics for {}...",
                        music.title
                    )])),
                },
                Some(Action::ToQueryWorker(ToQueryWorker::new(
                    HighLevelQuery::GetLyrics(GetLyricsParams {
                        track_name: music.title,
                        artist_name: music.artist,
                        album_name: music.album,
                        length: music.duration,
                    }),
                ))),
            )
        } else {
            (
                Self {
                    vol,
                    speed,
                    pos,
                    music: music.clone(),
                    lyrics: LyricsSpace::Disabled,
                },
                None,
            )
        }
    }
}

impl HandleActionSimple for Playing {
    fn handle_action_simple(&mut self, action: Action) {
        if let Action::FromQueryWorker(res) = action {
            if let ResponseType::GetLyrics(lyrics) = res.res {
                self.lyrics = match lyrics {
                    Ok(content) => match content {
                        Some(found) => {
                            if let Some(synced) = found.synced_lyrics {
                                LyricsSpace::Found(Synced::new(synced))
                            } else if let Some(plain) = found.plain_lyrics {
                                LyricsSpace::Plain(Unsynced::new(plain))
                            } else {
                                LyricsSpace::NotFound(Centered::new(vec![
                                    "Lyrics not found!".to_string()
                                ]))
                            }
                        }
                        None => LyricsSpace::NotFound(Centered::new(vec![
                            "Lyrics not found!".to_string()
                        ])),
                    },
                    Err(e) => LyricsSpace::Error(Centered::new(vec![format!(
                        "Failed to find lyrics! Reason: {}",
                        e
                    )])),
                }
            }
        } else if let Action::FromPlayerWorker(FromPlayerWorker::StateChange(s)) = &action {
            match s {
                StateType::Position(pos) => self.pos = *pos,
                StateType::Volume(v) => self.vol = *v,
                StateType::Speed(s) => self.speed = *s,
                StateType::NowPlaying(Some(media)) => {
                    self.music = media.clone();
                }
                _ => {}
            };
            if let LyricsSpace::Found(l) = &mut self.lyrics {
                l.handle_action_simple(action.clone());
            }
        } else if matches!(action, Action::User(_)) {
            match &mut self.lyrics {
                LyricsSpace::Plain(l) => {
                    l.handle_action_simple(action);
                }
                _ => {}
            }
        };
    }
}

impl Renderable for Playing {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Max(1),
            Constraint::Length(1), // Padding
            Constraint::Length(3),
            Constraint::Length(1), // Padding
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
        match &mut self.lyrics {
            LyricsSpace::Found(lyrics) => lyrics.draw(frame, areas[3]),
            LyricsSpace::Plain(lyrics) => lyrics.draw(frame, areas[3]),
            LyricsSpace::Fetching(centered)
            | LyricsSpace::NotFound(centered)
            | LyricsSpace::Error(centered) => centered.draw(frame, areas[3]),
            LyricsSpace::Disabled => (),
        };
        if let Some(len) = self.music.duration {
            if len == 0 {
                let label = format!(
                    "{:02}:{:02} / 00:00",
                    self.pos.as_secs() / 60,
                    self.pos.as_secs() % 60,
                );
                frame.render_widget(Line::raw(label), areas[5]);
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
                frame.render_widget(Gauge::default().label(label).percent(adjusted), areas[5]);
            }
        } else {
            let label = format!(
                "{:02}:{:02} / ??:??",
                self.pos.as_secs() / 60,
                self.pos.as_secs() % 60,
            );
            frame.render_widget(Line::raw(label), areas[5]);
        }
    }
}
