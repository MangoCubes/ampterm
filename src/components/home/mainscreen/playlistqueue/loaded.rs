use crate::{
    action::{
        action::{Action, Mode, QueueAction, TargetedAction},
        localaction::PlaylistQueueAction,
    },
    components::{
        home::mainscreen::playlistqueue::PlaylistQueue,
        lib::{
            scrollbar::ScrollBar,
            visualtable::{VisualSelection, VisualTable},
        },
        traits::{
            focusable::Focusable,
            handlefilter::HandleFilter,
            handlekeyseq::{ComponentKeyHelp, HandleKeySeq, KeySeqResult},
            handlesearch::HandleSearch,
            renderable::Renderable,
        },
    },
    config::{keybindings::KeyBindings, Config},
    helper::selection::Selection,
    osclient::{
        response::getplaylist::{FullPlaylist, Media},
        types::MediaID,
    },
    playerworker::player::QueueLocation,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::GetPlaylistParams, ToQueryWorker},
    },
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Row, Table, TableState},
    Frame,
};

pub struct Loaded {
    name: String,
    playlist: FullPlaylist,
    enabled: bool,
    table: VisualTable,
    keymap: KeyBindings<PlaylistQueueAction>,
    filter: Option<(usize, String)>,
    search: Option<(usize, String)>,
    bar: ScrollBar,
    orig_location: usize,
}

impl Loaded {
    /// Adds selected items into the queue, resetting the current selection.
    fn add_selection_to_queue(
        &mut self,
        playpos: QueueLocation,
        randomise: bool,
    ) -> Option<Action> {
        let (selection, action) = self.table.get_selection_reset();
        let first = match selection {
            VisualSelection::Single(index) => Some(Action::Targeted(TargetedAction::Queue({
                let items = vec![self.playlist.entry[index].clone()];
                if randomise {
                    QueueAction::RandomAdd(items, playpos)
                } else {
                    QueueAction::Add(items, playpos)
                }
            }))),
            VisualSelection::Multiple { map, temp: _ } => {
                let items = self
                    .playlist
                    .entry
                    .iter()
                    .zip(map)
                    .filter(|(_, selected)| *selected)
                    .map(|(m, _)| m.clone())
                    .collect();
                Some(Action::Targeted(TargetedAction::Queue(QueueAction::Add(
                    items, playpos,
                ))))
            }
            VisualSelection::None => None,
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
                    if let Some(len) = item.duration {
                        format!("{:02}:{:02}", len / 60, len % 60)
                    } else {
                        "".to_string()
                    },
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
            config.clone(),
            rows,
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(2, 3),
                Constraint::Length(5),
                Constraint::Length(2),
            ]
            .to_vec(),
            table_proc,
        );
        let mut tablestate = TableState::default();
        tablestate.select(Some(0));

        Self {
            bar: ScrollBar::new(list.entry.len() as u32, 0),
            keymap: config.local.playlistqueue,
            name,
            enabled,
            playlist: list,
            table,
            filter: None,
            search: None,
            orig_location: 0,
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
            let (len, extra) = match &self.filter {
                Some((len, filter)) => (*len, format!(" (Filter: {})", filter)),
                None => (self.playlist.entry.len(), "".to_string()),
            };
            format!(
                "{} ({}/{}){}",
                self.name,
                if pos == usize::MAX || pos >= len {
                    len
                } else {
                    pos + 1
                },
                len,
                extra
            )
        } else {
            let (len, extra) = match &self.filter {
                Some((len, filter)) => (*len, format!(" (Filter: {})", filter)),
                None => (self.playlist.entry.len(), "".to_string()),
            };
            format!("{} ({}){}", self.name, len, extra)
        };
        let border = PlaylistQueue::gen_block(self.enabled, title);
        let inner = border.inner(area);
        frame.render_widget(border, area);
        let [list, bar] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(inner);
        self.table.draw(frame, list);
        self.bar.draw(frame, bar);
    }
}

impl HandleKeySeq<PlaylistQueueAction> for Loaded {
    fn get_other_helps(&self) -> Vec<ComponentKeyHelp> {
        self.table.get_help()
    }
    fn get_name(&self) -> &str {
        "PlaylistQueue"
    }
    fn pass_to_lower_comp(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        let res = self.table.handle_key_seq(keyseq);
        self.bar
            .update_pos(self.table.get_current().unwrap_or(0) as u32);
        res
    }

    fn handle_local_action(&mut self, action: PlaylistQueueAction) -> KeySeqResult {
        match action {
            PlaylistQueueAction::ViewInfo => {
                if let Some(pos) = self.table.get_current() {
                    KeySeqResult::ActionNeeded(Action::Targeted(TargetedAction::ViewMediaInfo(
                        self.playlist.entry[pos].clone(),
                    )))
                } else {
                    KeySeqResult::NoActionNeeded
                }
            }
            PlaylistQueueAction::ToggleStar => {
                let (selection, action) = self.table.get_selection_reset();
                let mut items: Vec<Action> = match selection {
                    VisualSelection::Single(idx) => {
                        let item = self.playlist.entry[idx].clone();
                        vec![(item.id, item.starred == None)]
                    }
                    VisualSelection::Multiple { map, temp: _ } => self
                        .playlist
                        .entry
                        .iter()
                        .zip(map)
                        .filter_map(|(m, selected)| {
                            if selected {
                                Some((m.id.clone(), m.starred == None))
                            } else {
                                None
                            }
                        })
                        .collect(),
                    VisualSelection::None => vec![],
                }
                .into_iter()
                .map(|(id, star)| {
                    Action::ToQuery(ToQueryWorker::new(HighLevelQuery::SetStar {
                        media: id,
                        star,
                    }))
                })
                .collect();

                if let Some(a) = action {
                    items.push(a);
                }

                KeySeqResult::ActionNeeded(Action::Multiple(items))
            }
            PlaylistQueueAction::Add(pos) => match self.add_selection_to_queue(pos, false) {
                Some(a) => KeySeqResult::ActionNeeded(a),
                None => KeySeqResult::NoActionNeeded,
            },
            PlaylistQueueAction::Refresh => {
                self.table.bump_cursor_pos();
                KeySeqResult::ActionNeeded(Action::ToQuery(ToQueryWorker::new(
                    HighLevelQuery::SelectPlaylist(GetPlaylistParams {
                        name: self.name.to_string(),
                        id: self.playlist.id.clone(),
                    }),
                )))
            }
            PlaylistQueueAction::RandomAdd(pos) => match self.add_selection_to_queue(pos, true) {
                Some(a) => KeySeqResult::ActionNeeded(a),
                None => KeySeqResult::NoActionNeeded,
            },
            PlaylistQueueAction::AddToPlaylist => {
                let (vs, action) = self.table.get_selection_reset();

                let selection = match vs {
                    VisualSelection::Single(index) => Selection::Single(index),
                    VisualSelection::Multiple { map, temp: _ } => Selection::Multiple(map),
                    VisualSelection::None => {
                        return match action {
                            Some(a) => KeySeqResult::ActionNeeded(a),
                            None => KeySeqResult::NoActionNeeded,
                        }
                    }
                };

                let ids = match selection {
                    Selection::Single(i) => vec![self.playlist.entry[i].id.clone()],
                    Selection::Multiple(items) => self
                        .playlist
                        .entry
                        .iter()
                        .zip(items)
                        .filter(|(_, bool)| *bool)
                        .map(|(m, _)| m.id.clone())
                        .collect(),
                };
                let request_popup = Action::Targeted(TargetedAction::PrepareAddToPlaylist(ids));

                let actions = if let Some(a) = action {
                    Action::Multiple(vec![a, request_popup])
                } else {
                    request_popup
                };

                KeySeqResult::ActionNeeded(actions)
            }
        }
    }

    fn get_keybinds(&self) -> &KeyBindings<PlaylistQueueAction> {
        &self.keymap
    }
}

impl Focusable for Loaded {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            if enable {
                self.table.bump_cursor_pos();
            } else {
                self.table.disable_visual_discard();
            }
        };
    }
}

impl HandleSearch for Loaded {
    fn init_search(&mut self) -> bool {
        self.orig_location = self.table.get_current().unwrap_or(0);
        true
    }

    // PlaylistQueueAction::Filter => {
    //     self.state = State::Filtering(Filter::new());
    //     KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Insert))
    // }
    // PlaylistQueueAction::ClearFilter => KeySeqResult::ActionNeeded(self.reset_filter()),
    // PlaylistQueueAction::Search => {
    //     self.state = State::Searching(
    //         Search::new(if let Some((_, search)) = &self.search {
    //             search.clone()
    //         } else {
    //             "".to_string()
    //         }),
    //         self.table.get_current().unwrap_or(0),
    //     );
    //     KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Insert))
    // }
    // PlaylistQueueAction::ClearSearch => KeySeqResult::ActionNeeded(self.clear_search()),
    fn test_search(&mut self, search: String) {
        if search.len() == 0 {
            self.search = Some((0, search));
            self.table.set_position(self.orig_location);
            self.table.reset_highlight();
        } else {
            let mut count = 0;
            let highlight: Vec<bool> = self
                .playlist
                .entry
                .iter()
                .map(|i| {
                    let a = i.title.to_lowercase().contains(&search.to_lowercase());
                    if a {
                        count += 1;
                    }
                    a
                })
                .collect();
            if let Some(idx) = highlight.iter().position(|x| *x) {
                self.table.set_position(idx);
            };
            self.search = Some((count, search));
            self.table.set_highlight(&highlight);
        }
    }

    fn revert_search(&mut self) {
        self.table.set_position(self.orig_location);
        self.search = None;
        self.table.reset_highlight();
        self.table.bump_cursor_pos();
    }
}

impl HandleFilter for Loaded {
    fn set_filter(&mut self, filter: String) -> Action {
        let mut count = 0;
        let visibility: Vec<bool> = self
            .playlist
            .entry
            .iter()
            .map(|i| {
                let a = i.title.to_lowercase().contains(&filter.to_lowercase());
                if a {
                    count += 1;
                }
                a
            })
            .collect();
        self.filter = Some((count, filter));
        self.table.set_visibility(&visibility);
        self.table.bump_cursor_pos();
        self.bar.update_max(count as u32);
        Action::ChangeMode(Mode::Normal)
    }

    fn reset_filter(&mut self) -> Action {
        self.filter = None;
        self.table.reset_visibility();
        self.table.bump_cursor_pos();
        self.bar.update_max(self.playlist.entry.len() as u32);
        Action::ChangeMode(Mode::Normal)
    }

    fn exit_filter(&mut self) -> Action {
        self.table.bump_cursor_pos();
        Action::ChangeMode(Mode::Normal)
    }
}
