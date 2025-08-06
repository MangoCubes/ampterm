mod error;
mod loaded;
mod loading;

use crate::{
    action::{
        getplaylists::{GetPlaylistsResponse, PlaylistID, SimplePlaylist},
        Action,
    },
    components::{
        home::mainscreen::playlistlist::{
            error::PlaylistListError, loaded::PlaylistListLoaded, loading::PlaylistListLoading,
        },
        Component,
    },
    focusable::Focusable,
    insert_action, local_action,
    playerworker::player::QueueLocation,
    queryworker::query::Query,
};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListState, Padding, Paragraph, Wrap},
    Frame,
};

pub trait PlaylistListComps: Focusable {}

pub struct PlaylistList {
    comp: Box<dyn PlaylistListComps>,
    enabled: bool,
}

impl PlaylistList {
    pub fn new(enabled: bool) -> Self {
        Self {
            comp: Box::new(PlaylistListLoading::new(enabled)),
            enabled,
        }
    }
}

impl Component for PlaylistList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::GetPlaylists(res) => match res {
                GetPlaylistsResponse::Success(simple_playlists) => {
                    self.comp = Box::new(PlaylistListLoaded::new(
                        self.enabled,
                        simple_playlists,
                        ListState::default().with_selected(Some(0)),
                        None,
                    ));
                    Ok(None)
                }
                GetPlaylistsResponse::Failure(error) => {
                    self.comp = Box::new(PlaylistListError::new(self.enabled, error));
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.comp.draw(frame, area)
    }
}

impl Focusable for PlaylistList {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            self.comp.set_enabled(enable);
        };
    }
}
