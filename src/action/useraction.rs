use serde::{Deserialize, Serialize};

use crate::playerworker::player::QueueLocation;

/// Actions that are always available to the user regardless of what the user is currently
/// focusing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Global {
    /// Action for deleting all key sequences currently stored
    /// It's like escape in Vim, and Ctrl+G in Emacs
    EndKeySeq,
    TapToBPM,
}

/// Common actions are actions that are applicable in more than one mode (Insert, Normal, etc), but
/// must not propagate to more than one components.
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

/// Visual mode exclusive actions
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

/// Normal mode exclusive actions
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
