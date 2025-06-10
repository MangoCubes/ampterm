mod error;
mod loaded;
mod loading;
mod notselected;

use crate::{
    action::{getplaylist::GetPlaylistResponse, Action},
    components::Component,
    focusable::Focusable,
    local_action,
    queryworker::query::Query,
};
use color_eyre::Result;
use error::Error;
use loaded::Loaded;
use loading::Loading;
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
            local_action!() => self.comp.update(action),
            Action::Query(q) => {
                if let Query::GetPlaylist { name, id } = q {
                    self.comp = Box::new(Loading::new(id.clone(), name.unwrap_or(id), self.enabled))
                };
                Ok(None)
            }
            Action::GetPlaylist(res) => match res {
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
