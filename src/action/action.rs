use serde::{Deserialize, Serialize};

use crate::{
    osclient::response::getplaylist::Media,
    playerworker::player::{FromPlayerWorker, QueueLocation, ToPlayerWorker},
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
pub enum TargetedAction {
    Play,
    Pause,
    Skip,
    Previous,
    Queue(QueueAction),

    // Action for moving between boxes
    WindowUp,
    WindowDown,
    WindowLeft,
    WindowRight,

    TapToBPM,
    FocusPlaylistList,
    FocusPlaylistQueue,
    FocusPlayQueue,
    OpenTasks,
    CloseTasks,
    ToggleTasks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryAction {
    /// Handle response from the player worker in response to the previous request
    FromPlayerWorker(FromPlayerWorker),
    /// Receive a response from the query worker in response to the previous request
    FromQueryWorker(FromQueryWorker),
    ToPlayerWorker(ToPlayerWorker),
    ToQueryWorker(ToQueryWorker),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Multiple(Vec<Action>),
    Targeted(TargetedAction),
    ToPlayerWorker(ToPlayerWorker),
    ToQueryWorker(ToQueryWorker),
    Suspend,
    Resume,
    ClearScreen,
    Quit,
    Resize(u16, u16),
    ChangeMode(Mode),
    Query(QueryAction),
}
