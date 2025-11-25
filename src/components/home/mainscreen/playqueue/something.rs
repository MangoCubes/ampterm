use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Row, Table},
    Frame,
};

use crate::{
    action::{
        useraction::{Common, Global, UserAction},
        Action, FromPlayerWorker, QueueAction, Selection,
    },
    components::{
        home::mainscreen::playqueue::PlayQueue,
        lib::visualtable::{VisualSelection, VisualTable},
        traits::{focusable::Focusable, fullcomp::FullComp, renderable::Renderable},
    },
    helper::selection::ModifiableList,
    osclient::response::getplaylist::Media,
    playerworker::player::{QueueLocation, ToPlayerWorker},
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
    /// The play cursor is placed next to the item specified by the index
    InQueue(usize),
}

#[derive(Clone)]
enum MediaStatus {
    /// Any item marked as Repeat(n) is repeated n times. Normal is equivalent to Repeat(1).
    Repeat(usize),
    /// Any item marked as Temporary is removed once it is finished.
    Temporary,
}

pub struct Something {
    list: ModifiableList<(Media, MediaStatus)>,
    now_playing: CurrentItem,
    table: VisualTable,
    enabled: bool,
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
    pub fn new(enabled: bool, list: Vec<Media>) -> (Self, Action) {
        fn table_proc(table: Table<'static>) -> Table<'static> {
            table
                .highlight_symbol(">")
                .row_highlight_style(Style::new().reversed())
        }
        let action = Action::ToQueryWorker(ToQueryWorker::new(HighLevelQuery::PlayMusicFromURL(
            list[0].clone(),
        )));
        let queue = list
            .into_iter()
            .map(|m| (m, MediaStatus::Repeat(1)))
            .collect();
        (
            Self {
                enabled,
                table: VisualTable::new(
                    Self::gen_rows_from(&queue, &CurrentItem::InQueue(0)),
                    [Constraint::Max(1), Constraint::Min(0), Constraint::Max(1)].to_vec(),
                    table_proc,
                ),
                list: ModifiableList::new(queue),
                now_playing: CurrentItem::InQueue(0),
            },
            action,
        )
    }
    pub fn set_star(&mut self, media: MediaID, star: bool) -> Option<Action> {
        self.list.0 = self
            .list
            .0
            .clone()
            .into_iter()
            .map(|(mut m, s)| {
                if m.id == media {
                    m.starred = if star {
                        Some("Starred".to_string())
                    } else {
                        None
                    };
                }
                (m, s)
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
        let action = if let CurrentItem::InQueue(idx) = to {
            match self.list.0.get(idx) {
                Some(m) => Action::ToQueryWorker(ToQueryWorker::new(
                    HighLevelQuery::PlayMusicFromURL(m.0.clone()),
                )),
                None => Action::ToPlayerWorker(ToPlayerWorker::Stop),
            }
        } else {
            Action::ToPlayerWorker(ToPlayerWorker::Stop)
        };
        self.now_playing = to;
        self.regen_rows();
        action
    }

    fn skip(&mut self, skip_by: i32) -> Action {
        let max_len = self.list.0.len() as i32;
        let index = match &self.now_playing {
            CurrentItem::BeforeFirst => -1 + skip_by,
            CurrentItem::AfterLast => max_len + skip_by,
            CurrentItem::InQueue(index) => *index as i32 + skip_by,
        };
        let cleaned = if index >= 0 {
            if index >= max_len {
                CurrentItem::AfterLast
            } else {
                CurrentItem::InQueue(index as usize)
            }
        } else {
            CurrentItem::BeforeFirst
        };
        self.skip_to(cleaned)
    }

    fn gen_rows_from(
        items: &Vec<(Media, MediaStatus)>,
        now_playing: &CurrentItem,
    ) -> Vec<Row<'static>> {
        let len = items.len();
        if len == 0 {
            return vec![];
        }
        let played = Style::new().fg(Color::DarkGray);
        let not_yet_played = Style::new();
        fn gen_rows_part(ms: &[(Media, MediaStatus)], style: Style) -> Vec<Row<'static>> {
            ms.iter()
                .map(|(m, _)| {
                    Row::new(vec![" ".to_string(), m.title.clone(), m.get_fav_marker()])
                        .style(style)
                })
                .collect()
        }
        fn gen_playing_item((m, _): &(Media, MediaStatus)) -> Row<'static> {
            let current = Style::new().bold();
            Row::new(vec!["▶".to_string(), m.title.clone(), m.get_fav_marker()]).style(current)
        }
        match now_playing {
            CurrentItem::BeforeFirst => gen_rows_part(&items, not_yet_played),
            CurrentItem::AfterLast => gen_rows_part(&items, played),
            CurrentItem::InQueue(idx) => match *idx {
                // This is the case where the currently plaing item's index matches the final item
                // in the sublist this function received. Therefore, everything but the last
                // element will be marked as "Played", and the last element will be marked as "Now
                // playing".
                i if (len - 1) == i => {
                    let mut list = gen_rows_part(&items[..i], played);
                    list.push(gen_playing_item(&items[i]));
                    list
                }
                // This is the case where the currently playing item's index matches the first item
                // in the sublist this function received. Therefore, everything but the first
                // element will be marked as "Not played", and the first element will be marked as
                // "Now playing".
                0 => {
                    let mut list = gen_rows_part(&items[1..], not_yet_played);
                    list.insert(0, gen_playing_item(&items[0]));
                    list
                }
                // This is the case where the sublist contains the music that is currently being
                // played, but is not the start nor the end
                i => {
                    let mut list = gen_rows_part(&items[..i], played);
                    list.append(&mut gen_rows_part(&items[(i + 1)..], not_yet_played));
                    list.insert(i, gen_playing_item(&items[i]));
                    list
                }
            },
        }
    }
}

impl Renderable for Something {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
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

impl FullComp for Something {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Queue(a) => match a {
                QueueAction::Add(items, at) => {
                    let max = self.list.0.len();
                    let idx = match at {
                        QueueLocation::Front => match self.now_playing {
                            CurrentItem::BeforeFirst => 0,
                            CurrentItem::AfterLast => max,
                            CurrentItem::InQueue(idx) => idx,
                        },
                        QueueLocation::Next => match self.now_playing {
                            CurrentItem::BeforeFirst => 0,
                            CurrentItem::AfterLast => max,
                            CurrentItem::InQueue(idx) => idx + 1,
                        },
                        QueueLocation::Last => max,
                    };
                    let len = items.len();
                    self.list.add_rows_at(
                        items
                            .into_iter()
                            .map(|m| (m, MediaStatus::Repeat(1)))
                            .collect(),
                        idx,
                    );
                    let rows = Self::gen_rows_from(&self.list.0, &self.now_playing);
                    self.table.add_rows_at(rows, idx, len);

                    if matches!(at, QueueLocation::Front) {
                        Ok(Some(self.skip_to(self.now_playing.clone())))
                    } else {
                        Ok(None)
                    }
                }
            },
            Action::ToQueryWorker(ToQueryWorker {
                dest: _,
                ticket: _,
                query,
            }) => match query {
                HighLevelQuery::SetStar { media, star } => Ok(self.set_star(media, star)),
                _ => Ok(None),
            },
            Action::FromPlayerWorker(a) => match a {
                FromPlayerWorker::Finished => Ok(Some(self.skip(1))),
                _ => Ok(None),
            },
            Action::User(UserAction::Global(a)) => match a {
                Global::Skip => Ok(Some(self.skip(1))),
                Global::Previous => Ok(Some(self.skip(-1))),
                _ => Ok(None),
            },
            Action::User(UserAction::Common(a)) => match a {
                Common::Delete => {
                    let (selection, action) = self.table.get_selection_reset();
                    let selection = match selection {
                        VisualSelection::Single(index) => Selection::Single(index),
                        VisualSelection::TempSelection(start, end) => Selection::Range(start, end),
                        VisualSelection::Selection(items) => Selection::Multiple(items),
                        VisualSelection::None { unselect: _ } => {
                            return Ok(action);
                        }
                    };
                    if let CurrentItem::InQueue(idx) = self.now_playing {
                        if let Some((newidx, deleted)) = self.list.move_item_to(&selection, idx) {
                            self.now_playing = if deleted {
                                CurrentItem::BeforeFirst
                                // CurrentItem::NotInQueue(newidx, self.list.0[idx].clone())
                            } else {
                                CurrentItem::InQueue(newidx)
                            };
                        } else {
                            self.now_playing = CurrentItem::BeforeFirst;
                        }
                    } /*  else if let CurrentItem::NotInQueue(idx, m) = &self.now_playing {
                          if let Some((newidx, _)) = self.list.move_item_to(&selection, *idx) {
                              self.now_playing = CurrentItem::NotInQueue(newidx, m.clone())
                          } else {
                              self.now_playing = CurrentItem::BeforeFirst;
                          }
                      } */
                    self.list.delete(&selection);
                    self.regen_rows();

                    Ok(action)
                }
                Common::ToggleStar => {
                    let (selection, action) = self.table.get_selection_reset();
                    let mut items: Vec<Action> = match selection {
                        VisualSelection::Single(idx) => {
                            let item = self.list.0[idx].0.clone();
                            vec![(item.id, item.starred == None)]
                        }
                        VisualSelection::TempSelection(start, end) => self.list.0[start..=end]
                            .iter()
                            .map(|(m, _)| (m.id.clone(), m.starred == None))
                            .collect(),
                        VisualSelection::Selection(items) => self
                            .list
                            .0
                            .iter()
                            .zip(items.iter())
                            .filter_map(|((m, _), &selected)| {
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
                        Action::ToQueryWorker(ToQueryWorker::new(HighLevelQuery::SetStar {
                            media: id,
                            star,
                        }))
                    })
                    .collect();

                    if let Some(a) = action {
                        items.push(a);
                    }

                    Ok(Some(Action::Multiple(items)))
                }
                Common::Confirm => match self.table.get_current() {
                    Some(idx) => Ok(Some(self.skip_to(CurrentItem::InQueue(idx)))),
                    None => Ok(None),
                },
                _ => self.table.update(Action::User(UserAction::Common(a))),
            },
            _ => self.table.update(action),
        }
    }
}

impl Focusable for Something {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
