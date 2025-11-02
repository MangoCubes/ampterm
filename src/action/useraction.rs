use serde::{Deserialize, Serialize};

use crate::playerworker::player::QueueLocation;

/// Global actions are actions that are always available to the user, and has the same overall
/// effect regardless of what component is currently being focused at the time of triggering the
/// action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Global {
    /// Action for deleting all key sequences currently stored
    /// It's like escape in Vim, and Ctrl+G in Emacs
    EndKeySeq,
    TapToBPM,
    FocusPlaylistList,
    FocusPlaylistQueue,
    FocusPlayQueue,
    OpenTasks,
    CloseTasks,
    ToggleTasks,
}

impl ToString for Global {
    fn to_string(&self) -> String {
        match self {
            Global::EndKeySeq => "Cancel command".to_string(),
            Global::TapToBPM => "Tap to BPM".to_string(),
            Global::FocusPlaylistList => "Focus playlist list".to_string(),
            Global::FocusPlaylistQueue => "Focus playlist queue".to_string(),
            Global::FocusPlayQueue => "Focus play queue".to_string(),
            Global::OpenTasks => "Open tasks view".to_string(),
            Global::CloseTasks => "Close tasks view".to_string(),
            Global::ToggleTasks => "Toggle tasks view".to_string(),
        }
    }
}

/// Common actions are actions that are generally available to the user in any mode, but have
/// different effect depending on the current state of the program, such as what the user is
/// currently focused at. For example, when the currently focused element is PlaylistQueue, then
/// action [`Common::Up`] would move the cursor in PlaylistQueue. However, when PlaylistList is
/// currently being focused, [`Common::Up`] moves the cursor in PlaylistList instead. They have
/// similar meaning, but ultimately different actions depending on the program state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Common {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    Top,
    Bottom,
    Refresh,
    ResetState,
    Help,
}

impl ToString for Common {
    fn to_string(&self) -> String {
        match self {
            Common::Up => "Move up".to_string(),
            Common::Down => "Move down".to_string(),
            Common::Left => "Move left".to_string(),
            Common::Right => "Move right".to_string(),
            Common::Confirm => "Confirm selection".to_string(),
            Common::Cancel => "Cancel action".to_string(),
            Common::Top => "Go to top".to_string(),
            Common::Bottom => "Go to bottom".to_string(),
            Common::Refresh => "Refresh content".to_string(),
            Common::ResetState => "Reset to default state".to_string(),
            Common::Help => "Display help".to_string(),
        }
    }
}

/// These actions are relevant only when the program is in Visual mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Visual {
    // Exit visual mode after applying changes
    ExitSave,
    // Exit visual mode after discarding changes
    ExitDiscard,
    // Add current selection to the queue, and discards the changes
    Add(QueueLocation),
    // Delete a temporarily selected region, exiting visual mode
    Delete,
}

impl ToString for Visual {
    fn to_string(&self) -> String {
        match self {
            Visual::ExitSave => "Exit visual mode, saving selection".to_string(),
            Visual::ExitDiscard => "Exit visual mode, discarding selection".to_string(),
            Visual::Add(_) => "Add selected elements".to_string(),
            Visual::Delete => "Delete selected elements".to_string(),
        }
    }
}

/// Similarly, these actions are relevant only when the program is in Normal mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Normal {
    // Action for moving between boxes
    WindowUp,
    WindowDown,
    WindowLeft,
    WindowRight,
    // Enter visual mode to select items
    SelectMode,
    // Enter visual mode to deselect items
    DeselectMode,
    // Add to the queue
    Add(QueueLocation),
    // Delete a selected region
    Delete,
    // Adds/removes a song from favourites
    ToggleStar,
}

impl ToString for Normal {
    fn to_string(&self) -> String {
        match self {
            Normal::WindowUp => "Move window up",
            Normal::WindowDown => "Move window down",
            Normal::WindowLeft => "Move window left",
            Normal::WindowRight => "Move window right",
            Normal::SelectMode => "Enter visual mode (select)",
            Normal::DeselectMode => "Enter visual mode (deselect)",
            Normal::Add(_) => "Add to queue",
            Normal::Delete => "Delete selected items",
            Normal::ToggleStar => "Toggle favourite",
        }
        .to_string()
    }
}

/// These actions corresponds to user actions
/// Additionally, these actions are limited to the currently focused component only
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserAction {
    Common(Common),
    Normal(Normal),
    Visual(Visual),
    Global(Global),
}
