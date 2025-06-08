use std::collections::HashSet;

use crate::{
    action::{
        getplaylist::{FullPlaylist, MediaID},
        Action,
    },
    components::Component,
    focusable::Focusable,
    local_action,
    playerworker::player::{PlayerAction, QueueLocation},
    queryworker::query::Query,
    visualmode::VisualMode,
};
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

pub struct Loaded {
    name: String,
    comp: List<'static>,
    list: FullPlaylist,
    list_state: ListState,
    // If the value is None, then the current mode is not visual mode
    // Otherwise, the list is filled with the items selected by the current visual mode
    visual: Option<HashSet<MediaID>>,
    // List of all selected media
    selected: Option<HashSet<MediaID>>,
    enabled: bool,
}

impl Loaded {
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
    fn select_music(&self, playpos: QueueLocation) -> Option<Action> {
        if let Some(pos) = self.list_state.selected() {
            Some(Action::Player(PlayerAction::AddToQueue {
                pos: playpos,
                music: vec![self.list.entry[pos].clone()],
            }))
        } else {
            None
        }
    }
    pub fn gen_list(
        list: &FullPlaylist,
        visual: &Option<HashSet<MediaID>>,
        selected: &Option<HashSet<MediaID>>,
        enabled: bool,
    ) -> List<'static> {
        let items: Vec<ListItem> = list
            .entry
            .iter()
            .map(|p| {
                let id = &p.id;
                let mut item = ListItem::from(p.title.clone());
                if let Some(s) = selected {
                    if s.contains(id) {
                        item = item.bold();
                    }
                }
                if let Some(r) = visual {
                    if r.contains(id) {
                        item = item.green();
                    }
                }
                item
            })
            .collect();
        List::new(items)
            .block(Self::gen_block(enabled, &list.name))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }
    pub fn new(
        name: String,
        list: FullPlaylist,
        list_state: ListState,
        visual: Option<HashSet<MediaID>>,
        selected: Option<HashSet<MediaID>>,
        enabled: bool,
    ) -> Self {
        Self {
            name,
            comp: Loaded::gen_list(&list, &None, &None, enabled),
            list,
            list_state: ListState::default(),
            visual: None,
            selected: None,
            enabled,
        }
    }
}

impl Component for Loaded {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            local_action!() => {
                match action {
                    Action::Up => {
                        self.list_state.select_previous();
                        Ok(None)
                    }
                    Action::Down => {
                        self.list_state.select_next();
                        Ok(None)
                    }
                    Action::Top => {
                        self.list_state.select_first();
                        Ok(None)
                    }
                    Action::Bottom => {
                        self.list_state.select_last();
                        Ok(None)
                    }
                    Action::Refresh => Ok(Some(Action::Query(Query::GetPlaylist {
                        name: Some(self.name.to_string()),
                        id: self.list.id.clone(),
                    }))),
                    Action::AddNext => Ok(self.select_music(QueueLocation::Next)),
                    Action::AddLast => Ok(self.select_music(QueueLocation::Last)),
                    Action::AddFront => Ok(self.select_music(QueueLocation::Front)),
                    Action::NormalMode => {
                        self.set_visual_mode(false);
                        Ok(None)
                    }
                    Action::VisualMode => {
                        let Some(i) = self.list_state.selected() else {
                            return Ok(None);
                        };
                        let Some(item) = self.list.entry.get(i) else {
                            return Ok(None);
                        };
                        let id = item.id.clone();
                        self.set_temp_selection(Some(HashSet::from([id])));
                        self.set_visual_mode(true);
                        // *comp = Loaded::gen_list(list, &None, &None, self.enabled);
                        Ok(None)
                    }
                    // TODO: Add horizontal text scrolling
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.list_state);
        Ok(())
    }
}

impl Focusable for Loaded {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            self.comp = Self::gen_list(&self.list, &self.visual, &self.selected, self.enabled);
            if !self.enabled {
                self.set_visual(false);
            }
        };
    }
}

impl VisualMode<MediaID> for Loaded {
    fn is_visual(&self) -> bool {
        matches!(self.visual, Some(_))
    }

    fn set_visual(&mut self, to: bool) {
        if matches!(self.visual, Some(_)) != to {
            self.visual = match &self.visual {
                Some(_) => None,
                None => Some(HashSet::new()),
            }
        }
    }

    fn get_temp_selection(&self) -> Option<&HashSet<MediaID>> {
        if let Some(ids) = &self.visual {
            Some(ids)
        } else {
            None
        }
    }

    fn get_selection(&self) -> Option<&HashSet<MediaID>> {
        if let Some(ids) = &self.selected {
            Some(ids)
        } else {
            None
        }
    }

    fn set_selection(&mut self, selection: Option<HashSet<MediaID>>) {
        self.selected = selection;
    }

    fn set_temp_selection(&mut self, selection: Option<HashSet<MediaID>>) {
        self.visual = selection;
    }
}
