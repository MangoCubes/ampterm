use crate::{
    action::{
        getplaylists::{GetPlaylistsResponse, SimplePlaylist},
        Action,
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
    Loading,
    Error(String),
    Loaded {
        comp: List<'static>,
        list: Vec<SimplePlaylist>,
        state: ListState,
    },
}

pub struct PlaylistList {
    action_tx: UnboundedSender<Action>,
    state: CompState,
}

impl PlaylistList {
    fn select_playlist(&self) {
        if let CompState::Loaded {
            comp: _,
            list,
            state,
        } = &self.state
        {
            if let Some(pos) = state.selected() {
                let key = (&list[pos].id).to_string();
                let _ = self.action_tx.send(Action::SelectPlaylist { key });
            };
        }
    }
    fn change_item(&mut self, down: bool) {
        if let CompState::Loaded {
            comp: _,
            list: _,
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
    fn gen_list(list: &Vec<SimplePlaylist>) -> List<'static> {
        let items: Vec<String> = list.iter().map(|p| p.name.clone()).collect();
        List::new(items)
            .block(Block::bordered().title("Playlist"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self {
            action_tx,
            state: CompState::Loading,
        }
    }
}

impl Component for PlaylistList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Up => self.change_item(false),
            Action::Down => self.change_item(true),
            Action::Confirm => self.select_playlist(),
            // TODO: Add horizontal text scrolling
            // Action::Left => todo!(),
            // Action::Right => todo!(),
            Action::GetPlaylists(res) => match res {
                GetPlaylistsResponse::Success(simple_playlists) => {
                    self.state = CompState::Loaded {
                        comp: PlaylistList::gen_list(&simple_playlists),
                        list: simple_playlists,
                        state: ListState::default().with_selected(Some(0)),
                    };
                }
                GetPlaylistsResponse::Failure(e) => self.state = CompState::Error(e),
            },
            _ => {}
        }
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &mut self.state {
            CompState::Loading => frame.render_widget(
                Paragraph::new("Loading...")
                    .block(Block::bordered().title("Playlist").padding(Padding::new(
                        0,
                        0,
                        area.height / 2,
                        0,
                    )))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false }),
                area,
            ),
            CompState::Error(e) => frame.render_widget(
                Paragraph::new(vec![
                    Line::raw("Error!"),
                    Line::raw(format!("{}", e)),
                    Line::raw(format!("Reload with 'R'")),
                ])
                .block(Block::bordered().title("Playlist").padding(Padding::new(
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
            } => frame.render_stateful_widget(&*comp, area, state),
        };
        Ok(())
    }
}
