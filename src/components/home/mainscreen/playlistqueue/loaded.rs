use crate::{
    action::action::{Action, QueryAction, QueueAction, TargetedAction},
    components::{
        home::mainscreen::playlistqueue::PlaylistQueue,
        lib::visualtable::{VisualSelection, VisualTable},
        traits::{
            focusable::Focusable,
            handlekeyseq::{HandleKeySeq, KeySeqResult},
            renderable::Renderable,
        },
    },
    config::{keybindings::KeyBindings, localkeybinds::PlaylistQueueAction, Config},
    osclient::response::getplaylist::{FullPlaylist, Media},
    playerworker::player::QueueLocation,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{
            getplaylist::{GetPlaylistParams, MediaID},
            ToQueryWorker,
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

pub struct Loaded {
    name: String,
    playlist: FullPlaylist,
    enabled: bool,
    table: VisualTable,
}

impl Loaded {
    /// Adds selected items into the queue, resetting the current selection.
    fn add_selection_to_queue(&mut self, playpos: QueueLocation) -> Option<Action> {
        let (selection, action) = self.table.get_selection_reset();
        let first = match selection {
            VisualSelection::Single(index) => Some(Action::Targeted(TargetedAction::Queue(
                QueueAction::Add(vec![self.playlist.entry[index].clone()], playpos),
            ))),
            VisualSelection::TempSelection(start, end) => {
                Some(Action::Targeted(TargetedAction::Queue(QueueAction::Add(
                    self.playlist.entry[start..=end].to_vec(),
                    playpos,
                ))))
            }
            VisualSelection::Selection(items) => {
                let items: Vec<Media> = items
                    .into_iter()
                    .enumerate()
                    .filter(|(_, selected)| *selected)
                    .filter_map(|(idx, _)| self.playlist.entry.get(idx))
                    .map(|m| m.clone())
                    .collect();
                Some(Action::Targeted(TargetedAction::Queue(QueueAction::Add(
                    items, playpos,
                ))))
            }
            VisualSelection::None { unselect: _ } => None,
        };
        if let Some(a) = first {
            if let Some(b) = action {
                Some(Action::Multiple(vec![a, b]))
            } else {
                Some(a)
            }
        } else {
            if let Some(b) = action {
                Some(b)
            } else {
                None
            }
        }
    }

    /// Generate rows so that they can be used by the table component
    pub fn gen_rows(items: &Vec<Media>) -> Vec<Row<'static>> {
        items
            .iter()
            .map(|item| {
                Row::new(vec![
                    item.artist.clone().unwrap_or("Unknown".to_string()),
                    item.title.clone(),
                    item.get_fav_marker(),
                ])
            })
            .collect()
    }

    pub fn new(config: Config, name: String, list: FullPlaylist, enabled: bool) -> Self {
        fn table_proc(table: Table<'static>) -> Table<'static> {
            table
                .highlight_symbol(">")
                .row_highlight_style(Style::new().reversed())
        }
        let rows = Self::gen_rows(&list.entry);
        let table = VisualTable::new(
            config,
            rows,
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(2, 3),
                Constraint::Min(1),
            ]
            .to_vec(),
            table_proc,
        );
        let mut tablestate = TableState::default();
        tablestate.select(Some(0));

        Self {
            name,
            enabled,
            playlist: list,
            table,
        }
    }

    pub fn set_star(&mut self, media: MediaID, star: bool) -> Option<Action> {
        let updated = self
            .playlist
            .entry
            .clone()
            .into_iter()
            .map(|mut m| {
                if m.id == media {
                    m.starred = if star {
                        Some("Starred".to_string())
                    } else {
                        None
                    };
                }
                m
            })
            .collect();
        self.playlist.entry = updated;
        let rows = Self::gen_rows(&self.playlist.entry);
        self.table.set_rows(rows);
        None
    }
}

impl Renderable for Loaded {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let title = if let Some(pos) = self.table.get_current() {
            let len = self.playlist.entry.len();
            format!(
                "{} ({}/{})",
                self.name,
                if pos == usize::MAX || pos >= len {
                    len
                } else {
                    pos + 1
                },
                len
            )
        } else {
            format!("{} ({})", self.name, self.playlist.entry.len())
        };
        let border = PlaylistQueue::gen_block(self.enabled, title);
        let inner = border.inner(area);
        frame.render_widget(border, area);
        self.table.draw(frame, inner)
    }
}

impl HandleKeySeq<PlaylistQueueAction> for Loaded {
    fn pass_to_lower_comp(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        self.table.handle_key_seq(keyseq)
    }

    fn handle_local_action(&mut self, action: PlaylistQueueAction) -> KeySeqResult {
        match action {
            PlaylistQueueAction::ToggleStar => {
                let (selection, action) = self.table.get_selection_reset();
                let mut items: Vec<Action> = match selection {
                    VisualSelection::Single(idx) => {
                        let item = self.playlist.entry[idx].clone();
                        vec![(item.id, item.starred == None)]
                    }
                    VisualSelection::TempSelection(start, end) => self.playlist.entry[start..=end]
                        .iter()
                        .map(|m| (m.id.clone(), m.starred == None))
                        .collect(),
                    VisualSelection::Selection(items) => self
                        .playlist
                        .entry
                        .iter()
                        .zip(items.iter())
                        .filter_map(|(m, &selected)| {
                            if selected {
                                Some((m.id.clone(), m.starred == None))
                            } else {
                                None
                            }
                        })
                        .collect(),
                    VisualSelection::None { unselect: _ } => vec![],
                }
                .into_iter()
                .map(|(id, star)| {
                    Action::Query(QueryAction::ToQueryWorker(ToQueryWorker::new(
                        HighLevelQuery::SetStar { media: id, star },
                    )))
                })
                .collect();

                if let Some(a) = action {
                    items.push(a);
                }

                KeySeqResult::ActionNeeded(Action::Multiple(items))
            }
            PlaylistQueueAction::Add(pos) => match self.add_selection_to_queue(pos) {
                Some(a) => KeySeqResult::ActionNeeded(a),
                None => KeySeqResult::NoActionNeeded,
            },
            PlaylistQueueAction::Refresh => {
                KeySeqResult::ActionNeeded(Action::Query(QueryAction::ToQueryWorker(
                    ToQueryWorker::new(HighLevelQuery::SelectPlaylist(GetPlaylistParams {
                        name: self.name.to_string(),
                        id: self.playlist.id.clone(),
                    })),
                )))
            }
        }
    }

    fn get_keybinds(&self) -> &KeyBindings<PlaylistQueueAction> {
        todo!()
    }
}

impl Focusable for Loaded {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            if !self.enabled {
                self.table.disable_visual_discard();
            }
        };
    }
}
