use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Padding, Paragraph, Wrap},
    Frame,
};

use crate::{
    components::{
        home::mainscreen::playlistqueue::PlaylistQueue,
        traits::{component::Component, focusable::Focusable},
    },
    queryworker::query::getplaylists::PlaylistID,
};

pub struct Loading {
    id: PlaylistID,
    name: String,
    enabled: bool,
}

impl Loading {
    pub fn new(id: PlaylistID, name: String, enabled: bool) -> Self {
        Self { id, name, enabled }
    }
}

impl Component for Loading {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new("Loading...")
                .block(
                    PlaylistQueue::gen_block(self.enabled, self.name.clone())
                        .padding(Padding::new(0, 0, area.height / 2, 0)),
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            area,
        );
        Ok(())
    }
}

impl Focusable for Loading {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
