use ratatui::{
    layout::{Constraint, Layout},
    text::Line,
};

use crate::components::traits::{component::Component, focusable::Focusable};

/// Checkbox element
/// Can be toggled with either enter or space
/// Insert mode must be enabled to interact with the checkbox
pub struct Checkbox {
    /// Specifies whether the component is focused by the user or not
    enabled: bool,
    /// True if the box is checked
    toggle: bool,
    /// Helper text next to the checkbox. Must fit in a single line
    label: Line<'static>,
}

impl Checkbox {
    pub fn new(enabled: bool, toggle: bool, label: String) -> Self {
        Checkbox {
            enabled,
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
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::eyre::Result<()> {
        let layout = Layout::horizontal([Constraint::Min(1), Constraint::Length(3)]);
        let areas = layout.split(area);
        frame.render_widget(self.label.clone(), areas[0]);
        frame.render_widget(Line::raw(if self.toggle { "[X]" } else { "[ ]" }), areas[1]);
        Ok(())
    }
}

impl Focusable for Checkbox {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
        }
    }
}
