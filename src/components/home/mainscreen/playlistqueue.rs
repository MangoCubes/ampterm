mod empty;
mod error;
mod loaded;
mod loading;
mod notselected;

use crate::{
    action::Action,
    compid::CompID,
    components::{
        home::mainscreen::playlistqueue::{empty::Empty, loading::Loading},
        traits::{focusable::Focusable, fullcomp::FullComp, renderable::Renderable},
    },
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::GetPlaylistResponse, ResponseType},
    },
};
use color_eyre::Result;
use error::Error;
use loaded::Loaded;
use notselected::NotSelected;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::Block,
    Frame,
};

enum Comp {
    Error(Error),
    Loaded(Loaded),
    Loading(Loading, usize),
    NotSelected(NotSelected),
    Empty(Empty),
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

    fn gen_block(enabled: bool, title: String) -> Block<'static> {
        let style = if enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };
        let title = Span::styled(
            title,
            if enabled {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            },
        );
        Block::bordered().title(title).border_style(style)
    }
}

impl Renderable for PlaylistQueue {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.comp {
            Comp::Error(error) => error.draw(frame, area),
            Comp::Loaded(loaded) => loaded.draw(frame, area),
            Comp::Loading(loading, _) => loading.draw(frame, area),
            Comp::NotSelected(not_selected) => not_selected.draw(frame, area),
            Comp::Empty(centered) => centered.draw(frame, area),
        }
    }
}

impl FullComp for PlaylistQueue {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ToQueryWorker(qw) => {
                if qw.dest.contains(&CompID::PlaylistQueue) {
                    match qw.query {
                        HighLevelQuery::SelectPlaylist(params) => {
                            self.comp =
                                Comp::Loading(Loading::new(params.name, self.enabled), qw.ticket);
                            Ok(None)
                        }
                        HighLevelQuery::SetStar { media, star } => {
                            if let Comp::Loaded(loaded) = &mut self.comp {
                                Ok(loaded.set_star(media, star))
                            } else {
                                Ok(None)
                            }
                        }
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            Action::FromQueryWorker(qw) => {
                if let ResponseType::GetPlaylist(res) = qw.res {
                    match res {
                        GetPlaylistResponse::Success(full_playlist) => {
                            if let Comp::Loading(_, ticket) = self.comp {
                                if ticket == qw.ticket {
                                    self.comp = Comp::Loaded(Loaded::new(
                                        full_playlist.name.clone(),
                                        full_playlist,
                                        self.enabled,
                                    ));
                                }
                            }
                        }
                        GetPlaylistResponse::Failure { id, name, msg } => {
                            self.comp = Comp::Error(Error::new(id, name, msg, self.enabled));
                        }
                        GetPlaylistResponse::Partial(simple_playlist) => {
                            self.comp = Comp::Empty(Empty::new(simple_playlist.name, self.enabled))
                        }
                    }
                };
                Ok(None)
            }
            _ => match &mut self.comp {
                Comp::Loaded(comp) => comp.update(action),
                Comp::Error(comp) => comp.update(action),
                _ => Ok(None),
            },
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
                Comp::Loading(loading, _) => loading.set_enabled(enable),
                Comp::NotSelected(not_selected) => not_selected.set_enabled(enable),
                Comp::Empty(empty) => empty.set_enabled(enable),
            }
        };
    }
}
