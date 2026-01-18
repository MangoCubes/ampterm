use serde::Deserialize;

use crate::{
    action::localaction::{
        HelpAction, ListAction, LyricsAction, PlayQueueAction, PlaylistListAction,
        PlaylistQueueAction, PopupAction, SelectPlaylistPopupAction,
    },
    config::keybindings::KeyBindings,
};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LocalKeyBinds {
    /// Technically they're tables, but they behave more like a list, which is why they are called
    /// list instead
    pub list: KeyBindings<ListAction>,
    pub list_visual: KeyBindings<ListAction>,

    pub help: KeyBindings<HelpAction>,

    pub playqueue: KeyBindings<PlayQueueAction>,

    pub lyrics: KeyBindings<LyricsAction>,

    pub playlistlist: KeyBindings<PlaylistListAction>,

    pub playlistqueue: KeyBindings<PlaylistQueueAction>,

    pub popup: KeyBindings<PopupAction>,

    pub select_playlist_popup: KeyBindings<SelectPlaylistPopupAction>,
}
