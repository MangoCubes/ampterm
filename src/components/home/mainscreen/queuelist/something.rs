use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Row, Table},
    Frame,
};

use crate::{
    action::{Action, FromPlayerWorker, QueueChange, StateType},
    components::{lib::visualtable::VisualTable, traits::component::Component},
    osclient::response::getplaylist::Media,
};

pub struct Something {
    list: Vec<Media>,
    index: usize,
    table: VisualTable,
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
    pub fn new(list: Vec<Media>) -> Self {
        fn table_proc(table: Table<'static>) -> Table<'static> {
            table
                .highlight_symbol(">")
                .row_highlight_style(Style::new().reversed())
        }
        Self {
            table: VisualTable::new(
                Self::gen_rows(&list, 0),
                [Constraint::Max(1), Constraint::Min(0), Constraint::Max(1)].to_vec(),
                table_proc,
            ),
            list,
            index: 0,
        }
    }
    fn gen_rows(items: &Vec<Media>, index: usize) -> Vec<Row<'static>> {
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
            // Current item is beyond the current playlist
            _ if len == index => gen_rows_part(&items, before),
            // Current item is the last item in the playlist
            idx if (len - 1) == index => {
                let mut list = gen_rows_part(&items[..idx], before);
                list.push(gen_current_item(&items[idx]));
                list
            }
            // Current item is the first element in the playlist
            0 => {
                let mut list = gen_rows_part(&items[1..], after);
                list.insert(0, gen_current_item(&items[0]));
                list
            }
            // Every other cases
            idx => {
                let mut list = gen_rows_part(&items[..idx], before);
                list.append(&mut gen_rows_part(&items[(idx + 1)..], after));
                list.insert(idx, gen_current_item(&items[idx]));
                list
            }
        }
    }
}

impl Component for Something {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.table.draw(frame, area)
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FromPlayerWorker(FromPlayerWorker::StateChange(state_type)) => {
                match state_type {
                    StateType::Queue(queue_change) => match queue_change {
                        QueueChange::Add { items, at } => {
                            self.list.splice(at..at, items);
                            self.table
                                .add_rows_at(Self::gen_rows(&self.list, self.index), at);
                        }
                        QueueChange::Del { from, to } => todo!(),
                    },
                    StateType::NowPlaying { music, index } => {
                        self.index = index;
                        self.table.set_rows(Self::gen_rows(&self.list, self.index));
                    }
                    StateType::Volume(_) | StateType::Position(_) | StateType::Speed(_) => {}
                }
                Ok(None)
            }
            _ => self.table.update(action),
        }
    }
}
