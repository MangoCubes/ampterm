use crate::{
    action::{
        getplaylist::{FullPlaylist, GetPlaylistResponse},
        getplaylists::{GetPlaylistsResponse, SimplePlaylist},
        Action, LocalAction,
    },
    components::Component,
};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, List, ListState, Padding, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

enum CompState {
    Loading(String),
    NotSelected,
    Error {
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
    action_tx: UnboundedSender<Action>,
    state: CompState,
}

impl PlaylistQueue {
    fn select_playlist(&self) {}
    fn change_item(&mut self, down: bool) {
        if let CompState::Loaded {
            comp: _,
            list: _,
            name: _,
            state,
        } = &mut self.state
        {
            if down {
                state.select_next()
            } else {
                state.select_previous()
            };
        }
    }
    fn gen_list(list: &FullPlaylist) -> List<'static> {
        let items: Vec<String> = list.entry.iter().map(|p| p.title.clone()).collect();
        List::new(items)
            .block(Block::bordered().title(list.name.clone()))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self {
            action_tx,
            state: CompState::NotSelected,
        }
    }
}

impl Component for PlaylistQueue {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Local(l) => match l {
                LocalAction::Up => self.change_item(false),
                LocalAction::Down => self.change_item(true),
                LocalAction::Confirm => self.select_playlist(),
                // TODO: Add horizontal text scrolling
                _ => {}
            },
            Action::GetPlaylist(res) => match res {
                GetPlaylistResponse::Success(full_playlist) => {
                    self.state = CompState::Loaded {
                        comp: PlaylistQueue::gen_list(&full_playlist),
                        name: full_playlist.name.clone(),
                        list: full_playlist,
                        state: ListState::default().with_selected(Some(0)),
                    }
                }
                GetPlaylistResponse::Failure { name, msg } => {
                    self.state = CompState::Error {
                        name: name.unwrap_or("Playlist Queue".to_string()),
                        error: msg,
                    };
                }
            },
            _ => {}
        }
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.state {
            CompState::NotSelected => frame.render_widget(
                Paragraph::new("Choose a playlist!")
                    .block(
                        Block::bordered()
                            .title("Playlist Queue")
                            .padding(Padding::new(0, 0, area.height / 2, 0)),
                    )
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false }),
                area,
            ),
            CompState::Loading(name) => frame.render_widget(
                Paragraph::new("Loading...")
                    .block(Block::bordered().title(name.clone()).padding(Padding::new(
                        0,
                        0,
                        area.height / 2,
                        0,
                    )))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false }),
                area,
            ),
            CompState::Error { name, error } => frame.render_widget(
                Paragraph::new(vec![
                    Line::raw("Error!"),
                    Line::raw(format!("{}", error)),
                    Line::raw(format!("Reload with 'R'")),
                ])
                .block(Block::bordered().title(name.clone()).padding(Padding::new(
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
                state,
                name,
            } => frame.render_stateful_widget(&*comp, area, state),
        };
        Ok(())
    }
}
