use ratatui::{layout::Rect, Frame};

/// Renderable trait means that the component can be drawn on the screen. It essentially means that
/// it is a component.
pub trait Renderable {
    fn draw(&mut self, frame: &mut Frame, area: Rect);
}
