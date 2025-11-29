use serde::{Deserialize, Serialize};

use crate::{
    action::FromPlayerWorker,
    osclient::response::getplaylist::Media,
    playerworker::player::{QueueLocation, ToPlayerWorker},
    queryworker::query::{FromQueryWorker, ToQueryWorker},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    Normal,
    Visual,
    Insert,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match &self {
            Mode::Normal => "NORMAL".to_string(),
            Mode::Visual => "VISUAL".to_string(),
            Mode::Insert => "INSERT".to_string(),
        }
    }
}

/// These actions are all related to modifying the queue in one way or another.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueueAction {
    Add(Vec<Media>, QueueLocation),
}

/// These actions are associated with a specific component in the program, and are usually
/// available regardles of the currently focused component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GlobalAction {
    Play,
    Pause,
    Queue(QueueAction),
    Suspend,
    Resume,
    Quit,
}

/// These actions wraps communication between components. These should never be invoked by the
/// user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InternalAction {
    /// Send a request to the player worker
    ToPlayerWorker(ToPlayerWorker),
    /// Handle response from the player worker in response to the previous request
    FromPlayerWorker(FromPlayerWorker),

    /// Send a request to the query worker
    ToQueryWorker(ToQueryWorker),
    /// Receive a response from the query worker in response to the previous request
    FromQueryWorker(FromQueryWorker),

    ClearScreen,
    Resize(u16, u16),
    ChangeMode(Mode),
}
