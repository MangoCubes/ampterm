use crate::{
    action::{
        getplaylists::{GetPlaylistsResponse, PlaylistID, SimplePlaylist},
        Action,
    },
    components::{home::mainscreen::playlistlist::PlaylistListComps, Component},
    focusable::Focusable,
    insert_action, local_action,
    playerworker::player::QueueLocation,
    queryworker::query::Query,
};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListState, Padding, Paragraph, Wrap},
    Frame,
};

pub struct PlaylistListLoaded {
    comp: List<'static>,
    list: Vec<SimplePlaylist>,
    state: ListState,
    adding_playlist: Option<(PlaylistID, QueueLocation)>,
    enabled: bool,
}

impl PlaylistListLoaded {
    fn select_playlist(&self) -> Option<Action> {
        if let Some(pos) = self.state.selected() {
            let key = self.list[pos].id.clone();
            let name = self.list[pos].name.clone();
            Some(Action::Query(Query::GetPlaylist {
                name: Some(name),
                id: key,
            }))
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
    pub fn new(
        enabled: bool,
        list: Vec<SimplePlaylist>,
        state: ListState,
        adding_playlist: Option<(PlaylistID, QueueLocation)>,
    ) -> Self {
        Self {
            enabled,
            comp: Self::gen_list(&list, enabled),
            list,
            state,
            adding_playlist,
        }
    }
    pub fn prepare_add_to_queue(&mut self, ql: QueueLocation) -> Option<Action> {
        if let Some(pos) = self.state.selected() {
            let key = self.list[pos].id.clone();
            self.adding_playlist = Some((key, ql));
        }
        None
    }
}

impl Component for PlaylistListLoaded {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            insert_action!() => {
                if let Some(id) = &self.adding_playlist {
                    match action {
                        Action::AddAsIs => todo!(),
                        Action::Randomise => todo!(),
                        Action::Reverse => todo!(),
                        _ => Ok(None),
                    }
                } else {
                    panic!(
                        "Program is in invalid state: Got insert mode action outside insert mode."
                    )
                }
            }
            local_action!() => {
                match action {
                    Action::Up => {
                        self.state.select_previous();
                        Ok(None)
                    }
                    Action::Down => {
                        self.state.select_next();
                        Ok(None)
                    }
                    Action::Add(pos) => Ok(self.prepare_add_to_queue(pos)),
                    Action::Confirm => Ok(self.select_playlist()),
                    Action::Top => {
                        self.state.select_first();
                        Ok(None)
                    }
                    Action::Bottom => {
                        self.state.select_last();
                        Ok(None)
                    }
                    // TODO: Add horizontal text scrolling
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
}

impl Focusable for PlaylistListLoaded {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            self.comp = Self::gen_list(&self.list, self.enabled);
        };
    }
}

impl PlaylistListComps for PlaylistListLoaded {}
