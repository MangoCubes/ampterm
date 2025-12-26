use std::time::Duration;

use crossterm::event::KeyEvent;
use ratatui::{prelude::Rect, Frame};

use crate::{
    action::action::Action,
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
    Fetching(usize, Centered),
    NotFound(Centered),
    Error(Centered),
    Plain(Unsynced),
}

pub struct Lyrics {
    state: State,
    config: Config,
}

impl Lyrics {
    pub fn make_query(music: Media) -> ToQueryWorker {
        ToQueryWorker::new(HighLevelQuery::GetLyrics(GetLyricsParams {
            track_name: music.title,
            artist_name: music.artist,
            album_name: music.album,
            length: music.duration,
        }))
    }
    pub fn new(config: Config, music: Media) -> (Self, Action) {
        let title = music.title.clone();
        let query = Self::make_query(music);
        (
            Self {
                state: State::Fetching(
                    query.ticket,
                    Centered::new(vec![format!("Searching for lyrics for {}...", title)]),
                ),
                config,
            },
            Action::ToQuery(query),
        )
    }

    pub fn handle_lyrics(
        &mut self,
        ticket: usize,
        lyrics: Result<Option<GetLyricsResponse>, String>,
    ) {
        if let State::Fetching(t, _) = self.state {
            if ticket == t {
                self.state = match lyrics {
                    Ok(content) => match content {
                        Some(found) => {
                            if let Some(synced) = found.synced_lyrics {
                                State::Found(Synced::new(synced))
                            } else if let Some(plain) = found.plain_lyrics {
                                State::Plain(Unsynced::new(self.config.clone(), plain))
                            } else {
                                State::NotFound(Centered::new(
                                    vec!["Lyrics not found!".to_string()],
                                ))
                            }
                        }
                        None => {
                            State::NotFound(Centered::new(vec!["Lyrics not found!".to_string()]))
                        }
                    },
                    Err(e) => State::Error(Centered::new(vec![format!(
                        "Failed to find lyrics! Reason: {}",
                        e
                    )])),
                }
            }
        }
    }

    pub fn set_pos(&mut self, pos: Duration) {
        if let State::Found(s) = &mut self.state {
            s.set_pos(pos);
        }
    }

    pub fn wait_for(&mut self, ticket: usize, title: String) {
        self.state = State::Fetching(
            ticket,
            Centered::new(vec![format!("Searching for lyrics for {}...", title)]),
        );
    }
}

impl Renderable for Lyrics {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match &mut self.state {
            State::Found(lyrics) => lyrics.draw(frame, area),
            State::Plain(lyrics) => lyrics.draw(frame, area),
            State::Fetching(_, centered) | State::NotFound(centered) | State::Error(centered) => {
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
