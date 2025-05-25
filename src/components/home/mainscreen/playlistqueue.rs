use crate::{
    action::{
        getplaylist::{FullPlaylist, GetPlaylistResponse},
        Action, LocalAction,
    },
    components::Component,
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
    fn select_music(&self) -> Option<Action> {
        if let CompState::Loaded {
            name: _,
            comp: _,
            list,
            state,
        } = &self.state
        {
            if let Some(pos) = state.selected() {
                Some(Action::Player(PlayerAction::AddToQueue {
                    pos: QueueLocation::Start,
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
    pub fn new() -> Self {
        Self {
            state: CompState::NotSelected,
            enabled: false,
        }
    }
}

impl Component for PlaylistQueue {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Local(l) => {
                if let CompState::Loaded {
                    name,
                    comp: _,
                    list,
                    state,
                } = &mut self.state
                {
                    match l {
                        LocalAction::Up => {
                            state.select_previous();
                            Ok(None)
                        }
                        LocalAction::Down => {
                            state.select_next();
                            Ok(None)
                        }
                        LocalAction::Confirm => Ok(self.select_music()),
                        LocalAction::Top => {
                            state.select_first();
                            Ok(None)
                        }
                        LocalAction::Bottom => {
                            state.select_last();
                            Ok(None)
                        }
                        LocalAction::Refresh => Ok(Some(Action::Query(Query::GetPlaylist {
                            name: Some(name.to_string()),
                            id: list.id.clone(),
                        }))),
                        // TODO: Add horizontal text scrolling
                        _ => Ok(None),
                    }
                } else if let CompState::Error { id, name, error: _ } = &self.state {
                    if let LocalAction::Refresh = l {
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
}

impl Stateful<bool> for PlaylistQueue {
    fn draw_state(&mut self, frame: &mut Frame, area: Rect, state: bool) -> Result<()> {
        match &mut self.state {
            CompState::NotSelected => frame.render_widget(
                Paragraph::new("Choose a playlist!")
                    .block(
                        Self::gen_block(state, "Playlist Queue").padding(Padding::new(
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
                    .block(Self::gen_block(state, name).padding(Padding::new(
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
                .block(Self::gen_block(state, name).padding(Padding::new(
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
                if self.enabled != state {
                    self.enabled = state;
                    *comp = Self::gen_list(list, self.enabled);
                };
                frame.render_stateful_widget(&*comp, area, ls);
            }
        };
        Ok(())
    }
}
