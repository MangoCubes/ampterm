mod filter;
mod search;
use crate::{
    action::{
        action::{Action, Mode, QueueAction, TargetedAction},
        localaction::PlaylistQueueAction,
    },
    components::{
        home::mainscreen::playlistqueue::{
            loaded::{
                filter::{Filter, FilterResult},
                search::{Search, SearchResult},
            },
            PlaylistQueue,
        },
        lib::{
            scrollbar::ScrollBar,
            visualtable::{VisualSelection, VisualTable},
        },
        traits::{
            focusable::Focusable,
            handlekeyseq::{ComponentKeyHelp, HandleKeySeq, KeySeqResult},
            handleraw::HandleRaw,
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
    widgets::{Clear, Row, Table, TableState},
    Frame,
};

enum State {
    Nothing,
    Filtering(Filter),
    Searching(Search, usize),
}

pub struct Loaded {
    name: String,
    playlist: FullPlaylist,
    enabled: bool,
    table: VisualTable,
    keymap: KeyBindings<PlaylistQueueAction>,
    state: State,
    filter: Option<(usize, String)>,
    search: Option<(usize, String)>,
    bar: ScrollBar,
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
            state: State::Nothing,
            filter: None,
            search: None,
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

    /// Executed when the user types something into the search bar
    fn apply_search(&mut self, search: String) -> Option<Action> {
        if search.len() == 0 {
            self.search = Some((0, search));
            self.table.reset_highlight();
            None
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
            None
        }
    }

    /// Executed when the user confirms the search by pressing enter
    fn confirm_search(&mut self, search: String) -> Action {
        self.apply_search(search);
        self.state = State::Nothing;
        Action::ChangeMode(Mode::Normal)
    }

    /// Executed whent the search is cancelled or removed
    fn clear_search(&mut self) -> Action {
        if let State::Searching(_, idx) = self.state {
            self.table.set_position(idx);
        };
        self.state = State::Nothing;
        self.search = None;
        self.table.reset_highlight();
        self.table.bump_cursor_pos();
        Action::ChangeMode(Mode::Normal)
    }
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
        self.state = State::Nothing;
        self.table.set_visibility(&visibility);
        self.table.bump_cursor_pos();
        self.bar.update_max(count as u32);
        Action::ChangeMode(Mode::Normal)
    }
    fn reset_filter(&mut self) -> Action {
        self.state = State::Nothing;
        self.filter = None;
        self.table.reset_visibility();
        self.table.bump_cursor_pos();
        self.bar.update_max(self.playlist.entry.len() as u32);
        Action::ChangeMode(Mode::Normal)
    }
    fn exit_filter(&mut self) -> Action {
        self.state = State::Nothing;
        self.table.bump_cursor_pos();
        Action::ChangeMode(Mode::Normal)
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
        let inner = match &mut self.state {
            State::Nothing => {
                let inner = border.inner(area);
                frame.render_widget(border, area);
                inner
            }
            State::Filtering(filter) => {
                let layout =
                    Layout::default().constraints([Constraint::Min(1), Constraint::Length(3)]);
                let areas = layout.split(area);
                let inner = border.inner(areas[0]);
                frame.render_widget(border, areas[0]);
                frame.render_widget(Clear, areas[1]);
                filter.draw(frame, areas[1]);
                inner
            }
            State::Searching(search, _) => {
                let layout =
                    Layout::default().constraints([Constraint::Min(1), Constraint::Length(3)]);
                let areas = layout.split(area);
                let inner = border.inner(areas[0]);
                frame.render_widget(border, areas[0]);
                frame.render_widget(Clear, areas[1]);
                search.draw(frame, areas[1]);
                inner
            }
        };
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
            PlaylistQueueAction::Filter => {
                self.state = State::Filtering(Filter::new());
                KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Insert))
            }
            PlaylistQueueAction::ClearFilter => KeySeqResult::ActionNeeded(self.reset_filter()),
            PlaylistQueueAction::Search => {
                self.state = State::Searching(
                    Search::new(if let Some((_, search)) = &self.search {
                        search.clone()
                    } else {
                        "".to_string()
                    }),
                    self.table.get_current().unwrap_or(0),
                );
                KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Insert))
            }
            PlaylistQueueAction::ClearSearch => KeySeqResult::ActionNeeded(self.clear_search()),
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

impl HandleRaw for Loaded {
    fn handle_raw(&mut self, key: KeyEvent) -> Option<Action> {
        match &mut self.state {
            State::Nothing => unreachable!("Playlist queue should never enter insert mode without filtering or searching active."),
            State::Filtering(f) => {
                match f.handle_raw(key) {
                    FilterResult::NoChange => None,
                    FilterResult::ApplyFilter(filter) => {
                        Some(self.set_filter(filter))
                    },
                    FilterResult::ClearFilter => {
                        Some(self.reset_filter())
                    }
                    FilterResult::Exit => Some(self.exit_filter())
                }
            },
            State::Searching(s , idx) => {
                match s.handle_raw(key) {
                    SearchResult::ApplySearch(s) => self.apply_search(s),
                    SearchResult::ConfirmSearch(s, b) => {
                        if !b {
                            self.table.set_position(*idx);
                        }
                        Some(self.confirm_search(s))
                    }
                    SearchResult::CancelSearch => Some(self.clear_search()),
                }
            },
        }
    }
}
