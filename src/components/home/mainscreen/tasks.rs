use color_eyre::eyre::Result;
use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Clear, Row, Table},
    Frame,
};

use crate::{
    components::traits::component::Component,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{FromQueryWorker, ResponseType, ToQueryWorker},
    },
};

pub struct Tasks {
    border: Block<'static>,
    table: Table<'static>,
}

impl Tasks {
    pub fn new() -> Self {
        Self {
            border: Self::gen_block(),
            table: Table::new(
                [Row::new(vec!["ID", "Task", "Status"])],
                [Constraint::Max(4), Constraint::Min(1), Constraint::Max(7)],
            ),
        }
    }

    fn gen_block() -> Block<'static> {
        let style = Style::new().white();
        let title = Span::styled("Tasks", Style::default().add_modifier(Modifier::BOLD));
        Block::bordered().title(title).border_style(style)
    }

    pub fn register_task(&self, task: &ToQueryWorker) {
        match &task.query {
            HighLevelQuery::PlayMusicFromURL(media) => todo!(),
            HighLevelQuery::CheckCredentialValidity => todo!(),
            HighLevelQuery::SelectPlaylist(get_playlist_params) => todo!(),
            HighLevelQuery::AddPlaylistToQueue(get_playlist_params) => todo!(),
            HighLevelQuery::ListPlaylists => todo!(),
            HighLevelQuery::SetCredential(credential) => {}
        };
    }

    pub fn unregister_task(&self, task: &FromQueryWorker) {
        match &task.res {
            ResponseType::Ping(ping_response) => todo!(),
            ResponseType::GetPlaylists(get_playlists_response) => {
                todo!()
            }
            ResponseType::GetPlaylist(get_playlist_response) => todo!(),
            ResponseType::SetCredential(_) => todo!(),
        }
    }
}

impl Component for Tasks {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let vertical = Layout::vertical([Constraint::Percentage(80)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(80)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        frame.render_widget(&self.border, area);
        frame.render_widget(&self.table, self.border.inner(area));
        Ok(())
    }
}
