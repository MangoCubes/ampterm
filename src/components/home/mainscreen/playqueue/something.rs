use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Row, Table},
    Frame,
};

use crate::{
    action::{
        useraction::{Common, UserAction},
        Action, FromPlayerWorker, QueueAction, Selection, StateType,
    },
    components::{
        home::mainscreen::playqueue::PlayQueue,
        lib::visualtable::{VisualSelection, VisualTable},
        traits::{focusable::Focusable, fullcomp::FullComp, renderable::Renderable},
    },
    helper::selection::ModifiableList,
    osclient::response::getplaylist::Media,
    playerworker::player::QueueLocation,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::MediaID, ToQueryWorker},
    },
};

pub struct Something {
    list: ModifiableList<Media>,
    now_playing: Option<usize>,
    table: VisualTable,
    enabled: bool,
}

/// There are 4 unique states each item in the list can have:
/// 1. Position relative to the item currently being played
///    This is indicated by a ▶ at the front, with played items using grey as primary colour
/// 2. Temporary selection
///    This is indicated by colour inversion
/// 3. Selection
///    This is indicated with green (darker green used if the item has already been played)
/// 4. Current cursor position
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
        (
            Self {
                enabled,
                table: VisualTable::new(
                    Self::gen_rows_from(&list, Some(0), 0),
                    [Constraint::Max(1), Constraint::Min(0), Constraint::Max(1)].to_vec(),
                    table_proc,
                ),
                list: ModifiableList::new(list),
                now_playing: Some(0),
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
        let rows = Self::gen_rows_from(&self.list.0, self.now_playing, 0);
        self.table.set_rows(rows);
        None
    }
    fn gen_rows_from(
        items: &Vec<Media>,
        now_playing: Option<usize>,
        starting_from: usize,
    ) -> Vec<Row<'static>> {
        let len = items.len();
        let played = Style::new().fg(Color::DarkGray);
        let not_yet_played = Style::new();
        fn gen_rows_part(ms: &[Media], style: Style) -> Vec<Row<'static>> {
            ms.iter()
                .map(|m| {
                    Row::new(vec![" ".to_string(), m.title.clone(), m.get_fav_marker()])
                        .style(style)
                })
                .collect()
        }
        fn gen_playing_item(ms: &Media) -> Row<'static> {
            let current = Style::new().bold();
            Row::new(vec!["▶".to_string(), ms.title.clone(), ms.get_fav_marker()]).style(current)
        }
        match now_playing {
            Some(idx) => match idx {
                // This is the case where the currently plaing item's index matches the final item
                // in the sublist this function received. Therefore, everything but the last
                // element will be marked as "Played", and the last element will be marked as "Now
                // playing".
                i if (starting_from + len - 1) == idx => {
                    let now_playing = i - starting_from;
                    let mut list = gen_rows_part(&items[..now_playing], played);
                    list.push(gen_playing_item(&items[now_playing]));
                    list
                }
                // This is the case where the currently playing item's index matches the first item
                // in the sublist this function received. Therefore, everything but the first
                // element will be marked as "Not played", and the first element will be marked as
                // "Now playing".
                _ if (starting_from == idx) => {
                    let mut list = gen_rows_part(&items[1..], not_yet_played);
                    list.insert(0, gen_playing_item(&items[0]));
                    list
                }
                // This is the case where the sublist this function received is beyond the item
                // that is currently being played. Therefore, everything will be marked as "Not
                // played".
                _ if (starting_from > idx) => gen_rows_part(&items, not_yet_played),
                // This is the case where the entire sublist this function received is before the
                // item that is currently being played. Everything will be marked as such.
                _ if (starting_from + len - 1 < idx) => gen_rows_part(&items, played),
                // This is the case where the sublist contains the music that is currently being
                // played, but is not the start nor the end
                i => {
                    let mut list = gen_rows_part(&items[..(i - starting_from)], played);
                    list.append(&mut gen_rows_part(
                        &items[(i - starting_from + 1)..],
                        not_yet_played,
                    ));
                    list.insert(i, gen_playing_item(&items[i - starting_from]));
                    list
                }
            },
            // Current item is beyond the current playlist
            None => gen_rows_part(&items, played),
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
                    let idx = match self.now_playing {
                        Some(c) => match at {
                            QueueLocation::Front => c,
                            QueueLocation::Next => c + 1,
                            QueueLocation::Last => max,
                        },
                        None => max,
                    };
                    self.table
                        .add_rows_at(Self::gen_rows_from(&items, self.now_playing, idx), idx);
                    self.list.add_rows_at(items, idx);
                    Ok(None)
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
                FromPlayerWorker::StateChange(state_type) => {
                    match state_type {
                        StateType::NowPlaying(now_playing) => {
                            self.now_playing = if let Some(n) = now_playing {
                                Some(n.index)
                            } else {
                                None
                            };
                            self.table.set_rows(Self::gen_rows_from(
                                &self.list.0,
                                self.now_playing,
                                0,
                            ));
                        }
                        StateType::Volume(_) | StateType::Position(_) | StateType::Speed(_) => {}
                    }
                    Ok(None)
                }
                FromPlayerWorker::Finished => {
                    if let Some(idx) = self.now_playing {
                        if let Some(i) = self.list.0.get(idx + 1) {
                            self.now_playing = Some(idx + 1);
                            return Ok(Some(Action::ToQueryWorker(ToQueryWorker::new(
                                HighLevelQuery::PlayMusicFromURL(i.clone()),
                            ))));
                        } else {
                            self.now_playing = None;
                        }
                    };
                    Ok(None)
                }
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
                    self.list.delete(&selection);
                    Ok(action)
                }
                Common::ToggleStar => {
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
