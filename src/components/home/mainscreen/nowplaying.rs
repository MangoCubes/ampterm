mod playing;
mod stopped;

use color_eyre::Result;
use playing::Playing;
use ratatui::{layout::Rect, Frame};
use stopped::Stopped;

use crate::{action::Action, components::Component};

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
        if let Action::InQueue {
            current,
            next: _,
            vol,
            speed,
            pos,
        } = action
        {
            match current {
                Some(p) => {
                    self.state = CompState::Playing {
                        comp: Playing::new(
                            p.title,
                            p.artist.unwrap_or("Unknown".to_string()),
                            p.album.unwrap_or("Unknown".to_string()),
                            p.duration.unwrap_or(1),
                            vol,
                            speed,
                            pos,
                        ),
                    }
                }
                None => {
                    self.state = CompState::Stopped {
                        comp: Stopped::new(),
                    }
                }
            };
        } else {
            if let CompState::Playing { comp } = &mut self.state {
                return comp.update(action);
            }
        }
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.state {
            CompState::Stopped { comp } => comp.draw(frame, area),
            CompState::Playing { comp } => comp.draw(frame, area),
        }
    }
}
