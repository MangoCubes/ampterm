use crate::components::Component;
use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

pub trait Stateless: Component {
    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}
