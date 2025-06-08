use crate::{components::Component, focusable::Focusable};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

use super::PlaylistQueueComps;

pub struct NotSelected {
    enabled: bool,
}

impl NotSelected {
    fn gen_block(enabled: bool, title: &str) -> Block<'static> {
        let style = if enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };
        let title = Span::styled(
            title.to_string(),
            if enabled {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            },
        );
        Block::bordered().title(title).border_style(style)
    }
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Component for NotSelected {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new("Choose a playlist!")
                .block(
                    Self::gen_block(self.enabled, "Playlist Queue").padding(Padding::new(
                        0,
                        0,
                        area.height / 2,
                        0,
                    )),
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
impl PlaylistQueueComps for NotSelected {}
