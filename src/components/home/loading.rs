use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

use crate::components::{lib::centered::Centered, traits::renderable::Renderable};

pub struct Loading {
    comp: Centered,
}

impl Loading {
    pub fn new(url: String, username: String) -> Self {
        let comp = Centered::new(vec![
            "Logging in with the credentials in the configuration...".to_string(),
            format!("URL: {}", url),
            format!("Username: {}", username),
        ]);
        Self { comp }
    }
}
impl Renderable for Loading {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.comp.draw(frame, area)
    }
}
