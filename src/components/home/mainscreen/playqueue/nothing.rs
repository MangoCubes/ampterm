use ratatui::{layout::Rect, Frame};

use color_eyre::Result;

use crate::components::{
    home::mainscreen::playqueue::PlayQueue,
    lib::centered::Centered,
    traits::{component::Component, focusable::Focusable},
};

pub struct Nothing {
    comp: Centered,
    enabled: bool,
}

impl Nothing {
    pub fn new(enabled: bool) -> Self {
        Self {
            comp: Centered::new(vec!["Nothing in the queue".to_string()]),
            enabled,
        }
    }
}

impl Component for Nothing {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let border = PlayQueue::gen_block(self.enabled, "Queue".to_string());
        let inner = border.inner(area);
        frame.render_widget(border, area);
        self.comp.draw(frame, inner)
    }
}

impl Focusable for Nothing {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
