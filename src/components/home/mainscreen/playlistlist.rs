mod error;
mod loaded;
mod loading;

use crate::{
    action::action::{Action, QueryAction},
    components::{
        home::mainscreen::playlistlist::{error::Error, loaded::Loaded, loading::Loading},
        traits::{
            focusable::Focusable,
            handlekeyseq::{HandleKeySeq, KeySeqResult, PassKeySeq},
            handlequery::HandleQuery,
            renderable::Renderable,
        },
    },
    config::Config,
    queryworker::query::{getplaylists::GetPlaylistsResponse, ResponseType},
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, ListState},
    Frame,
};

enum Comp {
    Error(Error),
    Loaded(Loaded),
    Loading(Loading),
}

pub struct PlaylistList {
    comp: Comp,
    enabled: bool,
    config: Config,
}

impl PlaylistList {
    pub fn new(config: Config, enabled: bool) -> Self {
        Self {
            comp: Comp::Loading(Loading::new()),
            enabled,
            config,
        }
    }
    fn gen_block(&self) -> Block<'static> {
        let style = if self.enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };
        let title = Span::styled(
            "Playlist".to_string(),
            if self.enabled {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            },
        );
        Block::bordered().title(title).border_style(style)
    }
}

impl Renderable for PlaylistList {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let block = self.gen_block();
        let inner = block.inner(area);
        frame.render_widget(block, area);
        match &mut self.comp {
            Comp::Error(error) => error.draw(frame, inner),
            Comp::Loaded(loaded) => loaded.draw(frame, inner),
            Comp::Loading(loading) => loading.draw(frame, inner),
        }
    }
}

impl HandleQuery for PlaylistList {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
        match &action {
            QueryAction::FromQueryWorker(qw) => {
                if let ResponseType::GetPlaylists(res) = &qw.res {
                    match res {
                        GetPlaylistsResponse::Success(simple_playlists) => {
                            self.comp = Comp::Loaded(Loaded::new(
                                self.config.clone(),
                                simple_playlists.clone(),
                                ListState::default().with_selected(Some(0)),
                            ));
                        }
                        GetPlaylistsResponse::Failure(error) => {
                            self.comp = Comp::Error(Error::new(error.clone()));
                        }
                    }
                    None
                } else {
                    if let Comp::Loaded(comp) = &mut self.comp {
                        comp.handle_query(action)
                    } else {
                        None
                    }
                }
            }
            _ => None,
        }
    }
}

impl PassKeySeq for PlaylistList {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        match &mut self.comp {
            Comp::Loaded(comp) => comp.handle_key_seq(keyseq),
            _ => None,
        }
    }
}

impl Focusable for PlaylistList {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
        };
    }
}
