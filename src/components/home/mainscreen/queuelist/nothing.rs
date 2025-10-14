use ratatui::{layout::Rect, Frame};

use color_eyre::Result;

use crate::components::{lib::centered::Centered, traits::component::Component};

pub struct Nothing {
    comp: Centered,
}

impl Nothing {
    pub fn new() -> Self {
        Self {
            comp: Centered::new(vec!["Nothing in the queue".to_string()]),
        }
    }
}

impl Component for Nothing {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.comp.draw(frame, area)
    }
}
