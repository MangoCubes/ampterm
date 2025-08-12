mod error;
mod loaded;
mod loading;
mod notselected;

use crate::{
    action::{getplaylist::GetPlaylistResponse, Action, FromQueryWorker},
    components::{home::mainscreen::playlistqueue::loading::Loading, Component},
    focusable::Focusable,
    queryworker::query::ToQueryWorker,
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
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Local(_) => self.comp.update(action),
            Action::ToQueryWorker(ToQueryWorker::GetPlaylist { id, name }) => {
                self.comp = Box::new(Loading::new(
                    id,
                    name.unwrap_or("Playlist Queue".to_string()),
                    self.enabled,
                ));
                Ok(None)
            }
            Action::FromQueryWorker(FromQueryWorker::GetPlaylist(res)) => match res {
                GetPlaylistResponse::Success(full_playlist) => {
                    self.comp = Box::new(Loaded::new(
                        full_playlist.name.clone(),
                        full_playlist,
                        self.enabled,
                    ));
                    Ok(None)
                }
                GetPlaylistResponse::Failure { id, name, msg } => {
                    self.comp = Box::new(Error::new(
                        id,
                        name.unwrap_or("Playlist Queue".to_string()),
                        msg,
                        self.enabled,
                    ));
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

impl Focusable for PlaylistQueue {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            self.comp.set_enabled(enable);
        };
    }
}
