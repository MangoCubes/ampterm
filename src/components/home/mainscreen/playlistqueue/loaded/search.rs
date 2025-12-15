use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};
use tui_textarea::TextArea;

use crate::components::traits::renderable::Renderable;

pub struct Search {
    input: TextArea<'static>,
    original: String,
}

pub enum SearchResult {
    /// Search results so that results that matches this string appear
    ApplySearch(String),
    ConfirmSearch(String),
    /// Clear filter, and display all elements
    ClearSearch,
}

impl Search {
    pub fn handle_raw(&mut self, key: KeyEvent) -> SearchResult {
        match key.code {
            KeyCode::Esc => SearchResult::ConfirmSearch(self.original.clone()),
            KeyCode::Enter => SearchResult::ConfirmSearch(self.input.lines()[0].clone()),
            _ => {
                self.input.input(key);
                SearchResult::ApplySearch(self.input.lines()[0].clone())
            }
        }
    }
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
