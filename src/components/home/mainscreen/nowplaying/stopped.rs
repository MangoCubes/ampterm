use crate::components::{lib::centered::Centered, traits::renderable::Renderable};
use ratatui::{layout::Rect, Frame};

pub struct Stopped {
    comp: Centered,
}

impl Stopped {
    pub fn new() -> Self {
        Self {
            comp: Centered::new(vec!["Select a music!".to_string()]),
        }
    }
}

impl Renderable for Stopped {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.comp.draw(frame, area);
    }
}
