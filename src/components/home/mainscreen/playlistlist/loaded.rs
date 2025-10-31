use std::collections::HashMap;

use crate::{
    action::{
        useraction::{Common, Global, Normal, UserAction},
        Action,
    },
    components::traits::component::Component,
    config::Config,
    osclient::response::getplaylists::SimplePlaylist,
    playerworker::player::{QueueLocation, ToPlayerWorker},
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{
            getplaylist::{GetPlaylistParams, GetPlaylistResponse},
            getplaylists::PlaylistID,
            ResponseType, ToQueryWorker,
        },
    },
};
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{List, ListState},
    Frame,
};
use tracing::error;

pub struct Loaded {
    config: Config,
    comp: List<'static>,
    list: Vec<SimplePlaylist>,
    state: ListState,
    callback: HashMap<usize, (PlaylistID, QueueLocation)>,
}

impl Loaded {
    fn select_playlist(&self) -> Option<Action> {
        if let Some(pos) = self.state.selected() {
            let key = self.list[pos].id.clone();
            let name = self.list[pos].name.clone();
            if self.config.config.auto_focus {
                Some(Action::Multiple(vec![
                    Action::ToQueryWorker(ToQueryWorker::new(HighLevelQuery::SelectPlaylist(
                        GetPlaylistParams { name, id: key },
                    ))),
                    Action::User(UserAction::Global(Global::FocusPlaylistQueue)),
                ]))
            } else {
                Some(Action::ToQueryWorker(ToQueryWorker::new(
                    HighLevelQuery::SelectPlaylist(GetPlaylistParams { name, id: key }),
                )))
            }
        } else {
            None
        }
    }

    /// This needs to be a function not tied to &self because it needs to be used by [`Self::new`]
    fn gen_list(list: &Vec<SimplePlaylist>) -> List<'static> {
        let items: Vec<String> = list.iter().map(|p| p.name.clone()).collect();
        List::new(items)
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }

    pub fn new(config: Config, list: Vec<SimplePlaylist>, state: ListState) -> Self {
        Self {
            comp: Self::gen_list(&list),
            list,
            state,
            callback: HashMap::new(),
            config,
        }
    }
    pub fn add_to_queue(&mut self, ql: QueueLocation) -> Option<Action> {
        if let Some(pos) = self.state.selected() {
            let key = self.list[pos].id.clone();
            let name = self.list[pos].name.clone();
            let req = ToQueryWorker::new(HighLevelQuery::AddPlaylistToQueue(GetPlaylistParams {
                name,
                id: key.clone(),
            }));
            self.callback.insert(req.ticket, (key, ql));
            Some(Action::ToQueryWorker(req))
        } else {
            error!("Failed to add playlist to queue: No playlist selected");
            None
        }
    }
}

impl Component for Loaded {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FromQueryWorker(res) => {
                if let Some(cb) = self.callback.remove(&res.ticket) {
                    if let ResponseType::GetPlaylist(res) = res.res {
                        match res {
                            GetPlaylistResponse::Success(full_playlist) => {
                                return Ok(Some(Action::ToPlayerWorker(
                                    ToPlayerWorker::AddToQueue {
                                        music: full_playlist.entry,
                                        pos: cb.1,
                                    },
                                )));
                            }
                            GetPlaylistResponse::Failure {
                                id: _,
                                name: _,
                                msg,
                            } => {
                                error!("Failed to add playlist to queue: {msg}");
                            }
                            // This implies that the returned playlist is empty
                            GetPlaylistResponse::Partial(_simple_playlist) => return Ok(None),
                        }
                    }
                }
                Ok(None)
            }
            Action::User(UserAction::Common(local)) => {
                match local {
                    Common::Up => {
                        self.state.select_previous();
                        Ok(None)
                    }
                    Common::Down => {
                        self.state.select_next();
                        Ok(None)
                    }
                    Common::Confirm => Ok(self.select_playlist()),
                    Common::Top => {
                        self.state.select_first();
                        Ok(None)
                    }
                    Common::Bottom => {
                        self.state.select_last();
                        Ok(None)
                    }
                    // TODO: Add horizontal text scrolling
                    _ => Ok(None),
                }
            }
            Action::User(UserAction::Normal(normal)) => {
                if let Normal::Add(pos) = normal {
                    Ok(self.add_to_queue(pos))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
