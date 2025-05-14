mod nowplaying;
mod playlistlist;
mod playlistqueue;
mod queuelist;

use crate::{
    action::Action, components::Component, queryworker::query::Query, stateful::Stateful,
    stateless::Stateless,
};
use color_eyre::Result;
use nowplaying::NowPlaying;
use playlistlist::PlaylistList;
use playlistqueue::PlaylistQueue;
use queuelist::QueueList;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(PartialEq)]
enum CurrentlySelected {
    Playlists,
    PlaylistQueue,
    Queue,
}

pub struct MainScreen {
    state: CurrentlySelected,
    pl_list: PlaylistList,
    pl_queue: PlaylistQueue,
    now_playing: NowPlaying,
    queuelist: QueueList,
}

impl MainScreen {
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        let _ = action_tx.send(Action::Query(Query::GetPlaylists));
        Self {
            state: CurrentlySelected::Playlists,
            pl_list: PlaylistList::new(),
            pl_queue: PlaylistQueue::new(),
            queuelist: QueueList::new(),
            now_playing: NowPlaying::new(),
        }
    }
}

impl Component for MainScreen {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match &action {
            Action::MoveLeft => {
                self.state = match self.state {
                    CurrentlySelected::Playlists => CurrentlySelected::Queue,
                    CurrentlySelected::Queue => CurrentlySelected::PlaylistQueue,
                    CurrentlySelected::PlaylistQueue => CurrentlySelected::Playlists,
                };
                Ok(None)
            }
            Action::MoveRight => {
                self.state = match self.state {
                    CurrentlySelected::Playlists => CurrentlySelected::PlaylistQueue,
                    CurrentlySelected::PlaylistQueue => CurrentlySelected::Queue,
                    CurrentlySelected::Queue => CurrentlySelected::Playlists,
                };
                Ok(None)
            }
            Action::Local(_) => match self.state {
                CurrentlySelected::Playlists => self.pl_list.update(action),
                CurrentlySelected::PlaylistQueue => self.pl_queue.update(action),
                CurrentlySelected::Queue => self.queuelist.update(action),
            },
            _ => {
                self.pl_list.update(action.clone())?;
                self.pl_queue.update(action.clone())?;
                self.queuelist.update(action)?;
                Ok(None)
            }
        }
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
}

impl Stateless for MainScreen {
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

        self.pl_list.draw_state(
            frame,
            listareas[0],
            self.state == CurrentlySelected::Playlists,
        )?;
        self.pl_queue.draw_state(
            frame,
            listareas[1],
            self.state == CurrentlySelected::PlaylistQueue,
        )?;
        self.queuelist
            .draw_state(frame, listareas[2], self.state == CurrentlySelected::Queue)?;
        self.now_playing.draw(frame, areas[1])?;
        Ok(())
    }
}
