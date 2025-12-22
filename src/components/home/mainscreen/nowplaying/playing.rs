mod imagecomp;
mod lyrics;
use std::time::Duration;

use crate::{
    action::action::{Action, QueryAction},
    components::{
        home::mainscreen::nowplaying::playing::{imagecomp::ImageComp, lyrics::Lyrics},
        traits::{
            handlekeyseq::{ComponentKeyHelp, KeySeqResult, PassKeySeq},
            handlequery::HandleQuery,
            renderable::Renderable,
        },
    },
    config::Config,
    helper::strings::trim_long_str,
    osclient::response::getplaylist::Media,
    playerworker::player::{FromPlayerWorker, StateType},
    queryworker::query::ResponseType,
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::Gauge,
    Frame,
};

pub struct Playing {
    pos: Duration,
    playing: bool,
    music: Media,
    lyrics: Option<Lyrics>,
    cover: Option<ImageComp>,
}

impl Playing {
    pub fn new(playing: bool, music: Media, config: Config) -> (Self, Option<Action>) {
        let mut actions = vec![];
        let cover = if config.features.cover_art.enable {
            let (comp, action) = ImageComp::new(music.cover_art.clone());
            if let Some(a) = action {
                actions.push(a);
            };
            Some(comp)
        } else {
            None
        };

        let lyrics = if config.features.lyrics.enable {
            let (comp, action) = Lyrics::new(config, music.clone());
            actions.push(action);
            Some(comp)
        } else {
            None
        };
        (
            Self {
                playing,
                pos: Duration::from_secs(0),
                music: music,
                lyrics,
                cover,
            },
            Some(Action::Multiple(actions)),
        )
    }
}

impl PassKeySeq for Playing {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        if let Some(c) = &mut self.lyrics {
            c.handle_key_seq(keyseq)
        } else {
            None
        }
    }

    fn get_help(&self) -> Vec<ComponentKeyHelp> {
        if let Some(c) = &self.lyrics {
            c.get_help()
        } else {
            vec![]
        }
    }
}

impl HandleQuery for Playing {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
        if let QueryAction::FromQueryWorker(res) = action {
            if let ResponseType::GetCover(d) = res.res {
                if let Some(cover) = &mut self.cover {
                    cover.set_image(res.ticket, d);
                }
            } else if let ResponseType::GetLyrics(lyrics) = res.res {
                if let Some(c) = &mut self.lyrics {
                    c.handle_lyrics(res.ticket, lyrics);
                }
            }
        } else if let QueryAction::FromPlayerWorker(FromPlayerWorker::StateChange(s)) = action {
            match s {
                StateType::Jump(pos) | StateType::Position(pos) => {
                    self.pos = pos;
                    if let Some(l) = &mut self.lyrics {
                        l.set_pos(self.pos);
                    }
                }
                StateType::NowPlaying(Some(media)) => {
                    self.pos = Duration::from_secs(0);
                    self.music = media.clone();
                }
                StateType::Playing(p) => {
                    self.playing = p;
                }
                _ => {}
            };
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
        let area = if let Some(comp) = &mut self.cover {
            let horizontal = Layout::horizontal([Constraint::Length(18), Constraint::Fill(1)]);
            let areas = horizontal.split(area);
            comp.draw(frame, areas[0]);
            areas[1]
        } else {
            area
        };
        let info_area = vertical.split(area);
        let msg = trim_long_str(
            format!(
                "{} - {}",
                match &self.music.artist {
                    Some(v) => v,
                    None => "Unknown",
                },
                self.music.title
            ),
            50,
        );
        frame.render_widget(Line::raw(msg).bold(), info_area[0]);
        frame.render_widget(
            Line::raw(format!(
                "{}",
                self.music.album.clone().unwrap_or("Unknown".to_string())
            )),
            info_area[1],
        );

        if let Some(comp) = &mut self.lyrics {
            comp.draw(frame, info_area[3]);
        }

        let symbol = if self.playing { "▶" } else { "⏸" };
        if let Some(len) = self.music.duration {
            if len == 0 {
                let label = format!(
                    "{:02}:{:02} {} 00:00",
                    self.pos.as_secs() / 60,
                    self.pos.as_secs() % 60,
                    symbol
                );
                frame.render_widget(Line::raw(label), info_area[5]);
            } else {
                let label = format!(
                    "{:02}:{:02} {} {:02}:{:02}",
                    self.pos.as_secs() / 60,
                    self.pos.as_secs() % 60,
                    symbol,
                    len / 60,
                    len % 60,
                );
                let percent = ((self.pos.as_secs() as i32 * 100) / len) as u16;
                let adjusted = if percent > 100 { 100 } else { percent };
                frame.render_widget(
                    Gauge::default()
                        .gauge_style(Color::LightBlue)
                        .percent(adjusted)
                        .label(label),
                    info_area[5],
                );
            }
        } else {
            let label = format!(
                "{:02}:{:02} {} ??:??",
                self.pos.as_secs() / 60,
                self.pos.as_secs() % 60,
                symbol
            );
            frame.render_widget(Line::raw(label), info_area[5]);
        }
    }
}
