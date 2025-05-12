use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, List, ListState},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{getplaylist::Media, Action},
    components::Component,
    stateful::Stateful,
};
use color_eyre::Result;

pub struct QueueList {
    comp: List<'static>,
    list: Vec<Media>,
    state: ListState,
    action_tx: UnboundedSender<Action>,
    enabled: bool,
}

impl QueueList {
    fn gen_list(enabled: bool, list: Option<&Vec<Media>>) -> List<'static> {
        let comp = match list {
            Some(l) => {
                let items: Vec<String> = l.iter().map(|p| p.title.clone()).collect();
                List::new(items)
            }
            None => List::default(),
        };
        let style = if enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };
        comp.block(Block::bordered().border_style(style).title("Next Up"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }

    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        let empty = vec![];
        Self {
            state: ListState::default(),
            comp: Self::gen_list(false, None),
            list: empty,
            action_tx,
            enabled: false,
        }
    }
}

impl Component for QueueList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::InQueue(q) = action {
            self.list = q;
            self.comp = Self::gen_list(self.enabled, Some(&self.list))
        }
        Ok(None)
    }
}

impl Stateful<bool> for QueueList {
    fn draw_state(&mut self, frame: &mut Frame, area: Rect, state: bool) -> Result<()> {
        if self.enabled != state {
            self.enabled = state;
            self.comp = Self::gen_list(self.enabled, Some(&self.list))
        }
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
}
