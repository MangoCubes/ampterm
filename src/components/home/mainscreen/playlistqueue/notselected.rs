use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Padding, Paragraph, Wrap},
    Frame,
};

use crate::components::{
    home::mainscreen::playlistqueue::PlaylistQueue,
    traits::{focusable::Focusable, renderable::Renderable},
};

pub struct NotSelected {
    enabled: bool,
}

impl NotSelected {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Renderable for NotSelected {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new("Choose a playlist!")
                .block(
                    PlaylistQueue::gen_block(self.enabled, "Playlist Queue".to_string())
                        .padding(Padding::new(0, 0, area.height / 2, 0)),
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            area,
        );
        Ok(())
    }
}

impl Focusable for NotSelected {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
