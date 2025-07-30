use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Clear,
    Frame,
};

use crate::{components::Component, focusable::Focusable};

pub struct Popup {
    visible: bool,
    width: u16,
    height: u16,
    comp: Box<dyn Component>,
}

impl Popup {
    pub fn new(comp: Box<dyn Component>, width: u16, height: u16) -> Self {
        Self {
            visible: false,
            width,
            height,
            comp,
        }
    }
    pub fn default(comp: Box<dyn Component>) -> Self {
        Self {
            visible: false,
            width: 50,
            height: 30,
            comp,
        }
    }
}

impl Component for Popup {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }
        let vertical = Layout::vertical([Constraint::Percentage(self.width)]).flex(Flex::Center);
        let horizontal =
            Layout::horizontal([Constraint::Percentage(self.height)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        self.comp.draw(frame, area)
    }
}

impl Focusable for Popup {
    fn set_enabled(&mut self, enable: bool) {
        self.visible = enable;
    }
}
