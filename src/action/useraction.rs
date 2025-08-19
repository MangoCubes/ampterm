use serde::{Deserialize, Serialize};

use crate::playerworker::player::QueueLocation;

/// Common actions are actions that are applicable in more than one mode (Insert, Normal, etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visual {
    // Exit visual mode after applying changes
    ExitSave,
    // Exit visual mode after discarding changes
    ExitDiscard,
}

/// Normal mode exclusive actions
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// These actions corresponds to user actions
/// Additionally, these actions are limited to the currently focused component only
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAction {
    Common(Common),
    Normal(Normal),
    Visual(Visual),
}
