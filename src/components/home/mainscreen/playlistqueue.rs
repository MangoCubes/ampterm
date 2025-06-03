use crate::{
    action::{
        getplaylist::{FullPlaylist, GetPlaylistResponse},
        Action,
    },
    components::Component,
    local_action,
    playerworker::player::{PlayerAction, QueueLocation},
    queryworker::query::Query,
    stateful::Stateful,
};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListState, Padding, Paragraph, Wrap},
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
    fn gen_list(list: &FullPlaylist, enabled: bool) -> List<'static> {
        let items: Vec<String> = list.entry.iter().map(|p| p.title.clone()).collect();
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
                    comp: _,
                    list,
                    state,
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
                        comp: PlaylistQueue::gen_list(&full_playlist, self.enabled),
                        name: full_playlist.name.clone(),
                        list: full_playlist,
                        state: ListState::default().with_selected(Some(0)),
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
                list,
                state: ls,
                name: _,
            } => {
                frame.render_stateful_widget(&*comp, area, ls);
            }
        };
        Ok(())
    }
}

impl Stateful<bool> for PlaylistQueue {
    fn update_state(&mut self, state: bool) {
        if self.enabled != state {
            self.enabled = state;
            if let CompState::Loaded {
                name: _,
                comp,
                list,
                state: _,
            } = &mut self.state
            {
                *comp = Self::gen_list(list, self.enabled);
            };
        };
    }
}
