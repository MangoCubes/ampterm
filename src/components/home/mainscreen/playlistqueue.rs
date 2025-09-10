mod error;
mod loaded;
mod loading;
mod notselected;

use crate::{
    action::Action,
    components::{
        home::mainscreen::playlistqueue::loading::Loading,
        traits::{component::Component, focusable::Focusable, synccomp::SyncComp},
    },
    queryworker::query::{getplaylist::GetPlaylistResponse, QueryType, ResponseType},
};
use color_eyre::Result;
use error::Error;
use loaded::Loaded;
use notselected::NotSelected;
use ratatui::{layout::Rect, Frame};

pub trait PlaylistQueueComps: Focusable {}

pub struct PlaylistQueue {
    comp: Box<dyn PlaylistQueueComps>,
    enabled: bool,
}

impl PlaylistQueue {
    pub fn new(enabled: bool) -> Self {
        Self {
            comp: Box::new(NotSelected::new(enabled)),
            enabled,
        }
    }
}

impl Component for PlaylistQueue {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.comp.draw(frame, area)
    }
}

impl SyncComp for PlaylistQueue {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ToQueryWorker(qw) => {
                if let QueryType::GetPlaylist { id, name } = qw.query {
                    self.comp = Box::new(Loading::new(id, name, self.enabled));
                }
                Ok(None)
            }
            Action::FromQueryWorker(qw) => {
                if let ResponseType::GetPlaylist(res) = qw.res {
                    match res {
                        GetPlaylistResponse::Success(full_playlist) => {
                            self.comp = Box::new(Loaded::new(
                                full_playlist.name.clone(),
                                full_playlist,
                                self.enabled,
                            ));
                        }
                        GetPlaylistResponse::Failure { id, name, msg } => {
                            self.comp = Box::new(Error::new(id, name, msg, self.enabled));
                        }
                    }
                };
                Ok(None)
            }
            _ => self.comp.update(action),
        }
    }
}

impl Focusable for PlaylistQueue {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            self.comp.set_enabled(enable);
        };
    }
}
