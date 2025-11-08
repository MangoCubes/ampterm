use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Clear,
    Frame,
};

use crate::components::traits::fullcomp::FullComp;
pub struct Popup<T: FullComp> {
    width: u16,
    height: u16,
    comp: T,
}

impl<T: FullComp> Popup<T> {
    pub fn new(comp: T, width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            comp,
        }
    }
}

impl<T: FullComp> FullComp for Popup<T> {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical = Layout::vertical([Constraint::Percentage(self.width)]).flex(Flex::Center);
        let horizontal =
            Layout::horizontal([Constraint::Percentage(self.height)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        self.comp.draw(frame, area)
    }
}
