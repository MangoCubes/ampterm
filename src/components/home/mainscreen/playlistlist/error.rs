use crate::components::traits::{component::Component, focusable::Focusable};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};

pub struct Error {
    enabled: bool,
    error: String,
}

impl Error {
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
    pub fn new(enabled: bool, error: String) -> Self {
        Self { enabled, error }
    }
}

impl Component for Error {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw("Error!"),
                Line::raw(format!("{}", self.error)),
                Line::raw(format!("Reload with 'R'")),
            ])
            .block(
                Error::gen_block(self.enabled, "Playlist").padding(Padding::new(
                    0,
                    0,
                    (area.height / 2) - 1,
                    0,
                )),
            )
            .alignment(Alignment::Center),
            area,
        );
        Ok(())
    }
}

impl Focusable for Error {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
