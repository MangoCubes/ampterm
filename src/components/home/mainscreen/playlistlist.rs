use crate::{
    action::{
        getplaylists::{GetPlaylistsResponse, SimplePlaylist},
        Action, LocalAction,
    },
    components::Component,
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
    Loading,
    Error {
        error: String,
    },
    Loaded {
        comp: List<'static>,
        list: Vec<SimplePlaylist>,
        state: ListState,
    },
}

pub struct PlaylistList {
    state: CompState,
    enabled: bool,
}

impl PlaylistList {
    fn select_playlist(&self) -> Option<Action> {
        if let CompState::Loaded {
            comp: _,
            list,
            state,
        } = &self.state
        {
            if let Some(pos) = state.selected() {
                let key = list[pos].id.clone();
                let name = list[pos].name.clone();
                Some(Action::Query(Query::GetPlaylist {
                    name: Some(name),
                    id: key,
                }))
            } else {
                None
            }
        } else {
            None
        }
    }

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

    fn gen_list(list: &Vec<SimplePlaylist>, enabled: bool) -> List<'static> {
        let items: Vec<String> = list.iter().map(|p| p.name.clone()).collect();
        List::new(items)
            .block(Self::gen_block(enabled, "Playlist"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }
    pub fn new() -> Self {
        Self {
            state: CompState::Loading,
            enabled: false,
        }
    }
}

impl Component for PlaylistList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Local(l) => {
                if let CompState::Loaded {
                    comp: _,
                    list: _,
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
                        // LocalAction::AddNext => Ok(self.select_playlist()),
                        // LocalAction::AddLast => Ok(self.select_playlist()),
                        // LocalAction::AddFront => Ok(self.select_playlist()),
                        LocalAction::Confirm => Ok(self.select_playlist()),
                        LocalAction::Top => {
                            state.select_first();
                            Ok(None)
                        }
                        LocalAction::Bottom => {
                            state.select_last();
                            Ok(None)
                        }
                        // TODO: Add horizontal text scrolling
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            Action::GetPlaylists(res) => match res {
                GetPlaylistsResponse::Success(simple_playlists) => {
                    self.state = CompState::Loaded {
                        comp: PlaylistList::gen_list(&simple_playlists, self.enabled),
                        list: simple_playlists,
                        state: ListState::default().with_selected(Some(0)),
                    };
                    Ok(None)
                }
                GetPlaylistsResponse::Failure(error) => {
                    self.state = CompState::Error { error };
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.state {
            CompState::Loading => {
                // Cannot be cached since the area always changes
                frame.render_widget(
                    Paragraph::new("Loading...")
                        .block(
                            PlaylistList::gen_block(self.enabled, "Playlist")
                                .padding(Padding::new(0, 0, area.height / 2, 0)),
                        )
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false }),
                    area,
                )
            }
            // Cannot be cached since the area always changes
            CompState::Error { error } => frame.render_widget(
                Paragraph::new(vec![
                    Line::raw("Error!"),
                    Line::raw(format!("{}", error)),
                    Line::raw(format!("Reload with 'R'")),
                ])
                .block(
                    PlaylistList::gen_block(self.enabled, "Playlist").padding(Padding::new(
                        0,
                        0,
                        (area.height / 2) - 1,
                        0,
                    )),
                )
                .alignment(Alignment::Center),
                area,
            ),
            CompState::Loaded {
                comp,
                list,
                state: ls,
            } => {
                frame.render_stateful_widget(&*comp, area, ls);
            }
        };
        Ok(())
    }
}

impl Stateful<bool> for PlaylistList {
    fn update_state(&mut self, state: bool) {
        if self.enabled != state {
            self.enabled = state;
            if let CompState::Loaded {
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
