use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Row, Table, TableState},
    Frame,
};

use crate::{
    action::{
        useraction::{Common, Normal, UserAction, Visual},
        Action, FromPlayerWorker, PlayState,
    },
    app::Mode,
    components::{lib::visualtable::VisualTable, traits::component::Component},
    osclient::response::getplaylist::Media,
};

pub struct Something {
    list: PlayState,
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
    pub fn new(playstate: PlayState) -> Self {
        fn table_proc(table: Table<'static>) -> Table<'static> {
            table
                .highlight_symbol(">")
                .row_highlight_style(Style::new().reversed())
        }
        Self {
            table: VisualTable::new(
                Self::gen_rows(&playstate),
                [Constraint::Max(1), Constraint::Min(0), Constraint::Max(1)].to_vec(),
                table_proc,
            ),
            list: playstate,
        }
    }
    fn gen_current_item(ms: &Media) -> Row<'static> {
        let current = Style::new().bold();
        Row::new(vec!["▶".to_string(), ms.title.clone(), ms.get_fav_marker()]).style(current)
    }
    fn gen_rows(playstate: &PlayState) -> Vec<Row<'static>> {
        let len = playstate.items.len();
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
        match playstate.index {
            // Current item is beyond the current playlist
            _ if len == playstate.index => gen_rows_part(&playstate.items, before),
            // Current item is the last item in the playlist
            idx if (len - 1) == playstate.index => {
                let mut list = gen_rows_part(&playstate.items[..idx], before);
                list.push(Self::gen_current_item(&playstate.items[idx]));
                list
            }
            // Current item is the first element in the playlist
            0 => {
                let mut list = gen_rows_part(&playstate.items[1..], after);
                list.insert(0, Self::gen_current_item(&playstate.items[0]));
                list
            }
            // Every other cases
            idx => {
                let mut list = gen_rows_part(&playstate.items[..idx], before);
                list.append(&mut gen_rows_part(&playstate.items[(idx + 1)..], after));
                list.insert(idx, Self::gen_current_item(&playstate.items[idx]));
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
        self.table.update(action)
    }
}
