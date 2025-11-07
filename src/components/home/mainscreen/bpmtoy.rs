use std::time::{Duration, Instant};

use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Block,
    Frame,
};

use crate::{
    action::{
        useraction::{Global, UserAction},
        Action,
    },
    components::{
        lib::centered::Centered,
        traits::{component::Component, ontick::OnTick},
    },
    config::Config,
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
        interval_count: u32,
        total_len: f64,
        comp: Centered,
    },
}

pub struct BPMToy {
    state: State,
    init_msg: String,
}

impl OnTick for BPMToy {
    fn on_tick(&mut self) {
        match self.state {
            State::NeedToTapMore { last_tap, comp: _ } => {
                let elapsed = last_tap.elapsed();
                if elapsed > Duration::from_secs(3) {
                    self.state = State::Init(Centered::new(vec![self.init_msg.clone()]));
                }
            }
            State::Running {
                last_tap,
                interval_count,
                total_len,
                comp: _,
            } => {
                let elapsed = last_tap.elapsed();
                if elapsed > Duration::from_secs(3) {
                    let bpm = 60.0 / (total_len / (interval_count as f64));
                    self.state = State::Init(Centered::new(vec![format!("BPM: {:.2}", bpm)]));
                }
            }
            _ => {}
        };
    }
}

impl BPMToy {
    pub fn new(config: Config) -> Self {
        let keys = config
            .keybindings
            .find_action_str(Action::User(UserAction::Global(Global::TapToBPM)), None);
        let msg = match keys {
            Some(t) => format!("Tap {} for BPM", t),
            None => "Tap to BPM not bound!".to_string(),
        };

        Self {
            init_msg: msg.clone(),
            state: State::Init(Centered::new(vec![msg])),
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
        if let Action::User(UserAction::Global(Global::TapToBPM)) = action {
            self.state = match &self.state {
                State::Init(_centered) => State::NeedToTapMore {
                    last_tap: Instant::now(),
                    comp: Centered::new(vec!["Continue tapping...".to_string()]),
                },
                State::NeedToTapMore { last_tap, comp: _ } => {
                    let elapsed = last_tap.elapsed();
                    if elapsed > Duration::from_secs(3) {
                        State::NeedToTapMore {
                            last_tap: Instant::now(),
                            comp: Centered::new(vec!["Continue tapping...".to_string()]),
                        }
                    } else {
                        State::Running {
                            interval_count: 1,
                            last_tap: Instant::now(),
                            total_len: elapsed.as_secs_f64(),
                            comp: Centered::new(vec!["Continue tapping...".to_string()]),
                        }
                    }
                }
                State::Running {
                    last_tap,
                    total_len,
                    comp: _,
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
                            comp: Centered::new(vec![format!("BPM: {:.2}", bpm)]),
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
