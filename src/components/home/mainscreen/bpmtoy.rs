use std::time::{Duration, Instant};

use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};

use crate::{
    action::{
        useraction::{Common, UserAction},
        Action,
    },
    components::{lib::centered::Centered, traits::component::Component},
};
use color_eyre::Result;

enum State {
    Init(Centered),
    NeedToTapMore {
        last_tap: Instant,
        comp: Centered,
    },
    Running {
        last_tap: Instant,
        interval_count: u8,
        total_len: f64,
        comp: Centered,
    },
}

pub struct BPMToy {
    state: State,
}

impl BPMToy {
    pub fn new() -> Self {
        Self {
            state: State::Init(Centered::new(vec!["Tap [t] for BPM".to_string()])),
        }
    }
}

impl Component for BPMToy {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::bordered().border_style(Style::new().white());
        let inner = block.inner(area);
        frame.render_widget(block, area);
        match &mut self.state {
            State::Init(comp)
            | State::NeedToTapMore { comp, last_tap: _ }
            | State::Running {
                comp,
                last_tap: _,
                total_len: _,
                interval_count: _,
            } => comp.draw(frame, inner),
        }
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::User(UserAction::Common(Common::TapToBPM)) = action {
            self.state = match &self.state {
                State::Init(_centered) => State::NeedToTapMore {
                    last_tap: Instant::now(),
                    comp: Centered::new(vec!["Continue tapping...".to_string()]),
                },
                State::NeedToTapMore { last_tap, comp } => State::Running {
                    interval_count: 1,
                    last_tap: Instant::now(),
                    total_len: last_tap.elapsed().as_secs_f64(),
                    comp: Centered::new(vec!["Continue tapping...".to_string()]),
                },
                State::Running {
                    last_tap,
                    total_len,
                    comp,
                    interval_count,
                } => {
                    let elapsed = last_tap.elapsed();
                    if elapsed > Duration::from_secs(3) {
                        State::NeedToTapMore {
                            last_tap: Instant::now(),
                            comp: Centered::new(vec!["Continue tapping...".to_string()]),
                        }
                    } else {
                        let total_len = total_len + last_tap.elapsed().as_secs_f64();
                        let bpm = 60.0 / (total_len / ((*interval_count + 1) as f64));
                        State::Running {
                            interval_count: interval_count + 1,
                            last_tap: Instant::now(),
                            total_len,
                            comp: Centered::new(vec![format!("BPM: {}", bpm)]),
                        }
                    }
                }
            };
            Ok(None)
        } else {
            Ok(None)
        }
    }
}
