use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::osclient::response::getplaylist::Media;
use crate::playerworker::player::ToPlayerWorker;
use crate::queryworker::query::FromQueryWorker;
use crate::{app::Mode, playerworker::player::QueueLocation, queryworker::query::ToQueryWorker};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateType {
    Position(std::time::Duration),
    Volume(f32),
    Speed(f32),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayState {
    pub items: Vec<Media>,
    // Index is guaranteed to be in the range [0, items.len()]
    // In other words, items[index] may be invalid because index goes out of bound by 1
    pub index: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayOrder {
    Normal,
    Random,
    Reverse,
}

impl PlayState {
    pub fn default() -> Self {
        Self {
            items: Vec::new(),
            index: 0,
        }
    }
    pub fn new(items: Vec<Media>, index: usize) -> Self {
        Self { items, index }
    }
}

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

/// These actions are emitted by the playerworker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromPlayerWorker {
    // This action is used to synchronise the state of PlayerWorker with the components
    InQueue {
        play: PlayState,
        vol: f32,
        speed: f32,
        pos: Duration,
    },
    // This actions is used to send current position
    PlayerState(StateType),
    // Error sent out from the player to the components
    PlayerError(String),
    PlayerMessage(String),
}

/// FromQueryWorker enum is used to send responses from the QueryWorker to the components

/// These actions corresponds to user actions
/// Additionally, these actions are limited to the currently focused component only
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAction {
    Common(Common),
    Normal(Normal),
    Visual(Visual),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Suspend,
    Resume,
    Quit,
    ClearScreen,

    User(UserAction),

    FromQueryWorker(FromQueryWorker),
    ToQueryWorker(ToQueryWorker),

    FromPlayerWorker(FromPlayerWorker),
    ToPlayerWorker(ToPlayerWorker),
    // This action is fired from the components to the app
    ChangeMode(Mode),

    // Anything below this should not be used for keybinds, but feel free to experiment. Most are used to notify the system
    // System actions
    Tick,
    Render,
    // ModeChanged(InputMode),
    Resize(u16, u16),
    Error(String),
    // Action for deleting all key sequences currently stored
    // It's like escape in Vim, and Ctrl+G in Emacs
    EndKeySeq,
}
