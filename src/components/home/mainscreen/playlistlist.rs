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

enum DisplayedComponent {
    Loading,
    Error(String),
    Loaded {
        comp: List<'static>,
        list: Vec<SimplePlaylist>,
    },
}

pub struct PlaylistList {
    action_tx: UnboundedSender<Action>,
    state: DisplayedComponent,
}

impl PlaylistList {
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
            state: DisplayedComponent::Loading,
        }
    }
}

impl Component for PlaylistList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::GetPlaylists(res) = action {
            match res {
                GetPlaylistsResponse::Success(simple_playlists) => {
                    self.state = DisplayedComponent::Loaded {
                        comp: PlaylistList::gen_list(&simple_playlists),
                        list: simple_playlists,
                    };
                }
                GetPlaylistsResponse::Failure(_) => todo!(),
            }
        };
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match &self.state {
            DisplayedComponent::Loading => frame.render_widget(
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
            DisplayedComponent::Error(e) => frame.render_widget(
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
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
                area,
            ),
            DisplayedComponent::Loaded { comp, list: _ } => frame.render_widget(comp, area),
        };
        Ok(())
    }
}
