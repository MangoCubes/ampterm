mod loaded;

use crate::{
    action::{action::Action, localaction::PlaylistListAction},
    compid::CompID,
    components::{
        home::mainscreen::leftpanel::playlistlist::loaded::Loaded,
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
use ratatui::{layout::Rect, Frame};

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
}

impl Renderable for PlaylistList {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match &mut self.comp {
            Comp::Error(error) => error.draw(frame, area),
            Comp::Loaded(loaded) => loaded.draw(frame, area),
            Comp::Loading(loading) => loading.draw(frame, area),
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
                        .find_action_str(PlaylistListAction::Refresh)
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
