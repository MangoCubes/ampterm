use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::Block,
    Frame,
};

use crate::{
    action::{Action, FromPlayerWorker, QueueChange, StateType},
    components::{
        home::mainscreen::playqueue::{nothing::Nothing, something::Something},
        traits::{component::Component, focusable::Focusable},
    },
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{FromQueryWorker, ResponseType, ToQueryWorker},
    },
};
use color_eyre::Result;

mod nothing;
mod something;

enum Comp {
    Nothing(Nothing),
    Something(Something),
}

pub struct PlayQueue {
    comp: Comp,
    enabled: bool,
}

impl PlayQueue {
    fn gen_block(enabled: bool, title: String) -> Block<'static> {
        let style = if enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };
        let title = Span::styled(
            title,
            if enabled {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            },
        );
        Block::bordered().title(title).border_style(style)
    }

    pub fn new(enabled: bool) -> Self {
        Self {
            comp: Comp::Nothing(Nothing::new(enabled)),
            enabled,
        }
    }
}

impl Component for PlayQueue {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.comp {
            Comp::Nothing(nothing) => nothing.draw(frame, area),
            Comp::Something(something) => something.draw(frame, area),
        }
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Comp::Something(s) = &mut self.comp {
            s.update(action)
        } else {
            match action {
                Action::FromPlayerWorker(FromPlayerWorker::StateChange(StateType::Queue(
                    QueueChange::Add { items, at },
                ))) => {
                    let comp = Something::new(self.enabled, items);
                    self.comp = Comp::Something(comp);
                    Ok(None)
                }
                _ => Ok(None),
            }
        }
    }
}
impl Focusable for PlayQueue {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            match &mut self.comp {
                Comp::Nothing(comp) => comp.set_enabled(enable),
                Comp::Something(comp) => comp.set_enabled(enable),
            }
        };
    }
}
