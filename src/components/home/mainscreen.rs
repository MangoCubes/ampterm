mod playlist_list;

use crate::{action::Action, components::Component};
use color_eyre::Result;
use playlist_list::PlayListList;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

enum CurrentlySelected {
    CurrentlyPlaying,
    Queue,
}

pub struct MainScreen {
    state: CurrentlySelected,
    pl_list: PlayListList,
    action_tx: UnboundedSender<Action>,
}

impl MainScreen {
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self {
            state: CurrentlySelected::CurrentlyPlaying,
            pl_list: PlayListList::new(action_tx.clone()),
            action_tx,
        }
    }
}

impl Component for MainScreen {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let layout = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ]);
        let areas = layout.split(area);

        if let Err(err) = self.pl_list.draw(frame, areas[0]) {
            self.action_tx.send(Action::Error(err.to_string()))?;
        }
        Ok(())
    }
}
