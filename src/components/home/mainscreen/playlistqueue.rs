mod error;
mod loaded;
mod loading;
mod notselected;
use std::collections::HashSet;

use crate::{
    action::{
        getplaylist::{GetPlaylistResponse, MediaID},
        Action,
    },
    components::Component,
    focusable::Focusable,
    local_action,
    playerworker::player::QueueLocation,
    queryworker::query::Query,
    visualmode::VisualMode,
};
use color_eyre::Result;
use error::Error;
use loaded::Loaded;
use loading::Loading;
use notselected::NotSelected;
use ratatui::{
    layout::{Alignment, Rect},
    text::Line,
    widgets::{ListState, Padding, Paragraph, Wrap},
    Frame,
};

enum CompState {
    Loaded(Box<dyn Focusable>),
    Loading(Box<dyn Focusable>),
    Error(Box<dyn Focusable>),
    NotSelected(Box<dyn Focusable>),
}

pub struct PlaylistQueue {
    state: CompState,
    enabled: bool,
}

impl PlaylistQueue {
    pub fn new(enabled: bool) -> Self {
        Self {
            state: CompState::NotSelected(Box::new(NotSelected::new(enabled))),
            enabled,
        }
    }
}

// impl Component for PlaylistQueue {
//     fn update(&mut self, action: Action) -> Result<Option<Action>> {
//     }
// }
impl Component for PlaylistQueue {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            local_action!() => match &mut self.state {
                CompState::Loaded(c)
                | CompState::Loading(c)
                | CompState::Error(c)
                | CompState::NotSelected(c) => c.update(action),
            },
            Action::Query(q) => {
                if let Query::GetPlaylist { name, id } = q {
                    self.state = CompState::Loading(Box::new(Loading::new(
                        id.clone(),
                        name.unwrap_or(id),
                        self.enabled,
                    )))
                };
                Ok(None)
            }
            Action::GetPlaylist(res) => match res {
                GetPlaylistResponse::Success(full_playlist) => {
                    self.state = CompState::Loaded(Box::new(Loaded::new(
                        full_playlist.name.clone(),
                        full_playlist,
                        ListState::default().with_selected(Some(0)),
                        None,
                        None,
                        self.enabled,
                    )));
                    Ok(None)
                }
                GetPlaylistResponse::Failure { id, name, msg } => {
                    self.state = CompState::Error(Box::new(Error::new(
                        id,
                        name.unwrap_or("Playlist Queue".to_string()),
                        msg,
                        self.enabled,
                    )));
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.state {
            CompState::Loaded(c)
            | CompState::Loading(c)
            | CompState::Error(c)
            | CompState::NotSelected(c) => c.draw(frame, area),
        }
    }
}

impl Focusable for PlaylistQueue {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            match &mut self.state {
                CompState::Loaded(c)
                | CompState::Loading(c)
                | CompState::Error(c)
                | CompState::NotSelected(c) => c.set_enabled(enable),
            };
        };
    }
}
