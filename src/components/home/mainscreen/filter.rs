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

pub struct Filter {
    input: TextArea<'static>,
}

impl Filter {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("Filter"),
        );
        Self { input }
    }
}

impl Renderable for Filter {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(60)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        frame.render_widget(&self.input, area);
    }
}
impl HandleRaw for Filter {
    fn handle_raw(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc => Some(Action::Targeted(TargetedAction::CloseFilter)),
            KeyCode::Enter => {
                let filter = self.input.lines()[0].clone();
                if filter.len() == 0 {
                    Some(Action::Targeted(TargetedAction::ClearFilter))
                } else {
                    Some(Action::Targeted(TargetedAction::ApplyFilter(filter)))
                }
            }
            _ => {
                self.input.input(key);
                None
            }
        }
    }
}
