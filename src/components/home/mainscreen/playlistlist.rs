mod error;
mod loaded;
mod loading;

use crate::{
    action::Action,
    components::{
        home::mainscreen::playlistlist::{self, error::Error, loaded::Loaded, loading::Loading},
        traits::{component::Component, focusable::Focusable, synccomp::SyncComp},
    },
    queryworker::query::{getplaylists::GetPlaylistsResponse, FromQueryWorker, ResponseType},
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
            comp: Box::new(Loading::new(enabled)),
            enabled,
        }
    }
}

impl Component for PlaylistList {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.comp.draw(frame, area)
    }
}

impl SyncComp for PlaylistList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FromQueryWorker(qw) => {
                if let ResponseType::GetPlaylists(res) = qw.res {
                    match res {
                        GetPlaylistsResponse::Success(simple_playlists) => {
                            self.comp = Box::new(Loaded::new(
                                self.enabled,
                                simple_playlists,
                                ListState::default().with_selected(Some(0)),
                            ));
                        }
                        GetPlaylistsResponse::Failure(error) => {
                            self.comp = Box::new(Error::new(self.enabled, error));
                        }
                    }
                    Ok(None)
                } else {
                    self.comp.update(Action::FromQueryWorker(qw))
                }
            }
            _ => self.comp.update(action),
        }
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
