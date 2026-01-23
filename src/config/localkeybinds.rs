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
    #[serde(default)]
    pub list: KeyBindings<ListAction>,
    #[serde(default)]
    pub list_visual: KeyBindings<ListAction>,

    #[serde(default)]
    pub help: KeyBindings<HelpAction>,

    #[serde(default)]
    pub playqueue: KeyBindings<PlayQueueAction>,

    #[serde(default)]
    pub lyrics: KeyBindings<LyricsAction>,

    #[serde(default)]
    pub playlistlist: KeyBindings<PlaylistListAction>,

    #[serde(default)]
    pub playlistqueue: KeyBindings<PlaylistQueueAction>,

    #[serde(default)]
    pub popup: KeyBindings<PopupAction>,

    #[serde(default)]
    pub select_playlist_popup: KeyBindings<SelectPlaylistPopupAction>,
}
