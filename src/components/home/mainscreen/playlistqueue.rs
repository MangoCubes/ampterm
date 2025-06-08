use std::collections::HashSet;

use crate::{
    action::{
        getplaylist::{FullPlaylist, GetPlaylistResponse, Media, MediaID},
        Action,
    },
    add_to_queue,
    components::Component,
    focusable::Focusable,
    local_action, movements,
    playerworker::player::{PlayerAction, QueueLocation},
    queryworker::query::Query,
    visualmode::VisualMode,
};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Padding, Paragraph, Wrap},
    Frame,
};

enum CompState {
    Loading {
        id: String,
        name: String,
    },
    NotSelected,
    Error {
        id: String,
        name: String,
        error: String,
    },
    Loaded {
        name: String,
        comp: List<'static>,
        list: FullPlaylist,
        state: ListState,
        // If the value is None, then the current mode is not visual mode
        // Otherwise, the list is filled with the items selected by the current visual mode
        visual: Option<HashSet<MediaID>>,
        // List of all selected media
        selected: Option<HashSet<MediaID>>,
    },
}

pub struct PlaylistQueue {
    state: CompState,
    enabled: bool,
}

impl PlaylistQueue {
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
        if let CompState::Loaded {
            name: _,
            comp: _,
            list,
            state,
            visual: _,
            selected: _,
        } = &self.state
        {
            if let Some(pos) = state.selected() {
                Some(Action::Player(PlayerAction::AddToQueue {
                    pos: playpos,
                    music: vec![list.entry[pos].clone()],
                }))
            } else {
                None
            }
        } else {
            None
        }
    }
    fn gen_list(
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
    pub fn new(enabled: bool) -> Self {
        Self {
            state: CompState::NotSelected,
            enabled,
        }
    }
}

impl Component for PlaylistQueue {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            local_action!() => {
                if let CompState::Loaded {
                    name,
                    comp,
                    list,
                    state,
                    visual,
                    selected,
                } = &mut self.state
                {
                    match action {
                        Action::Up => {
                            state.select_previous();
                            Ok(None)
                        }
                        Action::Down => {
                            state.select_next();
                            Ok(None)
                        }
                        Action::Top => {
                            state.select_first();
                            Ok(None)
                        }
                        Action::Bottom => {
                            state.select_last();
                            Ok(None)
                        }
                        Action::Refresh => Ok(Some(Action::Query(Query::GetPlaylist {
                            name: Some(name.to_string()),
                            id: list.id.clone(),
                        }))),
                        Action::AddNext => Ok(self.select_music(QueueLocation::Next)),
                        Action::AddLast => Ok(self.select_music(QueueLocation::Last)),
                        Action::AddFront => Ok(self.select_music(QueueLocation::Front)),
                        Action::NormalMode => {
                            self.set_visual_mode(false);
                            Ok(None)
                        }
                        Action::VisualMode => {
                            let Some(i) = state.selected() else {
                                return Ok(None);
                            };
                            let Some(item) = list.entry.get(i) else {
                                return Ok(None);
                            };
                            let id = item.id.clone();
                            self.set_temp_selection(Some(HashSet::from([id])));
                            self.set_visual_mode(true);
                            // *comp = PlaylistQueue::gen_list(list, &None, &None, self.enabled);
                            Ok(None)
                        }
                        // TODO: Add horizontal text scrolling
                        _ => Ok(None),
                    }
                } else if let CompState::Error { id, name, error: _ } = &self.state {
                    if let Action::Refresh = action {
                        Ok(Some(Action::Query(Query::GetPlaylist {
                            name: Some(name.to_string()),
                            id: id.to_string(),
                        })))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Action::Query(q) => {
                if let Query::GetPlaylist { name, id } = q {
                    self.state = CompState::Loading {
                        id: id.clone(),
                        name: name.unwrap_or(id),
                    }
                };
                Ok(None)
            }
            Action::GetPlaylist(res) => match res {
                GetPlaylistResponse::Success(full_playlist) => {
                    self.state = CompState::Loaded {
                        comp: PlaylistQueue::gen_list(&full_playlist, &None, &None, self.enabled),
                        name: full_playlist.name.clone(),
                        list: full_playlist,
                        state: ListState::default().with_selected(Some(0)),
                        visual: None,
                        selected: None,
                    };
                    Ok(None)
                }
                GetPlaylistResponse::Failure { id, name, msg } => {
                    self.state = CompState::Error {
                        id,
                        name: name.unwrap_or("Playlist Queue".to_string()),
                        error: msg,
                    };
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.state {
            CompState::NotSelected => frame.render_widget(
                Paragraph::new("Choose a playlist!")
                    .block(
                        Self::gen_block(self.enabled, "Playlist Queue").padding(Padding::new(
                            0,
                            0,
                            area.height / 2,
                            0,
                        )),
                    )
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false }),
                area,
            ),
            CompState::Loading { id: _, name } => frame.render_widget(
                Paragraph::new("Loading...")
                    .block(Self::gen_block(self.enabled, name).padding(Padding::new(
                        0,
                        0,
                        area.height / 2,
                        0,
                    )))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false }),
                area,
            ),
            CompState::Error { id: _, name, error } => frame.render_widget(
                Paragraph::new(vec![
                    Line::raw("Error!"),
                    Line::raw(format!("{}", error)),
                    Line::raw(format!("Reload with 'R'")),
                ])
                .block(Self::gen_block(self.enabled, name).padding(Padding::new(
                    0,
                    0,
                    (area.height / 2) - 1,
                    0,
                )))
                .alignment(Alignment::Center),
                area,
            ),
            CompState::Loaded {
                comp,
                list: _,
                state: ls,
                name: _,
                visual: _,
                selected: _,
            } => {
                frame.render_stateful_widget(&*comp, area, ls);
            }
        };
        Ok(())
    }
}

impl Focusable for PlaylistQueue {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            if let CompState::Loaded {
                name: _,
                comp,
                list,
                state: _,
                visual,
                selected,
            } = &mut self.state
            {
                *comp = Self::gen_list(list, visual, selected, self.enabled);
                if !self.enabled {
                    self.set_visual(false);
                }
            };
        };
    }
}

impl VisualMode<MediaID> for PlaylistQueue {
    fn is_visual(&self) -> bool {
        if let CompState::Loaded {
            name: _,
            comp: _,
            list: _,
            state: _,
            visual,
            selected: _,
        } = &self.state
        {
            matches!(visual, Some(_))
        } else {
            false
        }
    }

    fn set_visual(&mut self, to: bool) {
        if let CompState::Loaded {
            name: _,
            comp: _,
            list: _,
            state: _,
            visual,
            selected: _,
        } = &mut self.state
        {
            if matches!(visual, Some(_)) != to {
                *visual = match visual {
                    Some(_) => None,
                    None => Some(HashSet::new()),
                }
            }
        }
    }

    fn get_temp_selection(&self) -> Option<&HashSet<MediaID>> {
        if let CompState::Loaded {
            name: _,
            comp: _,
            list: _,
            state: _,
            visual,
            selected: _,
        } = &self.state
        {
            if let Some(ids) = visual {
                Some(ids)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_selection(&self) -> Option<&HashSet<MediaID>> {
        if let CompState::Loaded {
            name: _,
            comp: _,
            list: _,
            state: _,
            visual: _,
            selected,
        } = &self.state
        {
            if let Some(ids) = selected {
                Some(ids)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn set_selection(&mut self, selection: Option<HashSet<MediaID>>) {
        if let CompState::Loaded {
            name: _,
            comp: _,
            list: _,
            state: _,
            visual: _,
            selected,
        } = &mut self.state
        {
            *selected = selection;
        };
    }

    fn set_temp_selection(&mut self, selection: Option<HashSet<MediaID>>) {
        if let CompState::Loaded {
            name: _,
            comp: _,
            list: _,
            state: _,
            visual,
            selected: _,
        } = &mut self.state
        {
            *visual = selection;
        };
    }
}
