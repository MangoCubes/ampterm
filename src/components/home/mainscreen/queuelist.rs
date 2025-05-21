use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

use crate::{
    action::{getplaylist::Media, Action},
    components::Component,
    stateful::Stateful,
};
use color_eyre::Result;

pub struct QueueList {
    comp: List<'static>,
    list: Vec<Media>,
    current: Option<Media>,
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
    fn gen_list(
        enabled: bool,
        current: &Option<Media>,
        list: Option<&Vec<Media>>,
    ) -> List<'static> {
        let comp = match list {
            Some(l) => {
                let mut items: Vec<ListItem> =
                    l.iter().map(|p| ListItem::from(p.title.clone())).collect();
                if let Some(p) = current {
                    items.push(ListItem::from(p.title.clone()).fg(Color::Green));
                };
                List::new(items)
            }
            None => List::default(),
        };
        comp.block(Self::gen_block(enabled, "Next Up"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }

    pub fn new() -> Self {
        let empty = vec![];
        Self {
            state: ListState::default(),
            comp: Self::gen_list(false, &None, None),
            current: None,
            list: empty,
            enabled: false,
        }
    }
}

impl Component for QueueList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::InQueue { current, next } = action {
            self.list = next;
            self.current = current;
            self.comp = Self::gen_list(self.enabled, &self.current, Some(&self.list))
        }
        Ok(None)
    }
}

impl Stateful<bool> for QueueList {
    fn draw_state(&mut self, frame: &mut Frame, area: Rect, state: bool) -> Result<()> {
        if self.enabled != state {
            self.enabled = state;
            self.comp = Self::gen_list(self.enabled, &self.current, Some(&self.list))
        }
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
}
