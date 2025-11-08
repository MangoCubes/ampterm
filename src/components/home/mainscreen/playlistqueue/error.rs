use crate::{
    action::{
        useraction::{Common, UserAction},
        Action,
    },
    components::{
        home::mainscreen::playlistqueue::PlaylistQueue,
        lib::centered::Centered,
        traits::{focusable::Focusable, fullcomp::FullComp, renderable::Renderable},
    },
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::GetPlaylistParams, getplaylists::PlaylistID, ToQueryWorker},
    },
};
use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

pub struct Error {
    id: PlaylistID,
    name: String,
    comp: Centered,
    enabled: bool,
}

impl Error {
    pub fn new(id: PlaylistID, name: String, error: String, enabled: bool) -> Self {
        Self {
            id,
            name,
            comp: Centered::new(vec![
                "Error!".to_string(),
                format!("{}", error),
                "Reload with 'R'".to_string(),
            ]),
            enabled,
        }
    }
}

impl Renderable for Error {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let border = PlaylistQueue::gen_block(self.enabled, self.name.clone());
        let inner = border.inner(area);
        frame.render_widget(border, area);
        self.comp.draw(frame, inner)
    }
}

impl FullComp for Error {
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
