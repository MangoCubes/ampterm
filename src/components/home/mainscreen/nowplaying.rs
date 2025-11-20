mod playing;
mod stopped;

use std::time::Duration;

use color_eyre::Result;
use playing::Playing;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};
use stopped::Stopped;

use crate::{
    action::{Action, FromPlayerWorker, StateType},
    components::traits::{
        focusable::Focusable, fullcomp::FullComp, renderable::Renderable, simplecomp::SimpleComp,
    },
    config::Config,
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

impl FullComp for NowPlaying {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FromPlayerWorker(FromPlayerWorker::StateChange(StateType::NowPlaying(
                now_playing,
            ))) => match now_playing {
                Some(n) => {
                    let (comp, action) = Playing::new(
                        n,
                        0.0,
                        0.0,
                        Duration::from_secs(0),
                        self.config.lyrics.enable,
                    );
                    self.comp = Comp::Playing(comp);
                    Ok(action)
                }
                None => {
                    self.comp = Comp::Stopped(Stopped::new());
                    Ok(None)
                }
            },
            _ => {
                if let Comp::Playing(comp) = &mut self.comp {
                    comp.update(action);
                }
                Ok(None)
            }
        }
    }
}

impl Renderable for NowPlaying {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
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
