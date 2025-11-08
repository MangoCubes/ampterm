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
    components::traits::{renderable::Renderable, simplecomp::SimpleComp},
};

enum Comp {
    Playing(Playing),
    Stopped(Stopped),
}

pub struct NowPlaying {
    comp: Comp,
}

impl NowPlaying {
    pub fn new() -> Self {
        Self {
            comp: Comp::Stopped(Stopped::new()),
        }
    }
    fn gen_block(&self) -> Block<'static> {
        Block::bordered().border_style(Style::new().white())
    }
}

impl SimpleComp for NowPlaying {
    fn update(&mut self, action: Action) {
        if let Action::FromPlayerWorker(FromPlayerWorker::StateChange(StateType::NowPlaying(
            now_playing,
        ))) = action
        {
            self.comp = match now_playing {
                Some(n) => Comp::Playing(Playing::new(n.music, 0.0, 0.0, Duration::from_secs(0))),
                None => Comp::Stopped(Stopped::new()),
            };
        } else {
            if let Comp::Playing(comp) = &mut self.comp {
                comp.update(action);
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
