use std::collections::HashMap;

use crate::{
    action::action::{Action, QueryAction, QueueAction, TargetedAction},
    components::traits::{
        handlekeyseq::{HandleKeySeq, KeySeqResult},
        handlequery::HandleQuery,
        renderable::Renderable,
    },
    config::{keybindings::KeyBindings, localkeybinds::PlaylistListAction, Config},
    osclient::response::getplaylists::SimplePlaylist,
    playerworker::player::QueueLocation,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{
            getplaylist::{GetPlaylistParams, GetPlaylistResponse},
            getplaylists::PlaylistID,
            ResponseType, ToQueryWorker,
        },
    },
};
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
            if self.config.behaviour.auto_focus {
                Some(Action::Multiple(vec![
                    Action::ToQueryWorker(ToQueryWorker::new(HighLevelQuery::SelectPlaylist(
                        GetPlaylistParams { name, id: key },
                    ))),
                    Action::Targeted(TargetedAction::FocusPlaylistQueue),
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

impl Renderable for Loaded {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
    }
}

impl HandleQuery for Loaded {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
        match action {
            QueryAction::FromQueryWorker(res) => {
                if let Some(cb) = self.callback.remove(&res.ticket) {
                    if let ResponseType::GetPlaylist(res) = res.res {
                        match res {
                            GetPlaylistResponse::Success(full_playlist) => {
                                return Some(Action::Targeted(TargetedAction::Queue(
                                    QueueAction::Add(full_playlist.entry, cb.1),
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
                            GetPlaylistResponse::Partial(_simple_playlist) => return None,
                        }
                    }
                }
            }
            _ => {}
        };
        None
    }
}

impl HandleKeySeq<PlaylistListAction> for Loaded {
    fn handle_local_action(&mut self, action: PlaylistListAction) -> KeySeqResult {
        match action {
            PlaylistListAction::Add(pos) => match self.add_to_queue(pos) {
                Some(a) => KeySeqResult::ActionNeeded(a),
                None => KeySeqResult::NoActionNeeded,
            },
            PlaylistListAction::Up => {
                self.state.select_previous();
                KeySeqResult::NoActionNeeded
            }
            PlaylistListAction::Down => {
                self.state.select_next();
                KeySeqResult::NoActionNeeded
            }
            PlaylistListAction::ViewSelected => match self.select_playlist() {
                Some(a) => KeySeqResult::ActionNeeded(a),
                None => KeySeqResult::NoActionNeeded,
            },
            PlaylistListAction::Top => {
                self.state.select_first();
                KeySeqResult::NoActionNeeded
            }
            PlaylistListAction::Bottom => {
                self.state.select_last();
                KeySeqResult::NoActionNeeded
            }
        }
    }

    fn get_keybinds(&self) -> &KeyBindings<PlaylistListAction> {
        &self.config.local.playlistlist
    }
}
