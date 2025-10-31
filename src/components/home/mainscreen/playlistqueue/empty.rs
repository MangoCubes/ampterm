use crate::components::{
    home::mainscreen::playlistqueue::PlaylistQueue,
    lib::centered::Centered,
    traits::{component::Component, focusable::Focusable},
};
use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

pub struct Empty {
    name: String,
    comp: Centered,
    enabled: bool,
}

impl Empty {
    pub fn new(name: String, enabled: bool) -> Self {
        Self {
            name,
            comp: Centered::new(vec!["Playlist is empty!".to_string()]),
            enabled,
        }
    }
}

impl Component for Empty {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let border = PlaylistQueue::gen_block(self.enabled, self.name.clone());
        let inner = border.inner(area);
        frame.render_widget(border, area);
        self.comp.draw(frame, inner)
    }
}

impl Focusable for Empty {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
