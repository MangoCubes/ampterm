use crate::{
    action::{
        getplaylists::{GetPlaylistsResponse, SimplePlaylist},
        Action,
    },
    components::Component,
};
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, List, ListState},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

pub struct PlaylistList {
    action_tx: UnboundedSender<Action>,
    list: List<'static>,
    playlists: Option<Vec<SimplePlaylist>>,
}

impl PlaylistList {
    fn gen_list(list: &Option<Vec<SimplePlaylist>>) -> List<'static> {
        let items: Vec<String> = match list {
            Some(ps) => ps.iter().map(|p| p.name.clone()).collect(),
            None => ["<Loading...>".to_owned()].to_vec(),
        };
        List::new(items)
            .block(Block::bordered().title("Playlist"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        // action_tx.send(
        Self {
            action_tx,
            list: PlaylistList::gen_list(&None),
            playlists: None,
        }
    }
}

impl Component for PlaylistList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::GetPlaylists(res) = action {
            match res {
                GetPlaylistsResponse::Success(simple_playlists) => {
                    self.playlists = Some(simple_playlists);
                    self.list = PlaylistList::gen_list(&self.playlists);
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
        frame.render_widget(&self.list, area);
        Ok(())
    }
}
