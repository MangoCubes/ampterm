use crate::{
    action::{
        getplaylists::{GetPlaylistsResponse, SimplePlaylist},
        Action, LocalAction,
    },
    components::Component,
    queryworker::query::Query,
    stateful::Stateful,
    stateless::Stateless,
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
                let key = list[pos].id.clone();
                let name = list[pos].name.clone();
                let _ = self.action_tx.send(Action::Query(Query::GetPlaylist {
                    name: Some(name),
                    id: key,
                }));
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
            Action::Local(l) => {
                if let CompState::Loaded {
                    comp: _,
                    list: _,
                    state,
                } = &mut self.state
                {
                    match l {
                        LocalAction::Up => state.select_previous(),
                        LocalAction::Down => state.select_next(),
                        LocalAction::Confirm => self.select_playlist(),
                        LocalAction::Top => state.select_first(),
                        LocalAction::Bottom => state.select_last(),
                        // TODO: Add horizontal text scrolling
                        _ => {}
                    }
                }
            }
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
}

impl Stateful<bool> for PlaylistList {
    fn draw_state(&mut self, frame: &mut Frame, area: Rect, state: bool) -> Result<()> {
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
