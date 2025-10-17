use crate::{
    action::{
        useraction::{Common, UserAction},
        Action,
    },
    components::{
        home::mainscreen::playlistqueue::PlaylistQueue,
        traits::{component::Component, focusable::Focusable},
    },
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::GetPlaylistParams, getplaylists::PlaylistID, ToQueryWorker},
    },
};
use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    text::Line,
    widgets::{Padding, Paragraph},
    Frame,
};

pub struct Error {
    id: PlaylistID,
    name: String,
    error: String,
    enabled: bool,
}

impl Error {
    pub fn new(id: PlaylistID, name: String, error: String, enabled: bool) -> Self {
        Self {
            id,
            name,
            error,
            enabled,
        }
    }
}

impl Component for Error {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw("Error!"),
                Line::raw(format!("{}", self.error)),
                Line::raw(format!("Reload with 'R'")),
            ])
            .block(
                PlaylistQueue::gen_block(self.enabled, self.name.clone()).padding(Padding::new(
                    0,
                    0,
                    (area.height / 2) - 1,
                    0,
                )),
            )
            .alignment(Alignment::Center),
            area,
        );
        Ok(())
    }
    fn update(&mut self, action: crate::action::Action) -> Result<Option<Action>> {
        if let Action::User(UserAction::Common(Common::Refresh)) = action {
            Ok(Some(Action::ToQueryWorker(ToQueryWorker::new(
                HighLevelQuery::SelectPlaylist(GetPlaylistParams {
                    name: self.name.clone(),
                    id: self.id.clone(),
                }),
            ))))
        } else {
            Ok(None)
        }
    }
}

impl Focusable for Error {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
