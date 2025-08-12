use crate::{
    action::getplaylists::{PlaylistID, SimplePlaylist},
    components::{
        home::mainscreen::playlistlist::PlaylistListComps,
        traits::{component::Component, focusable::Focusable},
    },
    playerworker::player::QueueLocation,
};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::Span,
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
        adding_playlist: Option<(PlaylistID, QueueLocation)>,
    },
}

pub struct PlaylistListLoading {
    enabled: bool,
}

impl PlaylistListLoading {
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
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Component for PlaylistListLoading {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new("Loading...")
                .block(
                    PlaylistListLoading::gen_block(self.enabled, "Playlist").padding(Padding::new(
                        0,
                        0,
                        area.height / 2,
                        0,
                    )),
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            area,
        );
        Ok(())
    }
}

impl Focusable for PlaylistListLoading {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}

impl PlaylistListComps for PlaylistListLoading {}
