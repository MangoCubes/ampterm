use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Row, Table, TableState, Widget},
    Frame,
};

use crate::{
    action::{
        useraction::{Common, Normal, UserAction, Visual},
        Action, FromPlayerWorker, PlayState,
    },
    app::Mode,
    components::{
        home::mainscreen::queuelist::{nothing::Nothing, something::Something},
        traits::{component::Component, focusable::Focusable},
    },
    playerworker::player::ToPlayerWorker,
};
use color_eyre::Result;

mod nothing;
mod something;

enum Comp {
    Nothing(Nothing),
    Something(Something),
}

pub struct QueueList {
    comp: Comp,
    enabled: bool,
}

impl QueueList {
    fn gen_block(&self) -> Block<'static> {
        let style = if self.enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };
        let title = Span::styled(
            "Queue".to_string(),
            if self.enabled {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            },
        );
        Block::bordered().title(title).border_style(style)
    }

    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            comp: Comp::Nothing(Nothing {}),
        }
    }
}

impl Component for QueueList {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = self.gen_block();
        let inner = block.inner(area);
        frame.render_widget(block, area);
        match &mut self.comp {
            Comp::Nothing(nothing) => nothing.draw(frame, inner),
            Comp::Something(something) => something.draw(frame, inner),
        }
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Comp::Something(s) = &mut self.comp {
            s.update(action)
        } else {
            match action {
                Action::FromPlayerWorker(a) => match a {
                    FromPlayerWorker::InQueue {
                        play,
                        vol,
                        speed,
                        pos,
                    } => {
                        let comp = Something::new(play);
                        self.comp = Comp::Something(comp)
                    }
                    _ => {}
                },
                _ => {}
            }
            Ok(None)
        }
    }
}

impl Focusable for QueueList {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
        }
    }
}
