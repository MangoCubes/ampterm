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

    Skip,
    Previous,
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
            Global::Skip => "Skip to next music".to_string(),
            Global::Previous => "Go back to previous music".to_string(),
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
    // Add to the queue
    Add(QueueLocation),
    // Delete a selected region
    Delete,
    // Adds/removes a song from favourites
    ToggleStar,
}

impl ToString for Common {
    fn to_string(&self) -> String {
        match self {
            Common::Up => "Move up",
            Common::Down => "Move down",
            Common::Left => "Move left",
            Common::Right => "Move right",
            Common::Confirm => "Confirm selection",
            Common::Cancel => "Cancel action",
            Common::Top => "Go to top",
            Common::Bottom => "Go to bottom",
            Common::Refresh => "Refresh content",
            Common::ResetState => "Reset to default state",
            Common::Help => "Display help",
            Common::Add(l) => match l {
                QueueLocation::Front => "Play items",
                QueueLocation::Next => "Play items next",
                QueueLocation::Last => "Add items to the end of queue",
            },
            Common::Delete => "Delete selected elements",
            Common::ToggleStar => "Toggle favourite",
        }
        .to_string()
    }
}

/// These actions are relevant only when the program is in Visual mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Visual {
    // Exit visual mode after applying changes
    ExitSave,
    // Exit visual mode after discarding changes
    ExitDiscard,
}

impl ToString for Visual {
    fn to_string(&self) -> String {
        match self {
            Visual::ExitSave => "Exit visual mode, saving selection".to_string(),
            Visual::ExitDiscard => "Exit visual mode, discarding selection".to_string(),
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
