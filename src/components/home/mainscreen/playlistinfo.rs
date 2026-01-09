use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Clear, Row, Table, TableState},
    Frame,
};

use crate::{
    action::localaction::ListAction,
    components::traits::{
        handlekeyseq::{HandleKeySeq, KeySeqResult},
        renderable::Renderable,
    },
    config::keybindings::KeyBindings,
    osclient::response::getplaylists::SimplePlaylist,
};

pub struct PlaylistInfo {
    table: Table<'static>,
    state: TableState,
    binds: KeyBindings<ListAction>,
    block: Block<'static>,
}

impl PlaylistInfo {
    pub fn new(playlist: SimplePlaylist, binds: KeyBindings<ListAction>) -> Self {
        let rows: Vec<Row<'static>> = [
            ["Name".to_string(), playlist.name],
            [
                "Owner".to_string(),
                if let Some(o) = playlist.owner {
                    o
                } else {
                    "".to_string()
                },
            ],
            [
                "Public".to_string(),
                if let Some(p) = playlist.public {
                    if p {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    }
                } else {
                    "".to_string()
                },
            ],
            ["Song Count".to_string(), playlist.song_count.to_string()],
            ["Duration".to_string(), {
                let secs = playlist.duration;
                if secs > (60 * 60) {
                    format!("{:02}:{:02}:{:02}", secs / (60 * 60), secs / 60, secs % 60)
                } else {
                    format!("{:02}:{:02}", secs / 60, secs % 60)
                }
            }],
            ["Created At".to_string(), playlist.created],
            ["Last Modified".to_string(), playlist.changed],
        ]
        .into_iter()
        .map(Row::new)
        .collect();
        Self {
            table: Table::new(rows, [Constraint::Fill(1), Constraint::Fill(1)]),
            state: TableState::default().with_offset(0),
            binds,
            block: {
                let style = Style::new().white();
                let title = Span::styled(
                    "Playlist Information",
                    Style::default().add_modifier(Modifier::BOLD),
                );
                Block::bordered().title(title).border_style(style)
            },
        }
    }
}

impl Renderable for PlaylistInfo {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical([Constraint::Percentage(50)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        frame.render_widget(&self.block, area);
        frame.render_stateful_widget(&self.table, self.block.inner(area), &mut self.state);
    }
}
impl HandleKeySeq<ListAction> for PlaylistInfo {
    fn get_name(&self) -> &str {
        "MediaInfo"
    }

    fn handle_local_action(&mut self, action: ListAction) -> KeySeqResult {
        match action {
            ListAction::Up => self.state.select_previous(),
            ListAction::Down => self.state.select_next(),
            ListAction::Top => self.state.select_first(),
            ListAction::Bottom => self.state.select_last(),
            _ => {}
        };
        KeySeqResult::NoActionNeeded
    }

    fn get_keybinds(&self) -> &KeyBindings<ListAction> {
        &self.binds
    }
}
