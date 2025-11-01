use color_eyre::eyre::Result;
use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Clear},
    Frame,
};

use crate::components::traits::component::Component;

pub struct Tasks {
    border: Block<'static>,
}

impl Tasks {
    pub fn new() -> Self {
        Self {
            border: Self::gen_block(),
        }
    }

    fn gen_block() -> Block<'static> {
        let style = Style::new().white();
        let title = Span::styled("Tasks", Style::default().add_modifier(Modifier::BOLD));
        Block::bordered().title(title).border_style(style)
    }
}

impl Component for Tasks {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical = Layout::vertical([Constraint::Percentage(80)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(80)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        frame.render_widget(&self.border, area);
        Ok(())
    }
}
