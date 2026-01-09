use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Clear, Row, Table, TableState},
    Frame,
};

use crate::{
    action::{
        action::{Action, TargetedAction},
        localaction::PopupAction,
    },
    components::traits::{
        handlekeyseq::{HandleKeySeq, KeySeqResult},
        renderable::Renderable,
    },
    config::keybindings::KeyBindings,
    osclient::response::getplaylist::Media,
};

pub struct MediaInfo {
    table: Table<'static>,
    state: TableState,
    binds: KeyBindings<PopupAction>,
    block: Block<'static>,
}

impl MediaInfo {
    pub fn new(media: Media, binds: KeyBindings<PopupAction>) -> Self {
        let rows: Vec<Row<'static>> = [
            ["Title".to_string(), media.title],
            ["ID".to_string(), media.id.to_string()],
            ["Album".to_string(), media.album.unwrap_or("".to_string())],
            ["Artist".to_string(), media.artist.unwrap_or("".to_string())],
            [
                "Track".to_string(),
                if let Some(track) = media.track {
                    track.to_string()
                } else {
                    "".to_string()
                },
            ],
            [
                "Year".to_string(),
                if let Some(year) = media.year {
                    year.to_string()
                } else {
                    "".to_string()
                },
            ],
            ["Genre".to_string(), media.genre.unwrap_or("".to_string())],
            [
                "Duration".to_string(),
                if let Some(duration) = media.duration {
                    duration.to_string()
                } else {
                    "".to_string()
                },
            ],
            [
                "Bit Rate".to_string(),
                if let Some(t) = media.bit_rate {
                    t.to_string()
                } else {
                    "".to_string()
                },
            ],
            [
                "File Size".to_string(),
                if let Some(s) = media.size {
                    s.to_string()
                } else {
                    "".to_string()
                },
            ],
            [
                "User Rating".to_string(),
                if let Some(rating) = media.user_rating {
                    rating.to_string()
                } else {
                    "".to_string()
                },
            ],
            [
                "Average Rating".to_string(),
                if let Some(rating) = media.average_rating {
                    rating.to_string()
                } else {
                    "".to_string()
                },
            ],
            [
                "Play Count".to_string(),
                if let Some(c) = media.play_count {
                    c.to_string()
                } else {
                    "".to_string()
                },
            ],
            [
                "Starred At".to_string(),
                if let Some(c) = media.starred {
                    c.to_string()
                } else {
                    "".to_string()
                },
            ],
        ]
        .into_iter()
        .map(Row::new)
        .collect();
        Self {
            table: Table::new(rows, [Constraint::Max(15), Constraint::Fill(1)]),
            state: TableState::default().with_selected(0),
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

impl Renderable for MediaInfo {
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

impl HandleKeySeq<PopupAction> for MediaInfo {
    fn get_name(&self) -> &str {
        "MediaInfo"
    }

    fn handle_local_action(&mut self, action: PopupAction) -> KeySeqResult {
        match action {
            PopupAction::Up => self.state.select_previous(),
            PopupAction::Down => self.state.select_next(),
            PopupAction::Top => self.state.select_first(),
            PopupAction::Bottom => self.state.select_last(),
            PopupAction::Close => {
                return KeySeqResult::ActionNeeded(Action::Targeted(TargetedAction::ClosePopup));
            }
        };
        KeySeqResult::NoActionNeeded
    }

    fn get_keybinds(&self) -> &KeyBindings<PopupAction> {
        &self.binds
    }
}
