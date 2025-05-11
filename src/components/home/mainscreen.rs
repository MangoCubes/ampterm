mod playlistlist;
mod playlistqueue;
mod queuelist;

use crate::{action::Action, components::Component, queryworker::query::Query};
use color_eyre::Result;
use playlistlist::PlaylistList;
use playlistqueue::PlaylistQueue;
use queuelist::QueueList;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

enum CurrentlySelected {
    Playlists,
    PlaylistQueue,
    Queue,
}

pub struct MainScreen {
    state: CurrentlySelected,
    pl_list: PlaylistList,
    pl_queue: PlaylistQueue,
    queuelist: QueueList,
    action_tx: UnboundedSender<Action>,
}

impl MainScreen {
    fn select_playlist(&self) {}
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        let _ = action_tx.send(Action::Query(Query::GetPlaylists));
        Self {
            state: CurrentlySelected::Playlists,
            pl_list: PlaylistList::new(action_tx.clone()),
            pl_queue: PlaylistQueue::new(action_tx.clone()),
            queuelist: QueueList::new(action_tx.clone()),
            action_tx,
        }
    }
}

impl Component for MainScreen {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match &action {
            Action::SelectPlaylist { key } => self.select_playlist(),
            Action::MoveLeft => {
                self.state = match self.state {
                    CurrentlySelected::Playlists => CurrentlySelected::Queue,
                    CurrentlySelected::Queue => CurrentlySelected::PlaylistQueue,
                    CurrentlySelected::PlaylistQueue => CurrentlySelected::Playlists,
                }
            }
            Action::MoveRight => {
                self.state = match self.state {
                    CurrentlySelected::Playlists => CurrentlySelected::PlaylistQueue,
                    CurrentlySelected::PlaylistQueue => CurrentlySelected::Queue,
                    CurrentlySelected::Queue => CurrentlySelected::Playlists,
                }
            }
            _ => {}
        };
        match &action {
            Action::Local(_) => match self.state {
                CurrentlySelected::Playlists => {
                    if let Some(action) = self.pl_list.update(action)? {
                        self.action_tx.send(action)?;
                    }
                }
                CurrentlySelected::PlaylistQueue => {
                    if let Some(action) = self.pl_queue.update(action)? {
                        self.action_tx.send(action)?;
                    }
                }
                CurrentlySelected::Queue => {
                    if let Some(action) = self.queuelist.update(action)? {
                        self.action_tx.send(action)?;
                    }
                }
            },
            _ => {
                if let Some(action) = self.pl_list.update(action.clone())? {
                    self.action_tx.send(action)?;
                }
                if let Some(action) = self.pl_queue.update(action.clone())? {
                    self.action_tx.send(action)?;
                }
                if let Some(action) = self.queuelist.update(action)? {
                    self.action_tx.send(action)?;
                }
            }
        };
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(10),
            Constraint::Length(1),
        ]);
        let horizontal = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ]);
        let areas = vertical.split(area);
        let listareas = horizontal.split(areas[0]);

        if let Err(err) = self.pl_list.draw(frame, listareas[0]) {
            self.action_tx.send(Action::Error(err.to_string()))?;
        }
        if let Err(err) = self.pl_queue.draw(frame, listareas[1]) {
            self.action_tx.send(Action::Error(err.to_string()))?;
        }
        if let Err(err) = self.queuelist.draw(frame, listareas[2]) {
            self.action_tx.send(Action::Error(err.to_string()))?;
        }
        Ok(())
    }
}
