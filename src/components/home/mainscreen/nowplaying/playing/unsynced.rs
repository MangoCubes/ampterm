use color_eyre::eyre::Result;
use ratatui::{
    prelude::Rect,
    style::{Style, Stylize},
    widgets::{List, ListState},
    Frame,
};

use crate::{
    action::{
        useraction::{Common, UserAction},
        Action,
    },
    components::traits::{renderable::Renderable, simplecomp::SimpleComp},
};

pub struct Unsynced {
    lyrics: Vec<String>,
    comp: List<'static>,
    state: ListState,
}

impl Unsynced {
    pub fn new(found: String) -> Self {
        let list: Vec<String> = found.lines().map(|line| line.to_string()).collect();
        let mut default = ListState::default();
        default.select_first();
        Self {
            lyrics: list.clone(),
            comp: Self::gen_list(list),
            state: default,
        }
    }
    fn gen_list(list: Vec<String>) -> List<'static> {
        List::new(list)
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
            .scroll_padding(1)
    }
}

impl Renderable for Unsynced {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
}

impl SimpleComp for Unsynced {
    fn update(&mut self, action: Action) {
        match action {
            Action::User(UserAction::Common(local)) => match local {
                Common::Up => {
                    self.state.select_previous();
                }
                Common::Down => {
                    self.state.select_next();
                }
                Common::Top => {
                    self.state.select_first();
                }
                Common::Bottom => {
                    self.state.select_last();
                }
                _ => {}
            },
            _ => {}
        }
    }
}
