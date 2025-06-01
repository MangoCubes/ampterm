use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

use crate::components::Component;

pub trait HasParams<T>: Component {
    fn draw_params(&mut self, frame: &mut Frame, area: Rect, state: T) -> Result<()>;
}
