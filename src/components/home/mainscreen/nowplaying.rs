mod playing;
mod stopped;

use std::time::Duration;

use color_eyre::Result;
use playing::{Playing, PlayingState};
use ratatui::{layout::Rect, Frame};
use stopped::Stopped;

use crate::{
    action::{Action, StateType},
    components::Component,
    hasparams::HasParams,
    noparams::NoParams,
};

enum CompState {
    Stopped {
        comp: Stopped,
    },
    Playing {
        comp: Playing,
        playing_state: PlayingState,
    },
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
        if let Action::PlayerState(v) = &action {
            if let CompState::Playing {
                comp: _,
                playing_state,
            } = &mut self.state
            {
                *playing_state = match v {
                    StateType::Position(duration) => PlayingState {
                        vol: playing_state.vol,
                        speed: playing_state.speed,
                        pos: *duration,
                    },
                    StateType::Volume(v) => PlayingState {
                        vol: *v,
                        speed: playing_state.speed,
                        pos: playing_state.pos,
                    },
                    StateType::Speed(s) => PlayingState {
                        vol: playing_state.vol,
                        speed: *s,
                        pos: playing_state.pos,
                    },
                };
            }
        } else if let Action::InQueue {
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
                        ),
                        playing_state: PlayingState { vol, speed, pos },
                    }
                }
                None => {
                    self.state = CompState::Stopped {
                        comp: Stopped::new(),
                    }
                }
            };
        }
        Ok(None)
    }
}

impl NoParams for NowPlaying {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.state {
            CompState::Stopped { comp } => comp.draw(frame, area),
            CompState::Playing {
                comp,
                playing_state,
            } => comp.draw_params(frame, area, playing_state),
        }
    }
}
