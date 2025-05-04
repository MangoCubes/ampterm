mod playlistlist;

use crate::{
    action::{getplaylists::GetPlaylistsResponse, Action},
    components::Component,
    queryworker::query::Query,
    trace_dbg,
};
use color_eyre::Result;
use playlistlist::PlaylistList;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

enum CurrentlySelected {
    Playlists,
    CurrentlyPlaying,
    Queue,
}

pub struct MainScreen {
    state: CurrentlySelected,
    pl_list: PlaylistList,
    action_tx: UnboundedSender<Action>,
}

impl MainScreen {
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        let _ = action_tx.send(Action::Query(Query::GetPlaylists));
        Self {
            state: CurrentlySelected::Playlists,
            pl_list: PlaylistList::new(action_tx.clone()),
            action_tx,
        }
    }
}

impl Component for MainScreen {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Some(action) = self.pl_list.update(action)? {
            self.action_tx.send(action)?;
        }
        // if let Action::GetPlaylists(r) = action {
        //     match r {
        //         GetPlaylistsResponse::Success(p) => {
        //             trace_dbg!(p);
        //         }
        //         GetPlaylistsResponse::Failure(e) => {
        //             trace_dbg!(e);
        //         }
        //     };
        // };
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
