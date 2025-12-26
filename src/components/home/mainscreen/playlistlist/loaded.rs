use std::collections::HashMap;

use crate::{
    action::{
        action::{Action, QueueAction, TargetedAction},
        localaction::PlaylistListAction,
    },
    compid::CompID,
    components::{
        lib::visualtable::VisualTable,
        traits::{
            handlekeyseq::{HandleKeySeq, KeySeqResult},
            handlequery::HandleQuery,
            renderable::Renderable,
        },
    },
    config::{keybindings::KeyBindings, Config},
    osclient::{response::getplaylists::SimplePlaylist, types::PlaylistID},
    playerworker::player::QueueLocation,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{
            getplaylist::{GetPlaylistParams, GetPlaylistResponse},
            QueryStatus, ResponseType, ToQueryWorker,
        },
    },
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Row, Table, TableState},
    Frame,
};
use tracing::error;

pub struct Loaded {
    autofocus: bool,
    table: VisualTable,
    keymap: KeyBindings<PlaylistListAction>,
    list: Vec<SimplePlaylist>,
    callback: HashMap<usize, (PlaylistID, QueueLocation, bool)>,
}

impl Loaded {
    fn select_playlist(&self) -> Option<Action> {
        if let Some(pos) = self.table.get_current() {
            let key = self.list[pos].id.clone();
            let name = self.list[pos].name.clone();
            if self.autofocus {
                Some(Action::Multiple(vec![
                    Action::ToQuery(ToQueryWorker::new(HighLevelQuery::SelectPlaylist(
                        GetPlaylistParams { name, id: key },
                    ))),
                    Action::Targeted(TargetedAction::FocusPlaylistQueue),
                ]))
            } else {
                Some(Action::ToQuery(ToQueryWorker::new(
                    HighLevelQuery::SelectPlaylist(GetPlaylistParams { name, id: key }),
                )))
            }
        } else {
            None
        }
    }

    /// This needs to be a function not tied to &self because it needs to be used by [`Self::new`]
    pub fn gen_rows(items: &Vec<SimplePlaylist>) -> Vec<Row<'static>> {
        items
            .iter()
            .map(|item| Row::new(vec![item.name.clone(), item.song_count.to_string()]))
            .collect()
    }

    pub fn new(config: Config, list: Vec<SimplePlaylist>) -> Self {
        fn table_proc(table: Table<'static>) -> Table<'static> {
            table
                .highlight_symbol(">")
                .row_highlight_style(Style::new().reversed())
        }
        let rows = Self::gen_rows(&list);
        let table = VisualTable::new(
            config.clone(),
            rows,
            [Constraint::Fill(1), Constraint::Max(6)].to_vec(),
            table_proc,
        );
        let mut tablestate = TableState::default();
        tablestate.select(Some(0));
        Self {
            list,
            table,
            callback: HashMap::new(),
            autofocus: config.behaviour.auto_focus,
            keymap: config.local.playlistlist.clone(),
        }
    }
    pub fn add_to_queue(&mut self, ql: QueueLocation, randomise: bool) -> Option<Action> {
        if let Some(pos) = self.table.get_current() {
            let key = self.list[pos].id.clone();
            let name = self.list[pos].name.clone();
            let req = ToQueryWorker::new(HighLevelQuery::AddPlaylistToQueue(GetPlaylistParams {
                name,
                id: key.clone(),
            }));
            self.callback.insert(req.ticket, (key, ql, randomise));
            Some(Action::ToQuery(req))
        } else {
            None
        }
    }
}

impl Renderable for Loaded {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.table.draw(frame, area);
    }
}

impl HandleQuery for Loaded {
    fn handle_query(&mut self, _dest: CompID, ticket: usize, res: QueryStatus) -> Option<Action> {
        if let QueryStatus::Finished(ResponseType::GetPlaylist(res)) = res {
            if let Some(cb) = self.callback.remove(&ticket) {
                match res {
                    GetPlaylistResponse::Success(full_playlist) => {
                        return Some(Action::Targeted(TargetedAction::Queue(if cb.2 {
                            QueueAction::RandomAdd(full_playlist.entry, cb.1)
                        } else {
                            QueueAction::Add(full_playlist.entry, cb.1)
                        })));
                    }
                    GetPlaylistResponse::Failure {
                        id: _,
                        name: _,
                        msg,
                    } => {
                        error!("Failed to add playlist to queue: {msg}");
                    }
                    // This implies that the returned playlist is empty
                    GetPlaylistResponse::Partial(_) => return None,
                }
            }
        };
        None
    }
}

impl HandleKeySeq<PlaylistListAction> for Loaded {
    fn get_name(&self) -> &str {
        "PlaylistList"
    }
    fn pass_to_lower_comp(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        self.table.handle_key_seq(keyseq)
    }
    fn handle_local_action(&mut self, action: PlaylistListAction) -> KeySeqResult {
        match action {
            PlaylistListAction::Add(pos) => match self.add_to_queue(pos, false) {
                Some(a) => KeySeqResult::ActionNeeded(a),
                None => KeySeqResult::NoActionNeeded,
            },
            PlaylistListAction::ViewSelected => match self.select_playlist() {
                Some(a) => KeySeqResult::ActionNeeded(a),
                None => KeySeqResult::NoActionNeeded,
            },
            PlaylistListAction::RandomAdd(pos) => match self.add_to_queue(pos, true) {
                Some(a) => KeySeqResult::ActionNeeded(a),
                None => KeySeqResult::NoActionNeeded,
            },
        }
    }

    fn get_keybinds(&self) -> &KeyBindings<PlaylistListAction> {
        &self.keymap
    }
}
