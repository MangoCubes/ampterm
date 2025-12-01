use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::Block,
    Frame,
};

use crate::{
    action::action::{Action, QueryAction, QueueAction, TargetedAction},
    components::{
        home::mainscreen::playqueue::{nothing::Nothing, something::Something},
        traits::{
            focusable::Focusable,
            handleaction::HandleAction,
            handlekeyseq::{HandleKeySeq, KeySeqResult, PassKeySeq},
            handlequery::HandleQuery,
            renderable::Renderable,
        },
    },
    config::Config,
};

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

impl HandleQuery for PlayQueue {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
        match action {
            QueryAction::FromPlayerWorker(_) => match &mut self.comp {
                Comp::Nothing(_) => None,
                Comp::Something(something) => something.handle_query(action),
            },
            _ => None,
        }
    }
}

impl PassKeySeq for PlayQueue {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        match &mut self.comp {
            Comp::Something(something) => something.handle_key_seq(keyseq),
            Comp::Nothing(_) => None,
        }
    }
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
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match &mut self.comp {
            Comp::Nothing(nothing) => nothing.draw(frame, area),
            Comp::Something(something) => something.draw(frame, area),
        }
    }
}

impl HandleAction for PlayQueue {
    fn handle_action(&mut self, action: TargetedAction) -> Option<Action> {
        if let Comp::Something(s) = &mut self.comp {
            s.handle_action(action)
        } else {
            match action {
                TargetedAction::Queue(QueueAction::Add(items, _)) => {
                    let (comp, action) = Something::new(self.enabled, items, self.config.clone());
                    self.comp = Comp::Something(comp);
                    Some(action)
                }
                _ => None,
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
