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
    FocusQueuelist,
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
