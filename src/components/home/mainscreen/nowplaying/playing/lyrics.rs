use std::time::Duration;

use crossterm::event::KeyEvent;
use ratatui::{prelude::Rect, Frame};

use crate::{
    action::action::{Action, QueryAction},
    components::{
        home::mainscreen::nowplaying::playing::lyrics::{synced::Synced, unsynced::Unsynced},
        lib::centered::Centered,
        traits::{
            handlekeyseq::{ComponentKeyHelp, HandleKeySeq, KeySeqResult, PassKeySeq},
            renderable::Renderable,
        },
    },
    config::Config,
    lyricsclient::getlyrics::{GetLyricsParams, GetLyricsResponse},
    osclient::response::getplaylist::Media,
    queryworker::{highlevelquery::HighLevelQuery, query::ToQueryWorker},
};

mod synced;
mod unsynced;

enum State {
    Found(Synced),
    Fetching(Centered),
    NotFound(Centered),
    Error(Centered),
    Plain(Unsynced),
}

pub struct Lyrics {
    state: State,
    config: Config,
}

impl Lyrics {
    pub fn new(config: Config, music: Media) -> (Self, Action) {
        (
            Self {
                state: State::Fetching(Centered::new(vec![format!(
                    "Searching for lyrics for {}...",
                    music.title
                )])),
                config,
            },
            Action::Query(QueryAction::ToQueryWorker(ToQueryWorker::new(
                HighLevelQuery::GetLyrics(GetLyricsParams {
                    track_name: music.title,
                    artist_name: music.artist,
                    album_name: music.album,
                    length: music.duration,
                }),
            ))),
        )
    }

    pub fn handle_lyrics(&mut self, lyrics: Result<Option<GetLyricsResponse>, String>) {
        self.state = match lyrics {
            Ok(content) => match content {
                Some(found) => {
                    if let Some(synced) = found.synced_lyrics {
                        State::Found(Synced::new(synced))
                    } else if let Some(plain) = found.plain_lyrics {
                        State::Plain(Unsynced::new(self.config.clone(), plain))
                    } else {
                        State::NotFound(Centered::new(vec!["Lyrics not found!".to_string()]))
                    }
                }
                None => State::NotFound(Centered::new(vec!["Lyrics not found!".to_string()])),
            },
            Err(e) => State::Error(Centered::new(vec![format!(
                "Failed to find lyrics! Reason: {}",
                e
            )])),
        }
    }

    pub fn set_pos(&mut self, pos: Duration) {
        if let State::Found(s) = &mut self.state {
            s.set_pos(pos);
        }
    }
}

impl Renderable for Lyrics {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match &mut self.state {
            State::Found(lyrics) => lyrics.draw(frame, area),
            State::Plain(lyrics) => lyrics.draw(frame, area),
            State::Fetching(centered) | State::NotFound(centered) | State::Error(centered) => {
                centered.draw(frame, area)
            }
        };
    }
}

impl PassKeySeq for Lyrics {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        match &mut self.state {
            State::Plain(unsynced) => unsynced.handle_key_seq(keyseq),
            _ => None,
        }
    }

    fn get_help(&self) -> Vec<ComponentKeyHelp> {
        match &self.state {
            State::Plain(unsynced) => unsynced.get_help(),
            _ => vec![],
        }
    }
}
