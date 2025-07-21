mod nowplaying;
mod playlistlist;
mod playlistqueue;
mod queuelist;

use crate::{
    action::Action, components::Component, focusable::Focusable, local_action,
    queryworker::query::Query,
};
use color_eyre::Result;
use nowplaying::NowPlaying;
use playlistlist::PlaylistList;
use playlistqueue::PlaylistQueue;
use queuelist::QueueList;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Wrap},
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
    message: String,
}

impl MainScreen {
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        let _ = action_tx.send(Action::Query(Query::GetPlaylists));
        Self {
            state: CurrentlySelected::Playlists,
            pl_list: PlaylistList::new(true),
            pl_queue: PlaylistQueue::new(false),
            queuelist: QueueList::new(false),
            now_playing: NowPlaying::new(),
            message: "You are now logged in.".to_string(),
        }
    }
}

impl Component for MainScreen {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match &action {
            Action::PlayerError(m) => {
                self.message = m.clone();
                Ok(None)
            }
            Action::PlayerMessage(m) => {
                self.message = m.clone();
                Ok(None)
            }
            Action::WindowLeft => {
                self.state = match self.state {
                    CurrentlySelected::Playlists => CurrentlySelected::Queue,
                    CurrentlySelected::Queue => CurrentlySelected::PlaylistQueue,
                    CurrentlySelected::PlaylistQueue => CurrentlySelected::Playlists,
                };
                self.pl_list
                    .set_enabled(self.state == CurrentlySelected::Playlists);
                self.pl_queue
                    .set_enabled(self.state == CurrentlySelected::PlaylistQueue);
                self.queuelist
                    .set_enabled(self.state == CurrentlySelected::Queue);
                Ok(None)
            }
            Action::WindowRight => {
                self.state = match self.state {
                    CurrentlySelected::Playlists => CurrentlySelected::PlaylistQueue,
                    CurrentlySelected::PlaylistQueue => CurrentlySelected::Queue,
                    CurrentlySelected::Queue => CurrentlySelected::Playlists,
                };
                self.pl_list
                    .set_enabled(self.state == CurrentlySelected::Playlists);
                self.pl_queue
                    .set_enabled(self.state == CurrentlySelected::PlaylistQueue);
                self.queuelist
                    .set_enabled(self.state == CurrentlySelected::Queue);
                Ok(None)
            }
            local_action!() => {
                if let Action::ExitVisualModeDiscard | Action::ExitVisualModeSave = action {
                    self.message = "Selection discarded.".to_owned();
                };
                if let Action::ExitVisualModeSave = action {
                    self.message = "Selection applied.".to_owned();
                };
                if let Action::VisualSelectMode = action {
                    self.message = "Hover over items to include...".to_owned();
                };
                if let Action::VisualDeselectMode = action {
                    self.message = "Hover over items to exclude...".to_owned();
                };
                match self.state {
                    CurrentlySelected::Playlists => self.pl_list.update(action),
                    CurrentlySelected::PlaylistQueue => self.pl_queue.update(action),
                    CurrentlySelected::Queue => self.queuelist.update(action),
                }
            }
            _ => {
                self.pl_list.update(action.clone())?;
                self.pl_queue.update(action.clone())?;
                self.queuelist.update(action.clone())?;
                self.now_playing.update(action)?;
                Ok(None)
            }
        }
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

        self.pl_list.draw(frame, listareas[0])?;
        self.pl_queue.draw(frame, listareas[1])?;
        self.queuelist.draw(frame, listareas[2])?;
        self.now_playing.draw(frame, areas[1])?;
        frame.render_widget(
            Paragraph::new(self.message.clone()).wrap(Wrap { trim: false }),
            areas[2],
        );
        Ok(())
    }
}
