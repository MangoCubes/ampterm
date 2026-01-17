use serde::Deserialize;
use strum::Display;

use crate::{osclient::types::PlaylistID, playerworker::player::QueueLocation};

/// Actions for all of the visual lists (lists where you can select multiple items). Not applicable
/// for lists where you cannot select items.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum ListAction {
    /// Exit visual mode, applying the selection
    ExitSave,
    /// Exit visual mode, discarding the selection
    ExitDiscard,
    /// Move one up in the list
    Up,
    /// Move one down in the list
    Down,
    /// Move to the top of the list
    Top,
    /// Move to the bottom of the list
    Bottom,
    /// Unselect everything
    ResetSelection,
    /// Enter selection mode. When you exit visual mode with ExitSave, the selected items will be
    /// selected.
    SelectMode,
    /// Enter selection mode. When you exit visual mode with ExitSave, the selected items will be
    /// unselected.
    DeselectMode,

    SearchNext,
    SearchPrev,
}

impl ToString for ListAction {
    fn to_string(&self) -> String {
        match self {
            ListAction::SearchNext => "Jump to the next item that matches the keyword",
            ListAction::SearchPrev => "Jump to the previous item that matches the keyword",
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
    /// Delete selected items from the play queue
    Delete,
    /// Star or unstar selected elements
    ToggleStar,
    /// Jump to a specific location in the play queue
    PlaySelected,
    /// Move the cursor to the currently playing item
    FocusPlaying,
    /// Randomise the order of everything that comes after item that is currently being played
    Randomise,
    /// Display metadata of the highlighted song
    ViewInfo,
    /// Add selected items to a playlist
    AddToPlaylist,
}

impl ToString for PlayQueueAction {
    fn to_string(&self) -> String {
        match self {
            PlayQueueAction::Delete => "Delete items from queue",
            PlayQueueAction::ToggleStar => "Star/unstar items",
            PlayQueueAction::PlaySelected => "Jump to the cursor's position",
            PlayQueueAction::FocusPlaying => "Focus currently playing item",
            PlayQueueAction::Randomise => "Randomise queued items",
            PlayQueueAction::ViewInfo => "Display metadata of the highlighted song",
            PlayQueueAction::AddToPlaylist => "Add items to a playlist",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum LyricsAction {
    /// Move up one line of the lyrics
    Up,
    /// Move down one line of the lyrics
    Down,
    /// Jump to the top of the lyrics
    Top,
    /// Jump to the bottom of the lyrics
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
    /// Add the entire playlist to the queue
    Add(QueueLocation),
    /// Shuffle the entire playlist, then add it to the queue
    RandomAdd(QueueLocation),
    /// Open the seleced playlist in the playlist queue view
    ViewSelected,

    ViewInfo,
}

impl ToString for PlaylistListAction {
    fn to_string(&self) -> String {
        match self {
            PlaylistListAction::Add(queue_location) => match queue_location {
                QueueLocation::Front => "Play the entire playlist immediately",
                QueueLocation::Next => "Play the entire playlist next",
                QueueLocation::Last => "Append the entire playlist to the end of the queue",
            },
            PlaylistListAction::ViewSelected => "Open selected playlist",
            PlaylistListAction::RandomAdd(queue_location) => match queue_location {
                QueueLocation::Front => "Shuffle the playlist and play it immediately",
                QueueLocation::Next => "Shuffle the playlist and play it next",
                QueueLocation::Last => "Shuffle the playlist and append it to the queue",
            },
            PlaylistListAction::ViewInfo => "View details of the current playlist",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum PlaylistQueueAction {
    /// Refresh the playlist queue
    Refresh,
    /// Add selected items to the playlist queue
    Add(QueueLocation),
    /// Star or unstar selected items
    ToggleStar,
    /// Shuffle the selected items, and add them to the queue
    RandomAdd(QueueLocation),
    /// Go into filter mode
    Filter,
    /// Remove filter
    ClearFilter,
    /// Search for a keyword. Unlike filter, this is applied immediately and does not hide
    /// non-matching elements. User can jump between matched items with SearchNext and SearchPrev.
    Search,
    ClearSearch,
    ViewInfo,
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
            PlaylistQueueAction::RandomAdd(queue_location) => match queue_location {
                QueueLocation::Front => "Shuffle the selected items and play it immediately",
                QueueLocation::Next => "Shuffle the selected items and play it next",
                QueueLocation::Last => "Shuffle the selected items and append it to the queue",
            },
            PlaylistQueueAction::ToggleStar => "Star/unstar items",
            PlaylistQueueAction::Filter => "Filter items (hides items that do not match)",
            PlaylistQueueAction::ClearFilter => "Remove filter",
            PlaylistQueueAction::Search => "Search for a specific keyword",
            PlaylistQueueAction::ClearSearch => "Clear the current search",
            PlaylistQueueAction::ViewInfo => "View details of the current media",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Display)]
pub enum HelpAction {
    /// Move up one item in the help menu
    Up,
    /// Move down one item in the help menu
    Down,
    /// See keybinds for the previous component
    Left,
    /// See keybinds for the next component
    Right,
    /// Close help page
    Close,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum PopupAction {
    Up,
    Down,
    Top,
    Bottom,
    Close,
}

impl ToString for PopupAction {
    fn to_string(&self) -> String {
        match self {
            PopupAction::Up => "Move up",
            PopupAction::Down => "Move down",
            PopupAction::Top => "Move to top",
            PopupAction::Bottom => "Move to bottom",
            PopupAction::Close => "Close popup",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum SelectPlaylistPopupAction {
    Up,
    Down,
    Top,
    Bottom,
    Cancel,
    Confirm,
    SelectID(PlaylistID, String),
}

impl ToString for SelectPlaylistPopupAction {
    fn to_string(&self) -> String {
        match self {
            SelectPlaylistPopupAction::Up => "Move up".to_string(),
            SelectPlaylistPopupAction::Down => "Move down".to_string(),
            SelectPlaylistPopupAction::Top => "Move to top".to_string(),
            SelectPlaylistPopupAction::Bottom => "Move to bottom".to_string(),
            SelectPlaylistPopupAction::Cancel => "Cancel action".to_string(),
            SelectPlaylistPopupAction::Confirm => "Proceed with the selected playlist".to_string(),
            SelectPlaylistPopupAction::SelectID(_, name) => {
                format!("Select playlist {}", name.clone())
            }
        }
    }
}
