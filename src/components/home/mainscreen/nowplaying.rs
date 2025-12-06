mod playing;
mod stopped;

use std::time::Duration;

use crossterm::event::KeyEvent;
use playing::Playing;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};
use stopped::Stopped;

use crate::{
    action::action::{Action, QueryAction},
    components::traits::{
        focusable::Focusable,
        handlekeyseq::{ComponentKeyHelp, KeySeqResult, PassKeySeq},
        handlequery::HandleQuery,
        renderable::Renderable,
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
            config,
            enabled,
            comp: Comp::Stopped(Stopped::new()),
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
                StateType::NowPlaying(now_playing),
            )) => match now_playing {
                Some(n) => {
                    let (comp, action) = Playing::new(
                        n,
                        0.0,
                        0.0,
                        Duration::from_secs(0),
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
            _ => match &mut self.comp {
                Comp::Playing(playing) => playing.handle_query(action),
                Comp::Stopped(_) => None,
            },
        }
    }
}

impl Renderable for NowPlaying {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let block = self.gen_block();
        let inner = block.inner(area);
        frame.render_widget(block, area);
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
