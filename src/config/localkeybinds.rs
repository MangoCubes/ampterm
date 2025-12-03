use serde::Deserialize;
use strum::Display;

use crate::{config::keybindings::KeyBindings, playerworker::player::QueueLocation};

#[derive(Clone, Debug, Deserialize, PartialEq, Display)]
pub enum ListAction {
    ExitSave,
    ExitDiscard,
    Up,
    Down,
    Top,
    Bottom,
    ResetSelection,
    SelectMode,
    DeselectMode,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Display)]
pub enum PlayQueueAction {
    Delete,
    ToggleStar,
    PlaySelected,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Display)]
pub enum LyricsAction {
    Up,
    Down,
    Top,
    Bottom,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Display)]
pub enum PlaylistListAction {
    Up,
    Down,
    Top,
    Bottom,
    Add(QueueLocation),
    ViewSelected,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Display)]
pub enum PlaylistQueueAction {
    Refresh,
    Add(QueueLocation),
    ToggleStar,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Display)]
pub enum HelpAction {
    Up,
    Down,
    Left,
    Right,
}

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
}
