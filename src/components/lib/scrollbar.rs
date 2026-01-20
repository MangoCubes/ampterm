use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::Block,
    Frame,
};

use crate::components::traits::renderable::Renderable;

/// Scrollbar component
/// Receives max size of x and index between 0 and x - 1. Scrollbar is drawn so that the bar is
/// placed at a specified index.
pub struct ScrollBar {
    /// Max index. Internally, unlike arrays, [`ScrollBar::max`] may be equal to [`ScrollBar::current`].
    max: u32,
    current: u32,
    block: Block<'static>,
}

impl ScrollBar {
    pub fn new(max: u32, current: u32) -> Self {
        Self {
            max: max - 1,
            current,
            block: Block::default().bg(Color::White),
        }
    }
    pub fn update_pos(&mut self, current: u32) {
        self.current = if self.max <= current {
            self.max
        } else {
            current
        };
    }
}

impl Renderable for ScrollBar {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [_, bar, _] = Layout::vertical([
            Constraint::Ratio(self.current, self.max),
            Constraint::Length(3),
            Constraint::Ratio(self.max - self.current, self.max),
        ])
        .areas(area);

        frame.render_widget(&self.block, bar);
    }
}
