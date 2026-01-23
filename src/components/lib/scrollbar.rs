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
    max: u32,
    current: u32,
    block: Block<'static>,
}

impl ScrollBar {
    pub fn new(max: u32, current: u32) -> Self {
        Self {
            max,
            current,
            block: Block::default().bg(Color::White),
        }
    }
    pub fn update_pos(&mut self, current: u32) {
        self.current = if self.max == 0 {
            0
        } else if self.max <= current {
            self.max - 1
        } else {
            current
        };
    }
    pub fn update_max(&mut self, max: u32) {
        if max == 0 {
            self.current = 0;
        } else if max < self.current {
            self.current = max - 1;
        }
        self.max = max;
    }
}

impl Renderable for ScrollBar {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let bar = if self.max == 0 {
            area
        } else {
            let [_, bar, _] = Layout::vertical([
                Constraint::Ratio(self.current, self.max),
                Constraint::Min(3),
                Constraint::Ratio(self.max - 1 - self.current, self.max),
            ])
            .areas(area);
            bar
        };

        frame.render_widget(&self.block, bar);
    }
}
