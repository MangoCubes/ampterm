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
    stateless::Stateless,
};
use color_eyre::Result;

pub struct QueueList {
    comp: List<'static>,
    list: Vec<Media>,
    state: ListState,
    action_tx: UnboundedSender<Action>,
}

impl QueueList {
    fn gen_list(&self) -> List<'static> {
        let items: Vec<String> = self.list.iter().map(|p| p.title.clone()).collect();
        List::new(items)
            .block(Block::bordered().title("Next Up"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        let empty = vec![];
        Self {
            state: ListState::default(),
            comp: List::default()
                .block(Block::bordered().title("Next Up"))
                .highlight_style(Style::new().reversed())
                .highlight_symbol(">"),
            list: empty,
            action_tx,
        }
    }
}

impl Component for QueueList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::InQueue(q) = action {
            self.list = q;
            self.comp = self.gen_list()
        }
        Ok(None)
    }
}

impl Stateless for QueueList {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
}
