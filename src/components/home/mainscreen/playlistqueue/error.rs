use crate::{
    action::action::{Action, QueryAction},
    components::{
        home::mainscreen::playlistqueue::PlaylistQueue,
        lib::centered::Centered,
        traits::{
            focusable::Focusable,
            handlekeyseq::{HandleKeySeq, KeySeqResult},
            renderable::Renderable,
        },
    },
    config::{keybindings::KeyBindings, localkeybinds::PlaylistQueueAction, Config},
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::GetPlaylistParams, getplaylists::PlaylistID, ToQueryWorker},
    },
};
use ratatui::{layout::Rect, Frame};

pub struct Error {
    id: PlaylistID,
    name: String,
    comp: Centered,
    enabled: bool,
    config: Config,
}

impl Error {
    pub fn new(config: Config, id: PlaylistID, name: String, error: String, enabled: bool) -> Self {
        Self {
            id,
            name,
            comp: Centered::new(vec![
                "Error!".to_string(),
                format!("{}", error),
                "Reload with 'R'".to_string(),
            ]),
            enabled,
            config,
        }
    }
}

impl Renderable for Error {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let border = PlaylistQueue::gen_block(self.enabled, self.name.clone());
        let inner = border.inner(area);
        frame.render_widget(border, area);
        self.comp.draw(frame, inner)
    }
}

impl HandleKeySeq<PlaylistQueueAction> for Error {
    fn get_name(&self) -> &str {
        "PlaylistQueue"
    }
    fn handle_local_action(&mut self, action: PlaylistQueueAction) -> KeySeqResult {
        match action {
            PlaylistQueueAction::Refresh => {
                KeySeqResult::ActionNeeded(Action::Query(QueryAction::ToQueryWorker(
                    ToQueryWorker::new(HighLevelQuery::SelectPlaylist(GetPlaylistParams {
                        name: self.name.clone(),
                        id: self.id.clone(),
                    })),
                )))
            }
            _ => KeySeqResult::NoActionNeeded,
        }
    }

    fn get_keybinds(&self) -> &KeyBindings<PlaylistQueueAction> {
        &self.config.local.playlistqueue
    }
}

impl Focusable for Error {
    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
