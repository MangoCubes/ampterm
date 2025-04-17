use crate::{action::Action, components::Component};
use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

enum CurrentlySelected {
    CurrentlyPlaying,
    Queue,
}

pub struct MainScreen {
    state: CurrentlySelected,
}

impl MainScreen {
    pub fn new() -> Self {
        Self {
            state: CurrentlySelected::CurrentlyPlaying,
        }
    }
}

impl Component for MainScreen {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        Ok(())
    }
}
