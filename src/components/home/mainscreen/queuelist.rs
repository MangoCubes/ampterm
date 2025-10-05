use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, List, ListItem, ListState, Row, Table, TableState},
    Frame,
};

use crate::{
    action::{Action, FromPlayerWorker, PlayState},
    components::traits::{component::Component, focusable::Focusable},
    osclient::response::getplaylist::Media,
};
use color_eyre::Result;

pub struct QueueList {
    comp: Table<'static>,
    list: PlayState,
    state: TableState,
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

impl QueueList {
    fn gen_block(enabled: bool, title: &str) -> Block<'static> {
        let style = if enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };
        let title = Span::styled(
            title.to_string(),
            if enabled {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            },
        );
        Block::bordered().title(title).border_style(style)
    }
    fn gen_table(&self) -> Table<'static> {
        let block = Self::gen_block(self.enabled, "Next Up");
        let len = self.list.items.len();
        let before = Style::new().fg(Color::DarkGray);
        let after = Style::new();
        fn gen_items(ms: &[Media], style: Style) -> Vec<Row<'static>> {
            ms.iter()
                .map(|m| {
                    Row::new(vec![" ".to_string(), " ".to_string(), m.title.clone()]).style(style)
                })
                .collect()
        }
        fn gen_current_item(ms: &Media) -> Row<'static> {
            let current = Style::new().bold();
            Row::new(vec!["▶".to_string(), " ".to_string(), ms.title.clone()]).style(current)
        }
        let items = match self.list.index {
            // Current item is beyond the current playlist
            _ if len == self.list.index => gen_items(&self.list.items, before),
            // Current item is the last item in the playlist
            idx if (len - 1) == self.list.index => {
                let mut list = gen_items(&self.list.items[..idx], before);
                list.push(gen_current_item(&self.list.items[idx]));
                list
            }
            // Current item is the first element in the playlist
            0 => {
                let mut list = gen_items(&self.list.items[1..], after);
                list.insert(0, gen_current_item(&self.list.items[0]));
                list
            }
            // Every other cases
            idx => {
                let mut list = gen_items(&self.list.items[..idx], before);
                list.append(&mut gen_items(&self.list.items[(idx + 1)..], after));
                list.insert(idx, gen_current_item(&self.list.items[idx]));
                list
            }
        };
        let comp = Table::new(
            items,
            [Constraint::Max(1), Constraint::Max(1), Constraint::Min(0)].to_vec(),
        );
        comp.highlight_symbol(">")
            .row_highlight_style(Style::new().reversed())
            .block(block)
    }

    pub fn new(enabled: bool) -> Self {
        let list = PlayState::default();
        Self {
            state: TableState::default(),
            comp: Table::default().block(Self::gen_block(false, "Next Up")),
            list,
            enabled,
        }
    }
}

impl Component for QueueList {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::FromPlayerWorker(FromPlayerWorker::InQueue {
            play,
            vol,
            speed,
            pos,
        }) = action
        {
            self.list = play;
            self.comp = self.gen_table()
        }
        Ok(None)
    }
}

impl Focusable for QueueList {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            self.comp = self.gen_table()
        }
    }
}
