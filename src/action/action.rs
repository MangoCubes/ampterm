use serde::{Deserialize, Serialize};
use strum::Display;

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display)]
pub enum TargetedAction {
    Play,
    Pause,
    PlayOrPause,
    Skip,
    Previous,
    Queue(QueueAction),
    GoToStart,
    ChangeVolume(f32),

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

    EndKeySeq,

    OpenHelp,
    CloseHelp,
    ToggleHelp,

    Suspend,
    Resume,
    ClearScreen,
    Quit,
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
    Resize(u16, u16),
    ChangeMode(Mode),
    Query(QueryAction),
}
