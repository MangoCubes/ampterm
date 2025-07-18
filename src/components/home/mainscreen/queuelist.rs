use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

use crate::{
    action::{getplaylist::Media, Action, PlayState},
    components::Component,
    focusable::Focusable,
};
use color_eyre::Result;

pub struct QueueList {
    comp: List<'static>,
    list: PlayState,
    state: ListState,
    enabled: bool,
}

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
    fn gen_list(&self) -> List<'static> {
        let len = self.list.items.len();
        let before = Style::new().fg(Color::Gray);
        let current = Style::new().fg(Color::Green).bold();
        let after = Style::new();
        fn gen_items(ms: &[Media], style: Style) -> Vec<ListItem<'static>> {
            ms.iter()
                .map(|m| ListItem::from(m.title.clone()).style(style))
                .collect()
        }
        let items = match self.list.index {
            // Current item is beyond the current playlist
            _ if len == self.list.index => gen_items(&self.list.items, before),
            // Current item is the last item in the playlist
            idx if (len - 1) == self.list.index => {
                let mut list = gen_items(&self.list.items[..idx], before);
                list.push(ListItem::from((&self.list.items[idx].title).clone()).style(current));
                list
            }
            // Current item is the first element in the playlist
            0 => {
                let mut list = gen_items(&self.list.items[..1], before);
                list.insert(
                    0,
                    ListItem::from((&self.list.items[0].title).clone()).style(current),
                );
                list
            }
            // Every other cases
            idx => {
                let mut list = gen_items(&self.list.items[..idx], before);
                list.append(&mut gen_items(&self.list.items[(idx + 1)..], after));
                list.insert(
                    idx,
                    ListItem::from((&self.list.items[idx].title).clone()).style(current),
                );
                list
            }
        };
        let comp = List::new(items);
        let block = Self::gen_block(self.enabled, "Next Up");

        comp.block(block)
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }

    pub fn new(enabled: bool) -> Self {
        let list = PlayState::default();
        Self {
            state: ListState::default(),
            comp: List::default(),
            list,
            enabled,
        }
    }
}

impl Component for QueueList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::InQueue {
            play,
            vol: _,
            speed: _,
            pos: _,
        } = action
        {
            self.list = play;
            self.comp = self.gen_list()
        }
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
}

impl Focusable for QueueList {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            self.comp = self.gen_list()
        }
    }
}
