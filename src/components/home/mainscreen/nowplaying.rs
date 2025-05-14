mod stopped;

use crate::{action::Action, components::Component, stateless::Stateless};
use color_eyre::Result;
use ratatui::{layout::Rect, Frame};
use stopped::Stopped;
use tokio::sync::mpsc::UnboundedSender;

enum CompState {
    Stopped { comp: Stopped },
}

pub struct NowPlaying {
    state: CompState,
}

impl NowPlaying {
    pub fn new() -> Self {
        Self {
            state: CompState::Stopped {
                comp: Stopped::new(),
            },
        }
    }
}

impl Component for NowPlaying {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
}

impl Stateless for NowPlaying {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.state {
            CompState::Stopped { comp } => {
                let _ = comp.draw(frame, area);
            }
        };
        Ok(())
    }
}
