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
        Action, FromPlayerWorker, QueueChange, StateType,
    },
    app::Mode,
    components::{
        home::mainscreen::playqueue::PlayQueue,
        lib::visualtable::{SelectionType, VisualTable},
        traits::{focusable::Focusable, fullcomp::FullComp, renderable::Renderable},
    },
    osclient::response::getplaylist::Media,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::MediaID, ToQueryWorker},
    },
};

pub struct Something {
    list: Vec<Media>,
    index: Option<usize>,
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
    pub fn new(enabled: bool, list: Vec<Media>) -> Self {
        fn table_proc(table: Table<'static>) -> Table<'static> {
            table
                .highlight_symbol(">")
                .row_highlight_style(Style::new().reversed())
        }
        Self {
            enabled,
            table: VisualTable::new(
                Self::gen_rows(&list, Some(0)),
                [Constraint::Max(1), Constraint::Min(0), Constraint::Max(1)].to_vec(),
                table_proc,
            ),
            list,
            index: Some(0),
        }
    }
    pub fn set_star(&mut self, media: MediaID, star: bool) -> Option<Action> {
        let updated = self
            .list
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
        self.list = updated;
        let rows = Self::gen_rows(&self.list, self.index);
        self.table.set_rows(rows);
        None
    }
    fn gen_rows(items: &Vec<Media>, index: Option<usize>) -> Vec<Row<'static>> {
        let len = items.len();
        let before = Style::new().fg(Color::DarkGray);
        let after = Style::new();
        fn gen_rows_part(ms: &[Media], style: Style) -> Vec<Row<'static>> {
            ms.iter()
                .map(|m| {
                    Row::new(vec![" ".to_string(), m.title.clone(), m.get_fav_marker()])
                        .style(style)
                })
                .collect()
        }
        fn gen_current_item(ms: &Media) -> Row<'static> {
            let current = Style::new().bold();
            Row::new(vec!["▶".to_string(), ms.title.clone(), ms.get_fav_marker()]).style(current)
        }
        match index {
            Some(idx) => match idx {
                // Current item is the last item in the playlist
                i if (len - 1) == idx => {
                    let mut list = gen_rows_part(&items[..i], before);
                    list.push(gen_current_item(&items[i]));
                    list
                }
                // Current item is the first element in the playlist
                0 => {
                    let mut list = gen_rows_part(&items[1..], after);
                    list.insert(0, gen_current_item(&items[0]));
                    list
                }
                // Every other cases
                i => {
                    let mut list = gen_rows_part(&items[..i], before);
                    list.append(&mut gen_rows_part(&items[(i + 1)..], after));
                    list.insert(i, gen_current_item(&items[i]));
                    list
                }
            },
            // Current item is beyond the current playlist
            None => gen_rows_part(&items, before),
        }
    }
}

impl Renderable for Something {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let title = if let Some(pos) = self.table.get_current() {
            let len = self.list.len();
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
            format!("Queue ({})", self.list.len())
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
            Action::ToQueryWorker(ToQueryWorker {
                dest: _,
                ticket: _,
                query,
            }) => match query {
                HighLevelQuery::SetStar { media, star } => Ok(self.set_star(media, star)),
                _ => Ok(None),
            },
            Action::FromPlayerWorker(FromPlayerWorker::StateChange(state_type)) => {
                match state_type {
                    StateType::Queue(queue_change) => match queue_change {
                        QueueChange::Add { mut items, at } => {
                            let len = self.list.len();
                            if at > len {
                                self.list.append(&mut items);
                            } else {
                                self.list.splice(at..at, items);
                            }
                            self.table
                                .add_rows_at(Self::gen_rows(&self.list, self.index), at, len);
                        }
                        QueueChange::Del { from, to } => {}
                    },
                    StateType::NowPlaying(now_playing) => {
                        self.index = if let Some(n) = now_playing {
                            Some(n.index)
                        } else {
                            None
                        };
                        self.table.set_rows(Self::gen_rows(&self.list, self.index));
                    }
                    StateType::Volume(_) | StateType::Position(_) | StateType::Speed(_) => {}
                }
                Ok(None)
            }
            Action::User(UserAction::Common(a)) => match a {
                Common::Delete => todo!(),
                Common::ToggleStar => {
                    let (selection, action) = self.table.get_selection_reset();
                    let mut items: Vec<Action> = match selection {
                        SelectionType::Single(idx) => {
                            let item = self.list[idx].clone();
                            vec![(item.id, item.starred == None)]
                        }
                        SelectionType::TempSelection(start, end) => self.list[start..=end]
                            .iter()
                            .map(|m| (m.id.clone(), m.starred == None))
                            .collect(),
                        SelectionType::Selection(items) => self
                            .list
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
                        SelectionType::None { unselect: _ } => vec![],
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
