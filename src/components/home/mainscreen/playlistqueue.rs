mod error;
mod loaded;
mod loading;
mod notselected;

use crate::{
    action::Action,
    components::{
        home::mainscreen::playlistqueue::loading::Loading,
        traits::{
            asynccomp::AsyncComp, component::Component, focusable::Focusable, synccomp::SyncComp,
        },
    },
    queryworker::query::{getplaylist::GetPlaylistResponse, QueryType, ResponseType},
};
use color_eyre::Result;
use error::Error;
use loaded::Loaded;
use notselected::NotSelected;
use ratatui::{layout::Rect, Frame};

enum Comp {
    Error(Error),
    Loaded(Loaded<'static>),
    Loading(Loading),
    NotSelected(NotSelected),
}

pub struct PlaylistQueue {
    comp: Comp,
    enabled: bool,
}

impl PlaylistQueue {
    pub fn new(enabled: bool) -> Self {
        Self {
            comp: Comp::NotSelected(NotSelected::new(enabled)),
            enabled,
        }
    }
}

impl Component for PlaylistQueue {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.comp {
            Comp::Error(error) => error.draw(frame, area),
            Comp::Loaded(loaded) => loaded.draw(frame, area),
            Comp::Loading(loading) => loading.draw(frame, area),
            Comp::NotSelected(not_selected) => not_selected.draw(frame, area),
        }
    }
}

impl AsyncComp for PlaylistQueue {
    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ToQueryWorker(qw) => {
                if let QueryType::GetPlaylist { id, name } = qw.query {
                    self.comp = Comp::Loading(Loading::new(id, name, self.enabled));
                }
                Ok(None)
            }
            Action::FromQueryWorker(qw) => {
                if let ResponseType::GetPlaylist(res) = qw.res {
                    match res {
                        GetPlaylistResponse::Success(full_playlist) => {
                            self.comp = Comp::Loaded(Loaded::new(
                                full_playlist.name.clone(),
                                full_playlist,
                                self.enabled,
                            ));
                        }
                        GetPlaylistResponse::Failure { id, name, msg } => {
                            self.comp = Comp::Error(Error::new(id, name, msg, self.enabled));
                        }
                    }
                };
                Ok(None)
            }
            _ => {
                if let Comp::Loaded(comp) = &mut self.comp {
                    comp.update(action).await
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl Focusable for PlaylistQueue {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            match &mut self.comp {
                Comp::Error(error) => error.set_enabled(enable),
                Comp::Loaded(loaded) => loaded.set_enabled(enable),
                Comp::Loading(loading) => loading.set_enabled(enable),
                Comp::NotSelected(not_selected) => not_selected.set_enabled(enable),
            }
        };
    }
}
