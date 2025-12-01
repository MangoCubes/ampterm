mod synced;
mod unsynced;
use std::time::Duration;

use crate::{
    action::action::{Action, QueryAction},
    components::{
        home::mainscreen::nowplaying::playing::{synced::Synced, unsynced::Unsynced},
        lib::centered::Centered,
        traits::{
            handlekeyseq::{HandleKeySeq, KeySeqResult, PassKeySeq},
            handlequery::HandleQuery,
            renderable::Renderable,
        },
    },
    config::Config,
    helper::strings::trim_long_str,
    lyricsclient::getlyrics::GetLyricsParams,
    osclient::response::getplaylist::Media,
    playerworker::player::{FromPlayerWorker, StateType},
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{ResponseType, ToQueryWorker},
    },
};
use crossterm::event::KeyEvent;
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
    config: Config,
}

impl Playing {
    pub fn new(
        music: Media,
        vol: f32,
        speed: f32,
        pos: Duration,
        enable_lyrics: bool,
        config: Config,
    ) -> (Self, Option<Action>) {
        if enable_lyrics {
            (
                Self {
                    config,
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
                    config,
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

impl PassKeySeq for Playing {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        match &mut self.lyrics {
            LyricsSpace::Plain(unsynced) => unsynced.handle_key_seq(keyseq),
            _ => None,
        }
    }
}

impl HandleQuery for Playing {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
        if let QueryAction::FromQueryWorker(res) = action {
            if let ResponseType::GetLyrics(lyrics) = res.res {
                self.lyrics = match lyrics {
                    Ok(content) => match content {
                        Some(found) => {
                            if let Some(synced) = found.synced_lyrics {
                                LyricsSpace::Found(Synced::new(synced))
                            } else if let Some(plain) = found.plain_lyrics {
                                LyricsSpace::Plain(Unsynced::new(self.config.clone(), plain))
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
        } else if let QueryAction::FromPlayerWorker(FromPlayerWorker::StateChange(s)) = &action {
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
                l.handle_query(action.clone());
            }
        };
        None
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
