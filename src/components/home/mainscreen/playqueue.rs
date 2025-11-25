use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::Block,
    Frame,
};

use crate::{
    action::{Action, QueueAction},
    components::{
        home::mainscreen::playqueue::{nothing::Nothing, something::Something},
        traits::{focusable::Focusable, fullcomp::FullComp, renderable::Renderable},
    },
    config::Config,
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
    config: Config,
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

    pub fn new(enabled: bool, config: Config) -> Self {
        Self {
            comp: Comp::Nothing(Nothing::new(enabled)),
            enabled,
            config,
        }
    }
}

impl Renderable for PlayQueue {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.comp {
            Comp::Nothing(nothing) => nothing.draw(frame, area),
            Comp::Something(something) => something.draw(frame, area),
        }
    }
}

impl FullComp for PlayQueue {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Comp::Something(s) = &mut self.comp {
            s.update(action)
        } else {
            match action {
                Action::Queue(QueueAction::Add(items, _)) => {
                    let (comp, action) = Something::new(self.enabled, items, self.config.clone());
                    self.comp = Comp::Something(comp);
                    Ok(Some(action))
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
