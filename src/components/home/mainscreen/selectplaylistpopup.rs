use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Clear, List, ListState},
    Frame,
};

use crate::{
    action::{
        action::{Action, TargetedAction},
        localaction::SelectPlaylistPopupAction,
    },
    components::{
        lib::centered::Centered,
        traits::{
            handlekeyseq::{HandleKeySeq, KeySeqResult},
            renderable::Renderable,
        },
    },
    config::{keybindings::KeyBindings, keyparser::KeyParser},
    osclient::{
        response::getplaylists::SimplePlaylist,
        types::{MediaID, PlaylistID},
    },
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{updateplaylist::UpdatePlaylistParams, ToQueryWorker},
    },
};

struct State {
    playlists: Vec<SimplePlaylist>,
    list: List<'static>,
    liststate: ListState,
}

pub struct SelectPlaylistPopup {
    binds: KeyBindings<SelectPlaylistPopupAction>,
    block: Block<'static>,
    items: Vec<MediaID>,
    state: Option<State>,
    quick_select: List<'static>,
}

impl SelectPlaylistPopup {
    pub fn new(
        items: Vec<MediaID>,
        binds: KeyBindings<SelectPlaylistPopupAction>,
        title: String,
    ) -> (Self, Action) {
        let list: Vec<String> = binds
            .0
            .iter()
            .filter_map(|(key, action)| {
                if let SelectPlaylistPopupAction::SelectID { id: _, name } = action {
                    Some(format!("{}: {}", KeyParser::keyseq_to_string(key), name))
                } else {
                    None
                }
            })
            .collect();
        (
            Self {
                items,
                binds,
                block: {
                    let style = Style::new().white();
                    let title = Span::styled(title, Style::default().add_modifier(Modifier::BOLD));
                    Block::bordered().title(title).border_style(style)
                },
                state: None,
                quick_select: List::new(list),
            },
            Action::ToQuery(ToQueryWorker::new(HighLevelQuery::ListPlaylistsPopup(
                false,
            ))),
        )
    }

    pub fn update_playlist(&mut self, p: Vec<SimplePlaylist>) {
        let list: Vec<String> = p.iter().map(|p| p.name.clone()).collect();
        self.state = Some(State {
            playlists: p,
            list: List::new(list)
                .highlight_style(Style::new().reversed())
                .highlight_symbol(">")
                .scroll_padding(1),
            liststate: ListState::default().with_selected(Some(0)),
        });
    }

    fn get_id(&self) -> Option<PlaylistID> {
        let Some(state) = &self.state else {
            return None;
        };
        let idx = state.liststate.selected()?;
        let playlist = state.playlists.get(idx)?;
        Some(playlist.id.clone())
    }
}

impl Renderable for SelectPlaylistPopup {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical([Constraint::Percentage(60)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(60)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        frame.render_widget(&self.block, area);
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(self.block.inner(area));
        match &mut self.state {
            Some(state) => {
                frame.render_stateful_widget(&state.list, right, &mut state.liststate);
            }
            None => Centered::new(vec!["Loading playlists...".to_string()]).draw(frame, area),
        }
        frame.render_widget(&self.quick_select, left);
    }
}

impl HandleKeySeq<SelectPlaylistPopupAction> for SelectPlaylistPopup {
    fn get_name(&self) -> &str {
        "SelectPlaylistPopup"
    }

    fn handle_local_action(&mut self, action: SelectPlaylistPopupAction) -> KeySeqResult {
        match action {
            SelectPlaylistPopupAction::Up => {
                if let Some(state) = &mut self.state {
                    state.liststate.select_previous();
                }
            }
            SelectPlaylistPopupAction::Down => {
                if let Some(state) = &mut self.state {
                    state.liststate.select_next();
                }
            }
            SelectPlaylistPopupAction::Top => {
                if let Some(state) = &mut self.state {
                    state.liststate.select_first();
                }
            }
            SelectPlaylistPopupAction::Bottom => {
                if let Some(state) = &mut self.state {
                    state.liststate.select_last();
                }
            }
            SelectPlaylistPopupAction::Cancel => {
                return KeySeqResult::ActionNeeded(Action::Targeted(TargetedAction::ClosePopup))
            }
            SelectPlaylistPopupAction::Confirm => {
                let Some(id) = self.get_id() else {
                    return KeySeqResult::ActionNeeded(Action::Targeted(
                        TargetedAction::ClosePopup,
                    ));
                };

                return KeySeqResult::ActionNeeded(Action::Multiple(vec![
                    Action::Targeted(TargetedAction::ClosePopup),
                    Action::ToQuery(ToQueryWorker::new(HighLevelQuery::UpdatePlaylist(
                        UpdatePlaylistParams {
                            playlist_id: id,
                            name: None,
                            comment: None,
                            public: None,
                            song_id_to_add: Some(self.items.clone()),
                            song_index_to_remove: None,
                        },
                    ))),
                ]));
            }
            SelectPlaylistPopupAction::SelectID { id, name: _ } => {
                return KeySeqResult::ActionNeeded(Action::Multiple(vec![
                    Action::Targeted(TargetedAction::ClosePopup),
                    Action::ToQuery(ToQueryWorker::new(HighLevelQuery::UpdatePlaylist(
                        UpdatePlaylistParams {
                            playlist_id: id,
                            name: None,
                            comment: None,
                            public: None,
                            song_id_to_add: Some(self.items.clone()),
                            song_index_to_remove: None,
                        },
                    ))),
                ]));
            }
        };
        KeySeqResult::NoActionNeeded
    }

    fn get_keybinds(&self) -> &KeyBindings<SelectPlaylistPopupAction> {
        &self.binds
    }
}
