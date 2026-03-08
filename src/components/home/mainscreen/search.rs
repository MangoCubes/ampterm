use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear},
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
    pub fn new(original: Option<String>) -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("Search"),
        );
        Self {
            input,
            original: original.unwrap_or("".to_string()),
        }
    }
}

impl Renderable for Search {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(60)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        frame.render_widget(&self.input, area);
    }
}

impl HandleRaw for Search {
    fn handle_raw(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc => Some(Action::Targeted(TargetedAction::ApplySearch(
                self.original.clone(),
                true,
            ))),
            KeyCode::Enter => Some(Action::Targeted(TargetedAction::ApplySearch(
                self.input.lines()[0].clone(),
                true,
            ))),
            KeyCode::Backspace => {
                let search = &self.input.lines()[0];
                if search.len() == 0 {
                    Some(Action::Targeted(TargetedAction::ApplySearch(
                        self.original.clone(),
                        true,
                    )))
                } else {
                    self.input.input(key);
                    Some(Action::Targeted(TargetedAction::ApplySearch(
                        self.input.lines()[0].clone(),
                        false,
                    )))
                }
            }
            _ => {
                self.input.input(key);
                Some(Action::Targeted(TargetedAction::ApplySearch(
                    self.input.lines()[0].clone(),
                    false,
                )))
            }
        }
    }
}
