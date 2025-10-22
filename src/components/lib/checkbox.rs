use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
};

use crate::components::traits::{component::Component, focusable::Focusable};

/// Checkbox element
/// Can be toggled with space
/// Insert mode must be enabled to interact with the checkbox
pub struct Checkbox {
    /// Specifies whether the component is focused by the user or not
    focused: bool,
    /// True if the box is checked
    toggle: bool,
    /// Helper text next to the checkbox. Must fit in a single line
    label: Line<'static>,
}

impl Checkbox {
    pub fn new(focused: bool, toggle: bool, label: String) -> Self {
        Checkbox {
            focused,
            toggle,
            label: Line::raw(label),
        }
    }
    pub fn toggle(&mut self) {
        self.toggle = !self.toggle;
    }
    pub fn get_toggle(&self) -> bool {
        self.toggle
    }
}

impl Component for Checkbox {
    fn handle_key_event(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::eyre::Result<Option<crate::action::Action>> {
        if let KeyCode::Char(' ') = key.code {
            self.toggle()
        };
        Ok(None)
    }
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::eyre::Result<()> {
        let layout = Layout::horizontal([Constraint::Min(1), Constraint::Length(3)]);
        let areas = layout.split(area);
        frame.render_widget(self.label.clone(), areas[0]);
        let mut checkbox = Line::raw(if self.toggle { "[X]" } else { "[ ]" });
        if self.focused {
            checkbox = checkbox.reversed();
        }
        frame.render_widget(checkbox, areas[1]);
        Ok(())
    }
}

impl Focusable for Checkbox {
    fn set_enabled(&mut self, enable: bool) {
        if self.focused != enable {
            self.focused = enable;
        }
    }
}
