mod error;
mod loaded;
mod loading;

use crate::{
    action::Action,
    components::{
        home::mainscreen::playlistlist::{error::Error, loaded::Loaded, loading::Loading},
        traits::{component::Component, focusable::Focusable},
    },
    queryworker::query::{getplaylists::GetPlaylistsResponse, ResponseType},
};
use color_eyre::Result;
use ratatui::{layout::Rect, widgets::ListState, Frame};

enum Comp {
    Error(Error),
    Loaded(Loaded),
    Loading(Loading),
}

pub struct PlaylistList {
    comp: Comp,
    enabled: bool,
}

impl PlaylistList {
    pub fn new(enabled: bool) -> Self {
        Self {
            comp: Comp::Loading(Loading::new(enabled)),
            enabled,
        }
    }
}

impl Component for PlaylistList {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.comp {
            Comp::Error(error) => error.draw(frame, area),
            Comp::Loaded(loaded) => loaded.draw(frame, area),
            Comp::Loading(loading) => loading.draw(frame, area),
        }
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FromQueryWorker(qw) => {
                if let ResponseType::GetPlaylists(res) = qw.res {
                    match res {
                        GetPlaylistsResponse::Success(simple_playlists) => {
                            self.comp = Comp::Loaded(Loaded::new(
                                self.enabled,
                                simple_playlists,
                                ListState::default().with_selected(Some(0)),
                            ));
                        }
                        GetPlaylistsResponse::Failure(error) => {
                            self.comp = Comp::Error(Error::new(self.enabled, error));
                        }
                    }
                    Ok(None)
                } else {
                    if let Comp::Loaded(comp) = &mut self.comp {
                        comp.update(Action::FromQueryWorker(qw))
                    } else {
                        Ok(None)
                    }
                }
            }
            _ => {
                if let Comp::Loaded(comp) = &mut self.comp {
                    comp.update(action)
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl Focusable for PlaylistList {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            match &mut self.comp {
                Comp::Error(error) => error.set_enabled(enable),
                Comp::Loaded(loaded) => loaded.set_enabled(enable),
                Comp::Loading(loading) => loading.set_enabled(enable),
            }
        };
    }
}
