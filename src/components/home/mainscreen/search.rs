use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};
use tui_textarea::TextArea;

use crate::{
    action::action::{Action, TargetedAction},
    components::traits::{handleraw::HandleRaw, renderable::Renderable},
};

pub struct Search {
    input: TextArea<'static>,
    original: String,
}

impl Search {
    pub fn new(original: String) -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("Search"),
        );
        Self { input, original }
    }
}

impl Renderable for Search {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.input, area);
    }
}

impl HandleRaw for Search {
    fn handle_raw(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc => Some(Action::Targeted(TargetedAction::ApplySearch(
                self.original.clone(),
            ))),
            KeyCode::Enter => Some(Action::Targeted(TargetedAction::ApplySearch(
                self.input.lines()[0].clone(),
            ))),
            KeyCode::Backspace => {
                let search = &self.input.lines()[0];
                if search.len() == 0 {
                    Some(Action::Targeted(TargetedAction::ApplySearch(
                        self.original.clone(),
                    )))
                } else {
                    self.input.input(key);
                    Some(Action::Targeted(TargetedAction::ApplySearch(
                        self.input.lines()[0].clone(),
                    )))
                }
            }
            _ => {
                self.input.input(key);
                Some(Action::Targeted(TargetedAction::ApplySearch(
                    self.input.lines()[0].clone(),
                )))
            }
        }
    }
}
