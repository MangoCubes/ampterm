use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

use crate::{
    action::{Action, PlayState},
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
    fn gen_list(enabled: bool, state: &PlayState) -> List<'static> {
        let mut items: Vec<ListItem> = state
            .next
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != state.index)
            .map(|(_, p)| ListItem::from(p.title.clone()))
            .collect();
        if let Some(m) = state.next.get(state.index) {
            items.insert(
                state.index,
                ListItem::from(m.title.clone()).fg(Color::Green),
            );
        };
        let comp = List::new(items);

        comp.block(Self::gen_block(enabled, "Next Up"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }

    pub fn new(enabled: bool) -> Self {
        let list = PlayState::default();
        Self {
            state: ListState::default(),
            comp: Self::gen_list(false, &list),
            list,
            enabled,
        }
    }
}

impl Component for QueueList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::InQueue {
            play,
            vol,
            speed,
            pos,
        } = action
        {
            self.list = play;
            self.comp = Self::gen_list(self.enabled, &self.list)
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
            self.comp = Self::gen_list(self.enabled, &self.list)
        }
    }
}
