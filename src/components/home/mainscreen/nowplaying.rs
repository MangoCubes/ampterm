mod playing;
mod stopped;

use color_eyre::Result;
use playing::Playing;
use ratatui::{layout::Rect, Frame};
use stopped::Stopped;

use crate::{action::Action, components::Component, noparams::NoParams};

enum CompState {
    Stopped { comp: Stopped },
    Playing { comp: Playing },
}

pub struct NowPlaying {
    state: CompState,
}

impl NowPlaying {
    pub fn new() -> Self {
        Self {
            state: CompState::Stopped {
                comp: Stopped::new(),
            },
        }
    }
}

impl Component for NowPlaying {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::InQueue { current, next: _ } = action {
            match current {
                Some(p) => {
                    self.state = CompState::Playing {
                        comp: Playing::new(
                            p.title,
                            p.artist.unwrap_or("Unknown".to_string()),
                            p.album.unwrap_or("Unknown".to_string()),
                        ),
                    }
                }
                None => {
                    self.state = CompState::Stopped {
                        comp: Stopped::new(),
                    }
                }
            };
        };
        Ok(None)
    }
}

impl NoParams for NowPlaying {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.state {
            CompState::Stopped { comp } => comp.draw(frame, area),
            CompState::Playing { comp } => comp.draw(frame, area),
        }
    }
}
