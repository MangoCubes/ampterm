mod loaded;

use crate::{
    action::{action::Action, localaction::PlaylistListAction},
    compid::CompID,
    components::{
        home::mainscreen::playlistlist::loaded::Loaded,
        lib::centered::Centered,
        traits::{
            focusable::Focusable,
            handlekeyseq::{ComponentKeyHelp, HandleKeySeq, KeySeqResult, PassKeySeq},
            handlequery::HandleQuery,
            renderable::Renderable,
        },
    },
    config::Config,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{QueryStatus, ResponseType, ToQueryWorker},
    },
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::Block,
    Frame,
};

enum Comp {
    Error(Centered),
    Loaded(Loaded),
    Loading(Centered),
}

pub struct PlaylistList {
    comp: Comp,
    enabled: bool,
    config: Config,
}

impl PlaylistList {
    pub fn new(config: Config, enabled: bool) -> (Self, Action) {
        let query = ToQueryWorker::new(HighLevelQuery::ListPlaylists);
        (
            Self {
                comp: Comp::Loading(Centered::new(vec!["Loading...".to_string()])),
                enabled,
                config,
            },
            Action::ToQuery(query),
        )
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
    fn handle_query(&mut self, dest: CompID, ticket: usize, res: QueryStatus) -> Option<Action> {
        if let QueryStatus::Finished(ResponseType::GetPlaylists(res)) = res {
            match res {
                Ok(simple_playlists) => {
                    if let Comp::Loaded(c) = &mut self.comp {
                        c.set_rows(&simple_playlists);
                    } else {
                        self.comp = Comp::Loaded(Loaded::new(
                            self.config.clone(),
                            simple_playlists.clone(),
                        ));
                    }
                }
                Err(error) => {
                    let mut msg = vec!["Error!".to_string(), error];
                    if let Some(keyseq) = self
                        .config
                        .local
                        .playlistlist
                        .find_action_str(PlaylistListAction::ViewSelected)
                    {
                        msg.push(format!("Reload with {}", keyseq));
                    }

                    self.comp = Comp::Error(Centered::new(msg));
                }
            }
            None
        } else {
            if let Comp::Loaded(comp) = &mut self.comp {
                comp.handle_query(dest, ticket, res)
            } else {
                None
            }
        }
    }
}

impl PassKeySeq for PlaylistList {
    fn get_help(&self) -> Vec<ComponentKeyHelp> {
        match &self.comp {
            Comp::Loaded(comp) => comp.get_help(),
            _ => vec![],
        }
    }
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
