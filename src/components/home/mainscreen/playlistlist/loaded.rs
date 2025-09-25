use std::collections::HashMap;

use crate::{
    action::{
        useraction::{Common, Normal, UserAction},
        Action,
    },
    compid,
    components::traits::{component::Component, focusable::Focusable},
    osclient::response::getplaylists::SimplePlaylist,
    playerworker::player::{QueueLocation, ToPlayerWorker},
    queryworker::query::{
        getplaylist::GetPlaylistResponse, getplaylists::PlaylistID, QueryType, ResponseType,
        ToQueryWorker,
    },
};
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, List, ListState},
    Frame,
};
use tracing::error;

pub struct Loaded {
    comp: List<'static>,
    list: Vec<SimplePlaylist>,
    state: ListState,
    enabled: bool,
    callback: HashMap<usize, (PlaylistID, QueueLocation)>,
}

impl Loaded {
    fn select_playlist(&self) -> Option<Action> {
        if let Some(pos) = self.state.selected() {
            let key = self.list[pos].id.clone();
            let name = self.list[pos].name.clone();
            Some(Action::ToQueryWorker(ToQueryWorker::new(
                // When a playlist is selected, its content should update the playlist queue
                compid::PLAYLISTQUEUE,
                QueryType::GetPlaylist { name, id: key },
            )))
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
    pub fn new(enabled: bool, list: Vec<SimplePlaylist>, state: ListState) -> Self {
        Self {
            enabled,
            comp: Self::gen_list(&list, enabled),
            list,
            state,
            callback: HashMap::new(),
        }
    }
    pub fn add_to_queue(&mut self, ql: QueueLocation) -> Option<Action> {
        if let Some(pos) = self.state.selected() {
            let key = self.list[pos].id.clone();
            let name = self.list[pos].name.clone();
            let req = ToQueryWorker::new(
                compid::PLAYLISTLIST,
                QueryType::GetPlaylist {
                    name,
                    id: key.clone(),
                },
            );
            self.callback.insert(req.ticket, (key, ql));
            Some(Action::ToQueryWorker(req))
        } else {
            error!("Failed to add playlist to queue: No playlist selected");
            None
        }
    }
}

impl Component for Loaded {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FromQueryWorker(res) => {
                if let Some(cb) = self.callback.remove(&res.ticket) {
                    if let ResponseType::GetPlaylist(res) = res.res {
                        match res {
                            GetPlaylistResponse::Success(full_playlist) => {
                                return Ok(Some(Action::ToPlayerWorker(
                                    ToPlayerWorker::AddToQueue {
                                        music: full_playlist.entry,
                                        pos: cb.1,
                                    },
                                )));
                            }
                            GetPlaylistResponse::Failure {
                                id: _,
                                name: _,
                                msg,
                            } => {
                                error!("Failed to add playlist to queue: {msg}");
                            }
                        }
                    }
                }
                Ok(None)
            }
            Action::User(UserAction::Common(local)) => {
                match local {
                    Common::Up => {
                        self.state.select_previous();
                        Ok(None)
                    }
                    Common::Down => {
                        self.state.select_next();
                        Ok(None)
                    }
                    Common::Confirm => Ok(self.select_playlist()),
                    Common::Top => {
                        self.state.select_first();
                        Ok(None)
                    }
                    Common::Bottom => {
                        self.state.select_last();
                        Ok(None)
                    }
                    // TODO: Add horizontal text scrolling
                    _ => Ok(None),
                }
            }
            Action::User(UserAction::Normal(normal)) => {
                if let Normal::Add(pos) = normal {
                    Ok(self.add_to_queue(pos))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

impl Focusable for Loaded {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            self.comp = Self::gen_list(&self.list, self.enabled);
        };
    }
}
