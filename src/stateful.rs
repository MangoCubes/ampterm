use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

use crate::components::Component;

pub trait Stateful<T>: Component {
    fn draw_state(&mut self, frame: &mut Frame, area: Rect, state: T) -> Result<()>;
}
