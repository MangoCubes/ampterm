mod playing;
mod stopped;

use std::time::Duration;

use color_eyre::Result;
use playing::Playing;
use ratatui::{layout::Rect, Frame};
use stopped::Stopped;

use crate::{
    action::{Action, FromPlayerWorker, StateType},
    components::traits::component::Component,
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
}

impl Component for NowPlaying {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.comp {
            Comp::Playing(playing) => playing.draw(frame, area),
            Comp::Stopped(stopped) => stopped.draw(frame, area),
        }
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::FromPlayerWorker(FromPlayerWorker::StateChange(StateType::NowPlaying {
            music,
            index: _,
        })) = action
        {
            self.comp = Comp::Playing(Playing::new(music, 0.0, 0.0, Duration::from_secs(0)));
            Ok(None)
        } else {
            if let Comp::Playing(comp) = &mut self.comp {
                comp.update(action)
            } else {
                Ok(None)
            }
        }
    }
}
