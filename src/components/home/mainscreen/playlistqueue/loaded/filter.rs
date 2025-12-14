use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};
use tui_textarea::TextArea;

use crate::components::traits::renderable::Renderable;

pub struct Filter {
    input: TextArea<'static>,
}

pub enum FilterResult {
    /// Filtered result should not change
    NoChange,
    /// Filter results so that results that matches this string appear
    ApplyFilter(String),
    /// Clear filter, and display all elements
    ClearFilter,
    /// Exit filter mode, but do not change the applied filter
    Exit,
}

impl Filter {
    pub fn handle_raw(&mut self, key: KeyEvent) -> FilterResult {
        match key.code {
            KeyCode::Esc => FilterResult::Exit,
            KeyCode::Enter => {
                let filter = self.input.lines()[0].clone();
                if filter.len() == 0 {
                    FilterResult::ClearFilter
                } else {
                    FilterResult::ApplyFilter(filter)
                }
            }
            _ => {
                self.input.input(key);
                FilterResult::NoChange
            }
        }
    }
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
        frame.render_widget(&self.input, area);
    }
}
