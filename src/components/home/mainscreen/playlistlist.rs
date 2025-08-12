mod error;
mod loaded;
mod loading;

use crate::{
    action::{getplaylists::GetPlaylistsResponse, Action, FromQueryWorker},
    components::{
        home::mainscreen::playlistlist::{
            error::PlaylistListError, loaded::PlaylistListLoaded, loading::PlaylistListLoading,
        },
        traits::{component::Component, focusable::Focusable},
    },
};
use color_eyre::Result;
use ratatui::{layout::Rect, widgets::ListState, Frame};

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
            Action::FromQueryWorker(FromQueryWorker::GetPlaylists(res)) => match res {
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
            _ => self.comp.update(action),
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
