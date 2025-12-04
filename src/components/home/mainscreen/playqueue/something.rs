use crossterm::event::KeyEvent;
use rand::seq::SliceRandom;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Row, Table},
    Frame,
};

use crate::{
    action::{
        action::{Action, QueryAction, QueueAction, TargetedAction},
        localaction::PlayQueueAction,
    },
    components::{
        home::mainscreen::playqueue::PlayQueue,
        lib::visualtable::{VisualSelection, VisualTable},
        traits::{
            focusable::Focusable,
            handleaction::HandleAction,
            handlekeyseq::{ComponentKeyHelp, HandleKeySeq, KeySeqResult},
            handlequery::HandleQuery,
            renderable::Renderable,
        },
    },
    config::{keybindings::KeyBindings, Config},
    helper::selection::{ModifiableList, Selection},
    osclient::response::getplaylist::Media,
    playerworker::player::{FromPlayerWorker, QueueLocation, ToPlayerWorker},
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::MediaID, ToQueryWorker},
    },
};

/// Specifies where the play cursor (▶) should be at
#[derive(Clone)]
enum CurrentItem {
    /// The play cursor is not in the playlist because it is before the entire list
    /// In other words, nothing in the queue has been played yet.
    BeforeFirst,
    /// The play cursor is not present because it is after the entire playlist.
    /// In other words, everything has been played.
    AfterLast,
    /// The play cursor is not present because the item being played right now is deleted by the
    /// user.
    /// The index is the last item that has already been played.
    /// In this state, skip forward behaves normally, but skip backward plays the currently
    /// selected item, instead of skipping back.
    /// Example: Item Three to Item Five are deleted.
    ///   Music One        Music One
    ///   Music Two      ▷ Music Two
    ///   Music Three      Music Six
    /// ▶ Music Four  ->   Music Seven
    ///   Music Five
    ///   Music Six
    ///   Music Seven
    NotInQueue(usize),
    /// The play cursor is placed next to the item specified by the index
    InQueue(usize),
}

pub struct Something {
    list: ModifiableList<Media>,
    now_playing: CurrentItem,
    table: VisualTable,
    enabled: bool,
    config: Config,
}

/// There are 4 unique states each item in the list can have:
/// 1. Position of the item being played (Play cursor position)
///    This is indicated by a ▶ at the front, with played items using grey as primary colour
/// 2. Temporary selection
///    This is indicated by colour inversion
/// 3. Selection
///    This is indicated with green (darker green used if the item has already been played)
/// 4. Current cursor position (Cursor position)
///    This is indicated with > and inversion
///
/// As a result, a dedicated list component has to be made
impl Something {
    pub fn new(enabled: bool, list: Vec<Media>, config: Config) -> (Self, Action) {
        fn table_proc(table: Table<'static>) -> Table<'static> {
            table
                .highlight_symbol(">")
                .row_highlight_style(Style::new().reversed())
        }
        let action = Action::Query(QueryAction::ToQueryWorker(ToQueryWorker::new(
            HighLevelQuery::PlayMusicFromURL(list[0].clone()),
        )));
        (
            Self {
                config: config.clone(),
                enabled,
                table: VisualTable::new(
                    config,
                    Self::gen_rows_from(&list, &CurrentItem::InQueue(0)),
                    [Constraint::Max(1), Constraint::Min(0), Constraint::Max(1)].to_vec(),
                    table_proc,
                ),
                list: ModifiableList::new(list),
                now_playing: CurrentItem::InQueue(0),
            },
            action,
        )
    }
    pub fn set_star(&mut self, media: &MediaID, star: bool) -> Option<Action> {
        self.list.0 = self
            .list
            .0
            .clone()
            .into_iter()
            .map(|mut m| {
                if m.id == *media {
                    m.starred = if star {
                        Some("Starred".to_string())
                    } else {
                        None
                    };
                }
                m
            })
            .collect();
        self.regen_rows();
        None
    }

    /// Regenerate all rows based on the current state, and rerender the table in full
    fn regen_rows(&mut self) {
        let rows = Self::gen_rows_from(&self.list.0, &self.now_playing);
        self.table.set_rows(rows);
    }

    fn skip_to(&mut self, to: CurrentItem) -> Action {
        let action = match to {
            CurrentItem::NotInQueue(idx) => {
                self.now_playing = CurrentItem::InQueue(idx + 1);
                match self.list.0.get(idx + 1) {
                    Some(m) => Action::Query(QueryAction::ToQueryWorker(ToQueryWorker::new(
                        HighLevelQuery::PlayMusicFromURL(m.clone()),
                    ))),
                    None => Action::Query(QueryAction::ToPlayerWorker(ToPlayerWorker::Stop)),
                }
            }
            CurrentItem::InQueue(idx) => {
                self.now_playing = CurrentItem::InQueue(idx);
                match self.list.0.get(idx) {
                    Some(m) => Action::Query(QueryAction::ToQueryWorker(ToQueryWorker::new(
                        HighLevelQuery::PlayMusicFromURL(m.clone()),
                    ))),
                    None => Action::Query(QueryAction::ToPlayerWorker(ToPlayerWorker::Stop)),
                }
            }
            _ => {
                self.now_playing = to;
                Action::Query(QueryAction::ToPlayerWorker(ToPlayerWorker::Stop))
            }
        };
        self.regen_rows();
        action
    }

    fn skip(&mut self, skip_by: i32) -> Action {
        let max_len = self.list.0.len() as i32;

        fn clean(index: i32, max_len: i32) -> CurrentItem {
            if index >= 0 {
                if index >= max_len {
                    CurrentItem::AfterLast
                } else {
                    CurrentItem::InQueue(index as usize)
                }
            } else {
                CurrentItem::BeforeFirst
            }
        }

        let ci = match &self.now_playing {
            CurrentItem::BeforeFirst => clean(skip_by - 1, max_len),
            CurrentItem::AfterLast => clean(max_len + skip_by, max_len),
            CurrentItem::InQueue(index) => clean(*index as i32 + skip_by, max_len),
            CurrentItem::NotInQueue(index) => {
                if skip_by <= 0 {
                    clean(*index as i32 + skip_by + 1, max_len)
                } else {
                    clean(*index as i32 + skip_by, max_len)
                }
            }
        };

        self.skip_to(ci)
    }

    /// Generate all the rows that appears in the table.
    fn gen_rows_from(items: &Vec<Media>, now_playing: &CurrentItem) -> Vec<Row<'static>> {
        let len = items.len();
        if len == 0 {
            return vec![];
        }
        // Theme for items that have already been played
        let played = Style::new().fg(Color::DarkGray);
        // Theme for items that have not been played yet
        let not_yet_played = Style::new();

        /// Function that generates a number of rows given a subarray and the style to apply
        fn gen_rows_part(ms: &[Media], style: Style) -> Vec<Row<'static>> {
            ms.iter()
                .map(|m| {
                    Row::new(vec![" ".to_string(), m.title.clone(), m.get_fav_marker()])
                        .style(style)
                })
                .collect()
        }

        /// Function that generates a single row with a specified cursor and style
        fn gen_playing_item(ms: &Media, cursor: String, style: Style) -> Row<'static> {
            Row::new(vec![cursor, ms.title.clone(), ms.get_fav_marker()]).style(style)
        }

        /// Function that generates the entire list given the list of media to show in the table
        fn gen_rows_with_cursor(
            idx: usize,
            len: usize,
            played: Style,
            not_yet_played: Style,
            items: &Vec<Media>,
            in_queue: bool,
        ) -> Vec<Row<'static>> {
            let (cursor, playing_style) = if in_queue {
                ("▶".to_string(), not_yet_played.bold())
            } else {
                ("▷".to_string(), played.bold())
            };
            match idx {
                // This is the case where the currently plaing item's index matches the final item
                // in the sublist this function received. Therefore, everything but the last
                // element will be marked as "Played", and the last element will be marked as "Now
                // playing".
                i if (len - 1) == i => {
                    let mut list = gen_rows_part(&items[..i], played);
                    list.push(gen_playing_item(&items[i], cursor, playing_style));
                    list
                }
                // This is the case where the currently playing item's index matches the first item
                // in the sublist this function received. Therefore, everything but the first
                // element will be marked as "Not played", and the first element will be marked as
                // "Now playing".
                0 => {
                    let mut list = gen_rows_part(&items[1..], not_yet_played);
                    list.insert(0, gen_playing_item(&items[0], cursor, playing_style));
                    list
                }
                // This is the case where the sublist contains the music that is currently being
                // played, but is not the start nor the end
                i => {
                    let mut list = gen_rows_part(&items[..i], played);
                    list.append(&mut gen_rows_part(&items[(i + 1)..], not_yet_played));
                    list.insert(i, gen_playing_item(&items[i], cursor, playing_style));
                    list
                }
            }
        }
        match now_playing {
            CurrentItem::BeforeFirst => gen_rows_part(&items, not_yet_played),
            CurrentItem::AfterLast => gen_rows_part(&items, played),
            CurrentItem::InQueue(idx) => {
                gen_rows_with_cursor(*idx, len, played, not_yet_played, items, true)
            }
            CurrentItem::NotInQueue(idx) => {
                gen_rows_with_cursor(*idx, len, played, not_yet_played, items, false)
            }
        }
    }
    fn add_to_queue(&mut self, items: Vec<Media>, at: QueueLocation) -> Option<Action> {
        let max = self.list.0.len();
        let idx = match at {
            QueueLocation::Front => match self.now_playing {
                CurrentItem::BeforeFirst => 0,
                CurrentItem::AfterLast => max,
                CurrentItem::InQueue(idx) => idx,
                CurrentItem::NotInQueue(idx) => idx + 1,
            },
            QueueLocation::Next => match self.now_playing {
                CurrentItem::BeforeFirst => 0,
                CurrentItem::AfterLast => max,
                CurrentItem::InQueue(idx) | CurrentItem::NotInQueue(idx) => idx + 1,
            },
            QueueLocation::Last => max,
        };
        let len = items.len();
        self.list.add_rows_at(items, idx);

        let rows = Self::gen_rows_from(&self.list.0, &self.now_playing);
        self.table.add_rows_at(rows, idx, len);

        if matches!(at, QueueLocation::Front) {
            Some(self.skip_to(self.now_playing.clone()))
        } else {
            None
        }
    }
}

impl Renderable for Something {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let title = if let Some(pos) = self.table.get_current() {
            let len = self.list.0.len();
            format!(
                "Queue ({}/{})",
                if pos == usize::MAX || pos >= len {
                    len
                } else {
                    pos + 1
                },
                len
            )
        } else {
            format!("Queue ({})", self.list.0.len())
        };
        let border = PlayQueue::gen_block(self.enabled, title);
        let inner = border.inner(area);
        frame.render_widget(border, area);
        self.table.draw(frame, inner)
    }
}

impl HandleQuery for Something {
    fn handle_query(&mut self, action: crate::action::action::QueryAction) -> Option<Action> {
        match action {
            QueryAction::FromPlayerWorker(FromPlayerWorker::Finished) => Some(self.skip(1)),
            _ => None,
        }
    }
}

impl HandleAction for Something {
    fn handle_action(&mut self, action: TargetedAction) -> Option<Action> {
        match action {
            TargetedAction::Skip => Some(self.skip(1)),
            TargetedAction::Previous => Some(self.skip(-1)),
            TargetedAction::Queue(a) => match a {
                QueueAction::Add(items, at) => self.add_to_queue(items, at),
                QueueAction::RandomAdd(mut items, at) => {
                    let mut rng = rand::rng();
                    items.shuffle(&mut rng);
                    self.add_to_queue(items, at)
                }
            },
            _ => None,
        }
    }
}

impl HandleKeySeq<PlayQueueAction> for Something {
    fn get_other_helps(&self) -> Vec<ComponentKeyHelp> {
        self.table.get_help()
    }
    fn get_name(&self) -> &str {
        "PlayQueue"
    }
    fn pass_to_lower_comp(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        self.table.handle_key_seq(keyseq)
    }

    fn handle_local_action(&mut self, action: PlayQueueAction) -> KeySeqResult {
        match action {
            PlayQueueAction::Delete => {
                let (vs, action) = self.table.get_selection_reset();
                let selection = match vs {
                    VisualSelection::Single(index) => Selection::Single(index),
                    VisualSelection::TempSelection(start, end) => Selection::Range(start, end),
                    VisualSelection::Selection(items) => Selection::Multiple(items),
                    VisualSelection::None { unselect: _ } => {
                        return match action {
                            Some(a) => KeySeqResult::ActionNeeded(a),
                            None => KeySeqResult::NoActionNeeded,
                        }
                    }
                };

                match self.now_playing {
                    CurrentItem::InQueue(idx) => {
                        if let Some((newidx, deleted)) = self.list.move_item_to(&selection, idx) {
                            self.now_playing = if deleted {
                                CurrentItem::NotInQueue(newidx)
                            } else {
                                CurrentItem::InQueue(newidx)
                            };
                        } else {
                            self.now_playing = CurrentItem::BeforeFirst;
                        }
                    }
                    CurrentItem::NotInQueue(idx) => {
                        if let Some((newidx, _)) = self.list.move_item_to(&selection, idx) {
                            self.now_playing = CurrentItem::NotInQueue(newidx)
                        } else {
                            self.now_playing = CurrentItem::BeforeFirst;
                        }
                    }
                    _ => (),
                }

                self.list.delete(&selection);
                self.regen_rows();

                match action {
                    Some(a) => KeySeqResult::ActionNeeded(a),
                    None => KeySeqResult::NoActionNeeded,
                }
            }
            PlayQueueAction::ToggleStar => {
                let (selection, action) = self.table.get_selection_reset();
                let mut items: Vec<Action> = match selection {
                    VisualSelection::Single(idx) => {
                        let item = self.list.0[idx].clone();
                        vec![(item.id, item.starred == None)]
                    }
                    VisualSelection::TempSelection(start, end) => self.list.0[start..=end]
                        .iter()
                        .map(|m| (m.id.clone(), m.starred == None))
                        .collect(),
                    VisualSelection::Selection(items) => self
                        .list
                        .0
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
                    self.set_star(&id, star);
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
            PlayQueueAction::PlaySelected => match self.table.get_current() {
                Some(idx) => KeySeqResult::ActionNeeded(self.skip_to(CurrentItem::InQueue(idx))),
                None => KeySeqResult::NoActionNeeded,
            },
        }
    }

    fn get_keybinds(&self) -> &KeyBindings<PlayQueueAction> {
        &self.config.local.playqueue
    }
}

impl Focusable for Something {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
