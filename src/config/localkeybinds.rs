use serde::Deserialize;
use strum::Display;

use crate::{config::keybindings::KeyBindings, playerworker::player::QueueLocation};

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

impl ToString for ListAction {
    fn to_string(&self) -> String {
        match self {
            ListAction::ExitSave => "Exit visual mode, apply selection",
            ListAction::ExitDiscard => "Exit visual mode, discard selection",
            ListAction::Up => "Move up",
            ListAction::Down => "Move down",
            ListAction::Top => "Move to top",
            ListAction::Bottom => "Move to bottom",
            ListAction::ResetSelection => "Reset selection",
            ListAction::SelectMode => "Enter selection mode",
            ListAction::DeselectMode => "Enter deselection mode",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum PlayQueueAction {
    Delete,
    ToggleStar,
    PlaySelected,
}

impl ToString for PlayQueueAction {
    fn to_string(&self) -> String {
        match self {
            PlayQueueAction::Delete => "Delete items from queue",
            PlayQueueAction::ToggleStar => "Star/unstar items",
            PlayQueueAction::PlaySelected => "Jump to the cursor's position",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum LyricsAction {
    Up,
    Down,
    Top,
    Bottom,
}

impl ToString for LyricsAction {
    fn to_string(&self) -> String {
        match self {
            LyricsAction::Up => "Move up",
            LyricsAction::Down => "Move down",
            LyricsAction::Top => "Move to top",
            LyricsAction::Bottom => "Move to bottom",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum PlaylistListAction {
    Up,
    Down,
    Top,
    Bottom,
    Add(QueueLocation),
    ViewSelected,
}

impl ToString for PlaylistListAction {
    fn to_string(&self) -> String {
        match self {
            PlaylistListAction::Up => "Move up",
            PlaylistListAction::Down => "Move down",
            PlaylistListAction::Top => "Move to top",
            PlaylistListAction::Bottom => "Move to bottom",
            PlaylistListAction::Add(queue_location) => match queue_location {
                QueueLocation::Front => "Play the entire playlist immediately",
                QueueLocation::Next => "Play the entire playlist next",
                QueueLocation::Last => "Append the entire playlist to the end of the queue",
            },
            PlaylistListAction::ViewSelected => "Open selected playlist",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum PlaylistQueueAction {
    Refresh,
    Add(QueueLocation),
    ToggleStar,
}

impl ToString for PlaylistQueueAction {
    fn to_string(&self) -> String {
        match self {
            PlaylistQueueAction::Refresh => "Fetch playlist from the server again",
            PlaylistQueueAction::Add(queue_location) => match queue_location {
                QueueLocation::Front => "Play selected items immediately",
                QueueLocation::Next => "Play selected items next",
                QueueLocation::Last => "Append selected items to the end of the queue",
            },
            PlaylistQueueAction::ToggleStar => "Star/unstar items",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Display)]
pub enum HelpAction {
    Up,
    Down,
    Left,
    Right,
    Close,
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
