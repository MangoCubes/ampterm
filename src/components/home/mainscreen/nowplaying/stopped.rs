use crate::components::{lib::centered::Centered, traits::component::Component};
use color_eyre::Result;
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

impl Component for Stopped {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.comp.draw(frame, area)
    }
}
