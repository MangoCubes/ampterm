mod playing;
mod speed;
mod stopped;
mod volume;

use std::time::Duration;

use crossterm::event::KeyEvent;
use playing::Playing;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};
use stopped::Stopped;

use crate::{
    action::action::{Action, QueryAction},
    components::{
        home::mainscreen::nowplaying::{speed::Speed, volume::Volume},
        traits::{
            focusable::Focusable,
            handlekeyseq::{ComponentKeyHelp, KeySeqResult, PassKeySeq},
            handlequery::HandleQuery,
            renderable::Renderable,
        },
    },
    config::Config,
    playerworker::player::{FromPlayerWorker, StateType},
};

enum Comp {
    Playing(Playing),
    Stopped(Stopped),
}

pub struct NowPlaying {
    comp: Comp,
    enabled: bool,
    config: Config,
    speed: Speed,
    volume: Volume,
}

impl PassKeySeq for NowPlaying {
    fn get_help(&self) -> Vec<ComponentKeyHelp> {
        match &self.comp {
            Comp::Playing(playing) => playing.get_help(),
            Comp::Stopped(_) => vec![],
        }
    }
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        match &mut self.comp {
            Comp::Playing(playing) => playing.handle_key_seq(keyseq),
            Comp::Stopped(_) => None,
        }
    }
}

impl NowPlaying {
    pub fn new(enabled: bool, config: Config) -> Self {
        Self {
            volume: Volume::new(config.init_state.volume),
            config,
            enabled,
            comp: Comp::Stopped(Stopped::new()),
            speed: Speed::new(1.0),
        }
    }
    fn gen_block(&self) -> Block<'static> {
        let style = if self.enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };

        Block::bordered().border_style(style)
    }
}

impl HandleQuery for NowPlaying {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
        match action {
            QueryAction::FromPlayerWorker(FromPlayerWorker::StateChange(
                StateType::NowPlaying(media),
            )) => match media {
                Some(n) => {
                    let (comp, action) = Playing::new(
                        n,
                        self.speed.get_speed(),
                        self.config.features.lyrics.enable,
                        self.config.clone(),
                    );
                    self.comp = Comp::Playing(comp);
                    action
                }
                None => {
                    self.comp = Comp::Stopped(Stopped::new());
                    None
                }
            },
            QueryAction::FromPlayerWorker(FromPlayerWorker::StateChange(StateType::Volume(v))) => {
                self.volume.set_volume(v);
                None
            }
            QueryAction::FromPlayerWorker(FromPlayerWorker::StateChange(StateType::Speed(s))) => {
                self.speed.set_speed(s);
                if let Comp::Playing(playing) = &mut self.comp {
                    playing.handle_query(action)
                } else {
                    None
                }
            }
            _ => match &mut self.comp {
                Comp::Playing(playing) => playing.handle_query(action),
                Comp::Stopped(_) => None,
            },
        }
    }
}

impl Renderable for NowPlaying {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let div = Layout::horizontal([Constraint::Min(1), Constraint::Max(5), Constraint::Max(5)]);
        let block = self.gen_block();
        let areas = div.split(area);
        let inner = block.inner(areas[0]);
        frame.render_widget(block, areas[0]);
        self.speed.draw(frame, areas[1]);
        self.volume.draw(frame, areas[2]);
        match &mut self.comp {
            Comp::Playing(playing) => playing.draw(frame, inner),
            Comp::Stopped(stopped) => stopped.draw(frame, inner),
        }
    }
}

impl Focusable for NowPlaying {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
        };
    }
}
