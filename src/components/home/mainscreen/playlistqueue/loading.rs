use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

use crate::components::{
    home::mainscreen::playlistqueue::PlaylistQueue,
    lib::centered::Centered,
    traits::{focusable::Focusable, renderable::Renderable},
};

pub struct Loading {
    name: String,
    enabled: bool,
    comp: Centered,
}

impl Loading {
    pub fn new(name: String, enabled: bool) -> Self {
        Self {
            name,
            comp: Centered::new(vec!["Loading...".to_string()]),
            enabled,
        }
    }
}

impl Renderable for Loading {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let border = PlaylistQueue::gen_block(self.enabled, self.name.clone());
        let inner = border.inner(area);
        frame.render_widget(border, area);
        self.comp.draw(frame, inner)
    }
}

impl Focusable for Loading {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
